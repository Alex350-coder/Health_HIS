//! Cross-service integration tests for the Medical History module (Rule 15.2), against a real
//! tempfile SQLCipher database. Business-rule branches (open-encounter enforcement, correction
//! auditing) are already exhaustively covered by the unit tests colocated in
//! `services/medical_history_service.rs`; this file focuses on the full multi-entity workflow and
//! the audit hash chain staying valid across it, mirroring `patients_integration.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use health_project::db::{connection, migrator};
use health_project::errors::AppError;
use health_project::services::{audit_service, bed_service, medical_history_service};
use health_project::validation::bed_validation::{
    AssignBedInput, CreateBedInput, CreateFloorInput, CreateRoomInput,
};
use health_project::validation::medical_history_validation::{
    CreateDiagnosisInput, CreateEncounterInput, CreateEvolutionInput, CreateTreatmentInput,
    DischargeEncounterInput,
};
use rusqlite::Connection;
use tempfile::tempdir;

const TEST_KEY: [u8; 32] = [17u8; 32];
const ACTOR_USER_ID: i64 = 1;
const PATIENT_ID: i64 = 1;

fn migrated_connection(dir: &std::path::Path, name: &str) -> Connection {
    let conn = connection::open(&dir.join(name), &TEST_KEY).unwrap();
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
fn a_full_encounter_workflow_persists_every_entity_and_discharges_cleanly() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "workflow.sqlite");

    let encounter = medical_history_service::create_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &CreateEncounterInput {
            patient_id: PATIENT_ID,
        },
    )
    .unwrap();
    let diagnosis = medical_history_service::create_diagnosis(
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
    medical_history_service::create_treatment(
        &mut conn,
        ACTOR_USER_ID,
        &CreateTreatmentInput {
            encounter_id: encounter.id,
            diagnosis_id: Some(diagnosis.id),
            description: "Appendectomy".to_string(),
            dosage: None,
            corrects_treatment_id: None,
            inventory_item_id: None,
            quantity: None,
        },
    )
    .unwrap();
    medical_history_service::create_evolution(
        &mut conn,
        ACTOR_USER_ID,
        &CreateEvolutionInput {
            encounter_id: encounter.id,
            note: "Patient stable post-op.".to_string(),
        },
    )
    .unwrap();
    let discharged = medical_history_service::discharge_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &DischargeEncounterInput {
            encounter_id: encounter.id,
            discharge_summary: Some("Recovered well, discharged home.".to_string()),
        },
    )
    .unwrap();

    assert_eq!(discharged.status, "discharged");
    let bundle = medical_history_service::get_by_patient(&conn, PATIENT_ID).unwrap();
    assert_eq!(bundle.encounters.len(), 1);
    assert_eq!(bundle.diagnoses.len(), 1);
    assert_eq!(bundle.treatments.len(), 1);
    assert_eq!(bundle.evolutions.len(), 1);
}

#[test]
fn a_correction_references_the_original_row_within_the_same_encounter() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "correction.sqlite");
    let encounter = medical_history_service::create_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &CreateEncounterInput {
            patient_id: PATIENT_ID,
        },
    )
    .unwrap();
    let original = medical_history_service::create_treatment(
        &mut conn,
        ACTOR_USER_ID,
        &CreateTreatmentInput {
            encounter_id: encounter.id,
            diagnosis_id: None,
            description: "Amoxicillin 500mg".to_string(),
            dosage: Some("3x/day".to_string()),
            corrects_treatment_id: None,
            inventory_item_id: None,
            quantity: None,
        },
    )
    .unwrap();

    let correction = medical_history_service::create_treatment(
        &mut conn,
        ACTOR_USER_ID,
        &CreateTreatmentInput {
            encounter_id: encounter.id,
            diagnosis_id: None,
            description: "Amoxicillin 875mg".to_string(),
            dosage: Some("2x/day".to_string()),
            corrects_treatment_id: Some(original.id),
            inventory_item_id: None,
            quantity: None,
        },
    )
    .unwrap();

    assert_eq!(correction.corrects_treatment_id, Some(original.id));
    let bundle = medical_history_service::get_by_patient(&conn, PATIENT_ID).unwrap();
    assert_eq!(bundle.treatments.len(), 2);
}

