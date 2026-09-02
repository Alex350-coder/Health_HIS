//! Operating Rooms orchestration (Database.md Section 3.4, IPC.md Section 2.1). Operating Rooms
//! owns the write path for `operating_rooms`/`or_reservations`, promoting an existing `room`
//! (owned by Beds) into a schedulable resource. Thin command layer calls into this module only.

use rusqlite::Connection;

use crate::db::DbError;
use crate::errors::app_error::correlation_id;
use crate::errors::AppError;
use crate::models::{OperatingRoom, OrReservation};
use crate::repositories::bed_repository;
use crate::repositories::encounter_repository;
use crate::repositories::operating_room_repository::{
    self, NewOperatingRoom, NewOrReservation, ReservationDetails,
};
use crate::repositories::patient_repository;
use crate::services::audit_service::{self, RecordInput};
use crate::validation::operating_room_validation::{
    self, CancelOrReservationInput, CreateOperatingRoomInput, CreateOrReservationInput,
    UpdateOrReservationInput,
};

pub fn create_operating_room(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateOperatingRoomInput,
) -> Result<OperatingRoom, AppError> {
    operating_room_validation::validate_create_operating_room(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let room = bed_repository::find_room_by_id(&tx, input.room_id)?.ok_or(AppError::NotFound {
        entity: "room".to_string(),
        id: input.room_id,
    })?;
    if room.room_type != "operating_room" {
        return Err(AppError::Validation {
            field: "roomId".to_string(),
            message: "room must have room_type 'operating_room'".to_string(),
        });
    }
    if operating_room_repository::find_operating_room_by_room_id(&tx, input.room_id)?.is_some() {
        return Err(AppError::Conflict {
            message: "room is already promoted to an operating room".to_string(),
        });
    }

    let operating_room_id = operating_room_repository::insert_operating_room(
        &tx,
        &NewOperatingRoom {
            room_id: input.room_id,
            name: &room.name,
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "operating_room.create",
            entity_type: "operating_room",
            entity_id: Some(operating_room_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_operating_room_or_die(conn, operating_room_id)
}

/// Enforces the overlap-prevention business rule (Database.md Section 3.4): no two
/// `scheduled`/`in_progress` reservations for the same operating room may overlap in time.
/// Checked explicitly here rather than inferred from a DB constraint, since SQLite has no
/// native exclusion constraints and `AppError::from(DbError)` maps every DB error generically.
pub fn reserve(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateOrReservationInput,
) -> Result<OrReservation, AppError> {
    operating_room_validation::validate_create_or_reservation(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    require_operating_room_exists(&tx, input.operating_room_id)?;
    require_patient_exists(&tx, input.patient_id)?;
    require_encounter_exists(&tx, input.encounter_id)?;

    if !operating_room_repository::find_overlapping_reservations(
        &tx,
        input.operating_room_id,
        &input.scheduled_start,
        &input.scheduled_end,
        None,
    )?
    .is_empty()
    {
        return Err(AppError::Conflict {
            message: "operating room already reserved for an overlapping time".to_string(),
        });
    }

    let reservation_id = operating_room_repository::insert_reservation(
        &tx,
        &NewOrReservation {
            operating_room_id: input.operating_room_id,
            patient_id: input.patient_id,
            encounter_id: input.encounter_id,
            procedure_description: &input.procedure_description,
            scheduled_start: &input.scheduled_start,
            scheduled_end: &input.scheduled_end,
            scheduled_by_user_id: actor_user_id,
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "or_reservation.create",
            entity_type: "or_reservation",
            entity_id: Some(reservation_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_reservation_or_die(conn, reservation_id)
}

pub fn update_reservation(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &UpdateOrReservationInput,
) -> Result<OrReservation, AppError> {
    operating_room_validation::validate_update_or_reservation(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let existing = operating_room_repository::find_reservation_by_id(&tx, input.id)?.ok_or(
        AppError::NotFound {
            entity: "or_reservation".to_string(),
            id: input.id,
        },
    )?;

    if !operating_room_repository::find_overlapping_reservations(
        &tx,
        existing.operating_room_id,
        &input.scheduled_start,
        &input.scheduled_end,
        Some(input.id),
    )?
    .is_empty()
    {
        return Err(AppError::Conflict {
            message: "operating room already reserved for an overlapping time".to_string(),
        });
    }

    operating_room_repository::update_reservation_details(
        &tx,
        input.id,
        &ReservationDetails {
            procedure_description: &input.procedure_description,
            scheduled_start: &input.scheduled_start,
            scheduled_end: &input.scheduled_end,
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "or_reservation.update",
            entity_type: "or_reservation",
            entity_id: Some(input.id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_reservation_or_die(conn, input.id)
}

pub fn cancel_reservation(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CancelOrReservationInput,
) -> Result<OrReservation, AppError> {
    operating_room_validation::validate_cancel_or_reservation(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let existing = operating_room_repository::find_reservation_by_id(&tx, input.id)?.ok_or(
        AppError::NotFound {
            entity: "or_reservation".to_string(),
            id: input.id,
        },
    )?;
    if existing.status == "cancelled" || existing.status == "completed" {
        return Err(AppError::Conflict {
            message: "reservation is already cancelled or completed".to_string(),
        });
    }

    operating_room_repository::update_reservation_status(&tx, input.id, "cancelled")?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "or_reservation.cancel",
            entity_type: "or_reservation",
            entity_id: Some(input.id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_reservation_or_die(conn, input.id)
}

pub fn list_operating_rooms(conn: &Connection) -> Result<Vec<OperatingRoom>, AppError> {
    Ok(operating_room_repository::list_operating_rooms(conn)?)
}

pub fn list_reservations(
    conn: &Connection,
    operating_room_id: Option<i64>,
) -> Result<Vec<OrReservation>, AppError> {
    Ok(operating_room_repository::list_reservations(
        conn,
        operating_room_id,
    )?)
}

/// Read-only helper for Billing's OR-charge aggregation (Plan.md Phase 12) — every reservation
/// for one encounter, including cancelled ones.
pub fn list_reservations_for_encounter(
    conn: &Connection,
    encounter_id: i64,
) -> Result<Vec<OrReservation>, AppError> {
    Ok(operating_room_repository::find_reservations_by_encounter_id(conn, encounter_id)?)
}

fn find_operating_room_or_die(conn: &Connection, id: i64) -> Result<OperatingRoom, AppError> {
    operating_room_repository::find_operating_room_by_id(conn, id)?
        .ok_or_else(|| unexpected_vanished("operating_room", id))
}

fn find_reservation_or_die(conn: &Connection, id: i64) -> Result<OrReservation, AppError> {
    operating_room_repository::find_reservation_by_id(conn, id)?
        .ok_or_else(|| unexpected_vanished("or_reservation", id))
}

fn require_operating_room_exists(
    conn: &Connection,
    operating_room_id: i64,
) -> Result<(), AppError> {
    operating_room_repository::find_operating_room_by_id(conn, operating_room_id)?.ok_or(
        AppError::NotFound {
            entity: "operating_room".to_string(),
            id: operating_room_id,
        },
    )?;
    Ok(())
}

fn require_patient_exists(conn: &Connection, patient_id: i64) -> Result<(), AppError> {
    patient_repository::find_by_id(conn, patient_id)?.ok_or(AppError::NotFound {
        entity: "patient".to_string(),
        id: patient_id,
    })?;
    Ok(())
}

fn require_encounter_exists(conn: &Connection, encounter_id: i64) -> Result<(), AppError> {
    encounter_repository::find_encounter_by_id(conn, encounter_id)?.ok_or(AppError::NotFound {
        entity: "encounter".to_string(),
        id: encounter_id,
    })?;
    Ok(())
}

fn unexpected_vanished(entity: &'static str, id: i64) -> AppError {
    let correlation_id = correlation_id();
    tracing::error!(
        correlation_id,
        entity,
        id,
        "row vanished immediately after being written"
    );
    AppError::Unexpected {
        message: "an unexpected error occurred".to_string(),
        correlation_id,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};
    use crate::repositories::audit_repository;

    const TEST_KEY: [u8; 32] = [31u8; 32];
    const ACTOR_USER_ID: i64 = 1;

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("or-service-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn
    }

    fn seed_user(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO users (full_name, username, password_hash, role) \
             VALUES ('Test Admin', 'test-admin', 'argon2id$dummy', 'admin')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn seed_patient(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO patients (medical_record_number, full_name, date_of_birth, sex) \
             VALUES ('MRN-1', 'Jane Doe', '1990-01-01', 'female')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn seed_encounter(conn: &Connection, patient_id: i64, created_by_user_id: i64) -> i64 {
        conn.execute(
            "INSERT INTO encounters (patient_id, created_by_user_id) VALUES (?1, ?2)",
            rusqlite::params![patient_id, created_by_user_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn seed_room(conn: &Connection, room_type: &str) -> i64 {
        conn.execute(
            "INSERT INTO floors (name, level_order) VALUES ('Ground Floor', 0)",
            [],
        )
        .unwrap();
        let floor_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO rooms (floor_id, name, room_type, map_x, map_y) \
             VALUES (?1, 'OR Suite', ?2, 0.5, 0.5)",
            rusqlite::params![floor_id, room_type],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn create_operating_room_input(room_id: i64) -> CreateOperatingRoomInput {
        CreateOperatingRoomInput { room_id }
    }

    fn reservation_input(
        operating_room_id: i64,
        patient_id: i64,
        encounter_id: i64,
    ) -> CreateOrReservationInput {
        CreateOrReservationInput {
            operating_room_id,
            patient_id,
            encounter_id,
            procedure_description: "Appendectomy".to_string(),
            scheduled_start: "2026-01-01T08:00:00".to_string(),
            scheduled_end: "2026-01-01T10:00:00".to_string(),
        }
    }

    #[test]
    fn create_operating_room_promotes_a_valid_room() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let room_id = seed_room(&conn, "operating_room");

        let operating_room =
            create_operating_room(&mut conn, user_id, &create_operating_room_input(room_id))
                .unwrap();

        assert_eq!(operating_room.room_id, room_id);
    }

    #[test]
    fn create_operating_room_rejects_a_room_with_the_wrong_type() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let room_id = seed_room(&conn, "ward");

        let result = create_operating_room(
            &mut conn,
            ACTOR_USER_ID,
            &create_operating_room_input(room_id),
        );

        assert!(matches!(result, Err(AppError::Validation { .. })));
    }

    #[test]
    fn create_operating_room_rejects_a_duplicate_promotion() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let room_id = seed_room(&conn, "operating_room");
        create_operating_room(&mut conn, user_id, &create_operating_room_input(room_id)).unwrap();

        let result =
            create_operating_room(&mut conn, user_id, &create_operating_room_input(room_id));

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn reserve_creates_a_scheduled_reservation() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let room_id = seed_room(&conn, "operating_room");
        let operating_room =
            create_operating_room(&mut conn, user_id, &create_operating_room_input(room_id))
                .unwrap();

        let reservation = reserve(
            &mut conn,
            user_id,
            &reservation_input(operating_room.id, patient_id, encounter_id),
        )
        .unwrap();

        assert_eq!(reservation.status, "scheduled");
    }

    #[test]
    fn reserve_an_overlapping_time_returns_conflict() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let room_id = seed_room(&conn, "operating_room");
        let operating_room =
            create_operating_room(&mut conn, user_id, &create_operating_room_input(room_id))
                .unwrap();
        reserve(
            &mut conn,
            user_id,
            &reservation_input(operating_room.id, patient_id, encounter_id),
        )
        .unwrap();

        let overlapping = CreateOrReservationInput {
            scheduled_start: "2026-01-01T09:00:00".to_string(),
            scheduled_end: "2026-01-01T11:00:00".to_string(),
            ..reservation_input(operating_room.id, patient_id, encounter_id)
        };
        let result = reserve(&mut conn, user_id, &overlapping);

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn cancel_reservation_sets_status_to_cancelled() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let room_id = seed_room(&conn, "operating_room");
        let operating_room =
            create_operating_room(&mut conn, user_id, &create_operating_room_input(room_id))
                .unwrap();
        let reservation = reserve(
            &mut conn,
            user_id,
            &reservation_input(operating_room.id, patient_id, encounter_id),
        )
        .unwrap();

        let cancelled = cancel_reservation(
            &mut conn,
            user_id,
            &CancelOrReservationInput { id: reservation.id },
        )
        .unwrap();

        assert_eq!(cancelled.status, "cancelled");
    }

    #[test]
    fn cancel_reservation_twice_returns_conflict() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let room_id = seed_room(&conn, "operating_room");
        let operating_room =
            create_operating_room(&mut conn, user_id, &create_operating_room_input(room_id))
                .unwrap();
        let reservation = reserve(
            &mut conn,
            user_id,
            &reservation_input(operating_room.id, patient_id, encounter_id),
        )
        .unwrap();
        cancel_reservation(
            &mut conn,
            user_id,
            &CancelOrReservationInput { id: reservation.id },
        )
        .unwrap();

        let result = cancel_reservation(
            &mut conn,
            user_id,
            &CancelOrReservationInput { id: reservation.id },
        );

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn update_reservation_persists_new_details() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let room_id = seed_room(&conn, "operating_room");
        let operating_room =
            create_operating_room(&mut conn, user_id, &create_operating_room_input(room_id))
                .unwrap();
        let reservation = reserve(
            &mut conn,
            user_id,
            &reservation_input(operating_room.id, patient_id, encounter_id),
        )
        .unwrap();

        let updated = update_reservation(
            &mut conn,
            user_id,
            &UpdateOrReservationInput {
                id: reservation.id,
                procedure_description: "Cholecystectomy".to_string(),
                scheduled_start: "2026-01-02T08:00:00".to_string(),
                scheduled_end: "2026-01-02T10:00:00".to_string(),
            },
        )
        .unwrap();

        assert_eq!(updated.procedure_description, "Cholecystectomy");
    }

    #[test]
    fn the_audit_chain_stays_valid_after_a_reserve_and_cancel_sequence() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let room_id = seed_room(&conn, "operating_room");
        let operating_room =
            create_operating_room(&mut conn, user_id, &create_operating_room_input(room_id))
                .unwrap();
        let reservation = reserve(
            &mut conn,
            user_id,
            &reservation_input(operating_room.id, patient_id, encounter_id),
        )
        .unwrap();
        let overlapping = CreateOrReservationInput {
            scheduled_start: "2026-01-01T09:00:00".to_string(),
            scheduled_end: "2026-01-01T11:00:00".to_string(),
            ..reservation_input(operating_room.id, patient_id, encounter_id)
        };
        let _ = reserve(&mut conn, user_id, &overlapping);
        cancel_reservation(
            &mut conn,
            user_id,
            &CancelOrReservationInput { id: reservation.id },
        )
        .unwrap();

        let verification = audit_service::verify_chain(&conn).unwrap();
        assert!(verification.is_valid);
        assert!(audit_repository::list_all_ordered(&conn).unwrap().len() >= 3);
    }
}
