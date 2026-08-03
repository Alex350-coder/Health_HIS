//! Pure SQL data access for `audit_log`. Exposes `insert` and read methods only — no update, no
//! delete — the repository-level half of the append-only guarantee (Audit.md Section 5); the DB
//! triggers in `0002_audit.sql` are the other, defense-in-depth half.

use rusqlite::{params, Connection, OptionalExtension, Row, ToSql};

use crate::db::DbError;
use crate::models::AuditLogEntry;

pub struct NewAuditEntry<'a> {
    /// Explicit, not `DEFAULT (datetime('now'))` — `audit_service::record` must hash the exact
    /// timestamp value that ends up stored, so it reads `datetime('now')` itself first and passes
    /// it through here rather than letting two independent evaluations of "now" risk drifting.
    pub timestamp: &'a str,
    pub user_id: Option<i64>,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_id: Option<i64>,
    pub before_state: Option<&'a str>,
    pub after_state: Option<&'a str>,
    pub result: &'a str,
    pub prev_hash: &'a str,
    pub row_hash: &'a str,
}

#[derive(Default)]
pub struct ListFilter<'a> {
    pub entity_type: Option<&'a str>,
    pub user_id: Option<i64>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
    pub limit: i64,
    pub offset: i64,
}

pub fn insert(conn: &Connection, entry: &NewAuditEntry) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO audit_log \
         (timestamp, user_id, action, entity_type, entity_id, before_state, after_state, \
          result, prev_hash, row_hash) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            entry.timestamp,
            entry.user_id,
            entry.action,
            entry.entity_type,
            entry.entity_id,
            entry.before_state,
            entry.after_state,
            entry.result,
            entry.prev_hash,
            entry.row_hash,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// The `prev_hash` for the next row to be written — `None` on an empty log, meaning the caller
/// must use the fixed genesis constant (Audit.md Section 4).
pub fn latest_row_hash(conn: &Connection) -> Result<Option<String>, DbError> {
    conn.query_row(
        "SELECT row_hash FROM audit_log ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get(0),
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list(conn: &Connection, filter: &ListFilter) -> Result<Vec<AuditLogEntry>, DbError> {
    let mut sql = String::from(
        "SELECT id, timestamp, user_id, action, entity_type, entity_id, before_state, \
         after_state, result, prev_hash, row_hash FROM audit_log WHERE 1 = 1",
    );
    let mut bindings: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(entity_type) = filter.entity_type {
        sql.push_str(" AND entity_type = ?");
        bindings.push(Box::new(entity_type.to_string()));
    }
    if let Some(user_id) = filter.user_id {
        sql.push_str(" AND user_id = ?");
        bindings.push(Box::new(user_id));
    }
    if let Some(from) = filter.from {
        sql.push_str(" AND timestamp >= ?");
        bindings.push(Box::new(from.to_string()));
    }
    if let Some(to) = filter.to {
        sql.push_str(" AND timestamp <= ?");
        bindings.push(Box::new(to.to_string()));
    }
    sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");
    bindings.push(Box::new(filter.limit));
    bindings.push(Box::new(filter.offset));

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn ToSql> = bindings.iter().map(AsRef::as_ref).collect();
    let rows = stmt.query_map(param_refs.as_slice(), map_row_to_entry)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// Every row, oldest first — the order `audit_service::verify_chain` recomputes hashes in.
pub fn list_all_ordered(conn: &Connection) -> Result<Vec<AuditLogEntry>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, user_id, action, entity_type, entity_id, before_state, \
         after_state, result, prev_hash, row_hash FROM audit_log ORDER BY id ASC",
    )?;
    let rows = stmt.query_map([], map_row_to_entry)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

fn map_row_to_entry(row: &Row) -> rusqlite::Result<AuditLogEntry> {
    Ok(AuditLogEntry {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        user_id: row.get(2)?,
        action: row.get(3)?,
        entity_type: row.get(4)?,
        entity_id: row.get(5)?,
        before_state: row.get(6)?,
        after_state: row.get(7)?,
        result: row.get(8)?,
        prev_hash: row.get(9)?,
        row_hash: row.get(10)?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [4u8; 32];
    const GENESIS_PREV_HASH: &str = "GENESIS";

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("audit-repo-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn
    }

    fn entry<'a>(action: &'a str, prev_hash: &'a str, row_hash: &'a str) -> NewAuditEntry<'a> {
        NewAuditEntry {
            timestamp: "2026-01-01T00:00:00",
            user_id: None,
            action,
            entity_type: "user",
            entity_id: None,
            before_state: None,
            after_state: None,
            result: "success",
            prev_hash,
            row_hash,
        }
    }

    #[test]
    fn latest_row_hash_is_none_on_an_empty_log() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        assert!(latest_row_hash(&conn).unwrap().is_none());
    }

    #[test]
    fn insert_then_latest_row_hash_returns_the_newest_hash() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        insert(&conn, &entry("auth.login", GENESIS_PREV_HASH, "hash-1")).unwrap();
        insert(&conn, &entry("auth.logout", "hash-1", "hash-2")).unwrap();

        assert_eq!(latest_row_hash(&conn).unwrap(), Some("hash-2".to_string()));
    }

    #[test]
    fn list_all_ordered_returns_rows_oldest_first() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        insert(&conn, &entry("auth.login", GENESIS_PREV_HASH, "hash-1")).unwrap();
        insert(&conn, &entry("auth.logout", "hash-1", "hash-2")).unwrap();

        let rows = list_all_ordered(&conn).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].action, "auth.login");
        assert_eq!(rows[1].action, "auth.logout");
    }

    #[test]
    fn list_filters_by_entity_type() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        insert(&conn, &entry("auth.login", GENESIS_PREV_HASH, "hash-1")).unwrap();
        insert(
            &conn,
            &NewAuditEntry {
                entity_type: "patient",
                ..entry("patient.create", "hash-1", "hash-2")
            },
        )
        .unwrap();

        let rows = list(
            &conn,
            &ListFilter {
                entity_type: Some("patient"),
                limit: 10,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].entity_type, "patient");
    }

    #[test]
    fn list_respects_limit_and_offset() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        insert(&conn, &entry("auth.login", GENESIS_PREV_HASH, "hash-1")).unwrap();
        insert(&conn, &entry("auth.logout", "hash-1", "hash-2")).unwrap();
        insert(&conn, &entry("auth.login", "hash-2", "hash-3")).unwrap();

        let page = list(
            &conn,
            &ListFilter {
                limit: 1,
                offset: 1,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(page.len(), 1);
        assert_eq!(page[0].row_hash, "hash-2");
    }
}
