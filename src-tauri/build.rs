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
    "patients_create",
    "patients_update",
    "patients_get",
    "patients_list",
    "medical_history_create_encounter",
    "medical_history_discharge_encounter",
    "medical_history_get_by_patient",
    "medical_history_create_diagnosis",
    "medical_history_create_treatment",
    "medical_history_create_evolution",
    "hospital_map_get_layout",
    "hospital_map_get_room_status",
    "beds_create_floor",
    "beds_create_room",
    "beds_create",
    "beds_set_status",
    "beds_list",
    "beds_assign",
    "beds_release",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .unwrap_or_else(|error| panic!("failed to run tauri-build codegen: {error}"));
}
