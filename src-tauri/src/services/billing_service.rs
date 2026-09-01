//! Billing (Simulation) orchestration (Database.md Section 3.8, IPC.md Section 2.1). Billing
//! owns the write path for `billing_simulations`/`billing_items` only — it aggregates read-only
//! data from Beds, Medical History, Inventory, and Operating Rooms through the small additive
//! read functions those modules expose (Architecture.md Section 3: cross-module reads go through
//! the owning module's service, never a direct repository import). This is a **simulation only**
//! — no real payment processing (CLAUDE.md Section 2).
//!
//! Charge amounts use fixed illustrative rate constants below. No source table carries a price
//! column, and adding one was out of scope for this phase (Plan.md Phase 12 decision record) —
//! a real billing engine would replace these constants with a priced rate table.

use rusqlite::Connection;
use serde::Serialize;

use crate::db::DbError;
use crate::errors::app_error::correlation_id;
use crate::errors::AppError;
use crate::models::{BillingItem, BillingSimulation};
use crate::repositories::billing_repository::{self, NewBillingItem};
use crate::repositories::encounter_repository;
use crate::services::audit_service::{self, RecordInput};
use crate::services::{
    bed_service, inventory_service, medical_history_service, operating_room_service,
};
use crate::validation::billing_validation::{
    self, FinalizeBillingSimulationInput, GenerateBillingSimulationInput,
};

const ROOM_RATE_PER_DAY: f64 = 150.0;
const TREATMENT_FLAT_RATE: f64 = 75.0;
const OR_RATE_PER_HOUR: f64 = 500.0;
const INVENTORY_RATE_PER_UNIT: f64 = 10.0;

/// A simulation with its itemization, the response shape for both `billing_generate_simulation`
/// and `billing_get_simulation` (mirrors `MedicalHistoryBundle`'s aggregate-response precedent).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingSimulationDetail {
    pub simulation: BillingSimulation,
    pub items: Vec<BillingItem>,
}

/// One charge computed by `collect_charge_items`, owning its description so it can be built
/// from formatted, per-encounter data before being handed to the repository as a borrowed
/// `NewBillingItem`.
struct ChargeItem {
    description: String,
    source: &'static str,
    source_entity_id: Option<i64>,
    amount: f64,
}

