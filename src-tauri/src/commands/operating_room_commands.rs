//! Operating Rooms commands (IPC.md Section 2.1). Every mutation emits an
//! `operating-rooms:*` event so both the Operating Rooms and Hospital Map query caches
//! invalidate (StateManagement.md / `shared/lib/event-query-map.ts`).

use std::sync::Mutex;

use rusqlite::Connection;

use crate::commands::{lock_connection, require_active_session};
use crate::events::emitter;
use crate::models::{OperatingRoom, OrReservation};
use crate::services::operating_room_service;
use crate::validation::operating_room_validation::{
    CancelOrReservationInput, CreateOperatingRoomInput, CreateOrReservationInput,
    UpdateOrReservationInput,
};
use crate::ActiveSession;

#[tauri::command]
pub fn operating_rooms_create(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateOperatingRoomInput,
) -> Result<OperatingRoom, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let operating_room =
        operating_room_service::create_operating_room(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "operating-rooms:room:created", &operating_room)?;
    Ok(operating_room)
}

#[tauri::command]
pub fn operating_rooms_list(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
) -> Result<Vec<OperatingRoom>, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    operating_room_service::list_operating_rooms(&conn)
}

#[tauri::command]
pub fn operating_rooms_list_reservations(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    operating_room_id: Option<i64>,
) -> Result<Vec<OrReservation>, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    operating_room_service::list_reservations(&conn, operating_room_id)
}

#[tauri::command]
pub fn operating_rooms_reserve(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateOrReservationInput,
) -> Result<OrReservation, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let reservation = operating_room_service::reserve(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "operating-rooms:reservation:created", &reservation)?;
    Ok(reservation)
}

#[tauri::command]
pub fn operating_rooms_update_reservation(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: UpdateOrReservationInput,
) -> Result<OrReservation, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let reservation =
        operating_room_service::update_reservation(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "operating-rooms:reservation:updated", &reservation)?;
    Ok(reservation)
}

#[tauri::command]
pub fn operating_rooms_cancel_reservation(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CancelOrReservationInput,
) -> Result<OrReservation, crate::errors::AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let reservation =
        operating_room_service::cancel_reservation(&mut conn, authenticated.user_id, &input)?;
    emitter::emit(&app, "operating-rooms:reservation:cancelled", &reservation)?;
    Ok(reservation)
}
