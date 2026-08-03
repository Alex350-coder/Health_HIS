//! Session issuance and validation (Security.md Section 4). The opaque 256-bit token is returned
//! to the client exactly once, at creation; only its SHA-256 hash is ever persisted, so a stolen
//! database file alone cannot be replayed as a valid session. Expiry (8h absolute) and idle
//! timeout (15min since last activity) are both evaluated in SQL against `datetime('now')`, per
//! this phase's decision to keep relative-time arithmetic at the query site (no new time crate).

use rand::rngs::OsRng;
use rand::TryRngCore;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use crate::db::DbError;
use crate::errors::app_error::correlation_id;
use crate::errors::AppError;

const TOKEN_LEN_BYTES: usize = 32;

pub struct IssuedSession {
    pub token: String,
    pub user_id: i64,
}

pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
    pub role: String,
}

pub fn create_session(conn: &Connection, user_id: i64) -> Result<IssuedSession, AppError> {
    let token = generate_token()?;
    let token_hash = hash_token(&token);

    conn.execute(
        "INSERT INTO sessions (user_id, token_hash, expires_at, last_active_at) \
         VALUES (?1, ?2, datetime('now', '+8 hours'), datetime('now'))",
        params![user_id, token_hash],
    )
    .map_err(DbError::from)?;

    Ok(IssuedSession { token, user_id })
}

pub fn require_session(conn: &Connection, token: &str) -> Result<AuthenticatedUser, AppError> {
    let token_hash = hash_token(token);

    let found = conn
        .query_row(
            "SELECT sessions.id, sessions.user_id, users.username, users.role, users.is_active \
             FROM sessions \
             JOIN users ON users.id = sessions.user_id \
             WHERE sessions.token_hash = ?1 \
               AND sessions.expires_at > datetime('now') \
               AND sessions.last_active_at > datetime('now', '-15 minutes')",
            params![token_hash],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()
        .map_err(DbError::from)?;

    let (session_id, user_id, username, role, is_active) = found.ok_or(AppError::Unauthorized)?;

    if is_active == 0 {
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])
            .map_err(DbError::from)?;
        return Err(AppError::Unauthorized);
    }

    conn.execute(
        "UPDATE sessions SET last_active_at = datetime('now') WHERE id = ?1",
        params![session_id],
    )
    .map_err(DbError::from)?;

    Ok(AuthenticatedUser {
        user_id,
        username,
        role,
    })
}

pub fn delete_session(conn: &Connection, token: &str) -> Result<(), AppError> {
    let token_hash = hash_token(token);
    conn.execute(
        "DELETE FROM sessions WHERE token_hash = ?1",
        params![token_hash],
    )
    .map_err(DbError::from)?;
    Ok(())
}

fn generate_token() -> Result<String, AppError> {
    let mut bytes = [0u8; TOKEN_LEN_BYTES];
    OsRng.try_fill_bytes(&mut bytes).map_err(|_| {
        let correlation_id = correlation_id();
        tracing::error!(
            correlation_id,
            "CSPRNG failure while generating a session token"
        );
        AppError::Unexpected {
            message: "failed to generate a session token".to_string(),
            correlation_id,
        }
    })?;
    Ok(encode_hex(&bytes))
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    encode_hex(&hasher.finalize())
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

    const TEST_KEY: [u8; 32] = [7u8; 32];

    fn seeded_connection(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("session-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn.execute(
            "INSERT INTO users (full_name, username, password_hash, role) \
             VALUES ('Test User', 'test-user', 'argon2id$dummy', 'admin')",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn issues_a_session_that_require_session_accepts() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        let issued = create_session(&conn, user_id).unwrap();
        let authenticated = require_session(&conn, &issued.token).unwrap();

        assert_eq!(authenticated.user_id, user_id);
        assert_eq!(authenticated.username, "test-user");
    }

    #[test]
    fn rejects_an_unknown_token() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());

        let result = require_session(&conn, "not-a-real-token");
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn rejects_an_expired_session() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        let issued = create_session(&conn, user_id).unwrap();
        conn.execute(
            "UPDATE sessions SET expires_at = datetime('now', '-1 minute') WHERE user_id = ?1",
            params![user_id],
        )
        .unwrap();

        let result = require_session(&conn, &issued.token);
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn rejects_a_session_idle_for_more_than_fifteen_minutes() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        let issued = create_session(&conn, user_id).unwrap();
        conn.execute(
            "UPDATE sessions SET last_active_at = datetime('now', '-16 minutes') \
             WHERE user_id = ?1",
            params![user_id],
        )
        .unwrap();

        let result = require_session(&conn, &issued.token);
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn rejects_a_session_belonging_to_a_deactivated_user() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        let issued = create_session(&conn, user_id).unwrap();
        conn.execute(
            "UPDATE users SET is_active = 0 WHERE id = ?1",
            params![user_id],
        )
        .unwrap();

        let result = require_session(&conn, &issued.token);
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn logout_deletes_the_session_immediately() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        let issued = create_session(&conn, user_id).unwrap();
        delete_session(&conn, &issued.token).unwrap();

        let result = require_session(&conn, &issued.token);
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn two_sessions_for_the_same_user_get_distinct_tokens() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        let first = create_session(&conn, user_id).unwrap();
        let second = create_session(&conn, user_id).unwrap();

        assert_ne!(first.token, second.token);
    }
}
