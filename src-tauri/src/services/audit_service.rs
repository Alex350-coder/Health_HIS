//! Audit hash-chain writer/verifier (Audit.md Sections 3-4). `record` is the single code path
//! every other service calls to write an audit row (Rule 17.4) — it must be called from inside
//! the same transaction as the action it describes, except for pre-transaction validation
//! failures, which are audit-worthy on their own (Audit.md Section 3).

use rusqlite::Connection;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::db::DbError;
use crate::errors::AppError;
use crate::models::AuditLogEntry;
use crate::repositories::audit_repository::{self, ListFilter, NewAuditEntry};

const GENESIS_PREV_HASH: &str = "GENESIS";

pub struct RecordInput<'a> {
    pub user_id: Option<i64>,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_id: Option<i64>,
    pub before_state: Option<&'a str>,
    pub after_state: Option<&'a str>,
    pub result: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainVerification {
    pub is_valid: bool,
    pub broken_at_id: Option<i64>,
}

struct HashInput<'a> {
    prev_hash: &'a str,
    timestamp: &'a str,
    user_id: Option<i64>,
    action: &'a str,
    entity_type: &'a str,
    entity_id: Option<i64>,
    before_state: Option<&'a str>,
    after_state: Option<&'a str>,
    result: &'a str,
}

pub fn record(conn: &Connection, input: &RecordInput) -> Result<(), AppError> {
    let prev_hash = audit_repository::latest_row_hash(conn)
        .map_err(AppError::from)?
        .unwrap_or_else(|| GENESIS_PREV_HASH.to_string());
    let timestamp = current_timestamp(conn)?;
    let row_hash = compute_row_hash(&HashInput {
        prev_hash: &prev_hash,
        timestamp: &timestamp,
        user_id: input.user_id,
        action: input.action,
        entity_type: input.entity_type,
        entity_id: input.entity_id,
        before_state: input.before_state,
        after_state: input.after_state,
        result: input.result,
    });

    audit_repository::insert(
        conn,
        &NewAuditEntry {
            timestamp: &timestamp,
            user_id: input.user_id,
            action: input.action,
            entity_type: input.entity_type,
            entity_id: input.entity_id,
            before_state: input.before_state,
            after_state: input.after_state,
            result: input.result,
            prev_hash: &prev_hash,
            row_hash: &row_hash,
        },
    )
    .map_err(AppError::from)?;

    Ok(())
}

pub fn list(conn: &Connection, filter: &ListFilter) -> Result<Vec<AuditLogEntry>, AppError> {
    audit_repository::list(conn, filter).map_err(AppError::from)
}

/// Recomputes the chain from the first row and flags the first row where the stored `prev_hash`
/// no longer matches the previous row's `row_hash`, or the stored `row_hash` no longer matches
/// what the formula produces for that row's content.
pub fn verify_chain(conn: &Connection) -> Result<ChainVerification, AppError> {
    let rows = audit_repository::list_all_ordered(conn).map_err(AppError::from)?;
    let mut expected_prev_hash = GENESIS_PREV_HASH.to_string();

    for row in &rows {
        if row.prev_hash != expected_prev_hash {
            return Ok(ChainVerification {
                is_valid: false,
                broken_at_id: Some(row.id),
            });
        }

        let recomputed = compute_row_hash(&HashInput {
            prev_hash: &row.prev_hash,
            timestamp: &row.timestamp,
            user_id: row.user_id,
            action: &row.action,
            entity_type: &row.entity_type,
            entity_id: row.entity_id,
            before_state: row.before_state.as_deref(),
            after_state: row.after_state.as_deref(),
            result: &row.result,
        });

        if recomputed != row.row_hash {
            return Ok(ChainVerification {
                is_valid: false,
                broken_at_id: Some(row.id),
            });
        }

        expected_prev_hash.clone_from(&row.row_hash);
    }

    Ok(ChainVerification {
        is_valid: true,
        broken_at_id: None,
    })
}

fn current_timestamp(conn: &Connection) -> Result<String, AppError> {
    conn.query_row("SELECT datetime('now')", [], |row| row.get(0))
        .map_err(DbError::from)
        .map_err(AppError::from)
}

