//! Cross-service integration tests for the Patients module (Rule 15.2), against a real tempfile
//! SQLCipher database. `commands::patient_commands` is intentionally not exercised here for the
//! same reason `auth_audit_integration.rs` doesn't exercise `commands::auth_commands` — it is a
//! documented thin adapter with no branching logic, and requires a live Tauri runtime.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use health_project::db::{connection, migrator};
use health_project::errors::AppError;
use health_project::services::{audit_service, patient_service};
use health_project::validation::patient_validation::{
    CreatePatientInput, ListPatientsInput, UpdatePatientInput,
};
use rusqlite::Connection;
use tempfile::tempdir;

const TEST_KEY: [u8; 32] = [13u8; 32];
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

fn create_input() -> CreatePatientInput {
    CreatePatientInput {
        medical_record_number: "MRN-1000".to_string(),
        full_name: "Ada Lovelace".to_string(),
        date_of_birth: "1990-01-01".to_string(),
        sex: "female".to_string(),
        national_id: None,
        phone: None,
        address: None,
        emergency_contact_name: None,
        emergency_contact_phone: None,
        blood_type: None,
        allergies: None,
    }
}

#[test]
fn create_succeeds_and_is_audited() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "create.sqlite");

    let patient = patient_service::create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();

    assert_eq!(patient.medical_record_number, "MRN-1000");
    let rows = health_project::repositories::audit_repository::list_all_ordered(&conn).unwrap();
    assert!(rows.iter().any(|row| row.action == "patient.create"));
}

#[test]
fn create_with_a_duplicate_medical_record_number_returns_conflict() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "duplicate-mrn.sqlite");
    patient_service::create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();

    let result = patient_service::create(&mut conn, ACTOR_USER_ID, &create_input());

    assert!(matches!(result, Err(AppError::Conflict { .. })));
}

#[test]
fn create_with_a_future_date_of_birth_returns_validation_error() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "future-dob.sqlite");
    let input = CreatePatientInput {
        date_of_birth: "2999-01-01".to_string(),
        ..create_input()
    };

    let result = patient_service::create(&mut conn, ACTOR_USER_ID, &input);

    assert!(matches!(result, Err(AppError::Validation { .. })));
}

#[test]
fn create_with_a_missing_full_name_returns_validation_error() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "missing-name.sqlite");
    let input = CreatePatientInput {
        full_name: String::new(),
        ..create_input()
    };

    let result = patient_service::create(&mut conn, ACTOR_USER_ID, &input);

    assert!(matches!(result, Err(AppError::Validation { .. })));
}

#[test]
fn update_succeeds_and_is_audited() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "update.sqlite");
    let created = patient_service::create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();

    let update_input = UpdatePatientInput {
        id: created.id,
        full_name: "Ada Byron".to_string(),
        medical_record_number: created.medical_record_number.clone(),
        date_of_birth: created.date_of_birth.clone(),
        sex: created.sex.clone(),
        national_id: None,
        phone: None,
        address: None,
        emergency_contact_name: None,
        emergency_contact_phone: None,
        blood_type: None,
        allergies: None,
    };
    let updated = patient_service::update(&mut conn, ACTOR_USER_ID, &update_input).unwrap();

    assert_eq!(updated.full_name, "Ada Byron");
    let rows = health_project::repositories::audit_repository::list_all_ordered(&conn).unwrap();
    assert!(rows.iter().any(|row| row.action == "patient.update"));
}

#[test]
fn update_of_a_nonexistent_patient_returns_not_found() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "update-missing.sqlite");

    let update_input = UpdatePatientInput {
        id: 999,
        full_name: "Nobody".to_string(),
        medical_record_number: "MRN-9999".to_string(),
        date_of_birth: "1990-01-01".to_string(),
        sex: "unknown".to_string(),
        national_id: None,
        phone: None,
        address: None,
        emergency_contact_name: None,
        emergency_contact_phone: None,
        blood_type: None,
        allergies: None,
    };
    let result = patient_service::update(&mut conn, ACTOR_USER_ID, &update_input);

    assert!(matches!(result, Err(AppError::NotFound { .. })));
}

#[test]
fn get_returns_a_previously_created_patient() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "get.sqlite");
    let created = patient_service::create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();

    let fetched = patient_service::get(&conn, created.id).unwrap();

    assert_eq!(fetched.id, created.id);
}

#[test]
fn get_of_a_nonexistent_patient_returns_not_found() {
    let dir = tempdir().unwrap();
    let conn = migrated_connection(dir.path(), "get-missing.sqlite");

    let result = patient_service::get(&conn, 999);

    assert!(matches!(result, Err(AppError::NotFound { .. })));
}

#[test]
fn list_filters_by_search_term() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "list-search.sqlite");
    patient_service::create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();
    patient_service::create(
        &mut conn,
        ACTOR_USER_ID,
        &CreatePatientInput {
            medical_record_number: "MRN-1001".to_string(),
            full_name: "Grace Hopper".to_string(),
            ..create_input()
        },
    )
    .unwrap();

    let results = patient_service::list(
        &conn,
        &ListPatientsInput {
            search: Some("Hopper".to_string()),
            limit: 10,
            offset: 0,
        },
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].full_name, "Grace Hopper");
}

#[test]
fn list_respects_limit_and_offset() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "list-page.sqlite");
    for i in 0..3 {
        patient_service::create(
            &mut conn,
            ACTOR_USER_ID,
            &CreatePatientInput {
                medical_record_number: format!("MRN-200{i}"),
                full_name: format!("Patient {i}"),
                ..create_input()
            },
        )
        .unwrap();
    }

    let page = patient_service::list(
        &conn,
        &ListPatientsInput {
            search: None,
            limit: 1,
            offset: 1,
        },
    )
    .unwrap();

    assert_eq!(page.len(), 1);
}

/// Create, then update, then a duplicate-conflict create attempt, then a failed validation
/// create attempt — the audit chain (which only records the successful writes) must still
/// verify as valid throughout.
#[test]
fn the_audit_chain_stays_valid_after_a_sequence_of_patient_writes() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "audit-chain.sqlite");

    let created = patient_service::create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();
    patient_service::update(
        &mut conn,
        ACTOR_USER_ID,
        &UpdatePatientInput {
            id: created.id,
            full_name: "Ada Byron".to_string(),
            medical_record_number: created.medical_record_number.clone(),
            date_of_birth: created.date_of_birth.clone(),
            sex: created.sex.clone(),
            national_id: None,
            phone: None,
            address: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            blood_type: None,
            allergies: None,
        },
    )
    .unwrap();
    let _ = patient_service::create(&mut conn, ACTOR_USER_ID, &create_input());
    let _ = patient_service::create(
        &mut conn,
        ACTOR_USER_ID,
        &CreatePatientInput {
            full_name: String::new(),
            ..create_input()
        },
    );

    let verification = audit_service::verify_chain(&conn).unwrap();
    assert!(
        verification.is_valid,
        "the hash chain must still verify after a create, an update, a conflict, and a \
         validation failure"
    );
}
