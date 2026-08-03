//! Shared plumbing for command handlers (Architecture.md — commands are thin adapters: lock
//! state, call one service method, map the result). Session-token handling here is the one
//! deviation worth flagging explicitly: Security.md Section 6 says the session token is "passed
//! via Tauri managed state," and Section 9.1/IPC.md's `auth_current_user` note ("restore session
//! context if token still in memory") only makes sense if the backend, not just the frontend,
//! retains the active token — a single WebView, single-session desktop app (Decision 3, Phase 2
//! plan) has exactly one concurrent session, so the Rust process holding it in `ActiveSession`
//! (Rule 17.7 — the one sanctioned mutable-singleton mechanism) is both possible and normal.
//! `auth_login`/`auth_bootstrap_admin` still return the raw token to the frontend per IPC.md's
//! documented response shape, but no other command requires the frontend to pass it back.

pub mod audit_commands;
pub mod auth_commands;

use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

use crate::errors::app_error::correlation_id;
use crate::errors::AppError;
use crate::security::session::{self, AuthenticatedUser};
use crate::ActiveSession;

pub(crate) fn lock_connection<'a>(
    state: &'a tauri::State<'_, Mutex<Connection>>,
) -> Result<MutexGuard<'a, Connection>, AppError> {
    state.lock().map_err(|_| poisoned("database connection"))
}

pub(crate) fn require_active_session(
    conn: &Connection,
    active_session: &tauri::State<'_, ActiveSession>,
) -> Result<AuthenticatedUser, AppError> {
    let token = active_session
        .0
        .lock()
        .map_err(|_| poisoned("active session"))?
        .clone()
        .ok_or(AppError::Unauthorized)?;
    session::require_session(conn, &token)
}

pub(crate) fn set_active_token(
    active_session: &tauri::State<'_, ActiveSession>,
    token: String,
) -> Result<(), AppError> {
    let mut guard = active_session
        .0
        .lock()
        .map_err(|_| poisoned("active session"))?;
    *guard = Some(token);
    Ok(())
}

pub(crate) fn clear_active_token(
    active_session: &tauri::State<'_, ActiveSession>,
) -> Result<Option<String>, AppError> {
    let mut guard = active_session
        .0
        .lock()
        .map_err(|_| poisoned("active session"))?;
    Ok(guard.take())
}

fn poisoned(what: &'static str) -> AppError {
    let correlation_id = correlation_id();
    tracing::error!(correlation_id, what, "mutex was poisoned by a prior panic");
    AppError::Unexpected {
        message: "an unexpected error occurred".to_string(),
        correlation_id,
    }
}
