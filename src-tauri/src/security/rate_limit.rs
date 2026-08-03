//! Login rate limiting / brute-force lockout (Security.md Section 3). The schema carries no
//! separate "how many times has this account been locked" counter — `failed_login_attempts` is
//! deliberately never reset except on a successful login, so the lockout number a given failure
//! triggers (and thus its escalating duration) is derived from the attempt count itself
//! (`attempts / 5`), rather than adding a new column for the same information.

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::DbError;
use crate::errors::AppError;

const LOCKOUT_THRESHOLD: i64 = 5;

/// The pure decision made after one failed login attempt, given the account's attempt count
/// *before* this failure. Unit-testable without a database.
pub struct LockoutDecision {
    pub failed_attempts: i64,
    pub lockout_offset: Option<&'static str>,
}

pub fn decide_after_failure(attempts_before: i64) -> LockoutDecision {
    let failed_attempts = attempts_before + 1;
    let lockout_offset = if failed_attempts % LOCKOUT_THRESHOLD == 0 {
        let lockout_number = failed_attempts / LOCKOUT_THRESHOLD;
        Some(match lockout_number {
            1 => "+1 minutes",
            2 => "+5 minutes",
            _ => "+15 minutes",
        })
    } else {
        None
    };

    LockoutDecision {
        failed_attempts,
        lockout_offset,
    }
}

pub fn record_login_failure(
    conn: &Connection,
    user_id: i64,
    attempts_before: i64,
) -> Result<(), AppError> {
    let decision = decide_after_failure(attempts_before);

    match decision.lockout_offset {
        Some(offset) => conn.execute(
            "UPDATE users SET failed_login_attempts = ?1, locked_until = datetime('now', ?2) \
             WHERE id = ?3",
            params![decision.failed_attempts, offset, user_id],
        ),
        None => conn.execute(
            "UPDATE users SET failed_login_attempts = ?1 WHERE id = ?2",
            params![decision.failed_attempts, user_id],
        ),
    }
    .map_err(DbError::from)?;

    Ok(())
}

pub fn record_login_success(conn: &Connection, user_id: i64) -> Result<(), AppError> {
    conn.execute(
        "UPDATE users SET failed_login_attempts = 0, locked_until = NULL WHERE id = ?1",
        params![user_id],
    )
    .map_err(DbError::from)?;

    Ok(())
}

/// `Some(retry_after_secs)` when the account is currently locked, `None` otherwise. The caller
/// (`auth_service::login`) maps `Some` to `AppError::AccountLocked` *before* touching the
/// password hash, so a locked-out attempt never pays (or leaks the timing of) a hash comparison.
pub fn currently_locked_for(conn: &Connection, user_id: i64) -> Result<Option<i64>, AppError> {
    let retry_after_secs: Option<i64> = conn
        .query_row(
            "SELECT CAST(\
                (julianday(locked_until) - julianday('now')) * 86400 AS INTEGER\
             ) \
             FROM users \
             WHERE id = ?1 AND locked_until IS NOT NULL AND locked_until > datetime('now')",
            params![user_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(DbError::from)?;

    Ok(retry_after_secs.filter(|secs| *secs > 0))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [9u8; 32];

    fn seeded_connection(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("rate-limit-test.sqlite"), &TEST_KEY).unwrap();
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
    fn first_four_failures_do_not_lock() {
        for attempts_before in 0..4 {
            let decision = decide_after_failure(attempts_before);
            assert!(decision.lockout_offset.is_none());
        }
    }

    #[test]
    fn fifth_failure_locks_for_one_minute() {
        let decision = decide_after_failure(4);
        assert_eq!(decision.failed_attempts, 5);
        assert_eq!(decision.lockout_offset, Some("+1 minutes"));
    }

    #[test]
    fn tenth_failure_locks_for_five_minutes() {
        let decision = decide_after_failure(9);
        assert_eq!(decision.failed_attempts, 10);
        assert_eq!(decision.lockout_offset, Some("+5 minutes"));
    }

    #[test]
    fn fifteenth_failure_locks_for_fifteen_minutes() {
        let decision = decide_after_failure(14);
        assert_eq!(decision.failed_attempts, 15);
        assert_eq!(decision.lockout_offset, Some("+15 minutes"));
    }

    #[test]
    fn twentieth_failure_stays_capped_at_fifteen_minutes() {
        let decision = decide_after_failure(19);
        assert_eq!(decision.failed_attempts, 20);
        assert_eq!(decision.lockout_offset, Some("+15 minutes"));
    }

    #[test]
    fn record_login_failure_persists_the_attempt_count() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        record_login_failure(&conn, user_id, 0).unwrap();

        let attempts: i64 = conn
            .query_row(
                "SELECT failed_login_attempts FROM users WHERE id = ?1",
                params![user_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(attempts, 1);
    }

    #[test]
    fn fifth_recorded_failure_locks_the_account() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        for attempts_before in 0..5 {
            record_login_failure(&conn, user_id, attempts_before).unwrap();
        }

        let retry_after = currently_locked_for(&conn, user_id).unwrap();
        assert!(retry_after.is_some());
    }

    #[test]
    fn success_resets_attempts_and_clears_lockout() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        for attempts_before in 0..5 {
            record_login_failure(&conn, user_id, attempts_before).unwrap();
        }
        record_login_success(&conn, user_id).unwrap();

        let attempts: i64 = conn
            .query_row(
                "SELECT failed_login_attempts FROM users WHERE id = ?1",
                params![user_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(attempts, 0);
        assert!(currently_locked_for(&conn, user_id).unwrap().is_none());
    }

    #[test]
    fn an_account_with_no_lockout_is_not_locked() {
        let dir = tempdir().unwrap();
        let conn = seeded_connection(dir.path());
        let user_id = conn.last_insert_rowid();

        assert!(currently_locked_for(&conn, user_id).unwrap().is_none());
    }
}
