//! Cross-service integration tests for Operating Rooms: room promotion, reservation
//! scheduling, and the overlap-prevention rule against a real tempfile SQLCipher database
//! (Rule 15.2).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use health_project::db::{connection, migrator};
use health_project::errors::AppError;
use health_project::services::{audit_service, operating_room_service};
use health_project::validation::operating_room_validation::{
    CancelOrReservationInput, CreateOperatingRoomInput, CreateOrReservationInput,
    UpdateOrReservationInput,
};
use rusqlite::Connection;
use tempfile::tempdir;

const TEST_KEY: [u8; 32] = [23u8; 32];
const ACTOR_USER_ID: i64 = 1;

fn migrated_connection(dir: &std::path::Path, name: &str) -> Connection {
    let conn = connection::open(&dir.join(name), &TEST_KEY).unwrap();
    migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
    conn.execute(
        "INSERT INTO users (full_name, username, password_hash, role) \
         VALUES ('Test Admin', 'test-admin', 'argon2id$dummy', 'admin')",
        [],
    )
    .unwrap();
    conn
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

#[test]
fn a_room_can_be_promoted_reserved_updated_and_cancelled() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "full-flow.sqlite");
    let room_id = seed_room(&conn, "operating_room");
    let patient_id = seed_patient(&conn);
    let encounter_id = seed_encounter(&conn, patient_id, ACTOR_USER_ID);

    let operating_room = operating_room_service::create_operating_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOperatingRoomInput { room_id },
    )
    .unwrap();

    let reservation = operating_room_service::reserve(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOrReservationInput {
            operating_room_id: operating_room.id,
            patient_id,
            encounter_id,
            procedure_description: "Appendectomy".to_string(),
            scheduled_start: "2026-01-01T08:00:00".to_string(),
            scheduled_end: "2026-01-01T10:00:00".to_string(),
        },
    )
    .unwrap();
    assert_eq!(reservation.status, "scheduled");

    let updated = operating_room_service::update_reservation(
        &mut conn,
        ACTOR_USER_ID,
        &UpdateOrReservationInput {
            id: reservation.id,
            procedure_description: "Appendectomy — revised".to_string(),
            scheduled_start: "2026-01-01T09:00:00".to_string(),
            scheduled_end: "2026-01-01T11:00:00".to_string(),
        },
    )
    .unwrap();
    assert_eq!(updated.procedure_description, "Appendectomy — revised");

    let cancelled = operating_room_service::cancel_reservation(
        &mut conn,
        ACTOR_USER_ID,
        &CancelOrReservationInput { id: updated.id },
    )
    .unwrap();
    assert_eq!(cancelled.status, "cancelled");
}

#[test]
fn reserving_an_overlapping_time_returns_conflict() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "overlap.sqlite");
    let room_id = seed_room(&conn, "operating_room");
    let patient_id = seed_patient(&conn);
    let encounter_id = seed_encounter(&conn, patient_id, ACTOR_USER_ID);
    let operating_room = operating_room_service::create_operating_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOperatingRoomInput { room_id },
    )
    .unwrap();
    operating_room_service::reserve(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOrReservationInput {
            operating_room_id: operating_room.id,
            patient_id,
            encounter_id,
            procedure_description: "Appendectomy".to_string(),
            scheduled_start: "2026-01-01T08:00:00".to_string(),
            scheduled_end: "2026-01-01T10:00:00".to_string(),
        },
    )
    .unwrap();

    let result = operating_room_service::reserve(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOrReservationInput {
            operating_room_id: operating_room.id,
            patient_id,
            encounter_id,
            procedure_description: "Cholecystectomy".to_string(),
            scheduled_start: "2026-01-01T09:00:00".to_string(),
            scheduled_end: "2026-01-01T11:00:00".to_string(),
        },
    );

    assert!(matches!(result, Err(AppError::Conflict { .. })));
}

#[test]
fn the_audit_chain_stays_valid_after_a_reserve_update_and_cancel_sequence() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "audit-chain.sqlite");
    let room_id = seed_room(&conn, "operating_room");
    let patient_id = seed_patient(&conn);
    let encounter_id = seed_encounter(&conn, patient_id, ACTOR_USER_ID);
    let operating_room = operating_room_service::create_operating_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOperatingRoomInput { room_id },
    )
    .unwrap();
    let reservation = operating_room_service::reserve(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOrReservationInput {
            operating_room_id: operating_room.id,
            patient_id,
            encounter_id,
            procedure_description: "Appendectomy".to_string(),
            scheduled_start: "2026-01-01T08:00:00".to_string(),
            scheduled_end: "2026-01-01T10:00:00".to_string(),
        },
    )
    .unwrap();
    let _ = operating_room_service::reserve(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOrReservationInput {
            operating_room_id: operating_room.id,
            patient_id,
            encounter_id,
            procedure_description: "Cholecystectomy".to_string(),
            scheduled_start: "2026-01-01T09:00:00".to_string(),
            scheduled_end: "2026-01-01T11:00:00".to_string(),
        },
    );
    operating_room_service::cancel_reservation(
        &mut conn,
        ACTOR_USER_ID,
        &CancelOrReservationInput { id: reservation.id },
    )
    .unwrap();

    let verification = audit_service::verify_chain(&conn).unwrap();
    assert!(
        verification.is_valid,
        "the hash chain must still verify after a sequence of OR writes and a rejected \
         overlapping reserve"
    );
}
