# Testing.md

## Purpose

Defines the complete testing strategy across every layer and the coverage gates enforced in CI.

## Dependencies

[Rules](Rules.md) Section 15, [Architecture](Architecture.md) Section 6 (test tooling decisions).

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md), [Database](Database.md), [Security](Security.md), [ErrorHandling](ErrorHandling.md).

## Referenced By

[Audit](Audit.md), [Database](Database.md), [DefinitionOfDone](DefinitionOfDone.md), [DevelopmentWorkflow](DevelopmentWorkflow.md), [ErrorHandling](ErrorHandling.md), [IPC](IPC.md), [Plan](Plan.md), [README_Project](README_Project.md), [Rules](Rules.md), [Security](Security.md), [StateManagement](StateManagement.md), [Tasks](Tasks.md), [UI](UI.md), [Validation](Validation.md).

## Documents that must be updated if changes occur

[DefinitionOfDone](DefinitionOfDone.md), [Plan](Plan.md) (per-phase automatic validation), [Tasks](Tasks.md) (per-task verification process).

## Priority

7.

---

## Contents

- [1. Test Types and Tooling](#1-test-types-and-tooling)
- [2. Why Integration Tests Are Primary (Not Mocks)](#2-why-integration-tests-are-primary-not-mocks)
- [3. Coverage Gates (CI-enforced, build fails below threshold)](#3-coverage-gates-ci-enforced-build-fails-below-threshold)
- [4. End-to-End Test Scenarios (minimum set)](#4-end-to-end-test-scenarios-minimum-set)
- [5. Error Path Coverage (cross-reference [ErrorHandling](ErrorHandling.md))](#5-error-path-coverage-cross-reference-errorhandling)
- [6. Test File Location and Naming](#6-test-file-location-and-naming)
- [7. CI Pipeline Gate Order](#7-ci-pipeline-gate-order)

---

## 1. Test Types and Tooling

| Type | Tooling | Scope |
|---|---|---|
| Unit (Rust) | `cargo test`, `rstest` for parameterized cases | Pure functions, validators, business-rule methods in isolation (repository calls mocked via a trait + in-memory fake only where a service method has no DB dependency to test in isolation — otherwise prefer Integration, see Section 2). |
| Unit (TS) | Vitest | Pure utils (`shared/lib`), Zod schema edge cases, event-query-map correctness. |
| Component | Vitest + React Testing Library | Component rendering, user interaction, loading/empty/error triad (Rule from [UI](UI.md) Section 6), accessibility roles present. |
| Integration | `cargo test` against a real temporary SQLite file (`tempfile` crate), migrations applied fresh per test | Every Tauri command, every service method that touches the DB. **Never a mocked database** (Rule 15.2) — this is the primary test type for business-rule correctness (e.g., bed double-assignment prevention, OR overlap prevention). |
| Database | Integration tests + a dedicated migration test that applies all migrations to an empty file, asserts success, and then inserts one row into every table in migration order (catching forward-referencing foreign keys per [Rules](Rules.md) 9.6), plus `EXPLAIN QUERY PLAN` assertions for the indexed queries listed in [Rules](Rules.md) 18.3 | Schema correctness, constraint enforcement, index usage. |
| Security | Integration tests | Rate limiting/lockout escalation, session expiry, unauthorized command access, SQL-injection-shaped input handling (proving parameterization holds), audit hash-chain verification, first-run bootstrap gating ([Security](Security.md) Section 9.1) including the concurrent double-bootstrap case. |
| Regression | Any of the above | Every bug fix adds a test reproducing the bug (Rule 15.3), tagged with the issue reference in the test name/comment. |
| End-to-End | `tauri-driver` + WebdriverIO | Full user workflows through the real built app — see Section 4. |

## 2. Why Integration Tests Are Primary (Not Mocks)

Per [Rules](Rules.md) 15.2 and the project's "no mock data" spirit extended to testing philosophy: business rules in this system (bed availability, OR overlap, encounter status gating) are fundamentally database-state-dependent. A mocked repository can silently drift from real SQLite/foreign-key/constraint behavior, producing false-positive passing tests (this is the exact failure mode the project explicitly guards against). Integration tests against a real temporary file-based SQLite database are therefore the default; unit tests with fakes are reserved for logic with zero DB interaction (e.g., a pure date-overlap calculation function extracted from `operating_room_service.rs`).

## 3. Coverage Gates (CI-enforced, build fails below threshold)

| Layer | Minimum |
|---|---|
| Rust services + repositories | 80% line coverage (`cargo tarpaulin` or `cargo llvm-cov`) |
| TypeScript business logic (hooks, `shared/lib`, validators) | 80% (`vitest --coverage`) |
| React components | 70% |

## 4. End-to-End Test Scenarios (minimum set)

0. `bootstrap.spec.ts` — starting from a **deleted database file**: `/setup` is reachable, the first admin is created, the session works, `/setup` is no longer reachable afterwards, and a second `auth_bootstrap_admin` call is rejected ([Security](Security.md) Section 9.1). This scenario runs first because every other scenario depends on a user existing.
1. `auth.spec.ts` — login success, login failure, lockout after 5 attempts, session expiry redirect.
2. `patient-admission-flow.spec.ts` — the full workflow from [README_Project](README_Project.md) Section 2, **including its own setup preamble** (bootstrap admin -> create floor/room/bed via `/facility` -> create an operating room -> create an inventory category and item), because [Rules](Rules.md) 17.1 forbids seeding this state and the workflow cannot run without it: register patient -> create encounter -> assign bed -> register diagnosis -> register treatment -> register evolution -> schedule OR -> generate billing simulation -> discharge -> verify medical history remains queryable post-discharge.
3. `inventory-shared-consultation.spec.ts` — pharmacy-role and lab-role user both consult and transact against the same inventory item, verifying real-time propagation (Tauri event -> cache invalidation, [StateManagement](StateManagement.md) Section 4) across two simulated windows/sessions.
4. `hospital-map-readonly.spec.ts` — verifies no mutation affordance exists anywhere in the Hospital Map UI.
5. `audit-integrity.spec.ts` — performs a series of actions, calls `audit_verify_integrity`, asserts chain valid; then (test-only, direct DB manipulation) tampers a row and asserts the verification detects the break.

## 5. Error Path Coverage (cross-reference [ErrorHandling](ErrorHandling.md))

Every `AppError` variant has at least one integration test asserting: (a) the condition that produces it, (b) correct serialization across IPC, (c) the frontend's corresponding presentation pattern (component test) per [ErrorHandling](ErrorHandling.md) Section 3.

## 6. Test File Location and Naming

- Rust: colocated `#[cfg(test)] mod tests` for pure unit tests; `src-tauri/tests/commands/<module>_commands.rs` and `src-tauri/tests/services/<module>_service.rs` for integration tests, per [FolderStructure](FolderStructure.md) Section 3.
- TypeScript: `*.test.ts(x)` colocated next to the file under test (e.g., `use-patient-query.test.ts` beside `use-patient-query.ts`).
- E2E: `tests/e2e/*.spec.ts` per [FolderStructure](FolderStructure.md) Section 4.

## 7. CI Pipeline Gate Order

`cargo fmt --check` -> `cargo clippy -D warnings` -> `cargo test` (unit+integration, with coverage) -> `eslint --max-warnings=0` -> `tsc --noEmit` -> `vitest --coverage` -> `npm audit` / `cargo audit` -> secret scan -> (on `main`/release branches only) E2E suite. Any failure blocks merge — this is the automatic-validation backbone referenced by every phase in [Plan](Plan.md) and by [DefinitionOfDone](DefinitionOfDone.md).

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#testingmd)
