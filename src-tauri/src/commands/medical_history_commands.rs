//! Medical History commands (IPC.md Section 2). Every command requires an active session (Rule
//! from IPC.md Section 1 — only the three bootstrap/login commands are session-exempt). Mutating
//! commands emit a `medical-history:<entity>:<action>` event after a successful service call
//! (Decision 3 — events are a command-layer concern, not threaded into services).

use std::sync::Mutex;

use rusqlite::Connection;

use crate::commands::{lock_connection, require_active_session};
use crate::events::emitter;
use crate::models::{Diagnosis, Encounter, Evolution, Treatment};
use crate::services::medical_history_service::{self, MedicalHistoryBundle};
use crate::validation::medical_history_validation::{
    CreateDiagnosisInput, CreateEncounterInput, CreateEvolutionInput, CreateTreatmentInput,
    DischargeEncounterInput, GetByPatientInput,
};
use crate::ActiveSession;

#[tauri::command]
pub fn medical_history_create_encounter(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateEncounterInput,
) -> Result<Encounter, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let encounter =
        medical_history_service::create_encounter(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "medical-history:encounter:created", &encounter)?;
    Ok(encounter)
}

#[tauri::command]
pub fn medical_history_discharge_encounter(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: DischargeEncounterInput,
) -> Result<Encounter, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let encounter =
        medical_history_service::discharge_encounter(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "medical-history:encounter:discharged", &encounter)?;
    Ok(encounter)
}

#[tauri::command]
pub fn medical_history_get_by_patient(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: GetByPatientInput,
) -> Result<MedicalHistoryBundle, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    medical_history_service::get_by_patient(&conn, input.patient_id)
}

#[tauri::command]
pub fn medical_history_create_diagnosis(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateDiagnosisInput,
) -> Result<Diagnosis, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let diagnosis =
        medical_history_service::create_diagnosis(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "medical-history:diagnosis:created", &diagnosis)?;
    Ok(diagnosis)
}

#[tauri::command]
pub fn medical_history_create_treatment(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateTreatmentInput,
) -> Result<Treatment, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let treatment =
        medical_history_service::create_treatment(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "medical-history:treatment:created", &treatment)?;
    Ok(treatment)
}

#[tauri::command]
pub fn medical_history_create_evolution(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateEvolutionInput,
) -> Result<Evolution, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let evolution =
        medical_history_service::create_evolution(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "medical-history:evolution:created", &evolution)?;
    Ok(evolution)
}
