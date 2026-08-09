//! Patient orchestration (Database.md Section 3.2, CLAUDE.md Section 7 — root entity of the
//! clinical workflow). Thin command layer calls into this module only.

use rusqlite::{Connection, ErrorCode};

use crate::db::DbError;
use crate::errors::app_error::correlation_id;
use crate::errors::AppError;
use crate::models::Patient;
use crate::repositories::patient_repository::{self, ListFilter, NewPatient, UpdatePatient};
use crate::services::audit_service::{self, RecordInput};
use crate::validation::patient_validation::{
    self, CreatePatientInput, ListPatientsInput, UpdatePatientInput,
};

pub fn create(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreatePatientInput,
) -> Result<Patient, AppError> {
    patient_validation::validate_create_patient(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let patient_id = patient_repository::insert(&tx, &new_patient(input))
        .map_err(|error| map_constraint_violation(error, "medical_record_number, national_id"))?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "patient.create",
            entity_type: "patient",
            entity_id: Some(patient_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_patient_or_die(conn, patient_id)
}

pub fn update(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &UpdatePatientInput,
) -> Result<Patient, AppError> {
    patient_validation::validate_update_patient(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let patient = patient_repository::update(&tx, input.id, &update_patient(input))
        .map_err(|error| map_constraint_violation(error, "medical_record_number, national_id"))?
        .ok_or(AppError::NotFound {
            entity: "patient".to_string(),
            id: input.id,
        })?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "patient.update",
            entity_type: "patient",
            entity_id: Some(input.id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    Ok(patient)
}

pub fn get(conn: &Connection, id: i64) -> Result<Patient, AppError> {
    patient_repository::find_by_id(conn, id)?.ok_or(AppError::NotFound {
        entity: "patient".to_string(),
        id,
    })
}

pub fn list(conn: &Connection, input: &ListPatientsInput) -> Result<Vec<Patient>, AppError> {
    let like_pattern = input.search.as_ref().map(|term| format!("%{term}%"));
    patient_repository::list(
        conn,
        &ListFilter {
            search: like_pattern.as_deref(),
            limit: input.limit,
            offset: input.offset,
        },
    )
    .map_err(AppError::from)
}

fn new_patient(input: &CreatePatientInput) -> NewPatient<'_> {
    NewPatient {
        medical_record_number: &input.medical_record_number,
        full_name: &input.full_name,
        date_of_birth: &input.date_of_birth,
        sex: &input.sex,
        national_id: input.national_id.as_deref(),
        phone: input.phone.as_deref(),
        address: input.address.as_deref(),
        emergency_contact_name: input.emergency_contact_name.as_deref(),
        emergency_contact_phone: input.emergency_contact_phone.as_deref(),
        blood_type: input.blood_type.as_deref(),
        allergies: input.allergies.as_deref(),
    }
}

fn update_patient(input: &UpdatePatientInput) -> UpdatePatient<'_> {
    UpdatePatient {
        medical_record_number: &input.medical_record_number,
        full_name: &input.full_name,
        date_of_birth: &input.date_of_birth,
        sex: &input.sex,
        national_id: input.national_id.as_deref(),
        phone: input.phone.as_deref(),
        address: input.address.as_deref(),
        emergency_contact_name: input.emergency_contact_name.as_deref(),
        emergency_contact_phone: input.emergency_contact_phone.as_deref(),
        blood_type: input.blood_type.as_deref(),
        allergies: input.allergies.as_deref(),
    }
}

/// Maps a SQLite `UNIQUE` constraint violation (`medical_record_number`/`national_id`) to a
/// user-actionable `AppError::Conflict`; every other `DbError` falls through to the generic
/// `AppError::Database` mapping (Decision 4).
fn map_constraint_violation(error: DbError, unique_columns: &str) -> AppError {
    if let DbError::Sqlite(rusqlite::Error::SqliteFailure(sqlite_error, Some(message))) = &error {
        if sqlite_error.code == ErrorCode::ConstraintViolation {
            let field = if message.contains("national_id") {
                "national_id"
            } else if message.contains("medical_record_number") {
                "medical_record_number"
            } else {
                unique_columns
            };
            return AppError::Conflict {
                message: format!("a patient with this {field} already exists"),
            };
        }
    }
    AppError::from(error)
}

/// A patient just written in this same connection/transaction cannot legitimately be missing;
/// treated as a technical invariant violation rather than a `NotFound`.
fn find_patient_or_die(conn: &Connection, id: i64) -> Result<Patient, AppError> {
    patient_repository::find_by_id(conn, id)?.ok_or_else(|| {
        let correlation_id = correlation_id();
        tracing::error!(
            correlation_id,
            id,
            "patient row vanished immediately after being written"
        );
        AppError::Unexpected {
            message: "an unexpected error occurred".to_string(),
            correlation_id,
        }
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};
    use crate::repositories::audit_repository;

    const TEST_KEY: [u8; 32] = [9u8; 32];
    const ACTOR_USER_ID: i64 = 1;

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("patient-service-test.sqlite"), &TEST_KEY).unwrap();
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
            medical_record_number: "MRN-0001".to_string(),
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
    fn create_persists_a_patient_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let patient = create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();

        assert_eq!(patient.medical_record_number, "MRN-0001");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "patient.create"));
    }

    #[test]
    fn create_rejects_a_duplicate_medical_record_number_as_conflict() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();

        let result = create(&mut conn, ACTOR_USER_ID, &create_input());

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn create_rejects_a_future_date_of_birth_as_validation_error() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let input = CreatePatientInput {
            date_of_birth: "2999-01-01".to_string(),
            ..create_input()
        };

        let result = create(&mut conn, ACTOR_USER_ID, &input);

        assert!(matches!(result, Err(AppError::Validation { .. })));
    }

    #[test]
    fn update_persists_changes_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let created = create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();

        let update_input = UpdatePatientInput {
            id: created.id,
            medical_record_number: created.medical_record_number.clone(),
            full_name: "Ada Byron".to_string(),
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
        let updated = update(&mut conn, ACTOR_USER_ID, &update_input).unwrap();

        assert_eq!(updated.full_name, "Ada Byron");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "patient.update"));
    }

    #[test]
    fn update_of_a_nonexistent_patient_returns_not_found() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let update_input = UpdatePatientInput {
            id: 999,
            medical_record_number: "MRN-9999".to_string(),
            full_name: "Nobody".to_string(),
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
        let result = update(&mut conn, ACTOR_USER_ID, &update_input);

        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn get_returns_not_found_for_a_missing_id() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let result = get(&conn, 999);
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn list_filters_by_search_term() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        create(&mut conn, ACTOR_USER_ID, &create_input()).unwrap();
        create(
            &mut conn,
            ACTOR_USER_ID,
            &CreatePatientInput {
                medical_record_number: "MRN-0002".to_string(),
                full_name: "Grace Hopper".to_string(),
                ..create_input()
            },
        )
        .unwrap();

        let results = list(
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
}
