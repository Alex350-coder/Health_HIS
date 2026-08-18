//! Cross-service integration tests for Beds facility configuration and the Hospital Map read
//! path (Rule 15.2), against a real tempfile SQLCipher database.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use health_project::db::{connection, migrator};
use health_project::errors::AppError;
use health_project::repositories::bed_repository;
use health_project::services::{audit_service, bed_service};
use health_project::validation::bed_validation::{
    CreateBedInput, CreateFloorInput, CreateRoomInput, SetBedStatusInput,
};
use rusqlite::Connection;
use tempfile::tempdir;

const TEST_KEY: [u8; 32] = [17u8; 32];
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

#[test]
fn a_facility_layout_can_be_bootstrapped_from_an_empty_database() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "bootstrap.sqlite");

    let floor = bed_service::create_floor(
        &mut conn,
        ACTOR_USER_ID,
        &CreateFloorInput {
            name: "Ground Floor".to_string(),
            level_order: 0,
        },
    )
    .unwrap();
    let room = bed_service::create_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateRoomInput {
            floor_id: floor.id,
            name: "Ward A".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.25,
            map_y: 0.5,
        },
    )
    .unwrap();
    let bed = bed_service::create_bed(
        &mut conn,
        ACTOR_USER_ID,
        &CreateBedInput {
            room_id: room.id,
            label: "Bed 1A".to_string(),
        },
    )
    .unwrap();

    let layout = bed_repository::list_floors_with_rooms(&conn).unwrap();
    assert_eq!(layout.len(), 1);
    assert_eq!(layout[0].rooms.len(), 1);
    assert_eq!(layout[0].rooms[0].id, room.id);

    let status = bed_repository::get_room_status(&conn, room.id)
        .unwrap()
        .unwrap();
    assert_eq!(status.beds.len(), 1);
    assert_eq!(status.beds[0].id, bed.id);
    assert_eq!(status.available_count, 1);
}

#[test]
fn set_status_transitions_a_bed_to_maintenance_and_is_audited() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "set-status.sqlite");
    let floor = bed_service::create_floor(
        &mut conn,
        ACTOR_USER_ID,
        &CreateFloorInput {
            name: "Ground Floor".to_string(),
            level_order: 0,
        },
    )
    .unwrap();
    let room = bed_service::create_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateRoomInput {
            floor_id: floor.id,
            name: "Ward A".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.25,
            map_y: 0.5,
        },
    )
    .unwrap();
    let bed = bed_service::create_bed(
        &mut conn,
        ACTOR_USER_ID,
        &CreateBedInput {
            room_id: room.id,
            label: "Bed 1A".to_string(),
        },
    )
    .unwrap();

    let updated = bed_service::set_status(
        &mut conn,
        ACTOR_USER_ID,
        &SetBedStatusInput {
            bed_id: bed.id,
            status: "maintenance".to_string(),
        },
    )
    .unwrap();

    assert_eq!(updated.status, "maintenance");
    let status = bed_repository::get_room_status(&conn, room.id)
        .unwrap()
        .unwrap();
    assert_eq!(status.maintenance_count, 1);
    assert_eq!(status.available_count, 0);

    let rows = health_project::repositories::audit_repository::list_all_ordered(&conn).unwrap();
    assert!(rows.iter().any(|row| row.action == "bed.status_change"));
}

#[test]
fn create_room_against_a_nonexistent_floor_returns_not_found() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "room-missing-floor.sqlite");

    let result = bed_service::create_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateRoomInput {
            floor_id: 999,
            name: "Ward A".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.25,
            map_y: 0.5,
        },
    );

    assert!(matches!(result, Err(AppError::NotFound { .. })));
}

#[test]
fn the_audit_chain_stays_valid_after_a_sequence_of_facility_writes() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "audit-chain.sqlite");

    let floor = bed_service::create_floor(
        &mut conn,
        ACTOR_USER_ID,
        &CreateFloorInput {
            name: "Ground Floor".to_string(),
            level_order: 0,
        },
    )
    .unwrap();
    let room = bed_service::create_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateRoomInput {
            floor_id: floor.id,
            name: "Ward A".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.25,
            map_y: 0.5,
        },
    )
    .unwrap();
    let bed = bed_service::create_bed(
        &mut conn,
        ACTOR_USER_ID,
        &CreateBedInput {
            room_id: room.id,
            label: "Bed 1A".to_string(),
        },
    )
    .unwrap();
    bed_service::set_status(
        &mut conn,
        ACTOR_USER_ID,
        &SetBedStatusInput {
            bed_id: bed.id,
            status: "maintenance".to_string(),
        },
    )
    .unwrap();
    let _ = bed_service::create_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateRoomInput {
            floor_id: 999,
            name: "Ward B".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.25,
            map_y: 0.5,
        },
    );

    let verification = audit_service::verify_chain(&conn).unwrap();
    assert!(
        verification.is_valid,
        "the hash chain must still verify after a sequence of facility writes and a NotFound \
         failure"
    );
}
