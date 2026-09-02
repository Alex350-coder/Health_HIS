//! Forward-only SQL migration runner (Database.md Section 5, Rules.md 9.2).

use rusqlite::Connection;

use super::DbError;

/// Migrations embedded at compile time, in application order. Later phases append new
/// `(filename, sql)` entries here as their migration files are added under `migrations/` —
/// never edit or remove an existing entry (Rules.md 9.2: migrations are forward-only).
pub fn embedded_migrations() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "0000_schema_migrations.sql",
            include_str!("../../migrations/0000_schema_migrations.sql"),
        ),
        (
            "0001_init_auth.sql",
            include_str!("../../migrations/0001_init_auth.sql"),
        ),
        (
            "0002_audit.sql",
            include_str!("../../migrations/0002_audit.sql"),
        ),
        (
            "0003_patients_encounters.sql",
            include_str!("../../migrations/0003_patients_encounters.sql"),
        ),
        (
            "0004_medical_history.sql",
            include_str!("../../migrations/0004_medical_history.sql"),
        ),
        (
            "0005_hospital_map_beds.sql",
            include_str!("../../migrations/0005_hospital_map_beds.sql"),
        ),
        (
            "0006_operating_rooms.sql",
            include_str!("../../migrations/0006_operating_rooms.sql"),
        ),
        (
            "0007_inventory.sql",
            include_str!("../../migrations/0007_inventory.sql"),
        ),
        (
            "0008_notifications.sql",
            include_str!("../../migrations/0008_notifications.sql"),
        ),
        (
            "0009_billing.sql",
            include_str!("../../migrations/0009_billing.sql"),
        ),
    ]
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

        assert_eq!(migration_count(&conn), 10);
        let version: String = conn
            .query_row(
                "SELECT version FROM schema_migrations ORDER BY version LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "0000_schema_migrations.sql");
    }

    #[test]
    fn running_twice_is_idempotent() {
        let dir = tempdir().unwrap();
        let conn = connection::open(&dir.path().join("idempotent.sqlite"), &TEST_KEY).unwrap();

        run_migrations(&conn, embedded_migrations()).unwrap();
        run_migrations(&conn, embedded_migrations()).unwrap();

        assert_eq!(migration_count(&conn), 10);
    }

    /// Rules.md 9.6 — applies every migration to an empty file, then inserts one row into every
    /// table in migration order, catching forward-referencing foreign key bugs.
    #[test]
    fn inserts_one_row_into_every_table_in_migration_order() {
        let dir = tempdir().unwrap();
        let conn = connection::open(&dir.path().join("seed.sqlite"), &TEST_KEY).unwrap();

        run_migrations(&conn, embedded_migrations()).unwrap();

        conn.execute(
            "INSERT INTO users (full_name, username, password_hash, role) \
             VALUES ('Test Admin', 'test-admin', 'argon2id$dummy', 'admin')",
            [],
        )
        .unwrap();
        let user_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO sessions (user_id, token_hash, expires_at) \
             VALUES (?1, 'dummy-token-hash', datetime('now', '+8 hours'))",
            [user_id],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO audit_log \
             (user_id, action, entity_type, entity_id, result, prev_hash, row_hash) \
             VALUES (?1, 'auth.login', 'user', ?1, 'success', 'GENESIS', 'dummy-row-hash')",
            [user_id],
        )
        .unwrap();

        let audit_count: i64 = conn
            .query_row("SELECT count(*) FROM audit_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!(audit_count, 1);

        conn.execute(
            "INSERT INTO notifications (type, message) VALUES ('other', 'test notification')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO patients (medical_record_number, full_name, date_of_birth, sex) \
             VALUES ('MRN-0001', 'Test Patient', '1990-01-01', 'unknown')",
            [],
        )
        .unwrap();
        let patient_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO encounters (patient_id, created_by_user_id) VALUES (?1, ?2)",
            [patient_id, user_id],
        )
        .unwrap();
        let encounter_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO billing_simulations (encounter_id, generated_by_user_id) \
             VALUES (?1, ?2)",
            [encounter_id, user_id],
        )
        .unwrap();
        let simulation_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO billing_items (billing_simulation_id, description, source, amount) \
             VALUES (?1, 'test item', 'other', 1.0)",
            [simulation_id],
        )
        .unwrap();
    }

    #[test]
    fn audit_log_rejects_update_and_delete() {
        let dir = tempdir().unwrap();
        let conn = connection::open(&dir.path().join("append-only.sqlite"), &TEST_KEY).unwrap();
        run_migrations(&conn, embedded_migrations()).unwrap();

        conn.execute(
            "INSERT INTO audit_log (action, entity_type, result, prev_hash, row_hash) \
             VALUES ('auth.login', 'user', 'success', 'GENESIS', 'dummy-row-hash')",
            [],
        )
        .unwrap();

        let update_result =
            conn.execute("UPDATE audit_log SET action = 'tampered' WHERE id = 1", []);
        assert!(update_result.is_err());

        let delete_result = conn.execute("DELETE FROM audit_log WHERE id = 1", []);
        assert!(delete_result.is_err());
    }
}
