// Rules.md 8.3 — panics are denied repository-wide; enforced here at the crate root so the
// lint applies to every module added in later phases, not only to this file.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used)]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("fatal: failed to start the Tauri application: {error}");
            std::process::exit(1);
        });
}
