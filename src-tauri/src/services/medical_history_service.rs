//! Medical History orchestration (Database.md Section 3.2, Rule 9.5 — encounters, diagnoses,
//! treatments, evolutions are append-only). Thin command layer calls into this module only.
//!
//! Business rule enforced here (not expressible as a plain SQL constraint): diagnoses,
//! treatments, and evolutions may only be added to an **open** encounter, and an encounter can
//! only be discharged once. Both checks happen inside the same transaction as the write.
//!
//! `create_treatment` optionally links a treatment to inventory consumption: when
//! `CreateTreatmentInput.inventory_item_id`/`quantity` are set, it calls
//! `inventory_service::record_transaction_core` inside its own transaction so the treatment
//! insert and the inventory consumption commit or roll back together — insufficient stock
//! (`AppError::Conflict`) rolls back the treatment insert too.

use rusqlite::{Connection, ErrorCode};
use serde::Serialize;

use crate::db::DbError;
use crate::errors::app_error::correlation_id;
use crate::errors::AppError;
use crate::models::{Diagnosis, Encounter, Evolution, Treatment};
use crate::repositories::encounter_repository::{
    self, DischargeEncounter, NewDiagnosis, NewEncounter, NewEvolution, NewTreatment,
};
use crate::services::audit_service::{self, RecordInput};
use crate::services::inventory_service;
use crate::validation::medical_history_validation::{
    self, CreateDiagnosisInput, CreateEncounterInput, CreateEvolutionInput, CreateTreatmentInput,
    DischargeEncounterInput,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicalHistoryBundle {
    pub encounters: Vec<Encounter>,
    pub diagnoses: Vec<Diagnosis>,
    pub treatments: Vec<Treatment>,
    pub evolutions: Vec<Evolution>,
}

pub fn create_encounter(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateEncounterInput,
) -> Result<Encounter, AppError> {
    medical_history_validation::validate_create_encounter(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let encounter_id = encounter_repository::insert_encounter(
        &tx,
        &NewEncounter {
            patient_id: input.patient_id,
            created_by_user_id: actor_user_id,
        },
    )
    .map_err(map_constraint_violation)?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "encounter.create",
            entity_type: "encounter",
            entity_id: Some(encounter_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_encounter_or_die(conn, encounter_id)
}

pub fn discharge_encounter(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &DischargeEncounterInput,
) -> Result<Encounter, AppError> {
    medical_history_validation::validate_discharge_encounter(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    require_open_encounter(&tx, input.encounter_id)?;
    let encounter = encounter_repository::discharge_encounter(
        &tx,
        input.encounter_id,
        &DischargeEncounter {
            discharge_summary: input.discharge_summary.as_deref(),
        },
    )
    .map_err(AppError::from)?
    .ok_or(AppError::NotFound {
        entity: "encounter".to_string(),
        id: input.encounter_id,
    })?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "encounter.discharge",
            entity_type: "encounter",
            entity_id: Some(input.encounter_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    Ok(encounter)
}

pub fn get_by_patient(
    conn: &Connection,
    patient_id: i64,
) -> Result<MedicalHistoryBundle, AppError> {
    let encounters = encounter_repository::find_encounters_by_patient(conn, patient_id)?;

    let mut diagnoses = Vec::new();
    let mut treatments = Vec::new();
    let mut evolutions = Vec::new();
    for encounter in &encounters {
        diagnoses.extend(encounter_repository::find_diagnoses_by_encounter(
            conn,
            encounter.id,
        )?);
        treatments.extend(encounter_repository::find_treatments_by_encounter(
            conn,
            encounter.id,
        )?);
        evolutions.extend(encounter_repository::find_evolutions_by_encounter(
            conn,
            encounter.id,
        )?);
    }

    Ok(MedicalHistoryBundle {
        encounters,
        diagnoses,
        treatments,
        evolutions,
    })
}

/// Read-only helper for Billing's treatment-charge aggregation (Plan.md Phase 12) — every
/// treatment recorded for one encounter.
pub fn list_treatments_for_encounter(
    conn: &Connection,
    encounter_id: i64,
) -> Result<Vec<Treatment>, AppError> {
    Ok(encounter_repository::find_treatments_by_encounter(
        conn,
        encounter_id,
    )?)
}

pub fn create_diagnosis(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateDiagnosisInput,
) -> Result<Diagnosis, AppError> {
    medical_history_validation::validate_create_diagnosis(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    require_open_encounter(&tx, input.encounter_id)?;
    let diagnosis_id = encounter_repository::insert_diagnosis(
        &tx,
        &NewDiagnosis {
            encounter_id: input.encounter_id,
            description: &input.description,
            icd_code: input.icd_code.as_deref(),
            registered_by_user_id: actor_user_id,
            corrects_diagnosis_id: input.corrects_diagnosis_id,
        },
    )
    .map_err(map_constraint_violation)?;
    let action = if input.corrects_diagnosis_id.is_some() {
        "diagnosis.correct"
    } else {
        "diagnosis.create"
    };
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action,
            entity_type: "diagnosis",
            entity_id: Some(diagnosis_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_diagnosis_or_die(conn, diagnosis_id)
}

pub fn create_treatment(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateTreatmentInput,
) -> Result<Treatment, AppError> {
    medical_history_validation::validate_create_treatment(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    require_open_encounter(&tx, input.encounter_id)?;
    let treatment_id = encounter_repository::insert_treatment(
        &tx,
        &NewTreatment {
            encounter_id: input.encounter_id,
            diagnosis_id: input.diagnosis_id,
            description: &input.description,
            dosage: input.dosage.as_deref(),
            registered_by_user_id: actor_user_id,
            corrects_treatment_id: input.corrects_treatment_id,
        },
    )
    .map_err(map_constraint_violation)?;
    let action = if input.corrects_treatment_id.is_some() {
        "treatment.correct"
    } else {
        "treatment.create"
    };
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action,
            entity_type: "treatment",
            entity_id: Some(treatment_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    if let (Some(item_id), Some(quantity)) = (input.inventory_item_id, input.quantity) {
        inventory_service::record_transaction_core(
            &tx,
            actor_user_id,
            item_id,
            -quantity,
            "consumption",
            Some(input.encounter_id),
            Some(treatment_id),
        )?;
    }
    tx.commit().map_err(DbError::from)?;

    find_treatment_or_die(conn, treatment_id)
}

pub fn create_evolution(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateEvolutionInput,
) -> Result<Evolution, AppError> {
    medical_history_validation::validate_create_evolution(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    require_open_encounter(&tx, input.encounter_id)?;
    let evolution_id = encounter_repository::insert_evolution(
        &tx,
        &NewEvolution {
            encounter_id: input.encounter_id,
            note: &input.note,
            registered_by_user_id: actor_user_id,
        },
    )
    .map_err(map_constraint_violation)?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "evolution.create",
            entity_type: "evolution",
            entity_id: Some(evolution_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_evolution_or_die(conn, evolution_id)
}

/// Diagnoses/treatments/evolutions may only be registered against, and encounters may only be
/// discharged from, an encounter that exists and is still `open`.
fn require_open_encounter(conn: &Connection, encounter_id: i64) -> Result<Encounter, AppError> {
    let encounter = encounter_repository::find_encounter_by_id(conn, encounter_id)?.ok_or(
        AppError::NotFound {
            entity: "encounter".to_string(),
            id: encounter_id,
        },
    )?;
    if encounter.status != "open" {
        return Err(AppError::Conflict {
            message: "the encounter is already discharged".to_string(),
        });
    }
    Ok(encounter)
}

/// Maps a SQLite `FOREIGN KEY`/`CHECK` constraint violation to a user-actionable
/// `AppError::Validation`; every other `DbError` falls through to the generic `AppError::Database`
/// mapping (Decision 4).
fn map_constraint_violation(error: DbError) -> AppError {
    if let DbError::Sqlite(rusqlite::Error::SqliteFailure(sqlite_error, Some(_))) = &error {
        if sqlite_error.code == ErrorCode::ConstraintViolation {
            return AppError::Validation {
                field: "reference".to_string(),
                message: "a referenced record does not exist".to_string(),
            };
        }
    }
    AppError::from(error)
}

fn find_encounter_or_die(conn: &Connection, id: i64) -> Result<Encounter, AppError> {
    encounter_repository::find_encounter_by_id(conn, id)?.ok_or_else(|| vanished("encounter", id))
}

fn find_diagnosis_or_die(conn: &Connection, id: i64) -> Result<Diagnosis, AppError> {
    encounter_repository::find_diagnosis_by_id(conn, id)?.ok_or_else(|| vanished("diagnosis", id))
}

fn find_treatment_or_die(conn: &Connection, id: i64) -> Result<Treatment, AppError> {
    encounter_repository::find_treatment_by_id(conn, id)?.ok_or_else(|| vanished("treatment", id))
}

fn find_evolution_or_die(conn: &Connection, id: i64) -> Result<Evolution, AppError> {
    encounter_repository::find_evolution_by_id(conn, id)?.ok_or_else(|| vanished("evolution", id))
}

/// A row just written in this same connection/transaction cannot legitimately be missing;
/// treated as a technical invariant violation rather than a `NotFound`.
fn vanished(entity: &'static str, id: i64) -> AppError {
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
        let conn =
            connection::open(&dir.join("medical-history-service-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn.execute(
            "INSERT INTO users (full_name, username, password_hash, role) \
             VALUES ('Test Admin', 'test-admin', 'argon2id$dummy', 'admin')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO patients (medical_record_number, full_name, date_of_birth, sex) \
             VALUES ('MRN-0001', 'Ada Lovelace', '1990-01-01', 'female')",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn create_encounter_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();

        assert_eq!(encounter.status, "open");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "encounter.create"));
    }

    #[test]
    fn discharge_encounter_closes_it_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();

        let discharged = discharge_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &DischargeEncounterInput {
                encounter_id: encounter.id,
                discharge_summary: Some("Recovered well.".to_string()),
            },
        )
        .unwrap();

        assert_eq!(discharged.status, "discharged");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "encounter.discharge"));
    }

    #[test]
    fn discharging_an_already_discharged_encounter_returns_conflict() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();
        discharge_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &DischargeEncounterInput {
                encounter_id: encounter.id,
                discharge_summary: None,
            },
        )
        .unwrap();

        let result = discharge_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &DischargeEncounterInput {
                encounter_id: encounter.id,
                discharge_summary: None,
            },
        );

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn create_diagnosis_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();

        let diagnosis = create_diagnosis(
            &mut conn,
            ACTOR_USER_ID,
            &CreateDiagnosisInput {
                encounter_id: encounter.id,
                description: "Acute appendicitis".to_string(),
                icd_code: Some("K35.80".to_string()),
                corrects_diagnosis_id: None,
            },
        )
        .unwrap();

        assert_eq!(diagnosis.description, "Acute appendicitis");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "diagnosis.create"));
    }

    #[test]
    fn a_correction_diagnosis_is_audited_as_diagnosis_correct() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();
        let original = create_diagnosis(
            &mut conn,
            ACTOR_USER_ID,
            &CreateDiagnosisInput {
                encounter_id: encounter.id,
                description: "Acute appendicitis".to_string(),
                icd_code: None,
                corrects_diagnosis_id: None,
            },
        )
        .unwrap();

        create_diagnosis(
            &mut conn,
            ACTOR_USER_ID,
            &CreateDiagnosisInput {
                encounter_id: encounter.id,
                description: "Acute cholecystitis".to_string(),
                icd_code: None,
                corrects_diagnosis_id: Some(original.id),
            },
        )
        .unwrap();

        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "diagnosis.correct"));
    }

    #[test]
    fn create_diagnosis_on_a_discharged_encounter_returns_conflict() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();
        discharge_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &DischargeEncounterInput {
                encounter_id: encounter.id,
                discharge_summary: None,
            },
        )
        .unwrap();

        let result = create_diagnosis(
            &mut conn,
            ACTOR_USER_ID,
            &CreateDiagnosisInput {
                encounter_id: encounter.id,
                description: "Late diagnosis".to_string(),
                icd_code: None,
                corrects_diagnosis_id: None,
            },
        );

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn create_diagnosis_on_a_nonexistent_encounter_returns_not_found() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let result = create_diagnosis(
            &mut conn,
            ACTOR_USER_ID,
            &CreateDiagnosisInput {
                encounter_id: 999,
                description: "Late diagnosis".to_string(),
                icd_code: None,
                corrects_diagnosis_id: None,
            },
        );

        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn create_treatment_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();

        let treatment = create_treatment(
            &mut conn,
            ACTOR_USER_ID,
            &CreateTreatmentInput {
                encounter_id: encounter.id,
                diagnosis_id: None,
                description: "Appendectomy".to_string(),
                dosage: None,
                corrects_treatment_id: None,
                inventory_item_id: None,
                quantity: None,
            },
        )
        .unwrap();

        assert_eq!(treatment.description, "Appendectomy");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "treatment.create"));
    }

    fn seed_inventory_item(conn: &mut Connection) -> i64 {
        let category = inventory_service::create_category(
            conn,
            ACTOR_USER_ID,
            &crate::validation::inventory_validation::CreateInventoryCategoryInput {
                name: "Analgesics".to_string(),
                kind: "medicine".to_string(),
            },
        )
        .unwrap();
        let item = inventory_service::create_item(
            conn,
            ACTOR_USER_ID,
            &crate::validation::inventory_validation::CreateInventoryItemInput {
                category_id: category.id,
                name: "Ibuprofen 400mg".to_string(),
                unit: "box".to_string(),
                reorder_threshold: 0,
                expiration_date: None,
                location: None,
            },
        )
        .unwrap();
        item.id
    }

    #[test]
    fn create_treatment_with_a_valid_inventory_linkage_consumes_stock() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let item_id = seed_inventory_item(&mut conn);
        inventory_service::record_transaction(
            &mut conn,
            ACTOR_USER_ID,
            &crate::validation::inventory_validation::CreateInventoryTransactionInput {
                item_id,
                quantity_delta: 10,
                reason: "restock".to_string(),
                encounter_id: None,
                treatment_id: None,
            },
        )
        .unwrap();
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();

        create_treatment(
            &mut conn,
            ACTOR_USER_ID,
            &CreateTreatmentInput {
                encounter_id: encounter.id,
                diagnosis_id: None,
                description: "Appendectomy".to_string(),
                dosage: None,
                corrects_treatment_id: None,
                inventory_item_id: Some(item_id),
                quantity: Some(3),
            },
        )
        .unwrap();

        let item = inventory_service::list_items(&conn, None, false).unwrap();
        assert_eq!(item[0].quantity, 7);
    }

    #[test]
    fn create_treatment_with_insufficient_stock_rolls_back_the_treatment_insert() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let item_id = seed_inventory_item(&mut conn);
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();

        let result = create_treatment(
            &mut conn,
            ACTOR_USER_ID,
            &CreateTreatmentInput {
                encounter_id: encounter.id,
                diagnosis_id: None,
                description: "Appendectomy".to_string(),
                dosage: None,
                corrects_treatment_id: None,
                inventory_item_id: Some(item_id),
                quantity: Some(1),
            },
        );

        assert!(matches!(result, Err(AppError::Conflict { .. })));
        let bundle = get_by_patient(&conn, 1).unwrap();
        assert!(bundle.treatments.is_empty());
    }

    #[test]
    fn create_evolution_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();

        let evolution = create_evolution(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEvolutionInput {
                encounter_id: encounter.id,
                note: "Patient stable post-op.".to_string(),
            },
        )
        .unwrap();

        assert_eq!(evolution.note, "Patient stable post-op.");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "evolution.create"));
    }

    #[test]
    fn get_by_patient_aggregates_every_entity() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let encounter = create_encounter(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEncounterInput { patient_id: 1 },
        )
        .unwrap();
        create_diagnosis(
            &mut conn,
            ACTOR_USER_ID,
            &CreateDiagnosisInput {
                encounter_id: encounter.id,
                description: "Acute appendicitis".to_string(),
                icd_code: None,
                corrects_diagnosis_id: None,
            },
        )
        .unwrap();
        create_treatment(
            &mut conn,
            ACTOR_USER_ID,
            &CreateTreatmentInput {
                encounter_id: encounter.id,
                diagnosis_id: None,
                description: "Appendectomy".to_string(),
                dosage: None,
                corrects_treatment_id: None,
                inventory_item_id: None,
                quantity: None,
            },
        )
        .unwrap();
        create_evolution(
            &mut conn,
            ACTOR_USER_ID,
            &CreateEvolutionInput {
                encounter_id: encounter.id,
                note: "Patient stable post-op.".to_string(),
            },
        )
        .unwrap();

        let bundle = get_by_patient(&conn, 1).unwrap();

        assert_eq!(bundle.encounters.len(), 1);
        assert_eq!(bundle.diagnoses.len(), 1);
        assert_eq!(bundle.treatments.len(), 1);
        assert_eq!(bundle.evolutions.len(), 1);
    }
}
