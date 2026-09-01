//! Cross-service integration tests for Inventory: category/item creation, restock and
//! consumption transactions, the sufficient-stock rule, and maintenance scheduling against a
//! real tempfile SQLCipher database (Rule 15.2).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use health_project::db::{connection, migrator};
use health_project::errors::AppError;
use health_project::services::{audit_service, inventory_service};
use health_project::validation::inventory_validation::{
    CreateInventoryCategoryInput, CreateInventoryItemInput, CreateInventoryTransactionInput,
    ScheduleMaintenanceInput,
};
use rusqlite::Connection;
use tempfile::tempdir;

const TEST_KEY: [u8; 32] = [67u8; 32];
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

fn category_input() -> CreateInventoryCategoryInput {
    CreateInventoryCategoryInput {
        name: "Analgesics".to_string(),
        kind: "medicine".to_string(),
    }
}

fn item_input(category_id: i64) -> CreateInventoryItemInput {
    CreateInventoryItemInput {
        category_id,
        name: "Ibuprofen 400mg".to_string(),
        unit: "box".to_string(),
        reorder_threshold: 10,
        expiration_date: None,
        location: Some("Pharmacy Shelf B2".to_string()),
    }
}

#[test]
fn a_category_and_item_can_be_created_restocked_and_consumed() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "full-flow.sqlite");

    let category =
        inventory_service::create_category(&mut conn, ACTOR_USER_ID, &category_input()).unwrap();
    let item =
        inventory_service::create_item(&mut conn, ACTOR_USER_ID, &item_input(category.id)).unwrap();
    assert_eq!(item.quantity, 0);

    let restock = inventory_service::record_transaction(
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
    assert_eq!(restock.quantity_delta, 50);

    let consumption = inventory_service::record_transaction(
        &mut conn,
        ACTOR_USER_ID,
        &CreateInventoryTransactionInput {
            item_id: item.id,
            quantity_delta: -20,
            reason: "consumption".to_string(),
            encounter_id: None,
            treatment_id: None,
        },
    )
    .unwrap();
    assert_eq!(consumption.quantity_delta, -20);

    let items = inventory_service::list_items(&conn, Some(category.id), false).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].quantity, 30);
}

#[test]
fn over_consumption_is_rejected_and_leaves_quantity_unchanged() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "over-consumption.sqlite");
    let category =
        inventory_service::create_category(&mut conn, ACTOR_USER_ID, &category_input()).unwrap();
    let item =
        inventory_service::create_item(&mut conn, ACTOR_USER_ID, &item_input(category.id)).unwrap();
    inventory_service::record_transaction(
        &mut conn,
        ACTOR_USER_ID,
        &CreateInventoryTransactionInput {
            item_id: item.id,
            quantity_delta: 10,
            reason: "restock".to_string(),
            encounter_id: None,
            treatment_id: None,
        },
    )
    .unwrap();

    let result = inventory_service::record_transaction(
        &mut conn,
        ACTOR_USER_ID,
        &CreateInventoryTransactionInput {
            item_id: item.id,
            quantity_delta: -11,
            reason: "consumption".to_string(),
            encounter_id: None,
            treatment_id: None,
        },
    );
    assert!(matches!(result, Err(AppError::Conflict { .. })));

    let items = inventory_service::list_items(&conn, Some(category.id), false).unwrap();
    assert_eq!(items[0].quantity, 10);
}

#[test]
fn a_maintenance_schedule_can_be_created_for_an_existing_item() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "maintenance.sqlite");
    let category =
        inventory_service::create_category(&mut conn, ACTOR_USER_ID, &category_input()).unwrap();
    let item =
        inventory_service::create_item(&mut conn, ACTOR_USER_ID, &item_input(category.id)).unwrap();

    let schedule = inventory_service::schedule_maintenance(
        &mut conn,
        ACTOR_USER_ID,
        &ScheduleMaintenanceInput {
            inventory_item_id: item.id,
            scheduled_date: "2026-03-01".to_string(),
            notes: Some("Routine calibration".to_string()),
        },
    )
    .unwrap();

    assert_eq!(schedule.inventory_item_id, item.id);
}

#[test]
fn the_audit_chain_stays_valid_after_a_full_inventory_write_sequence() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "audit-chain.sqlite");
    let category =
        inventory_service::create_category(&mut conn, ACTOR_USER_ID, &category_input()).unwrap();
    let item =
        inventory_service::create_item(&mut conn, ACTOR_USER_ID, &item_input(category.id)).unwrap();
    inventory_service::record_transaction(
        &mut conn,
        ACTOR_USER_ID,
        &CreateInventoryTransactionInput {
            item_id: item.id,
            quantity_delta: 10,
            reason: "restock".to_string(),
            encounter_id: None,
            treatment_id: None,
        },
    )
    .unwrap();
    let _ = inventory_service::record_transaction(
        &mut conn,
        ACTOR_USER_ID,
        &CreateInventoryTransactionInput {
            item_id: item.id,
            quantity_delta: -11,
            reason: "consumption".to_string(),
            encounter_id: None,
            treatment_id: None,
        },
    );
    inventory_service::schedule_maintenance(
        &mut conn,
        ACTOR_USER_ID,
        &ScheduleMaintenanceInput {
            inventory_item_id: item.id,
            scheduled_date: "2026-03-01".to_string(),
            notes: None,
        },
    )
    .unwrap();

    let verification = audit_service::verify_chain(&conn).unwrap();
    assert!(
        verification.is_valid,
        "the hash chain must still verify after a sequence of inventory writes and a rejected \
         over-consumption"
    );
}
