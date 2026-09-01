//! Inventory commands (IPC.md Section 2.1). Every mutation emits an `inventory:*` event so both
//! the Inventory module and any future Pharmacy/Lab consumer views invalidate their query caches
//! (StateManagement.md / `shared/lib/event-query-map.ts`).

use std::sync::Mutex;

use rusqlite::Connection;

use crate::commands::{lock_connection, require_active_session};
use crate::events::emitter;
use crate::models::{InventoryCategory, InventoryItem, InventoryTransaction, MaintenanceSchedule};
use crate::services::{inventory_service, notification_service};
use crate::validation::inventory_validation::{
    CreateInventoryCategoryInput, CreateInventoryItemInput, CreateInventoryTransactionInput,
    ScheduleMaintenanceInput,
};
use crate::ActiveSession;

/// Runs the inventory-alert detection sweep and emits one `notifications:notification:created`
/// event per newly-created row, so the top-bar bell updates without a manual refresh
/// (StateManagement.md / `shared/lib/event-query-map.ts`). Called after every Inventory mutation
/// command — see `notification_service::check_inventory_alerts`'s doc comment for why this lives
/// here rather than inside `inventory_service.rs`.
fn raise_inventory_alerts(
    app: &tauri::AppHandle,
    conn: &mut Connection,
) -> Result<(), crate::errors::AppError> {
    for notification in notification_service::check_inventory_alerts(conn)? {
        emitter::emit(app, "notifications:notification:created", &notification)?;
    }
    Ok(())
}

#[tauri::command]
pub fn inventory_list_categories(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
) -> Result<Vec<InventoryCategory>, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    inventory_service::list_categories(&conn)
}

#[tauri::command]
pub fn inventory_create_category(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateInventoryCategoryInput,
) -> Result<InventoryCategory, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let category = inventory_service::create_category(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "inventory:category:created", &category)?;
    Ok(category)
}

#[tauri::command]
pub fn inventory_list_items(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    category_id: Option<i64>,
    low_stock_only: bool,
) -> Result<Vec<InventoryItem>, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    inventory_service::list_items(&conn, category_id, low_stock_only)
}

#[tauri::command]
pub fn inventory_create_item(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateInventoryItemInput,
) -> Result<InventoryItem, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let item = inventory_service::create_item(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "inventory:item:created", &item)?;
    raise_inventory_alerts(&app, &mut conn)?;
    Ok(item)
}

#[tauri::command]
pub fn inventory_record_transaction(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateInventoryTransactionInput,
) -> Result<InventoryTransaction, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let transaction =
        inventory_service::record_transaction(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "inventory:transaction:created", &transaction)?;
    raise_inventory_alerts(&app, &mut conn)?;
    Ok(transaction)
}

#[tauri::command]
pub fn inventory_schedule_maintenance(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: ScheduleMaintenanceInput,
) -> Result<MaintenanceSchedule, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let schedule =
        inventory_service::schedule_maintenance(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "inventory:maintenance:created", &schedule)?;
    raise_inventory_alerts(&app, &mut conn)?;
    Ok(schedule)
}

#[tauri::command]
pub fn inventory_list_transactions(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    item_id: i64,
) -> Result<Vec<InventoryTransaction>, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    inventory_service::list_transactions_for_item(&conn, item_id)
}

#[tauri::command]
pub fn inventory_list_maintenance_schedules(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    item_id: i64,
) -> Result<Vec<MaintenanceSchedule>, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    inventory_service::list_maintenance_schedules_for_item(&conn, item_id)
}
