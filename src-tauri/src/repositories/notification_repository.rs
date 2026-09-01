//! Pure SQL data access for `notifications` (Architecture.md — repositories hold no business
//! rules and make no cross-repository calls). Database.md Section 3.6.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::DbError;
use crate::models::Notification;

pub struct NewNotification<'a> {
    pub kind: &'a str,
    pub target_role: Option<&'a str>,
    pub message: &'a str,
    pub related_entity_type: Option<&'a str>,
    pub related_entity_id: Option<i64>,
}

const COLUMNS: &str = "id, type, target_role, message, related_entity_type, related_entity_id, \
     is_read, created_at";

pub fn insert(conn: &Connection, new_notification: &NewNotification) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO notifications \
         (type, target_role, message, related_entity_type, related_entity_id) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            new_notification.kind,
            new_notification.target_role,
            new_notification.message,
            new_notification.related_entity_type,
            new_notification.related_entity_id,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<Notification>, DbError> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM notifications WHERE id = ?1"),
        params![id],
        map_row,
    )
    .optional()
    .map_err(DbError::from)
}

/// Backs `notifications_list` (IPC.md), optionally scoped to unread rows only.
pub fn list(conn: &Connection, unread_only: bool) -> Result<Vec<Notification>, DbError> {
    let where_clause = if unread_only { "WHERE is_read = 0" } else { "" };
    let sql =
        format!("SELECT {COLUMNS} FROM notifications {where_clause} ORDER BY created_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn mark_read(conn: &Connection, id: i64) -> Result<Option<Notification>, DbError> {
    conn.execute(
        "UPDATE notifications SET is_read = 1 WHERE id = ?1",
        params![id],
    )?;
    find_by_id(conn, id)
}

/// Idempotency check used by `notification_service::check_inventory_alerts` — a new alert for
/// the same `(type, related_entity_type, related_entity_id)` triple is only raised when no
/// unread one already exists for it.
pub fn exists_unread(
    conn: &Connection,
    kind: &str,
    related_entity_type: &str,
    related_entity_id: i64,
) -> Result<bool, DbError> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM notifications \
         WHERE type = ?1 AND related_entity_type = ?2 AND related_entity_id = ?3 \
         AND is_read = 0",
        params![kind, related_entity_type, related_entity_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn map_row(row: &Row) -> rusqlite::Result<Notification> {
    Ok(Notification {
        id: row.get(0)?,
        kind: row.get(1)?,
        target_role: row.get(2)?,
        message: row.get(3)?,
        related_entity_type: row.get(4)?,
        related_entity_id: row.get(5)?,
        is_read: row.get(6)?,
        created_at: row.get(7)?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [61u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("notification-repo-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn
    }

    fn seed_notification(conn: &Connection) -> i64 {
        insert(
            conn,
            &NewNotification {
                kind: "low_stock",
                target_role: None,
                message: "Ibuprofen 400mg is at or below its reorder threshold",
                related_entity_type: Some("inventory_item"),
                related_entity_id: Some(1),
            },
        )
        .unwrap()
    }

    #[test]
    fn insert_then_find_by_id_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        let id = seed_notification(&conn);
        let found = find_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.kind, "low_stock");
        assert!(!found.is_read);
    }

    #[test]
    fn list_defaults_to_all_notifications() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let id = seed_notification(&conn);
        mark_read(&conn, id).unwrap();

        let all = list(&conn, false).unwrap();
        let unread = list(&conn, true).unwrap();

        assert_eq!(all.len(), 1);
        assert!(unread.is_empty());
    }

    #[test]
    fn mark_read_flips_the_flag() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let id = seed_notification(&conn);

        let updated = mark_read(&conn, id).unwrap().unwrap();

        assert!(updated.is_read);
    }

    #[test]
    fn exists_unread_is_true_only_for_a_matching_unread_row() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let id = seed_notification(&conn);

        assert!(exists_unread(&conn, "low_stock", "inventory_item", 1).unwrap());
        assert!(!exists_unread(&conn, "low_stock", "inventory_item", 2).unwrap());

        mark_read(&conn, id).unwrap();
        assert!(!exists_unread(&conn, "low_stock", "inventory_item", 1).unwrap());
    }
}
