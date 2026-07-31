# Tasks.md — Execution Guide

## Purpose

Breaks every phase in [Plan](Plan.md) into atomic implementation tasks. This is what an implementer actually executes, one task at a time.

## Dependencies

[Plan](Plan.md), [Rules](Rules.md), [DefinitionOfDone](DefinitionOfDone.md).

## Documents that must be read before using it

[Plan](Plan.md) (read the specific phase's full entry before its tasks), [Rules](Rules.md).

## Referenced By

[Database](Database.md), [DefinitionOfDone](DefinitionOfDone.md), [DevelopmentWorkflow](DevelopmentWorkflow.md), [IPC](IPC.md), [Plan](Plan.md), [Progress](Progress.md), [README_Project](README_Project.md), [Routes](Routes.md), [StateManagement](StateManagement.md), [Testing](Testing.md).

## Documents that must be updated if changes occur

[Progress](Progress.md) (mark each task's status), [New_files](New_files.md) (log files created per task).

## Priority

10.

---

## Contents

- [0. Atomic Task Template](#0-atomic-task-template)
- [1. Vertical-Slice Pattern (applies to every module phase, 4–5, 7, 9–12)](#1-vertical-slice-pattern-applies-to-every-module-phase-45-7-912)
- [Phase 0 — Project Bootstrap & Tooling](#phase-0--project-bootstrap--tooling)
- [Phase 1 — Database & Migration Engine Foundation](#phase-1--database--migration-engine-foundation)
- [Phase 2 — Security Foundation, Authentication & Audit Modules](#phase-2--security-foundation-authentication--audit-modules)
- [Phases 4–5, 7, 9–12 — Module Vertical Slices](#phases-45-7-912--module-vertical-slices)
- [Phase 3 — Frontend Application Shell & Design System](#phase-3--frontend-application-shell--design-system)
- [Phase 8 — Cross-Module Integration Checkpoint](#phase-8--cross-module-integration-checkpoint)
- [Phase 13 — Patient Discharge & Permanent History Preservation](#phase-13--patient-discharge--permanent-history-preservation)
- [Phase 14 — Hardening, Full Regression & Release](#phase-14--hardening-full-regression--release)

---

## 0. Atomic Task Template

Every task below (and every task added later) follows this shape:

- **Objective** — one sentence, single responsibility.
- **Implementation** — concrete steps/files.
- **Dependencies** — which prior tasks/phases must be done first.
- **Affected files** — exact paths.
- **Rules involved** — specific [Rules](Rules.md) section numbers.
- **Security checks** — specific [Security](Security.md)/[Validation](Validation.md) sections, or "none beyond standard session check."
- **Database checks** — specific [Database](Database.md) sections, or "none."
- **Expected result** — observable outcome.
- **Verification process** — exact command(s)/test(s) to run.
- **Completion criteria** — [DefinitionOfDone](DefinitionOfDone.md) Section 1, in full, every time.
- **Possible implementation risks** — the one or two things most likely to go wrong.

## 1. Vertical-Slice Pattern (applies to every module phase, 4–5, 7, 9–12)

Every module phase decomposes into the same eight atomic tasks, applied to that module's entities. This pattern is documented once here rather than repeated in full per phase, to keep this document usable while remaining exhaustive (Rule: no duplicated logic, applied to documentation itself).

1. **Migration task** — write and apply the module's `NNNN_*.sql` migration per [Database](Database.md) Section 5/6.
2. **Model task** — write `models/<entity>.rs` structs mirroring the schema.
3. **Repository task** — write `repositories/<entity>_repository.rs` (parameterized queries only, Rule 9.1).
4. **Service task** — write `services/<module>_service.rs` (business rules, transaction boundaries, audit write, event emission).
5. **Command task** — write `commands/<module>_commands.rs` (one command per [IPC](IPC.md) catalog row for the module; thin, re-validates input).
6. **Validation task** — write the Zod schema (`modules/<module>/types/*-schemas.ts`) and the Rust validator (`validation/<module>_validation.rs`), and add the row to [Validation](Validation.md) Section 3.
7. **Frontend API task** — write `modules/<module>/api/*` (query/mutation hooks), register events in `shared/lib/event-query-map.ts` per [IPC](IPC.md) Section 3.
8. **Frontend UI task** — write `modules/<module>/components/*` and the route-level page, applying the loading/empty/error triad ([UI](UI.md) Section 6).

Each of the 8 sub-tasks gets its own integration/unit test per [Testing](Testing.md), and all 8 together must satisfy [DefinitionOfDone](DefinitionOfDone.md) Section 1 before the phase can close.

---

## Phase 0 — Project Bootstrap & Tooling

### Task 0.1 — Initialize Tauri + React + TypeScript project

- **Objective:** Scaffold the base project matching [FolderStructure](FolderStructure.md).
- **Implementation:** `npm create tauri-app` (React + TS template), then reshape output to match [FolderStructure](FolderStructure.md) exactly; configure `tsconfig.json` per Rule 7.1; configure path aliases per Rule 4.3.
- **Dependencies:** none.
- **Affected files:** repo root config, `src/`, `src-tauri/` skeletons.
- **Rules involved:** [Rules](Rules.md) Sections 1–2, 7.
- **Security checks:** none beyond confirming no telemetry/analytics dependency is bundled by default.
- **Database checks:** none.
- **Expected result:** `npm run tauri dev` opens an empty window.
- **Verification process:** manual run; `tsc --noEmit` passes on the empty scaffold.
- **Completion criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 1.
- **Possible implementation risks:** scaffold tool defaults diverging from [FolderStructure](FolderStructure.md) — must be manually reshaped, not left as-is.

### Task 0.2 — Configure ESLint, Prettier, lint-staged, husky

- **Objective:** Enforce [CodingStandards](CodingStandards.md) Section 1 automatically.
- **Implementation:** install and configure per [CodingStandards](CodingStandards.md) Section 1 exact ruleset; wire `lint-staged` + `husky` pre-commit per [DevelopmentWorkflow](DevelopmentWorkflow.md) Section 5.
- **Dependencies:** 0.1.
- **Affected files:** `.eslintrc.cjs`, `.prettierrc`, `.husky/`, `package.json`.
- **Rules involved:** [Rules](Rules.md) Sections 3, 4, 20; verification mechanism per Rules Section 21.
- **Security checks:** none.
- **Database checks:** none.
- **Expected result:** `npm run lint` runs clean on the scaffold; a deliberately-bad test file is rejected.
- **Verification process:** `npm run lint`; commit a deliberately malformed file, confirm pre-commit hook blocks it, then remove the test file.
- **Completion criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 1.
- **Possible implementation risks:** overly strict rule defaults blocking legitimate patterns used later (e.g., `import/no-cycle` false positives) — tune only with documented justification.

### Task 0.3 — Configure Rust toolchain (rustfmt, clippy) and Cargo workspace

- **Objective:** Enforce [CodingStandards](CodingStandards.md) Section 2.
- **Implementation:** add `rustfmt.toml` (defaults), configure `clippy::unwrap_used`/`expect_used` deny in `main.rs` lint attributes per Rule 8.3.
- **Dependencies:** 0.1.
- **Affected files:** `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`.
- **Rules involved:** [Rules](Rules.md) Sections 3, 8, 20.
- **Security checks:** none.
- **Database checks:** none.
- **Expected result:** `cargo clippy -D warnings` passes on the empty scaffold.
- **Verification process:** `cargo fmt --check && cargo clippy -D warnings`.
- **Completion criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 1.
- **Possible implementation risks:** none significant at this stage.

### Task 0.4 — Set up Vitest and cargo test scaffolding

- **Objective:** Establish test runners per [Testing](Testing.md) Section 1.
- **Implementation:** configure `vitest.config.ts`; confirm `cargo test` runs (even zero tests).
- **Dependencies:** 0.1.
- **Affected files:** `vitest.config.ts`, `package.json`.
- **Rules involved:** [Rules](Rules.md) Section 15.
- **Security checks:** none. **Database checks:** none.
- **Expected result:** both test commands exit 0.
- **Verification process:** `npm run test`, `cargo test`.
- **Completion criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 1.
- **Possible implementation risks:** none significant.

### Task 0.5 — Stand up CI pipeline

- **Objective:** Implement the exact gate order from [Testing](Testing.md) Section 7.
- **Implementation:** `.github/workflows/ci.yml` running fmt/clippy/test/eslint/tsc/vitest/audits/secret-scan in order, failing fast.
- **Dependencies:** 0.2, 0.3, 0.4.
- **Affected files:** `.github/workflows/ci.yml`.
- **Rules involved:** [Rules](Rules.md) Section 21 (verification mechanism).
- **Security checks:** secret-scan step present (Rule 10.5). **Database checks:** none.
- **Expected result:** CI runs green on the bootstrap commit.
- **Verification process:** push to a branch, observe CI result.
- **Completion criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 1.
- **Possible implementation risks:** `cargo audit`/`npm audit` false-positive noise on fresh scaffold dependencies — document any accepted exception explicitly, do not silently ignore.

---

## Phase 1 — Database & Migration Engine Foundation

### Task 1.1 — Implement SQLCipher-encrypted connection pool

- **Objective:** `db/connection.rs` opens/creates an encrypted SQLite file with WAL + foreign keys on.
- **Implementation:** `rusqlite` with `bundled-sqlcipher` feature; `PRAGMA key`, `PRAGMA journal_mode=WAL`, `PRAGMA foreign_keys=ON` on every connection open, per [Database](Database.md) Section 2.
- **Dependencies:** Phase 0.
- **Affected files:** `src-tauri/src/db/connection.rs`, `Cargo.toml`.
- **Rules involved:** [Rules](Rules.md) 9.1, 10.4.
- **Security checks:** [Security](Security.md) Sections 5, per Task 1.3's key retrieval.
- **Database checks:** [Database](Database.md) Section 2 (all pragmas set).
- **Expected result:** connection opens, pragmas verified via `PRAGMA` query round-trip in a test.
- **Verification process:** integration test asserting all three pragmas report the expected values.
- **Completion criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 1.
- **Possible implementation risks:** forgetting `foreign_keys=ON` per-connection (SQLite requires this per connection, not database-wide) — must be part of the pool's connection-init hook, not a one-time setup call.

### Task 1.2 — Implement migration runner

- **Objective:** `db/migrator.rs` applies `migrations/*.sql` in order, tracked in `schema_migrations`.
- **Implementation:** read migration files sorted by filename prefix, apply any not present in `schema_migrations`, insert a tracking row per applied file, all inside a transaction per file.
- **Dependencies:** 1.1.
- **Affected files:** `src-tauri/src/db/migrator.rs`, `src-tauri/migrations/0000_schema_migrations.sql`.
- **Rules involved:** [Rules](Rules.md) 9.2.
- **Security checks:** none. **Database checks:** [Database](Database.md) Section 5.
- **Expected result:** running the migrator twice in a row is idempotent (second run applies nothing new).
- **Verification process:** migration test per [Testing](Testing.md) Section 1 "Database."
- **Completion criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 1.
- **Possible implementation risks:** partial-apply on failure leaving `schema_migrations` inconsistent — must wrap each file's apply + tracking-row-insert in one transaction.

### Task 1.3 — Implement OS keychain key retrieval/generation

- **Objective:** `security/secrets.rs` generates a 256-bit key on first run, stores it via `keyring`, retrieves it on subsequent runs.
- **Implementation:** check keychain entry existence; if absent, generate via CSPRNG, store; always return the stored value to `connection.rs`.
- **Dependencies:** none (parallel to 1.1, consumed by it).
- **Affected files:** `src-tauri/src/security/secrets.rs`.
- **Rules involved:** [Rules](Rules.md) 10.4, 10.5.
- **Security checks:** [Security](Security.md) Section 5 in full.
- **Database checks:** none.
- **Expected result:** key persists across app restarts (same key retrieved, DB reopens successfully).
- **Verification process:** integration test: generate, close, reopen with retrieved key, confirm DB accessible; confirm wrong key fails to open.
- **Completion criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 1.
- **Possible implementation risks:** CI environment lacking a real OS keychain — provide an explicitly-named test-only key path (Section 5, [Security](Security.md)) never reachable from production code.

---

## Phase 2 — Security Foundation, Authentication & Audit Modules

### Task 2.1 — Migration `0001_init_auth.sql`

Creates `users`, `sessions` per [Database](Database.md) Section 3.1. Follows the standard Migration task shape (Section 1 above), Rules involved: 9.1–9.4.

### Task 2.2 — Migration `0002_audit.sql`

Creates `audit_log` + append-only triggers per [Database](Database.md) Section 3.7, [Audit](Audit.md) Section 5. Rules involved: 9.1–9.5.

### Task 2.3 — Implement Argon2id hashing wrapper

`security/hashing.rs`: `hash_password`, `verify_password` using parameters from [Security](Security.md) Section 2. Test: known-vector hash/verify round trip; verify wrong password rejected; verify hash format stored matches PHC string.

### Task 2.4 — Implement session issuance/validation

`security/session.rs`: `create_session`, `require_session` (used by every future command), expiry/idle-timeout logic per [Security](Security.md) Section 4. Test: valid session passes, expired session rejected, idle-timeout-exceeded session rejected.

### Task 2.5 — Implement rate limiting / lockout

`security/rate_limit.rs` + wiring into `auth_service.rs` per [Security](Security.md) Section 3. Test: 5 failures trigger lockout; escalating lockout durations verified; successful login resets counter.

### Task 2.6 — Implement `user_repository.rs`, `audit_repository.rs`

Standard Repository task shape. `audit_repository` exposes `insert` only (no update/delete function exists), per [Audit](Audit.md) Section 5.

### Task 2.7 — Implement `auth_service.rs`, `audit_service.rs` (incl. hash chain)

`audit_service::record` computes `prev_hash`/`row_hash` per [Audit](Audit.md) Section 4; `auth_service` orchestrates hashing + rate limiting + session issuance + audit write in one transaction where applicable. Test: hash chain verified correct across a sequence of writes; tampered-row detection test per [Testing](Testing.md) Section 4 scenario 5.

### Task 2.8 — Implement `auth_commands.rs`, `audit_commands.rs`

Standard Command task shape, per [IPC](IPC.md) auth/audit rows.

### Task 2.9 — Implement first-run bootstrap

`auth_bootstrap_status` + `auth_bootstrap_admin` per [Security](Security.md) Section 9.1. The empty-`users` check MUST be executed inside the same transaction as the `INSERT` (TOCTOU — see that section). Tests: bootstrap on empty DB succeeds and returns a usable session; second attempt returns `AppError::Conflict`; bootstrap rejected when any user exists; weak password rejected per [Validation](Validation.md) Section 5; the genesis audit row is written and `audit_verify_integrity` passes over it.
- **Possible implementation risks:** performing the count check before opening the transaction (silently correct in single-user manual testing, wrong under the concurrent test); forgetting that this is one of only three unauthenticated commands and wiring `require_session` into it, which would deadlock the installation permanently.

### Task 2.10 — Implement user management

`auth_create_user`, `auth_list_users`, `auth_deactivate_user` per [IPC](IPC.md) Section 2, audited as `user.create`/`user.deactivate` ([Audit](Audit.md) Section 2). `auth_list_users` must never serialize `password_hash` — assert this in the test, not just by inspection.

### Task 2.11 — Frontend: auth schemas, api hooks, login page, setup page, users page, session guard

Standard Validation/Frontend API/Frontend UI task shapes for Authentication, per [Routes](Routes.md) Sections 2–3, covering `/login`, `/setup` (gated on `auth_bootstrap_status`) and `/users`.

### Task 2.12 — Frontend: audit list/verify view

Standard Frontend API/UI tasks for Audit, per [UI](UI.md) Section 2, [Routes](Routes.md) Section 3 `/audit`.

*(Each task above still carries the full template fields — Dependencies/Affected files/Verification/Completion criteria — per Section 0; abbreviated here for length, but every implementer must fill in the full template per [DefinitionOfDone](DefinitionOfDone.md) Section 1 before marking done, recorded in [Progress](Progress.md).)*

---

## Phases 4–5, 7, 9–12 — Module Vertical Slices

Apply the 8-task Vertical-Slice Pattern (Section 1 above) to each module, using the specifics already declared in the corresponding [Plan](Plan.md) phase entry (migration file name, entities, files affected, required reading). Concretely:

| Phase | Module | Migration | Entities |
|---|---|---|---|
| 4 | Patients | `0003_patients_encounters.sql` | `patients` (encounters table created here, write path in Phase 5) |
| 5 | Medical History | `0004_medical_history.sql` | `encounters` (write path), `diagnoses`, `treatments`, `evolutions` |
| 6 | Hospital Map | `0005_hospital_map_beds.sql` (shared, read-only commands) | `floors`, `rooms` (read only) |
| 7 | Beds | `0005_hospital_map_beds.sql` (shared, write commands) | `beds`, `bed_assignments` |
| 9 | Operating Rooms | `0006_operating_rooms.sql` | `operating_rooms`, `or_reservations` |
| 10 | Inventory | `0007_inventory.sql` | `inventory_categories`, `inventory_items`, `inventory_transactions`, `maintenance_schedules` |
| 11 | Notifications | `0008_notifications.sql` | `notifications` |
| 12 | Billing | `0009_billing.sql` | `billing_simulations`, `billing_items` |

For each row: execute the 8 sub-tasks from Section 1, each individually verified and logged in [Progress](Progress.md)/[New_files](New_files.md), each meeting [DefinitionOfDone](DefinitionOfDone.md) Section 1.

**Phase 6 deviation.** For the *Hospital Map* read path, Phase 6 uses only sub-tasks 2–3 (model/repository, read-only) and 5, 7, 8 — no service business-rule task and no write validation task (see [Plan](Plan.md) Phase 6 note on the documented exception). Phase 6 additionally executes the **full** 8-sub-task slice for the Beds-owned facility-configuration commands (`beds_create_floor`, `beds_create_room`, `beds_create`, `beds_set_status`), which *are* ordinary validated, audited, event-emitting writes — see Task 6.1 below.

### Task 6.1 — Facility configuration (Beds-owned write path)

- **Objective:** Make it possible to create a hospital layout from an empty database without mock data.
- **Implementation:** `beds_create_floor`, `beds_create_room`, `beds_create`, `beds_set_status` in `bed_service.rs`/`bed_commands.rs` per [IPC](IPC.md) Section 2.1; `/facility` page (`FacilityConfigPage`) per [Routes](Routes.md) Section 3; emit `beds:facility:changed` and register it in `event-query-map.ts`.
- **Dependencies:** migration `0005_hospital_map_beds.sql` (same phase).
- **Affected files:** `services/bed_service.rs`, `commands/bed_commands.rs`, `repositories/bed_repository.rs`, `validation/bed_validation.rs`, `modules/beds/*`.
- **Rules involved:** [Rules](Rules.md) 1.5, 8.2, 9.1, 17.1.
- **Security checks:** standard session check; `beds_set_status` must reject `occupied` — occupancy is a consequence of assignment, never directly settable ([IPC](IPC.md) Section 2).
- **Database checks:** `rooms.map_x`/`map_y` normalized to 0..1 ([Database](Database.md) Section 3.3); `rooms.room_type` CHECK honoured.
- **Expected result:** a floor -> room -> bed chain is creatable through the UI and immediately visible on the Hospital Map.
- **Verification process:** integration tests per command; manual validation per [Plan](Plan.md) Phase 6.
- **Completion criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 1.
- **Possible implementation risks:** placing these commands in `hospital_map_commands.rs`, which would violate the module's read-only guarantee and fail the Phase 6 grep check — they belong in `bed_commands.rs`; allowing `beds_set_status` to set `occupied` and thereby desynchronise `beds.status` from `bed_assignments`.

---

## Phase 3 — Frontend Application Shell & Design System

### Task 3.1 — Build `shared/ui` primitives

One task per primitive listed in [UI](UI.md) Section 3 (`Button`, `Card`, `Dialog`, form primitives, `Table`/`VirtualizedTable`, `Badge`, `Toast`, `Skeleton`, `EmptyState`, `ErrorState`) — each with a component test verifying correct ARIA roles (jsx-a11y clean) and the states relevant to it.

### Task 3.2 — Build route tree skeleton

Implement `app/router.tsx` with all 14 routes from [Routes](Routes.md) Section 3 as stub pages; wire root `beforeLoad` guard per [Routes](Routes.md) Section 2.

### Task 3.3 — Build event subscription plumbing

`shared/hooks/use-event-subscription.ts`, `shared/lib/event-query-map.ts` (empty, populated per later module task per [StateManagement](StateManagement.md) Section 4).

### Task 3.4 — Build error mapping plumbing

`shared/errors/app-error.ts` (mirrors Rust `AppError`), `shared/errors/error-messages.ts` per [ErrorHandling](ErrorHandling.md) Sections 1, 3.

Each task above follows the full atomic template (Section 0); verification is primarily component tests + `eslint-plugin-jsx-a11y` + manual keyboard-navigation pass per [UI](UI.md) Section 8.

---

## Phase 8 — Cross-Module Integration Checkpoint

### Task 8.1 — Run anti-duplication checklist

Execute [StateManagement](StateManagement.md) Section 6 checklist against all code from Phases 4–7; file a fix task for each finding.

### Task 8.2 — Run partial E2E scenario

Execute `patient-admission-flow.spec.ts` through bed assignment ([Testing](Testing.md) Section 4); fix any failures as individually tracked fix tasks.

### Task 8.3 — Two-window manual propagation test

Manually verify cross-window event propagation per [Plan](Plan.md) Phase 8 manual validation step; file findings as fix tasks if propagation gaps are found.

---

## Phase 13 — Patient Discharge & Permanent History Preservation

### Task 13.1 — Implement discharge orchestration

`medical_history_service::discharge_encounter` calls `bed_service::release` for any active assignment, sets encounter status, all in one transaction; audited as `encounter.discharge`.

### Task 13.2 — Frontend discharge flow

Discharge confirmation dialog + billing-finalization prompt per [UI](UI.md) Section 4, [Routes](Routes.md) patient detail hub.

### Task 13.3 — Full workflow E2E scenario

Complete `patient-admission-flow.spec.ts` end to end (registration through post-discharge history query) per [Testing](Testing.md) Section 4.

---

## Phase 14 — Hardening, Full Regression & Release

### Task 14.1 — Dependency audit remediation

Run `npm audit`/`cargo audit`; remediate or document every finding.

### Task 14.2 — Exhaustive session-requirement audit

Cross-check every command in [IPC](IPC.md) Section 2 against `security::session::require_session` usage; no gaps permitted.

### Task 14.3 — Mock-data / dead-code / unused-file sweep

Run `ts-prune`/`knip`, `clippy` dead_code lint, and a manual grep for hardcoded arrays/`faker` usage outside `dev-seed`; remediate all findings per [Rules](Rules.md) Section 17.

### Task 14.4 — Full accessibility pass

Manual keyboard-only + screen-reader spot-check across all pages per [UI](UI.md) Section 8.

### Task 14.5 — Full E2E suite + release build

Run every scenario in [Testing](Testing.md) Section 4; produce `tauri build` release artifact; smoke-test on a clean machine/VM.

Each task above follows the full atomic template; completion of all five, plus every prior phase's tasks, satisfies [DefinitionOfDone](DefinitionOfDone.md) Section 3.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#tasksmd--execution-guide)
