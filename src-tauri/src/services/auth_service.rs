//! Authentication orchestration (Security.md Sections 3-4, 9.1). Thin command layer calls into
//! this module only — it composes hashing, rate limiting, session issuance and audit writes.
//!
//! Deviation note: Security.md Section 9.1 also asks for `auth_bootstrap_status`/
//! `auth_bootstrap_admin` to be rate-limited "keyed by installation rather than account." Before
//! any user row exists there is nothing to key a persistent counter to without adding a new
//! table outside this phase's schema (Database.md 3.1/3.7 define no such table), and Tasks.md's
//! Task 2.9 test list does not exercise it. Deferred; recorded in Progress.md rather than
//! silently implemented against an invented table.

use rusqlite::{Connection, Transaction};

use crate::db::DbError;
use crate::errors::app_error::correlation_id;
use crate::errors::AppError;
use crate::models::User;
use crate::repositories::user_repository::{self, NewUser};
use crate::security::{hashing, rate_limit, session};
use crate::services::audit_service::{self, RecordInput};
use crate::validation::auth_validation::{self, CreateUserInput, LoginInput};

pub struct SessionResponse {
    pub token: String,
    pub user: User,
}

pub fn bootstrap_status(conn: &Connection) -> Result<bool, AppError> {
    Ok(user_repository::count(conn)? == 0)
}

/// Security.md Section 9.1 — the empty-`users` check and the `INSERT` happen inside one
/// transaction so two concurrent calls cannot each observe an empty table and each insert a
/// "first" admin.
pub fn bootstrap_admin(
    conn: &mut Connection,
    input: &CreateUserInput,
) -> Result<SessionResponse, AppError> {
    auth_validation::validate_create_user(input)?;
    let password_hash = hashing::hash_password(&input.password)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    if user_repository::count(&tx)? != 0 {
        return Err(AppError::Conflict {
            message: "installation is already bootstrapped".to_string(),
        });
    }
    let user_id = user_repository::insert(
        &tx,
        &NewUser {
            full_name: &input.full_name,
            username: &input.username,
            password_hash: &password_hash,
            role: "admin",
        },
    )?;
    audit_service::record(&tx, &user_created(user_id, user_id))?;
    tx.commit().map_err(DbError::from)?;

    let user = find_user_or_die(conn, user_id)?;
    let issued = session::create_session(conn, user_id)?;
    Ok(SessionResponse {
        token: issued.token,
        user,
    })
}

/// Security.md Section 3 — rate-limit check, password verification, session issuance and the
/// audit write all happen inside one transaction, including on the failure paths, since a
/// failed attempt must be recorded even though the overall call returns `Err`.
pub fn login(conn: &mut Connection, input: &LoginInput) -> Result<SessionResponse, AppError> {
    auth_validation::validate_login(input)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let outcome = attempt_login(&tx, input);
    tx.commit().map_err(DbError::from)?;
    outcome
}

