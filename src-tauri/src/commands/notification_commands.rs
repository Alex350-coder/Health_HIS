//! Notification commands (IPC.md). `notifications_mark_read` is the one documented exception to
//! the no-optimistic-update policy (StateManagement.md Section 5) — the frontend mutation applies
//! the change locally before this round-trip resolves, so this command must stay small and fast.

use std::sync::Mutex;

use rusqlite::Connection;

use crate::commands::{lock_connection, require_active_session};
use crate::models::Notification;
use crate::services::notification_service;
use crate::validation::notification_validation::MarkNotificationReadInput;
use crate::ActiveSession;

#[tauri::command]
pub fn notifications_list(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    unread_only: bool,
) -> Result<Vec<Notification>, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    notification_service::list_notifications(&conn, unread_only)
}

#[tauri::command]
pub fn notifications_mark_read(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: MarkNotificationReadInput,
) -> Result<Notification, crate::errors::AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    notification_service::mark_read(&conn, &input)
}
