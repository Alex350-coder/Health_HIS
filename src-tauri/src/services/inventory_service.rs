//! Inventory orchestration (Database.md Section 3.5, IPC.md Section 2.1). Inventory owns the
//! write path for `inventory_categories`/`inventory_items`/`inventory_transactions`/
//! `maintenance_schedules`; Pharmacy and Lab consume these tables directly through this service,
//! not through separate schema. Thin command layer calls into this module only.
//!
//! `record_transaction_core` is `pub(crate)` rather than private because
//! `medical_history_service::create_treatment` calls it inside its own transaction to record
//! consumption tied to a treatment (Database.md Section 3.5's `treatment_id` column) — the two
//! writes must commit or roll back together.

use rusqlite::{Connection, Transaction};

use crate::db::DbError;
use crate::errors::app_error::correlation_id;
use crate::errors::AppError;
use crate::models::{InventoryCategory, InventoryItem, InventoryTransaction, MaintenanceSchedule};
use crate::repositories::inventory_repository::{
    self, NewInventoryCategory, NewInventoryItem, NewInventoryTransaction, NewMaintenanceSchedule,
};
use crate::services::audit_service::{self, RecordInput};
use crate::validation::inventory_validation::{
    self, CreateInventoryCategoryInput, CreateInventoryItemInput, CreateInventoryTransactionInput,
    ScheduleMaintenanceInput,
};