fn attempt_login(tx: &Transaction, input: &LoginInput) -> Result<SessionResponse, AppError> {
    let Some(record) = user_repository::find_by_username(tx, &input.username)? else {
        audit_service::record(tx, &failed_login(None))?;
        return Err(AppError::Unauthorized);
    };

    if !record.is_active {
        audit_service::record(tx, &failed_login(Some(record.id)))?;
        return Err(AppError::Unauthorized);
    }

    if let Some(retry_after_secs) = rate_limit::currently_locked_for(tx, record.id)? {
        return Err(AppError::AccountLocked { retry_after_secs });
    }

    if !hashing::verify_password(&input.password, &record.password_hash)? {
        rate_limit::record_login_failure(tx, record.id, record.failed_login_attempts)?;
        audit_service::record(tx, &failed_login(Some(record.id)))?;
        return Err(AppError::Unauthorized);
    }

    rate_limit::record_login_success(tx, record.id)?;
    let issued = session::create_session(tx, record.id)?;
    audit_service::record(
        tx,
        &RecordInput {
            user_id: Some(record.id),
            action: "auth.login",
            entity_type: "user",
            entity_id: Some(record.id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;

    let user = find_user_or_die(tx, record.id)?;
    Ok(SessionResponse {
        token: issued.token,
        user,
    })
}

pub fn logout(conn: &Connection, user_id: i64, token: &str) -> Result<(), AppError> {
    session::delete_session(conn, token)?;
    audit_service::record(
        conn,
        &RecordInput {
            user_id: Some(user_id),
            action: "auth.logout",
            entity_type: "user",
            entity_id: Some(user_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )
}

pub fn current_user(conn: &Connection, user_id: i64) -> Result<User, AppError> {
    user_repository::find_by_id(conn, user_id)?.ok_or(AppError::NotFound {
        entity: "user".to_string(),
        id: user_id,
    })
}

pub fn create_user(
    conn: &mut Connection,
    actor_user_id: i64,
    input: &CreateUserInput,
) -> Result<User, AppError> {
    auth_validation::validate_create_user(input)?;
    let password_hash = hashing::hash_password(&input.password)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    let user_id = user_repository::insert(
        &tx,
        &NewUser {
            full_name: &input.full_name,
            username: &input.username,
            password_hash: &password_hash,
            role: &input.role,
        },
    )?;
    audit_service::record(&tx, &user_created(actor_user_id, user_id))?;
    tx.commit().map_err(DbError::from)?;

    find_user_or_die(conn, user_id)
}

pub fn list_users(conn: &Connection) -> Result<Vec<User>, AppError> {
    user_repository::list(conn).map_err(AppError::from)
}

pub fn deactivate_user(
    conn: &mut Connection,
    actor_user_id: i64,
    target_user_id: i64,
) -> Result<User, AppError> {
    let tx = conn.transaction().map_err(DbError::from)?;
    let user = user_repository::deactivate(&tx, target_user_id)?.ok_or(AppError::NotFound {
        entity: "user".to_string(),
        id: target_user_id,
    })?;
    audit_service::record(
        &tx,
        &RecordInput {
            user_id: Some(actor_user_id),
            action: "user.deactivate",
            entity_type: "user",
            entity_id: Some(target_user_id),
            before_state: None,
            after_state: None,
            result: "success",
        },
    )?;
    tx.commit().map_err(DbError::from)?;
    Ok(user)
}

fn user_created(actor_user_id: i64, created_user_id: i64) -> RecordInput<'static> {
    RecordInput {
        user_id: Some(actor_user_id),
        action: "user.create",
        entity_type: "user",
        entity_id: Some(created_user_id),
        before_state: None,
        after_state: None,
        result: "success",
    }
}

fn failed_login(user_id: Option<i64>) -> RecordInput<'static> {
    RecordInput {
        user_id,
        action: "auth.login",
        entity_type: "user",
        entity_id: user_id,
        before_state: None,
        after_state: None,
        result: "failure",
    }
}

/// A user just written in this same connection/transaction cannot legitimately be missing;
/// treated as a technical invariant violation rather than a `NotFound`.
fn find_user_or_die(conn: &Connection, user_id: i64) -> Result<User, AppError> {
    user_repository::find_by_id(conn, user_id)?.ok_or_else(|| {
        let correlation_id = correlation_id();
        tracing::error!(
            correlation_id,
            user_id,
            "user row vanished immediately after being written"
        );
        AppError::Unexpected {
            message: "an unexpected error occurred".to_string(),
            correlation_id,
        }
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [7u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("auth-service-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn
    }

    fn admin_input() -> CreateUserInput {
        CreateUserInput {
            full_name: "Ada Lovelace".to_string(),
            username: "ada.lovelace".to_string(),
            password: "Correct-Horse-9".to_string(),
            role: "admin".to_string(),
        }
    }

    #[test]
    fn bootstrap_status_is_true_on_an_empty_database() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        assert!(bootstrap_status(&conn).unwrap());
    }

    #[test]
    fn bootstrap_admin_creates_an_admin_and_returns_a_usable_session() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let response = bootstrap_admin(&mut conn, &admin_input()).unwrap();

        assert_eq!(response.user.role, "admin");
        assert!(!bootstrap_status(&conn).unwrap());

        let rows = crate::repositories::audit_repository::list_all_ordered(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].action, "user.create");
    }

    #[test]
    fn a_second_bootstrap_attempt_is_rejected_with_conflict() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        bootstrap_admin(&mut conn, &admin_input()).unwrap();

        let second = CreateUserInput {
            username: "grace.hopper".to_string(),
            ..admin_input()
        };
        let result = bootstrap_admin(&mut conn, &second);

        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn bootstrap_admin_rejects_a_weak_password() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());

        let weak = CreateUserInput {
            password: "short".to_string(),
            ..admin_input()
        };
        let result = bootstrap_admin(&mut conn, &weak);

        assert!(matches!(result, Err(AppError::Validation { .. })));
        assert!(bootstrap_status(&conn).unwrap());
    }

    #[test]
    fn a_freshly_bootstrapped_admin_can_immediately_log_in() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        bootstrap_admin(&mut conn, &admin_input()).unwrap();

        let login_input = LoginInput {
            username: "ada.lovelace".to_string(),
            password: "Correct-Horse-9".to_string(),
        };
        let response = login(&mut conn, &login_input).unwrap();

        assert_eq!(response.user.username, "ada.lovelace");
    }

    #[test]
    fn login_with_the_wrong_password_is_rejected_and_audited_as_a_failure() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        bootstrap_admin(&mut conn, &admin_input()).unwrap();

        let login_input = LoginInput {
            username: "ada.lovelace".to_string(),
            password: "totally-wrong-password".to_string(),
        };
        let result = login(&mut conn, &login_input);

        assert!(matches!(result, Err(AppError::Unauthorized)));
        let rows = crate::repositories::audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows
            .iter()
            .any(|row| row.action == "auth.login" && row.result == "failure"));
    }

    #[test]
    fn five_consecutive_failures_lock_the_account() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        bootstrap_admin(&mut conn, &admin_input()).unwrap();

        let login_input = LoginInput {
            username: "ada.lovelace".to_string(),
            password: "totally-wrong-password".to_string(),
        };
        for _ in 0..5 {
            let _ = login(&mut conn, &login_input);
        }

        let correct_login = LoginInput {
            username: "ada.lovelace".to_string(),
            password: "Correct-Horse-9".to_string(),
        };
        let result = login(&mut conn, &correct_login);

        assert!(matches!(result, Err(AppError::AccountLocked { .. })));
    }

    #[test]
    fn create_user_then_list_users_round_trips_and_hides_the_password_hash_type() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let admin = bootstrap_admin(&mut conn, &admin_input()).unwrap();

        let nurse_input = CreateUserInput {
            full_name: "Grace Hopper".to_string(),
            username: "grace.hopper".to_string(),
            password: "Correct-Horse-9".to_string(),
            role: "nurse".to_string(),
        };
        let created = create_user(&mut conn, admin.user.id, &nurse_input).unwrap();

        let users = list_users(&conn).unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(created.username, "grace.hopper");
    }

    #[test]
    fn deactivate_user_soft_deletes_and_is_audited() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let admin = bootstrap_admin(&mut conn, &admin_input()).unwrap();

        let nurse_input = CreateUserInput {
            full_name: "Grace Hopper".to_string(),
            username: "grace.hopper".to_string(),
            password: "Correct-Horse-9".to_string(),
            role: "nurse".to_string(),
        };
        let created = create_user(&mut conn, admin.user.id, &nurse_input).unwrap();

        let deactivated = deactivate_user(&mut conn, admin.user.id, created.id).unwrap();

        assert!(!deactivated.is_active);
        let rows = crate::repositories::audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "user.deactivate"));
    }

    #[test]
    fn logout_deletes_the_session_and_is_audited() {
        let dir = tempdir().unwrap();
        let mut conn = open_migrated(dir.path());
        let admin = bootstrap_admin(&mut conn, &admin_input()).unwrap();

        logout(&conn, admin.user.id, &admin.token).unwrap();

        let result = session::require_session(&conn, &admin.token);
        assert!(matches!(result, Err(AppError::Unauthorized)));
        let rows = crate::repositories::audit_repository::list_all_ordered(&conn).unwrap();
        assert!(rows.iter().any(|row| row.action == "auth.logout"));
    }
}
