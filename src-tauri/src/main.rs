// Rules.md 8.3 — panics are denied repository-wide; enforced here at the crate root so the
// lint applies to every module added in later phases, not only to this file.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used)]

fn main() {
    health_project::run();
}
