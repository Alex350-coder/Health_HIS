//! Cross-service integration tests (Rule 15.2, Tasks.md Task 2.9) against a real tempfile
//! SQLCipher database, exercising multiple services together the way a real session would —
//! unlike the per-service unit tests, which each isolate one function's behavior.
//!
//! `commands::{auth_commands,audit_commands}` are intentionally not exercised here: they are
//! documented thin adapters (Architecture.md — "Commands are thin adapters: validate, call one
//! service method, map the result") with no branching logic of their own, and require a live
//! `tauri::App`/WebView2 runtime to invoke, which is unavailable in this headless test
//! environment. The service layer below is what commands call into and is the layer that
//! actually needs cross-cutting integration coverage.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};
use std::thread;

use health_project::db::{connection, migrator};
use health_project::errors::AppError;
use health_project::services::{audit_service, auth_service};
use health_project::validation::auth_validation::{CreateUserInput, LoginInput};
use rusqlite::Connection;
use tempfile::tempdir;

const TEST_KEY: [u8; 32] = [11u8; 32];

fn migrated_connection(dir: &std::path::Path, name: &str) -> Connection {
    let conn = connection::open(&dir.join(name), &TEST_KEY).unwrap();
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
fn concurrent_bootstrap_attempts_produce_exactly_one_winner() {
    let dir = tempdir().unwrap();
    let conn = migrated_connection(dir.path(), "race-bootstrap.sqlite");
    let db = Arc::new(Mutex::new(conn));

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                let mut conn = db.lock().unwrap();
                auth_service::bootstrap_admin(
                    &mut conn,
                    &CreateUserInput {
                        username: format!("admin-{i}"),
                        ..admin_input()
                    },
                )
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let conflicts = results
        .iter()
        .filter(|r| matches!(r, Err(AppError::Conflict { .. })))
        .count();

    assert_eq!(successes, 1, "exactly one concurrent bootstrap must win");
    assert_eq!(
        conflicts, 7,
        "every other attempt must observe the conflict"
    );
}

/// Bootstrap -> create a second user -> that user locks itself out -> the admin's own
/// activity plus the lockout are all reflected in a still-valid audit hash chain.
#[test]
fn a_full_bootstrap_create_lockout_and_audit_walkthrough_stays_consistent() {
    let dir = tempdir().unwrap();
    let mut conn = migrated_connection(dir.path(), "walkthrough.sqlite");

    let bootstrap = auth_service::bootstrap_admin(&mut conn, &admin_input()).unwrap();

    let nurse_input = CreateUserInput {
        full_name: "Grace Hopper".to_string(),
        username: "grace.hopper".to_string(),
        password: "Another-Str0ng-Pass".to_string(),
        role: "nurse".to_string(),
    };
    let nurse = auth_service::create_user(&mut conn, bootstrap.user.id, &nurse_input).unwrap();

    for _ in 0..5 {
        let outcome = auth_service::login(
            &mut conn,
            &LoginInput {
                username: nurse_input.username.clone(),
                password: "wrong-password".to_string(),
            },
        );
        assert!(outcome.is_err());
    }

    let locked_out = auth_service::login(
        &mut conn,
        &LoginInput {
            username: nurse_input.username.clone(),
            password: nurse_input.password.clone(),
        },
    );
    assert!(
        matches!(locked_out, Err(AppError::AccountLocked { .. })),
        "five failures must lock the account even with the correct password on the sixth try"
    );

    let users = auth_service::list_users(&conn).unwrap();
    assert_eq!(users.len(), 2);

    let deactivated =
        auth_service::deactivate_user(&mut conn, bootstrap.user.id, nurse.id).unwrap();
    assert!(!deactivated.is_active);

    let verification = audit_service::verify_chain(&conn).unwrap();
    assert!(
        verification.is_valid,
        "the hash chain must still verify after bootstrap, user creation, 6 login attempts, \
         and deactivation"
    );
}
