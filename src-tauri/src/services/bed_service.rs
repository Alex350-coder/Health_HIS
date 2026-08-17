//! Beds facility-configuration orchestration (Database.md Section 3.3, IPC.md Section 2.1).
//! Beds owns the write path for `floors`/`rooms`/`beds` even though the Hospital Map visualizes
//! the same tables (Architecture.md Section 3 — Hospital Map never writes, has no service here).
//! Thin command layer calls into this module only.

use rusqlite::Connection;

use crate::db::DbError;
use crate::errors::app_error::correlation_id;
use crate::errors::AppError;
use crate::models::{Bed, Floor, Room};
use crate::repositories::bed_repository::{self, NewBed, NewFloor, NewRoom};
use crate::services::audit_service::{self, RecordInput};
use crate::validation::bed_validation::{
    self, CreateBedInput, CreateFloorInput, CreateRoomInput, SetBedStatusInput,
};

pub fn create_floor(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateFloorInput,
) -> Result<Floor, AppError> {
    bed_validation::validate_create_floor(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let floor_id = bed_repository::insert_floor(
        &tx,
        &NewFloor {
            name: &input.name,
            level_order: input.level_order,
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "floor.create",
            entity_type: "floor",
            entity_id: Some(floor_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_floor_or_die(conn, floor_id)
}

pub fn create_room(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateRoomInput,
) -> Result<Room, AppError> {
    bed_validation::validate_create_room(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    require_floor_exists(&tx, input.floor_id)?;
    let room_id = bed_repository::insert_room(
        &tx,
        &NewRoom {
            floor_id: input.floor_id,
            name: &input.name,
            room_type: &input.room_type,
            map_x: input.map_x,
            map_y: input.map_y,
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "room.create",
            entity_type: "room",
            entity_id: Some(room_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_room_or_die(conn, room_id)
}

pub fn create_bed(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateBedInput,
) -> Result<Bed, AppError> {
    bed_validation::validate_create_bed(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    require_room_exists(&tx, input.room_id)?;
    let bed_id = bed_repository::insert_bed(
        &tx,
        &NewBed {
            room_id: input.room_id,
            label: &input.label,
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "bed.create",
            entity_type: "bed",
            entity_id: Some(bed_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_bed_or_die(conn, bed_id)
}

pub fn set_status(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &SetBedStatusInput,
) -> Result<Bed, AppError> {
    bed_validation::validate_set_bed_status(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let bed = bed_repository::update_bed_status(&tx, input.bed_id, &input.status)?.ok_or(
        AppError::NotFound {
            entity: "bed".to_string(),
            id: input.bed_id,
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "bed.status_change",
            entity_type: "bed",
            entity_id: Some(input.bed_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    Ok(bed)
}

fn require_floor_exists(conn: &Connection, floor_id: i64) -> Result<(), AppError> {
    bed_repository::find_floor_by_id(conn, floor_id)?.ok_or(AppError::NotFound {
        entity: "floor".to_string(),
        id: floor_id,
    })?;
    Ok(())
}

fn require_room_exists(conn: &Connection, room_id: i64) -> Result<(), AppError> {
    bed_repository::find_room_by_id(conn, room_id)?.ok_or(AppError::NotFound {
        entity: "room".to_string(),
        id: room_id,
    })?;
    Ok(())
}

/// A floor just written in this same connection/transaction cannot legitimately be missing;
/// treated as a technical invariant violation rather than a `NotFound`.
fn find_floor_or_die(conn: &Connection, id: i64) -> Result<Floor, AppError> {
    bed_repository::find_floor_by_id(conn, id)?.ok_or_else(|| unexpected_vanished("floor", id))
}

fn find_room_or_die(conn: &Connection, id: i64) -> Result<Room, AppError> {
    bed_repository::find_room_by_id(conn, id)?.ok_or_else(|| unexpected_vanished("room", id))
}

fn find_bed_or_die(conn: &Connection, id: i64) -> Result<Bed, AppError> {
    bed_repository::find_bed_by_id(conn, id)?.ok_or_else(|| unexpected_vanished("bed", id))
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

    const TEST_KEY: [u8; 32] = [11u8; 32];
    const ACTOR_USER_ID: i64 = 1;

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("bed-service-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn.execute(
            "INSERT INTO users (full_name, username, password_hash, role) \
             VALUES ('Test Admin', 'test-admin', 'argon2id$dummy', 'admin')",
            [],
        )
        .unwrap();
        conn
    }

    fn create_floor_input() -> CreateFloorInput {
        CreateFloorInput {
            name: "Ground Floor".to_string(),
            level_order: 0,
        }
    }

    fn create_room_input(floor_id: i64) -> CreateRoomInput {
        CreateRoomInput {
            floor_id,
            name: "Ward A".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.5,
            map_y: 0.5,
        }
    }

    fn create_bed_input(room_id: i64) -> CreateBedInput {
        CreateBedInput {
            room_id,
            label: "Bed 3A".to_string(),
        }
    }

    #[test]
    fn create_floor_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let floor = create_floor(&mut conn, ACTOR_USER_ID, &create_floor_input()).unwrap();

        assert_eq!(floor.name, "Ground Floor");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "floor.create"));
    }

    #[test]
    fn create_room_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let floor = create_floor(&mut conn, ACTOR_USER_ID, &create_floor_input()).unwrap();

        let room = create_room(&mut conn, ACTOR_USER_ID, &create_room_input(floor.id)).unwrap();

        assert_eq!(room.floor_id, floor.id);
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "room.create"));
    }

    #[test]
    fn create_room_against_a_nonexistent_floor_returns_not_found() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let result = create_room(&mut conn, ACTOR_USER_ID, &create_room_input(999));

        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn create_bed_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let floor = create_floor(&mut conn, ACTOR_USER_ID, &create_floor_input()).unwrap();
        let room = create_room(&mut conn, ACTOR_USER_ID, &create_room_input(floor.id)).unwrap();

        let bed = create_bed(&mut conn, ACTOR_USER_ID, &create_bed_input(room.id)).unwrap();

        assert_eq!(bed.status, "available");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "bed.create"));
    }

    #[test]
    fn create_bed_against_a_nonexistent_room_returns_not_found() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let result = create_bed(&mut conn, ACTOR_USER_ID, &create_bed_input(999));

        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn set_status_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let floor = create_floor(&mut conn, ACTOR_USER_ID, &create_floor_input()).unwrap();
        let room = create_room(&mut conn, ACTOR_USER_ID, &create_room_input(floor.id)).unwrap();
        let bed = create_bed(&mut conn, ACTOR_USER_ID, &create_bed_input(room.id)).unwrap();

        let updated = set_status(
            &mut conn,
            ACTOR_USER_ID,
            &SetBedStatusInput {
                bed_id: bed.id,
                status: "maintenance".to_string(),
            },
        )
        .unwrap();

        assert_eq!(updated.status, "maintenance");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "bed.status_change"));
    }

    #[test]
    fn set_status_of_a_nonexistent_bed_returns_not_found() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let result = set_status(
            &mut conn,
            ACTOR_USER_ID,
            &SetBedStatusInput {
                bed_id: 999,
                status: "maintenance".to_string(),
            },
        );

        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn the_audit_chain_stays_valid_after_a_sequence_of_facility_writes() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let floor = create_floor(&mut conn, ACTOR_USER_ID, &create_floor_input()).unwrap();
        let room = create_room(&mut conn, ACTOR_USER_ID, &create_room_input(floor.id)).unwrap();
        let bed = create_bed(&mut conn, ACTOR_USER_ID, &create_bed_input(room.id)).unwrap();
        set_status(
            &mut conn,
            ACTOR_USER_ID,
            &SetBedStatusInput {
                bed_id: bed.id,
                status: "maintenance".to_string(),
            },
        )
        .unwrap();

        let verification = audit_service::verify_chain(&conn).unwrap();
        assert!(verification.is_valid);
    }
}
