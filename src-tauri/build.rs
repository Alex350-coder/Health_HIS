const COMMANDS: &[&str] = &[
    "auth_bootstrap_status",
    "auth_bootstrap_admin",
    "auth_login",
    "auth_logout",
    "auth_current_user",
    "auth_create_user",
    "auth_list_users",
    "auth_deactivate_user",
    "audit_list",
    "audit_verify_integrity",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .unwrap_or_else(|error| panic!("failed to run tauri-build codegen: {error}"));
}
