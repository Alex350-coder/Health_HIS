//! Mechanically enforces Architecture.md Section 3 — "the Hospital Map never modifies data" — as
//! a CI-checked invariant rather than a convention. Every public command in
//! `commands/hospital_map_commands.rs` must be a read (`hospital_map_get_*`); any future
//! contributor adding a mutating `hospital_map_*` command fails this test.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;

#[test]
fn every_hospital_map_command_is_a_read() {
    let source = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/commands/hospital_map_commands.rs"
    ))
    .expect("hospital_map_commands.rs must exist");

    let command_names: Vec<&str> = source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("pub fn ")
                .and_then(|rest| rest.split(['(', '<']).next())
        })
        .collect();

    assert!(
        !command_names.is_empty(),
        "expected at least one hospital_map_* command to check"
    );

    for name in command_names {
        assert!(
            name.starts_with("hospital_map_get_"),
            "hospital_map_commands.rs must only expose read-only commands, found `{name}`"
        );
    }
}
