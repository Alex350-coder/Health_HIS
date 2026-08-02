//! Forward-only SQL migration runner (Database.md Section 5, Rules.md 9.2).

use rusqlite::Connection;

use super::DbError;

/// Migrations embedded at compile time, in application order. Later phases append new
/// `(filename, sql)` entries here as their migration files are added under `migrations/` —
/// never edit or remove an existing entry (Rules.md 9.2: migrations are forward-only).
pub fn embedded_migrations() -> &'static [(&'static str, &'static str)] {
    &[(
        "0000_schema_migrations.sql",
        include_str!("../../migrations/0000_schema_migrations.sql"),
    )]
}

/// Applies every migration in `migrations` that is not yet recorded in `schema_migrations`,
/// each inside its own transaction, in filename order.
pub fn run_migrations(conn: &Connection, migrations: &[(&str, &str)]) -> Result<(), DbError> {
    bootstrap_schema_migrations_table(conn)?;

    let mut ordered: Vec<&(&str, &str)> = migrations.iter().collect();
    ordered.sort_by_key(|(version, _)| *version);

    for (version, sql) in ordered {
        if is_applied(conn, version)? {
            continue;
        }
        apply_migration(conn, version, sql)?;
    }

    Ok(())
}

fn bootstrap_schema_migrations_table(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;
    Ok(())
}

fn is_applied(conn: &Connection, version: &str) -> Result<bool, DbError> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM schema_migrations WHERE version = ?1",
        [version],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn apply_migration(conn: &Connection, version: &str, sql: &str) -> Result<(), DbError> {
    // `unchecked_transaction` (shared-reference variant) is used because the migrator only
    // ever receives `&Connection` — the connection itself is owned by Tauri-managed state
    // shared across the app, not by this function.
    let tx = conn.unchecked_transaction()?;
    tx.execute_batch(sql)?;
    tx.execute(
        "INSERT INTO schema_migrations (version) VALUES (?1)",
        [version],
    )?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::connection;

    const TEST_KEY: [u8; 32] = [0x33; 32];

    fn migration_count(conn: &Connection) -> i64 {
        conn.query_row("SELECT count(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    #[test]
    fn applies_pending_migrations_to_a_fresh_database() {
        let dir = tempdir().unwrap();
        let conn = connection::open(&dir.path().join("fresh.sqlite"), &TEST_KEY).unwrap();

        run_migrations(&conn, embedded_migrations()).unwrap();

        assert_eq!(migration_count(&conn), 1);
        let version: String = conn
            .query_row("SELECT version FROM schema_migrations LIMIT 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, "0000_schema_migrations.sql");
    }

    #[test]
    fn running_twice_is_idempotent() {
        let dir = tempdir().unwrap();
        let conn = connection::open(&dir.path().join("idempotent.sqlite"), &TEST_KEY).unwrap();

        run_migrations(&conn, embedded_migrations()).unwrap();
        run_migrations(&conn, embedded_migrations()).unwrap();

        assert_eq!(migration_count(&conn), 1);
    }
}
