//! `audit_log` domain model (Database.md Section 3.7 / Audit.md). Matches the `AuditLogEntry`
//! return type named in IPC.md's `audit_list` row exactly.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub id: i64,
    pub timestamp: String,
    pub user_id: Option<i64>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub result: String,
    pub prev_hash: String,
    pub row_hash: String,
}
