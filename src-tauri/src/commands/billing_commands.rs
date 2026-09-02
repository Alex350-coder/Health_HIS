//! Billing commands (IPC.md Section 2.1). Every mutation emits a `billing:*` event so the
//! Billing tab invalidates its query cache (StateManagement.md / `shared/lib/event-query-map.ts`).
//! Simulation only — no real payment processing (CLAUDE.md Section 2).

use std::sync::Mutex;

use rusqlite::Connection;

use crate::commands::{lock_connection, require_active_session};
use crate::events::emitter;
use crate::services::billing_service::{self, BillingSimulationDetail};
use crate::validation::billing_validation::{
    FinalizeBillingSimulationInput, GenerateBillingSimulationInput,
};
use crate::ActiveSession;

#[tauri::command]
pub fn billing_generate_simulation(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: GenerateBillingSimulationInput,
) -> Result<BillingSimulationDetail, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let detail = billing_service::generate_simulation(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "billing:simulation:created", &detail.simulation)?;
    Ok(detail)
}

#[tauri::command]
pub fn billing_get_simulation(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    encounter_id: i64,
) -> Result<BillingSimulationDetail, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    billing_service::get_simulation(&conn, encounter_id)
}

#[tauri::command]
pub fn billing_finalize_simulation(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: FinalizeBillingSimulationInput,
) -> Result<BillingSimulationDetail, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let detail = billing_service::finalize_simulation(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "billing:simulation:finalized", &detail.simulation)?;
    Ok(detail)
}
