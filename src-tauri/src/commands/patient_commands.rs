//! Patient commands (IPC.md Section 2). Every command requires an active session (Rule from
//! IPC.md Section 1 — only the three bootstrap/login commands are session-exempt).
//! `patients_create`/`patients_update` also emit a `patients:record:*` event after a successful
//! service call (Decision 3 — events are a command-layer concern, not threaded into services).

use std::sync::Mutex;

use rusqlite::Connection;

use crate::commands::{lock_connection, require_active_session};
use crate::events::emitter;
use crate::models::Patient;
use crate::services::patient_service;
use crate::validation::patient_validation::{
    CreatePatientInput, GetPatientInput, ListPatientsInput, UpdatePatientInput,
};
use crate::ActiveSession;

#[tauri::command]
pub fn patients_create(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreatePatientInput,
) -> Result<Patient, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let patient = patient_service::create(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "patients:record:created", &patient)?;
    Ok(patient)
}

#[tauri::command]
pub fn patients_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: UpdatePatientInput,
) -> Result<Patient, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let patient = patient_service::update(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "patients:record:updated", &patient)?;
    Ok(patient)
}

#[tauri::command]
pub fn patients_get(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: GetPatientInput,
) -> Result<Patient, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    patient_service::get(&conn, input.id)
}

#[tauri::command]
pub fn patients_list(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: ListPatientsInput,
) -> Result<Vec<Patient>, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    patient_service::list(&conn, &input)
}
