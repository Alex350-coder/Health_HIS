//! Beds facility-configuration commands (IPC.md Section 2.1). Beds owns the write path for
//! `floors`/`rooms`/`beds` even though Hospital Map visualizes the same tables. Every mutation
//! emits `beds:facility:changed` so both the Beds and Hospital Map query caches invalidate
//! (StateManagement.md / `shared/lib/event-query-map.ts`).

use std::sync::Mutex;

use rusqlite::Connection;

use crate::commands::{lock_connection, require_active_session};
use crate::events::emitter;
use crate::models::{Bed, BedAssignment, BedSummary, Floor, Room};
use crate::services::bed_service;
use crate::validation::bed_validation::{
    AssignBedInput, CreateBedInput, CreateFloorInput, CreateRoomInput, ReleaseBedInput,
    SetBedStatusInput,
};
use crate::ActiveSession;

#[tauri::command]
pub fn beds_create_floor(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateFloorInput,
) -> Result<Floor, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let floor = bed_service::create_floor(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "beds:facility:changed", &floor)?;
    Ok(floor)
}

#[tauri::command]
pub fn beds_create_room(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateRoomInput,
) -> Result<Room, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let room = bed_service::create_room(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "beds:facility:changed", &room)?;
    Ok(room)
}

#[tauri::command]
pub fn beds_create(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateBedInput,
) -> Result<Bed, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let bed = bed_service::create_bed(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "beds:facility:changed", &bed)?;
    Ok(bed)
}

#[tauri::command]
pub fn beds_set_status(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: SetBedStatusInput,
) -> Result<Bed, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let bed = bed_service::set_status(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "beds:facility:changed", &bed)?;
    Ok(bed)
}

#[tauri::command]
pub fn beds_list(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    room_id: Option<i64>,
) -> Result<Vec<BedSummary>, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    bed_service::list(&conn, room_id)
}

#[tauri::command]
pub fn beds_assign(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: AssignBedInput,
) -> Result<BedAssignment, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let assignment = bed_service::assign(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "beds:assignment:created", &assignment)?;
    Ok(assignment)
}

#[tauri::command]
pub fn beds_release(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: ReleaseBedInput,
) -> Result<BedAssignment, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let assignment = bed_service::release(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "beds:assignment:released", &assignment)?;
    Ok(assignment)
}
