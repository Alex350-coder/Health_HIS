//! Audit commands (IPC.md Section 2). Both require a session — the audit trail is never a
//! second unauthenticated entry point.

use std::sync::Mutex;

use rusqlite::Connection;
use serde::Deserialize;

use crate::commands::{lock_connection, require_active_session};
use crate::errors::AppError;
use crate::models::AuditLogEntry;
use crate::repositories::audit_repository::ListFilter;
use crate::services::audit_service::{self, ChainVerification};
use crate::ActiveSession;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditListInput {
    pub entity_type: Option<String>,
    pub user_id: Option<i64>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

#[tauri::command]
pub fn audit_list(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: AuditListInput,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    audit_service::list(
        &conn,
        &ListFilter {
            entity_type: input.entity_type.as_deref(),
            user_id: input.user_id,
            from: input.from.as_deref(),
            to: input.to.as_deref(),
            limit: input.limit,
            offset: input.offset,
        },
    )
}

#[tauri::command]
pub fn audit_verify_integrity(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
) -> Result<ChainVerification, AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    audit_service::verify_chain(&conn)
}