#[test]
fn adding_a_diagnosis_to_a_discharged_encounter_fails() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "discharged.sqlite");
    let encounter = medical_history_service::create_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &CreateEncounterInput {
            patient_id: PATIENT_ID,
        },
    )
    .unwrap();
    medical_history_service::discharge_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &DischargeEncounterInput {
            encounter_id: encounter.id,
            discharge_summary: None,
        },
    )
    .unwrap();

    let result = medical_history_service::create_diagnosis(
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

/// Discharge must also release any bed still held for the encounter, in the same transaction as
/// the encounter status write (Database.md Section 7, Plan.md Phase 13 Task 13.1) — composes
/// `bed_service` with `medical_history_service` the way `billing_integration.rs` composes Billing
/// with every other module.
#[test]
fn discharging_an_encounter_releases_its_bed_assignment() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "discharge-releases-bed.sqlite");

    let encounter = medical_history_service::create_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &CreateEncounterInput {
            patient_id: PATIENT_ID,
        },
    )
    .unwrap();

    let floor = bed_service::create_floor(
        &mut conn,
        ACTOR_USER_ID,
        &CreateFloorInput {
            name: "Floor 1".to_string(),
            level_order: 0,
        },
    )
    .unwrap();
    let room = bed_service::create_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateRoomInput {
            floor_id: floor.id,
            name: "Room 101".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.5,
            map_y: 0.5,
        },
    )
    .unwrap();
    let bed = bed_service::create_bed(
        &mut conn,
        ACTOR_USER_ID,
        &CreateBedInput {
            room_id: room.id,
            label: "A".to_string(),
        },
    )
    .unwrap();
    let assignment = bed_service::assign(
        &mut conn,
        ACTOR_USER_ID,
        &AssignBedInput {
            bed_id: bed.id,
            patient_id: PATIENT_ID,
            encounter_id: encounter.id,
        },
    )
    .unwrap();

    medical_history_service::discharge_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &DischargeEncounterInput {
            encounter_id: encounter.id,
            discharge_summary: Some("Recovered well.".to_string()),
        },
    )
    .unwrap();

    let assignments = bed_service::list_assignments_for_encounter(&conn, encounter.id).unwrap();
    let released = assignments
        .iter()
        .find(|found| found.id == assignment.id)
        .unwrap();
    assert!(released.released_at.is_some());

    let verification = audit_service::verify_chain(&conn).unwrap();
    assert!(
        verification.is_valid,
        "the hash chain must still verify after a discharge that also releases a bed"
    );
}

/// Encounter create, diagnosis create, treatment correction, evolution create, then discharge —
/// the hash chain (which records only successful writes) must still verify as valid throughout.
#[test]
fn the_audit_chain_stays_valid_after_a_full_encounter_lifecycle() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "audit-chain.sqlite");

    let encounter = medical_history_service::create_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &CreateEncounterInput {
            patient_id: PATIENT_ID,
        },
    )
    .unwrap();
    let diagnosis = medical_history_service::create_diagnosis(
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
    let treatment = medical_history_service::create_treatment(
        &mut conn,
        ACTOR_USER_ID,
        &CreateTreatmentInput {
            encounter_id: encounter.id,
            diagnosis_id: Some(diagnosis.id),
            description: "Appendectomy".to_string(),
            dosage: None,
            corrects_treatment_id: None,
            inventory_item_id: None,
            quantity: None,
        },
    )
    .unwrap();
    medical_history_service::create_treatment(
        &mut conn,
        ACTOR_USER_ID,
        &CreateTreatmentInput {
            encounter_id: encounter.id,
            diagnosis_id: Some(diagnosis.id),
            description: "Appendectomy with drain".to_string(),
            dosage: None,
            corrects_treatment_id: Some(treatment.id),
            inventory_item_id: None,
            quantity: None,
        },
    )
    .unwrap();
    medical_history_service::create_evolution(
        &mut conn,
        ACTOR_USER_ID,
        &CreateEvolutionInput {
            encounter_id: encounter.id,
            note: "Patient stable post-op.".to_string(),
        },
    )
    .unwrap();
    medical_history_service::discharge_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &DischargeEncounterInput {
            encounter_id: encounter.id,
            discharge_summary: Some("Recovered well.".to_string()),
        },
    )
    .unwrap();

    let verification = audit_service::verify_chain(&conn).unwrap();
    assert!(
        verification.is_valid,
        "the hash chain must still verify after a full encounter lifecycle with a correction"
    );
}