pub fn create_category(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateInventoryCategoryInput,
) -> Result<InventoryCategory, AppError> {
    inventory_validation::validate_create_inventory_category(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let category_id = inventory_repository::insert_category(
        &tx,
        &NewInventoryCategory {
            name: &input.name,
            kind: &input.kind,
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "inventory_category.create",
            entity_type: "inventory_category",
            entity_id: Some(category_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_category_or_die(conn, category_id)
}

pub fn create_item(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateInventoryItemInput,
) -> Result<InventoryItem, AppError> {
    inventory_validation::validate_create_inventory_item(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    require_category_exists(&tx, input.category_id)?;

    let item_id = inventory_repository::insert_item(
        &tx,
        &NewInventoryItem {
            category_id: input.category_id,
            name: &input.name,
            unit: &input.unit,
            reorder_threshold: input.reorder_threshold,
            expiration_date: input.expiration_date.as_deref(),
            location: input.location.as_deref(),
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "inventory_item.create",
            entity_type: "inventory_item",
            entity_id: Some(item_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_item_or_die(conn, item_id)
}

pub fn record_transaction(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateInventoryTransactionInput,
) -> Result<InventoryTransaction, AppError> {
    inventory_validation::validate_create_inventory_transaction(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let transaction_id = record_transaction_core(
        &tx,
        actor_user_id,
        input.item_id,
        input.quantity_delta,
        &input.reason,
        input.encounter_id,
        input.treatment_id,
    )?;
    tx.commit().map_err(DbError::from)?;

    find_transaction_or_die(conn, transaction_id)
}

/// Inserts one `inventory_transactions` row, updates `inventory_items.quantity` to match, and
/// writes the audit entry — all inside the caller's transaction. Rejects a `quantity_delta` that
/// would drive `quantity` negative (SQLite has no cross-row check constraint for this, mirroring
/// `operating_room_service::reserve`'s overlap-prevention rationale). Returns the new
/// `inventory_transactions.id`.
pub(crate) fn record_transaction_core(
    tx: &Transaction,
    actor_user_id: i64,
    item_id: i64,
    quantity_delta: i64,
    reason: &str,
    encounter_id: Option<i64>,
    treatment_id: Option<i64>,
) -> Result<i64, AppError> {
    let item = inventory_repository::find_item_by_id(tx, item_id)?.ok_or(AppError::NotFound {
        entity: "inventory_item".to_string(),
        id: item_id,
    })?;
    let new_quantity = item.quantity + quantity_delta;
    if new_quantity < 0 {
        return Err(AppError::Conflict {
            message: "insufficient stock for this transaction".to_string(),
        });
    }

    let transaction_id = inventory_repository::insert_transaction(
        tx,
        &NewInventoryTransaction {
            item_id,
            quantity_delta,
            reason,
            encounter_id,
            treatment_id,
            performed_by_user_id: actor_user_id,
        },
    )?;
    inventory_repository::update_item_quantity(tx, item_id, new_quantity)?;
    audit_service::record(
        tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "inventory_transaction.create",
            entity_type: "inventory_transaction",
            entity_id: Some(transaction_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;

    Ok(transaction_id)
}

pub fn schedule_maintenance(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &ScheduleMaintenanceInput,
) -> Result<MaintenanceSchedule, AppError> {
    inventory_validation::validate_schedule_maintenance(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    require_item_exists(&tx, input.inventory_item_id)?;

    let schedule_id = inventory_repository::insert_maintenance_schedule(
        &tx,
        &NewMaintenanceSchedule {
            inventory_item_id: input.inventory_item_id,
            scheduled_date: &input.scheduled_date,
            notes: input.notes.as_deref(),
        },
    )?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "maintenance_schedule.create",
            entity_type: "maintenance_schedule",
            entity_id: Some(schedule_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;

    find_maintenance_schedule_or_die(conn, schedule_id)
}

pub fn list_categories(conn: &Connection) -> Result<Vec<InventoryCategory>, AppError> {
    Ok(inventory_repository::list_categories(conn)?)
}

pub fn list_items(
    conn: &Connection,
    category_id: Option<i64>,
    low_stock_only: bool,
) -> Result<Vec<InventoryItem>, AppError> {
    Ok(inventory_repository::list_items(
        conn,
        category_id,
        low_stock_only,
    )?)
}

/// Read-only detection query. Phase 11 (Notifications) calls this directly to decide when to
/// raise a low-stock alert — no `Notification` rows are written here or anywhere in Phase 10.
pub fn list_low_stock_items(conn: &Connection) -> Result<Vec<InventoryItem>, AppError> {
    Ok(inventory_repository::find_low_stock_items(conn)?)
}

/// Read-only detection query. Phase 11 (Notifications) calls this directly to decide when to
/// raise an expiration alert — no `Notification` rows are written here or anywhere in Phase 10.
pub fn list_expiring_items(
    conn: &Connection,
    before_date: &str,
) -> Result<Vec<InventoryItem>, AppError> {
    Ok(inventory_repository::find_expiring_items(
        conn,
        before_date,
    )?)
}

fn find_category_or_die(conn: &Connection, id: i64) -> Result<InventoryCategory, AppError> {
    inventory_repository::find_category_by_id(conn, id)?
        .ok_or_else(|| unexpected_vanished("inventory_category", id))
}

fn find_item_or_die(conn: &Connection, id: i64) -> Result<InventoryItem, AppError> {
    inventory_repository::find_item_by_id(conn, id)?
        .ok_or_else(|| unexpected_vanished("inventory_item", id))
}

fn find_transaction_or_die(conn: &Connection, id: i64) -> Result<InventoryTransaction, AppError> {
    inventory_repository::find_transaction_by_id(conn, id)?
        .ok_or_else(|| unexpected_vanished("inventory_transaction", id))
}

fn find_maintenance_schedule_or_die(
    conn: &Connection,
    id: i64,
) -> Result<MaintenanceSchedule, AppError> {
    inventory_repository::find_maintenance_schedule_by_id(conn, id)?
        .ok_or_else(|| unexpected_vanished("maintenance_schedule", id))
}

fn require_category_exists(conn: &Connection, category_id: i64) -> Result<(), AppError> {
    inventory_repository::find_category_by_id(conn, category_id)?.ok_or(AppError::NotFound {
        entity: "inventory_category".to_string(),
        id: category_id,
    })?;
    Ok(())
}

fn require_item_exists(conn: &Connection, item_id: i64) -> Result<(), AppError> {
    inventory_repository::find_item_by_id(conn, item_id)?.ok_or(AppError::NotFound {
        entity: "inventory_item".to_string(),
        id: item_id,
    })?;
    Ok(())
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

    const TEST_KEY: [u8; 32] = [51u8; 32];
    const ACTOR_USER_ID: i64 = 1;

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("inventory-service-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn.execute(
            "INSERT INTO users (full_name, username, password_hash, role) \
             VALUES ('Test Admin', 'test-admin', 'argon2id$dummy', 'admin')",
            [],
        )
        .unwrap();
        conn
    }

    fn create_category_input() -> CreateInventoryCategoryInput {
        CreateInventoryCategoryInput {
            name: "Analgesics".to_string(),
            kind: "medicine".to_string(),
        }
    }

    fn create_item_input(category_id: i64) -> CreateInventoryItemInput {
        CreateInventoryItemInput {
            category_id,
            name: "Ibuprofen 400mg".to_string(),
            unit: "box".to_string(),
            reorder_threshold: 10,
            expiration_date: None,
            location: None,
        }
    }

    #[test]
    fn create_category_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let category = create_category(&mut conn, ACTOR_USER_ID, &create_category_input()).unwrap();

        assert_eq!(category.name, "Analgesics");
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows
            .iter()
            .any(|row| row.action == "inventory_category.create"));
    }

    #[test]
    fn create_item_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let category = create_category(&mut conn, ACTOR_USER_ID, &create_category_input()).unwrap();

        let item = create_item(&mut conn, ACTOR_USER_ID, &create_item_input(category.id)).unwrap();

        assert_eq!(item.name, "Ibuprofen 400mg");
        assert_eq!(item.quantity, 0);
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "inventory_item.create"));
    }

    #[test]
    fn create_item_on_a_nonexistent_category_returns_not_found() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let result = create_item(&mut conn, ACTOR_USER_ID, &create_item_input(999));

        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn record_transaction_restock_increases_quantity() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let category = create_category(&mut conn, ACTOR_USER_ID, &create_category_input()).unwrap();
        let item = create_item(&mut conn, ACTOR_USER_ID, &create_item_input(category.id)).unwrap();

        let transaction = record_transaction(
            &mut conn,
            ACTOR_USER_ID,
            &CreateInventoryTransactionInput {
                item_id: item.id,
                quantity_delta: 100,
                reason: "restock".to_string(),
                encounter_id: None,
                treatment_id: None,
            },
        )
        .unwrap();

        assert_eq!(transaction.quantity_delta, 100);
        let updated = find_item_or_die(&conn, item.id).unwrap();
        assert_eq!(updated.quantity, 100);
    }

    #[test]
    fn record_transaction_over_consumption_returns_conflict() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let category = create_category(&mut conn, ACTOR_USER_ID, &create_category_input()).unwrap();
        let item = create_item(&mut conn, ACTOR_USER_ID, &create_item_input(category.id)).unwrap();

        let result = record_transaction(
            &mut conn,
            ACTOR_USER_ID,
            &CreateInventoryTransactionInput {
                item_id: item.id,
                quantity_delta: -1,
                reason: "consumption".to_string(),
                encounter_id: None,
                treatment_id: None,
            },
        );

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn schedule_maintenance_persists_and_writes_an_audit_row() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let category = create_category(&mut conn, ACTOR_USER_ID, &create_category_input()).unwrap();
        let item = create_item(&mut conn, ACTOR_USER_ID, &create_item_input(category.id)).unwrap();

        let schedule = schedule_maintenance(
            &mut conn,
            ACTOR_USER_ID,
            &ScheduleMaintenanceInput {
                inventory_item_id: item.id,
                scheduled_date: "2026-02-01".to_string(),
                notes: Some("Routine calibration".to_string()),
            },
        )
        .unwrap();

        assert_eq!(schedule.inventory_item_id, item.id);
        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows
            .iter()
            .any(|row| row.action == "maintenance_schedule.create"));
    }

    #[test]
    fn list_low_stock_items_returns_items_at_or_below_threshold() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let category = create_category(&mut conn, ACTOR_USER_ID, &create_category_input()).unwrap();
        let item = create_item(&mut conn, ACTOR_USER_ID, &create_item_input(category.id)).unwrap();
        record_transaction(
            &mut conn,
            ACTOR_USER_ID,
            &CreateInventoryTransactionInput {
                item_id: item.id,
                quantity_delta: 5,
                reason: "restock".to_string(),
                encounter_id: None,
                treatment_id: None,
            },
        )
        .unwrap();

        let low_stock = list_low_stock_items(&conn).unwrap();

        assert_eq!(low_stock.len(), 1);
        assert_eq!(low_stock[0].id, item.id);
    }

    #[test]
    fn list_expiring_items_returns_items_on_or_before_the_cutoff() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let category = create_category(&mut conn, ACTOR_USER_ID, &create_category_input()).unwrap();
        create_item(
            &mut conn,
            ACTOR_USER_ID,
            &CreateInventoryItemInput {
                expiration_date: Some("2026-06-01".to_string()),
                ..create_item_input(category.id)
            },
        )
        .unwrap();

        let expiring = list_expiring_items(&conn, "2026-12-31").unwrap();

        assert_eq!(expiring.len(), 1);
    }
}
