//! `sessions` domain model (Database.md Section 3.1). Deliberately excludes `token_hash` — no
//! IPC command returns session rows to the frontend (`security/session.rs` handles the token
//! itself), but the exclusion is kept here too in case a future admin-facing "active sessions"
//! view is added.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub created_at: String,
    pub expires_at: String,
    pub last_active_at: String,
}