/// Audit.md Section 4 — `SHA-256(prev_hash || timestamp || user_id || action || entity_type ||
/// entity_id || before_state || after_state || result)`. Absent optional fields contribute an
/// empty segment; this convention only has to be internally consistent between `record` (write)
/// and `verify_chain` (recompute), not match any external representation.
fn compute_row_hash(input: &HashInput) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.prev_hash.as_bytes());
    hasher.update(input.timestamp.as_bytes());
    hasher.update(opt_i64_to_string(input.user_id).as_bytes());
    hasher.update(input.action.as_bytes());
    hasher.update(input.entity_type.as_bytes());
    hasher.update(opt_i64_to_string(input.entity_id).as_bytes());
    hasher.update(input.before_state.unwrap_or("").as_bytes());
    hasher.update(input.after_state.unwrap_or("").as_bytes());
    hasher.update(input.result.as_bytes());
    encode_hex(&hasher.finalize())
}

fn opt_i64_to_string(value: Option<i64>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};
    use crate::repositories::user_repository::{self, NewUser};

    const TEST_KEY: [u8; 32] = [5u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("audit-service-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn
    }

    /// `audit_log.user_id` is a foreign key to `users(id)` — the hash-chain content itself
    /// doesn't care whether the referenced user exists, but the DB constraint does.
    fn seed_user(conn: &Connection) -> i64 {
        user_repository::insert(
            conn,
            &NewUser {
                full_name: "Ada Lovelace",
                username: "ada",
                password_hash: "argon2id$dummy",
                role: "admin",
            },
        )
        .unwrap()
    }

    fn login_success(user_id: i64) -> RecordInput<'static> {
        RecordInput {
            user_id: Some(user_id),
            action: "auth.login",
            entity_type: "user",
            entity_id: Some(user_id),
            before_state: None,
            after_state: None,
            result: "success",
        }
    }

    #[test]
    fn the_first_row_uses_the_genesis_prev_hash() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);

        record(&conn, &login_success(user_id)).unwrap();

        let rows = audit_repository::list_all_ordered(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].prev_hash, GENESIS_PREV_HASH);
    }

    #[test]
    fn a_freshly_recorded_chain_verifies_as_valid() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);

        record(&conn, &login_success(user_id)).unwrap();
        record(&conn, &login_success(user_id)).unwrap();
        record(&conn, &login_success(user_id)).unwrap();

        let verification = verify_chain(&conn).unwrap();
        assert!(verification.is_valid);
        assert!(verification.broken_at_id.is_none());
    }

    #[test]
    fn an_empty_log_verifies_as_valid() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        let verification = verify_chain(&conn).unwrap();
        assert!(verification.is_valid);
    }

    #[test]
    fn detects_a_row_whose_hash_does_not_match_its_content() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);

        record(&conn, &login_success(user_id)).unwrap();
        let good_prev_hash = audit_repository::latest_row_hash(&conn).unwrap().unwrap();

        // Simulates tampering reachable only by bypassing the application entirely (direct file
        // access) — the append-only trigger blocks UPDATE, so this models the tamper as an insert
        // whose row_hash was never actually produced by `compute_row_hash`.
        let tampered_id = audit_repository::insert(
            &conn,
            &NewAuditEntry {
                timestamp: "2026-01-01T00:00:01",
                user_id: Some(user_id),
                action: "auth.logout",
                entity_type: "user",
                entity_id: Some(user_id),
                before_state: None,
                after_state: None,
                result: "success",
                prev_hash: &good_prev_hash,
                row_hash: "not-a-real-hash",
            },
        )
        .unwrap();

        let verification = verify_chain(&conn).unwrap();
        assert!(!verification.is_valid);
        assert_eq!(verification.broken_at_id, Some(tampered_id));
    }

    #[test]
    fn detects_a_broken_prev_hash_link() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);

        record(&conn, &login_success(user_id)).unwrap();

        let tampered_id = audit_repository::insert(
            &conn,
            &NewAuditEntry {
                timestamp: "2026-01-01T00:00:01",
                user_id: Some(user_id),
                action: "auth.logout",
                entity_type: "user",
                entity_id: Some(user_id),
                before_state: None,
                after_state: None,
                result: "success",
                prev_hash: "wrong-prev-hash",
                row_hash: "irrelevant",
            },
        )
        .unwrap();

        let verification = verify_chain(&conn).unwrap();
        assert!(!verification.is_valid);
        assert_eq!(verification.broken_at_id, Some(tampered_id));
    }
}
