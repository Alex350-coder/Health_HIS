//! Authentication commands (IPC.md Section 2). Exactly three of these are session-exempt —
//! `auth_bootstrap_status`, `auth_bootstrap_admin`, `auth_login` — the complete and closed set
//! per Rule from IPC.md Section 1. Every other command here calls `require_active_session` first.

use std::sync::Mutex;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::commands::{
    clear_active_token, lock_connection, require_active_session, set_active_token,
};
use crate::errors::AppError;
use crate::models::User;
use crate::services::auth_service;
use crate::validation::auth_validation::{CreateUserInput, LoginInput};
use crate::ActiveSession;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapStatusResponse {
    pub needs_bootstrap: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponsePayload {
    pub token: String,
    pub user: User,
}

impl From<auth_service::SessionResponse> for SessionResponsePayload {
    fn from(response: auth_service::SessionResponse) -> Self {
        Self {
            token: response.token,
            user: response.user,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserResponse {
    pub user: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeactivateUserInput {
    pub id: i64,
}

#[tauri::command]
pub fn auth_bootstrap_status(
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<BootstrapStatusResponse, AppError> {
    let conn = lock_connection(&state)?;
    Ok(BootstrapStatusResponse {
        needs_bootstrap: auth_service::bootstrap_status(&conn)?,
    })
}

#[tauri::command]
pub fn auth_bootstrap_admin(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateUserInput,
) -> Result<SessionResponsePayload, AppError> {
    let mut conn = lock_connection(&state)?;
    let response = auth_service::bootstrap_admin(&mut conn, &input)?;
    set_active_token(&active_session, response.token.clone())?;
    Ok(response.into())
}

#[tauri::command]
pub fn auth_login(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: LoginInput,
) -> Result<SessionResponsePayload, AppError> {
    let mut conn = lock_connection(&state)?;
    let response = auth_service::login(&mut conn, &input)?;
    set_active_token(&active_session, response.token.clone())?;
    Ok(response.into())
}

#[tauri::command]
pub fn auth_logout(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
) -> Result<(), AppError> {
    let conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    let token = clear_active_token(&active_session)?.ok_or(AppError::Unauthorized)?;
    auth_service::logout(&conn, authenticated.user_id, &token)
}

#[tauri::command]
pub fn auth_current_user(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
) -> Result<CurrentUserResponse, AppError> {
    let conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    Ok(CurrentUserResponse {
        user: auth_service::current_user(&conn, authenticated.user_id)?,
    })
}

#[tauri::command]
pub fn auth_create_user(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: CreateUserInput,
) -> Result<User, AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    auth_service::create_user(&mut conn, authenticated.user_id, &input)
}

#[tauri::command]
pub fn auth_list_users(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
) -> Result<Vec<User>, AppError> {
    let conn = lock_connection(&state)?;
    require_active_session(&conn, &active_session)?;
    auth_service::list_users(&conn)
}

#[tauri::command]
pub fn auth_deactivate_user(
    state: tauri::State<'_, Mutex<Connection>>,
    active_session: tauri::State<'_, ActiveSession>,
    input: DeactivateUserInput,
) -> Result<User, AppError> {
    let mut conn = lock_connection(&state)?;
    let authenticated = require_active_session(&conn, &active_session)?;
    auth_service::deactivate_user(&mut conn, authenticated.user_id, input.id)
}
