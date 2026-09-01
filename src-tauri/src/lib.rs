// Rules.md 8.3 — panics are denied repository-wide; enforced here at the library root so the
// lint applies to every module, not only to `main.rs`.
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod commands;
pub mod db;
pub mod errors;
pub mod events;
pub mod models;
pub mod repositories;
pub mod security;
pub mod services;
pub mod validation;

use std::fs;
use std::sync::Mutex;

use tauri::Manager;

const DB_FILE_NAME: &str = "health.db";

/// The single active session's raw token, held in Rust process memory only (Rule 17.7 — the one
/// sanctioned mutable-singleton mechanism). See `commands/mod.rs` for the full rationale.
pub struct ActiveSession(pub Mutex<Option<String>>);

/// Bootstraps the encrypted database (open-or-create, then apply pending migrations) and
/// launches the Tauri application. Any failure here is fatal — the app cannot run without
/// a working, decrypted database connection.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::auth_commands::auth_bootstrap_status,
            commands::auth_commands::auth_bootstrap_admin,
            commands::auth_commands::auth_login,
            commands::auth_commands::auth_logout,
            commands::auth_commands::auth_current_user,
            commands::auth_commands::auth_create_user,
            commands::auth_commands::auth_list_users,
            commands::auth_commands::auth_deactivate_user,
            commands::audit_commands::audit_list,
            commands::audit_commands::audit_verify_integrity,
            commands::patient_commands::patients_create,
            commands::patient_commands::patients_update,
            commands::patient_commands::patients_get,
            commands::patient_commands::patients_list,
            commands::medical_history_commands::medical_history_create_encounter,
            commands::medical_history_commands::medical_history_discharge_encounter,
            commands::medical_history_commands::medical_history_get_by_patient,
            commands::medical_history_commands::medical_history_create_diagnosis,
            commands::medical_history_commands::medical_history_create_treatment,
            commands::medical_history_commands::medical_history_create_evolution,
            commands::hospital_map_commands::hospital_map_get_layout,
            commands::hospital_map_commands::hospital_map_get_room_status,
            commands::bed_commands::beds_create_floor,
            commands::bed_commands::beds_create_room,
            commands::bed_commands::beds_create,
            commands::bed_commands::beds_set_status,
            commands::bed_commands::beds_list,
            commands::bed_commands::beds_assign,
            commands::bed_commands::beds_release,
            commands::operating_room_commands::operating_rooms_create,
            commands::operating_room_commands::operating_rooms_list,
            commands::operating_room_commands::operating_rooms_list_reservations,
            commands::operating_room_commands::operating_rooms_reserve,
            commands::operating_room_commands::operating_rooms_update_reservation,
            commands::operating_room_commands::operating_rooms_cancel_reservation,
            commands::inventory_commands::inventory_list_categories,
            commands::inventory_commands::inventory_create_category,
            commands::inventory_commands::inventory_list_items,
            commands::inventory_commands::inventory_create_item,
            commands::inventory_commands::inventory_record_transaction,
            commands::inventory_commands::inventory_schedule_maintenance,
            commands::inventory_commands::inventory_list_transactions,
            commands::inventory_commands::inventory_list_maintenance_schedules,
            commands::notification_commands::notifications_list,
            commands::notification_commands::notifications_mark_read,
            commands::billing_commands::billing_generate_simulation,
            commands::billing_commands::billing_get_simulation,
            commands::billing_commands::billing_finalize_simulation,
        ])
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("failed to resolve app data directory: {error}"))?;
            fs::create_dir_all(&data_dir)
                .map_err(|error| format!("failed to create app data directory: {error}"))?;

            let db_path = data_dir.join(DB_FILE_NAME);
            let key = security::secrets::get_or_create_db_key()
                .map_err(|error| format!("failed to obtain database encryption key: {error}"))?;
            let conn = db::connection::open(&db_path, &key)
                .map_err(|error| format!("failed to open encrypted database: {error}"))?;
            db::migrator::run_migrations(&conn, db::migrator::embedded_migrations())
                .map_err(|error| format!("failed to apply database migrations: {error}"))?;

            app.manage(Mutex::new(conn));
            app.manage(ActiveSession(Mutex::new(None)));
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("fatal: failed to start the Tauri application: {error}");
            std::process::exit(1);
        });
}
