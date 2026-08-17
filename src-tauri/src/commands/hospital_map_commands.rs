//! Hospital Map commands (IPC.md Section 2, Architecture.md Section 3). Strictly read-only — every
//! function here calls `bed_repository` directly, never `bed_service`, and never emits an event.
//! `tests/hospital_map_read_only.rs` mechanically enforces that every `pub fn` in this file is
//! named `hospital_map_get_*`.

use std::sync::Mutex;

use rusqlite::Connection;

use crate::commands::{lock_connection, require_active_session};
use crate::errors::AppError;
use crate::models::{FloorLayout, RoomStatus};
use crate::repositories::bed_repository;
use crate::ActiveSession;

#[tauri::command]
pub fn hospital_map_get_layout(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
) -> Result<Vec<FloorLayout>, AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    bed_repository::list_floors_with_rooms(&conn).map_err(AppError::from)
}

#[tauri::command]
pub fn hospital_map_get_room_status(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    room_id: i64,
) -> Result<RoomStatus, AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    bed_repository::get_room_status(&conn, room_id)?.ok_or(AppError::NotFound {
        entity: "room".to_string(),
        id: room_id,
    })
}