pub fn generate_simulation(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &GenerateBillingSimulationInput,
) -> Result<BillingSimulationDetail, AppError> {
    billing_validation::validate_generate_billing_simulation(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    encounter_repository::find_encounter_by_id(&tx, input.encounter_id)?.ok_or(
        AppError::NotFound {
            entity: "encounter".to_string(),
            id: input.encounter_id,
        },
    )?;
    if billing_repository::find_simulation_by_encounter_id(&tx, input.encounter_id)?.is_some() {
        return Err(AppError::Conflict {
            message: "a billing simulation already exists for this encounter".to_string(),
        });
    }

    let charge_items = collect_charge_items(&tx, input.encounter_id)?;
    let total_amount: f64 = charge_items.iter().map(|item| item.amount).sum();

    let simulation_id =
        billing_repository::insert_simulation(&tx, input.encounter_id, actor_user_id)?;
    for item in &charge_items {
        billing_repository::insert_item(
            &tx,
            simulation_id,
            &NewBillingItem {
                description: &item.description,
                source: item.source,
                source_entity_id: item.source_entity_id,
                amount: item.amount,
            },
        )?;
    }
    billing_repository::update_simulation_total(&tx, simulation_id, total_amount)?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "billing_simulation.create",
            entity_type: "billing_simulation",
            entity_id: Some(simulation_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_simulation_detail_or_die(conn, simulation_id)
}

pub fn get_simulation(
    conn: &Connection,
    encounter_id: i64,
) -> Result<BillingSimulationDetail, AppError> {
    let simulation = billing_repository::find_simulation_by_encounter_id(conn, encounter_id)?
        .ok_or(AppError::NotFound {
            entity: "billing_simulation".to_string(),
            id: encounter_id,
        })?;
    let items = billing_repository::list_items_for_simulation(conn, simulation.id)?;
    Ok(BillingSimulationDetail { simulation, items })
}

pub fn finalize_simulation(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &FinalizeBillingSimulationInput,
) -> Result<BillingSimulationDetail, AppError> {
    billing_validation::validate_finalize_billing_simulation(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let simulation =
        billing_repository::find_simulation_by_id(&tx, input.id)?.ok_or(AppError::NotFound {
            entity: "billing_simulation".to_string(),
            id: input.id,
        })?;
    if simulation.status == "finalized" {
        return Err(AppError::Conflict {
            message: "billing simulation is already finalized".to_string(),
        });
    }

    billing_repository::update_simulation_status(&tx, input.id, "finalized")?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "billing_simulation.finalize",
            entity_type: "billing_simulation",
            entity_id: Some(input.id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_simulation_detail_or_die(conn, input.id)
}

/// Aggregates the four charge sources for one encounter, in a fixed
/// room -> treatment -> inventory -> operating-room order. No rows are persisted here — the
/// caller inserts them inside its own transaction.
fn collect_charge_items(conn: &Connection, encounter_id: i64) -> Result<Vec<ChargeItem>, AppError> {
    let mut items = Vec::new();
    items.extend(room_charge_items(conn, encounter_id)?);
    items.extend(treatment_charge_items(conn, encounter_id)?);
    items.extend(inventory_charge_items(conn, encounter_id)?);
    items.extend(or_charge_items(conn, encounter_id)?);
    Ok(items)
}

fn room_charge_items(conn: &Connection, encounter_id: i64) -> Result<Vec<ChargeItem>, AppError> {
    let assignments = bed_service::list_assignments_for_encounter(conn, encounter_id)?;
    assignments
        .iter()
        .map(|assignment| {
            let days = assignment_days(
                conn,
                assignment.assigned_at.as_str(),
                assignment.released_at.as_deref(),
            )?;
            Ok(ChargeItem {
                description: "Room charge".to_string(),
                source: "room",
                source_entity_id: Some(assignment.id),
                amount: ROOM_RATE_PER_DAY * days,
            })
        })
        .collect()
}

fn treatment_charge_items(
    conn: &Connection,
    encounter_id: i64,
) -> Result<Vec<ChargeItem>, AppError> {
    let treatments = medical_history_service::list_treatments_for_encounter(conn, encounter_id)?;
    Ok(treatments
        .iter()
        .map(|treatment| ChargeItem {
            description: format!("Treatment: {}", treatment.description),
            source: "treatment",
            source_entity_id: Some(treatment.id),
            amount: TREATMENT_FLAT_RATE,
        })
        .collect())
}

fn inventory_charge_items(
    conn: &Connection,
    encounter_id: i64,
) -> Result<Vec<ChargeItem>, AppError> {
    let transactions = inventory_service::list_transactions_for_encounter(conn, encounter_id)?;
    Ok(transactions
        .iter()
        .filter(|transaction| transaction.reason == "consumption")
        .map(|transaction| ChargeItem {
            description: "Inventory consumption".to_string(),
            source: "inventory",
            source_entity_id: Some(transaction.id),
            amount: INVENTORY_RATE_PER_UNIT * transaction.quantity_delta.unsigned_abs() as f64,
        })
        .collect())
}

fn or_charge_items(conn: &Connection, encounter_id: i64) -> Result<Vec<ChargeItem>, AppError> {
    let reservations = operating_room_service::list_reservations_for_encounter(conn, encounter_id)?;
    reservations
        .iter()
        .filter(|reservation| reservation.status != "cancelled")
        .map(|reservation| {
            let hours = interval_hours(
                conn,
                &reservation.scheduled_start,
                &reservation.scheduled_end,
            )?;
            Ok(ChargeItem {
                description: format!("Operating room: {}", reservation.procedure_description),
                source: "operating_room",
                source_entity_id: Some(reservation.id),
                amount: OR_RATE_PER_HOUR * hours,
            })
        })
        .collect()
}

/// Whole days between `assigned_at` and `released_at` (or now, if still active), rounded up,
/// minimum 1. SQLite's `julianday()` computes the day-count difference (no built-in `CEIL`, so
/// the rounding happens in Rust) to avoid pulling in a date/time crate for one calculation.
fn assignment_days(
    conn: &Connection,
    assigned_at: &str,
    released_at: Option<&str>,
) -> Result<f64, AppError> {
    let raw_days: f64 = conn
        .query_row(
            "SELECT julianday(COALESCE(?1, datetime('now'))) - julianday(?2)",
            rusqlite::params![released_at, assigned_at],
            |row| row.get(0),
        )
        .map_err(|error| AppError::from(DbError::from(error)))?;
    Ok(raw_days.ceil().max(1.0))
}

/// Fractional hours between two ISO timestamps, computed in SQL for the same reason as
/// `assignment_days`.
fn interval_hours(conn: &Connection, start: &str, end: &str) -> Result<f64, AppError> {
    conn.query_row(
        "SELECT (julianday(?1) - julianday(?2)) * 24.0",
        rusqlite::params![end, start],
        |row| row.get(0),
    )
    .map_err(|error| AppError::from(DbError::from(error)))
}

fn find_simulation_detail_or_die(
    conn: &Connection,
    simulation_id: i64,
) -> Result<BillingSimulationDetail, AppError> {
    let simulation = billing_repository::find_simulation_by_id(conn, simulation_id)?
        .ok_or_else(|| unexpected_vanished("billing_simulation", simulation_id))?;
    let items = billing_repository::list_items_for_simulation(conn, simulation_id)?;
    Ok(BillingSimulationDetail { simulation, items })
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
    use crate::repositories::{
        bed_repository::{self, NewAssignment, NewBed, NewFloor, NewRoom},
        encounter_repository::{self, NewEncounter, NewTreatment},
        inventory_repository::{
            self, NewInventoryCategory, NewInventoryItem, NewInventoryTransaction,
        },
        operating_room_repository::{self, NewOperatingRoom, NewOrReservation},
    };
    use crate::services::audit_service;

    const TEST_KEY: [u8; 32] = [23u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("billing-service-test.sqlite"), &TEST_KEY).unwrap();
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

    fn seed_encounter(conn: &mut Connection, patient_id: i64, user_id: i64) -> i64 {
        encounter_repository::insert_encounter(
            conn,
            &NewEncounter {
                patient_id,
                created_by_user_id: user_id,
            },
        )
        .unwrap()
    }

    fn seed_full_encounter_charges(conn: &mut Connection, user_id: i64, patient_id: i64) -> i64 {
        let encounter_id = seed_encounter(conn, patient_id, user_id);

        // Room charge: one released bed assignment.
        let floor_id = bed_repository::insert_floor(
            conn,
            &NewFloor {
                name: "Ground Floor",
                level_order: 0,
            },
        )
        .unwrap();
        let room_id = bed_repository::insert_room(
            conn,
            &NewRoom {
                floor_id,
                name: "Room 1",
                room_type: "ward",
                map_x: 0.0,
                map_y: 0.0,
            },
        )
        .unwrap();
        let bed_id = bed_repository::insert_bed(
            conn,
            &NewBed {
                room_id,
                label: "Bed 1",
            },
        )
        .unwrap();
        let assignment_id = bed_repository::insert_assignment(
            conn,
            &NewAssignment {
                bed_id,
                patient_id,
                encounter_id,
                assigned_by_user_id: user_id,
            },
        )
        .unwrap();
        bed_repository::release_assignment(conn, assignment_id).unwrap();

        // Treatment charge.
        encounter_repository::insert_treatment(
            conn,
            &NewTreatment {
                encounter_id,
                diagnosis_id: None,
                description: "IV fluids",
                dosage: None,
                registered_by_user_id: user_id,
                corrects_treatment_id: None,
            },
        )
        .unwrap();

        // Inventory charge: one consumption transaction.
        let category_id = inventory_repository::insert_category(
            conn,
            &NewInventoryCategory {
                name: "Analgesics",
                kind: "medicine",
            },
        )
        .unwrap();
        let item_id = inventory_repository::insert_item(
            conn,
            &NewInventoryItem {
                category_id,
                name: "Ibuprofen",
                unit: "tablet",
                reorder_threshold: 10,
                expiration_date: None,
                location: None,
            },
        )
        .unwrap();
        inventory_repository::insert_transaction(
            conn,
            &NewInventoryTransaction {
                item_id,
                quantity_delta: -3,
                reason: "consumption",
                encounter_id: Some(encounter_id),
                treatment_id: None,
                performed_by_user_id: user_id,
            },
        )
        .unwrap();

        // OR charge: one non-cancelled reservation.
        let or_room_id = bed_repository::insert_room(
            conn,
            &NewRoom {
                floor_id,
                name: "OR Suite",
                room_type: "operating_room",
                map_x: 0.5,
                map_y: 0.5,
            },
        )
        .unwrap();
        let operating_room_id = operating_room_repository::insert_operating_room(
            conn,
            &NewOperatingRoom {
                room_id: or_room_id,
                name: "OR 1",
            },
        )
        .unwrap();
        operating_room_repository::insert_reservation(
            conn,
            &NewOrReservation {
                operating_room_id,
                patient_id,
                encounter_id,
                procedure_description: "Appendectomy",
                scheduled_start: "2026-01-01T08:00:00",
                scheduled_end: "2026-01-01T10:00:00",
                scheduled_by_user_id: user_id,
            },
        )
        .unwrap();

        encounter_id
    }

    #[test]
    fn generate_simulation_aggregates_all_four_charge_sources() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_full_encounter_charges(&mut conn, user_id, patient_id);

        let detail = generate_simulation(
            &mut conn,
            user_id,
            &GenerateBillingSimulationInput { encounter_id },
        )
        .unwrap();

        assert_eq!(detail.items.len(), 4);
        let expected_total = ROOM_RATE_PER_DAY
            + TREATMENT_FLAT_RATE
            + INVENTORY_RATE_PER_UNIT * 3.0
            + OR_RATE_PER_HOUR * 2.0;
        // `julianday()` arithmetic on the OR reservation's fixed timestamps loses a few
        // nanoseconds of floating-point precision, so a generous-but-still-meaningful tolerance
        // is used rather than exact equality.
        assert!((detail.simulation.total_amount - expected_total).abs() < 0.01);
        assert_eq!(detail.simulation.status, "draft");
    }

    #[test]
    fn generate_simulation_rejects_a_duplicate_for_the_same_encounter() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&mut conn, patient_id, user_id);

        generate_simulation(
            &mut conn,
            user_id,
            &GenerateBillingSimulationInput { encounter_id },
        )
        .unwrap();
        let result = generate_simulation(
            &mut conn,
            user_id,
            &GenerateBillingSimulationInput { encounter_id },
        );

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn finalize_simulation_rejects_finalizing_twice() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&mut conn, patient_id, user_id);
        let detail = generate_simulation(
            &mut conn,
            user_id,
            &GenerateBillingSimulationInput { encounter_id },
        )
        .unwrap();

        finalize_simulation(
            &mut conn,
            user_id,
            &FinalizeBillingSimulationInput {
                id: detail.simulation.id,
            },
        )
        .unwrap();
        let result = finalize_simulation(
            &mut conn,
            user_id,
            &FinalizeBillingSimulationInput {
                id: detail.simulation.id,
            },
        );

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn finalize_simulation_returns_not_found_for_a_missing_id() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let result = finalize_simulation(&mut conn, 1, &FinalizeBillingSimulationInput { id: 999 });

        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn generate_simulation_keeps_the_audit_chain_valid() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&mut conn, patient_id, user_id);

        generate_simulation(
            &mut conn,
            user_id,
            &GenerateBillingSimulationInput { encounter_id },
        )
        .unwrap();

        assert!(audit_service::verify_chain(&conn).unwrap().is_valid);
    }
}
