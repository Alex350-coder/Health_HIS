//! Cross-service integration test for Billing (Simulation) (Rule 15.2), against a real tempfile
//! SQLCipher database. Exercises the full patient-centered workflow through the public service
//! layer of every module Billing aggregates from — Patients, Medical History, Beds, Inventory,
//! and Operating Rooms — then generates and finalizes a billing simulation and confirms the audit
//! hash chain stays valid throughout. Charge-source branch coverage (duplicate generation,
//! double finalize, not-found) is already exhaustive in `services/billing_service.rs`'s unit
//! tests; this file focuses on the full multi-module workflow, mirroring
//! `medical_history_integration.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use health_project::db::{connection, migrator};
use health_project::services::{
    audit_service, bed_service, billing_service, inventory_service, medical_history_service,
    operating_room_service, patient_service,
};
use health_project::validation::bed_validation::{
    AssignBedInput, CreateBedInput, CreateFloorInput, CreateRoomInput, ReleaseBedInput,
};
use health_project::validation::billing_validation::{
    FinalizeBillingSimulationInput, GenerateBillingSimulationInput,
};
use health_project::validation::inventory_validation::{
    CreateInventoryCategoryInput, CreateInventoryItemInput, CreateInventoryTransactionInput,
};
use health_project::validation::medical_history_validation::{
    CreateEncounterInput, CreateTreatmentInput,
};
use health_project::validation::operating_room_validation::{
    CreateOperatingRoomInput, CreateOrReservationInput,
};
use health_project::validation::patient_validation::CreatePatientInput;
use rusqlite::Connection;
use tempfile::tempdir;

const TEST_KEY: [u8; 32] = [29u8; 32];
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
fn a_full_patient_workflow_generates_and_finalizes_a_billing_simulation() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "billing-workflow.sqlite");

    let patient = patient_service::create(
        &mut conn,
        ACTOR_USER_ID,
        &CreatePatientInput {
            medical_record_number: "MRN-9001".to_string(),
            full_name: "John Smith".to_string(),
            date_of_birth: "1985-05-20".to_string(),
            sex: "male".to_string(),
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

    let encounter = medical_history_service::create_encounter(
        &mut conn,
        ACTOR_USER_ID,
        &CreateEncounterInput {
            patient_id: patient.id,
        },
    )
    .unwrap();

    // Room charge: admit into a ward bed, then release it.
    let floor = bed_service::create_floor(
        &mut conn,
        ACTOR_USER_ID,
        &CreateFloorInput {
            name: "Ground Floor".to_string(),
            level_order: 0,
        },
    )
    .unwrap();
    let ward_room = bed_service::create_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateRoomInput {
            floor_id: floor.id,
            name: "Room 1".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.0,
            map_y: 0.0,
        },
    )
    .unwrap();
    let bed = bed_service::create_bed(
        &mut conn,
        ACTOR_USER_ID,
        &CreateBedInput {
            room_id: ward_room.id,
            label: "Bed 1".to_string(),
        },
    )
    .unwrap();
    let assignment = bed_service::assign(
        &mut conn,
        ACTOR_USER_ID,
        &AssignBedInput {
            bed_id: bed.id,
            patient_id: patient.id,
            encounter_id: encounter.id,
        },
    )
    .unwrap();
    bed_service::release(
        &mut conn,
        ACTOR_USER_ID,
        &ReleaseBedInput {
            bed_assignment_id: assignment.id,
        },
    )
    .unwrap();

    // Treatment charge.
    medical_history_service::create_treatment(
        &mut conn,
        ACTOR_USER_ID,
        &CreateTreatmentInput {
            encounter_id: encounter.id,
            diagnosis_id: None,
            description: "IV fluids".to_string(),
            dosage: None,
            corrects_treatment_id: None,
            inventory_item_id: None,
            quantity: None,
        },
    )
    .unwrap();

    // Inventory charge: a consumption transaction tied to the encounter.
    let category = inventory_service::create_category(
        &mut conn,
        ACTOR_USER_ID,
        &CreateInventoryCategoryInput {
            name: "Analgesics".to_string(),
            kind: "medicine".to_string(),
        },
    )
    .unwrap();
    let item = inventory_service::create_item(
        &mut conn,
        ACTOR_USER_ID,
        &CreateInventoryItemInput {
            category_id: category.id,
            name: "Ibuprofen 400mg".to_string(),
            unit: "tablet".to_string(),
            reorder_threshold: 10,
            expiration_date: None,
            location: None,
        },
    )
    .unwrap();
    inventory_service::record_transaction(
        &mut conn,
        ACTOR_USER_ID,
        &CreateInventoryTransactionInput {
            item_id: item.id,
            quantity_delta: 50,
            reason: "restock".to_string(),
            encounter_id: None,
            treatment_id: None,
        },
    )
    .unwrap();
    inventory_service::record_transaction(
        &mut conn,
        ACTOR_USER_ID,
        &CreateInventoryTransactionInput {
            item_id: item.id,
            quantity_delta: -3,
            reason: "consumption".to_string(),
            encounter_id: Some(encounter.id),
            treatment_id: None,
        },
    )
    .unwrap();

    // OR charge: a two-hour, non-cancelled reservation.
    let or_room = bed_service::create_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateRoomInput {
            floor_id: floor.id,
            name: "OR Suite".to_string(),
            room_type: "operating_room".to_string(),
            map_x: 0.5,
            map_y: 0.5,
        },
    )
    .unwrap();
    let operating_room = operating_room_service::create_operating_room(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOperatingRoomInput {
            room_id: or_room.id,
        },
    )
    .unwrap();
    operating_room_service::reserve(
        &mut conn,
        ACTOR_USER_ID,
        &CreateOrReservationInput {
            operating_room_id: operating_room.id,
            patient_id: patient.id,
            encounter_id: encounter.id,
            procedure_description: "Appendectomy".to_string(),
            scheduled_start: "2026-01-01T08:00:00".to_string(),
            scheduled_end: "2026-01-01T10:00:00".to_string(),
        },
    )
    .unwrap();

    let generated = billing_service::generate_simulation(
        &mut conn,
        ACTOR_USER_ID,
        &GenerateBillingSimulationInput {
            encounter_id: encounter.id,
        },
    )
    .unwrap();

    assert_eq!(generated.simulation.status, "draft");
    assert_eq!(generated.items.len(), 4);
    let sources: Vec<&str> = generated
        .items
        .iter()
        .map(|item| item.source.as_str())
        .collect();
    assert!(sources.contains(&"room"));
    assert!(sources.contains(&"treatment"));
    assert!(sources.contains(&"inventory"));
    assert!(sources.contains(&"operating_room"));
    // Room 150.0 (1 day, min) + Treatment 75.0 + Inventory 10.0*3 + OR 500.0*2 hours = 1255.0.
    assert!((generated.simulation.total_amount - 1255.0).abs() < 0.01);

    let fetched = billing_service::get_simulation(&conn, encounter.id).unwrap();
    assert_eq!(fetched.simulation.id, generated.simulation.id);
    assert_eq!(fetched.items.len(), 4);

    let finalized = billing_service::finalize_simulation(
        &mut conn,
        ACTOR_USER_ID,
        &FinalizeBillingSimulationInput {
            id: generated.simulation.id,
        },
    )
    .unwrap();
    assert_eq!(finalized.simulation.status, "finalized");

    let verification = audit_service::verify_chain(&conn).unwrap();
    assert!(
        verification.is_valid,
        "the hash chain must still verify after a full cross-module billing workflow"
    );
}
