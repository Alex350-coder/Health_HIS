//! Pure SQL data access for `users` (Architecture.md — repositories hold no business rules and
//! make no cross-repository calls). Callers use the transactional `Connection`/`Transaction`
//! they were given, so `auth_service::bootstrap_admin`'s TOCTOU-safe `count` + `insert` can run
//! inside one transaction (Security.md Section 9.1).

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::DbError;
use crate::models::User;

pub struct NewUser<'a> {
    pub full_name: &'a str,
    pub username: &'a str,
    pub password_hash: &'a str,
    pub role: &'a str,
}

/// Includes `password_hash` — this type never leaves the backend (unlike `models::User`), it
/// exists solely for `auth_service::login` to verify a submitted password.
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub failed_login_attempts: i64,
}

pub fn count(conn: &Connection) -> Result<i64, DbError> {
    conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .map_err(DbError::from)
}

pub fn insert(conn: &Connection, new_user: &NewUser) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO users (full_name, username, password_hash, role) VALUES (?1, ?2, ?3, ?4)",
        params![
            new_user.full_name,
            new_user.username,
            new_user.password_hash,
            new_user.role
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_by_username(conn: &Connection, username: &str) -> Result<Option<UserRecord>, DbError> {
    conn.query_row(
        "SELECT id, username, password_hash, role, is_active, failed_login_attempts \
         FROM users WHERE username = ?1",
        params![username],
        map_row_to_record,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<User>, DbError> {
    conn.query_row(
        "SELECT id, full_name, username, role, is_active, created_at, updated_at \
         FROM users WHERE id = ?1",
        params![id],
        map_row_to_user,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list(conn: &Connection) -> Result<Vec<User>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, full_name, username, role, is_active, created_at, updated_at \
         FROM users ORDER BY id",
    )?;
    let rows = stmt.query_map([], map_row_to_user)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn deactivate(conn: &Connection, id: i64) -> Result<Option<User>, DbError> {
    conn.execute(
        "UPDATE users SET is_active = 0, updated_at = datetime('now') WHERE id = ?1",
        params![id],
    )?;
    find_by_id(conn, id)
}

fn map_row_to_record(row: &Row) -> rusqlite::Result<UserRecord> {
    Ok(UserRecord {
        id: row.get(0)?,
        username: row.get(1)?,
        password_hash: row.get(2)?,
        role: row.get(3)?,
        is_active: row.get::<_, i64>(4)? != 0,
        failed_login_attempts: row.get(5)?,
    })
}

fn map_row_to_user(row: &Row) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get(0)?,
        full_name: row.get(1)?,
        username: row.get(2)?,
        role: row.get(3)?,
        is_active: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [3u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("user-repo-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn
    }

    fn sample_user() -> NewUser<'static> {
        NewUser {
            full_name: "Ada Lovelace",
            username: "ada",
            password_hash: "argon2id$dummy",
            role: "admin",
        }
    }

    #[test]
    fn count_is_zero_on_an_empty_table() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        assert_eq!(count(&conn).unwrap(), 0);
    }

    #[test]
    fn insert_then_find_by_username_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        let id = insert(&conn, &sample_user()).unwrap();
        let found = find_by_username(&conn, "ada").unwrap().unwrap();

        assert_eq!(found.id, id);
        assert_eq!(found.password_hash, "argon2id$dummy");
        assert!(found.is_active);
    }

    #[test]
    fn find_by_username_returns_none_when_absent() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        assert!(find_by_username(&conn, "nobody").unwrap().is_none());
    }

    #[test]
    fn find_by_id_never_exposes_the_password_hash_type() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let id = insert(&conn, &sample_user()).unwrap();

        let user = find_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(user.username, "ada");
    }

    #[test]
    fn list_returns_every_user_in_id_order() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        insert(&conn, &sample_user()).unwrap();
        insert(
            &conn,
            &NewUser {
                full_name: "Grace Hopper",
                username: "grace",
                password_hash: "argon2id$dummy2",
                role: "nurse",
            },
        )
        .unwrap();

        let users = list(&conn).unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "ada");
        assert_eq!(users[1].username, "grace");
    }

    #[test]
    fn deactivate_sets_is_active_false_and_never_deletes() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let id = insert(&conn, &sample_user()).unwrap();

        let deactivated = deactivate(&conn, id).unwrap().unwrap();
        assert!(!deactivated.is_active);
        assert_eq!(count(&conn).unwrap(), 1);
    }
}
