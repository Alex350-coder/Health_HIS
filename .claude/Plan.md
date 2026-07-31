# Plan.md — Project Director

## Purpose

The authoritative, phase-by-phase execution plan for the entire HIS MVP. No implementation happens outside a phase defined here.

## Dependencies

[Rules](Rules.md), [Architecture](Architecture.md), [Security](Security.md), [Database](Database.md), and every document in the framework (each phase declares its own specific Required Reading).

## Documents that must be read before using it

[README_Project](README_Project.md), [Rules](Rules.md), [Architecture](Architecture.md).

## Referenced By

[Architecture](Architecture.md), [Database](Database.md), [DefinitionOfDone](DefinitionOfDone.md), [DevelopmentWorkflow](DevelopmentWorkflow.md), [Progress](Progress.md), [README_Project](README_Project.md), [StateManagement](StateManagement.md), [Tasks](Tasks.md), [Testing](Testing.md).

## Documents that must be updated if changes occur

[Tasks](Tasks.md) (atomic tasks per phase), [Progress](Progress.md) (status per phase), [DefinitionOfDone](DefinitionOfDone.md) (if exit criteria change globally).

## Priority

1.

---

## Contents

- [0. Phase Index](#0-phase-index)
- [Phase 0 — Project Bootstrap & Tooling](#phase-0--project-bootstrap--tooling)
- [Phase 1 — Database & Migration Engine Foundation](#phase-1--database--migration-engine-foundation)
- [Phase 2 — Security Foundation, Authentication & Audit Modules](#phase-2--security-foundation-authentication--audit-modules)
- [Phase 3 — Frontend Application Shell & Design System](#phase-3--frontend-application-shell--design-system)
- [Phase 4 — Patients Module](#phase-4--patients-module)
- [Phase 5 — Medical History Module](#phase-5--medical-history-module)
- [Phase 6 — Hospital Map Module](#phase-6--hospital-map-module)
- [Phase 7 — Beds Module](#phase-7--beds-module)
- [Phase 8 — Cross-Module Integration Checkpoint](#phase-8--cross-module-integration-checkpoint)
- [Phase 9 — Operating Rooms Module](#phase-9--operating-rooms-module)
- [Phase 10 — Inventory Module](#phase-10--inventory-module)
- [Phase 11 — Notifications Module](#phase-11--notifications-module)
- [Phase 12 — Billing Module (Simulation)](#phase-12--billing-module-simulation)
- [Phase 13 — Patient Discharge & Permanent History Preservation](#phase-13--patient-discharge--permanent-history-preservation)
- [Phase 14 — Hardening, Full Regression & Release](#phase-14--hardening-full-regression--release)

---

## 0. Phase Index

| # | Phase | Modules | Depends On |
| --- | --- | --- | --- |
| 0 | Project Bootstrap & Tooling | — | none |
| 1 | Database & Migration Engine Foundation | — | 0 |
| 2 | Security Foundation, Authentication, Audit | Authentication, Audit | 1 |
| 3 | Frontend Application Shell & Design System | — | 2 |
| 4 | Patients Module | Patients | 3 |
| 5 | Medical History Module | Medical History | 4 |
| 6 | Hospital Map Module | Hospital Map | 3 |
| 7 | Beds Module | Beds | 5, 6 |
| 8 | Cross-Module Integration Checkpoint | Patients, Medical History, Beds, Hospital Map | 7 |
| 9 | Operating Rooms Module | Operating Rooms | 8 |
| 10 | Inventory Module | Inventory | 8 |
| 11 | Notifications Module | Notifications | 9, 10 |
| 12 | Billing Module (Simulation) | Billing | 5, 9, 10 |
| 13 | Patient Discharge & Permanent History Preservation | Patients, Medical History, Beds, Billing | 12 |
| 14 | Hardening, Full Regression & Release | all | 13 |

Every phase below follows the same template. Full atomic breakdown of each phase's tasks lives in [Tasks](Tasks.md).

---

## Phase 0 — Project Bootstrap & Tooling

**Objective:** Stand up the repository skeleton, toolchain, and CI pipeline so every subsequent phase has a working, enforced quality gate from day one.

**Description:** Initialize the Tauri + React + TypeScript project, configure ESLint/Prettier/TypeScript strict mode, configure Rust toolchain (rustfmt/clippy), set up Vitest and cargo test scaffolding, and stand up the CI pipeline exactly as defined in [Testing](Testing.md) Section 7.

**Scope:** Tooling only. No business/domain code.

**Modules involved:** None (infrastructure).

**Files affected:** Repository root config files, `src-tauri/Cargo.toml`, `package.json`, `.github/workflows/`, initial `src/app/App.tsx` shell, initial `src-tauri/src/main.rs`.

**Required Reading:** [Rules](Rules.md), [Architecture](Architecture.md), [FolderStructure](FolderStructure.md), [CodingStandards](CodingStandards.md), [Testing](Testing.md) Section 7.

**Documents to update after completion:** [FolderStructure](FolderStructure.md) (confirm tree matches reality), [New_files](New_files.md).

**Dependencies:** None.

**Security implications:** Establish CSP and Tauri capability baseline (empty/minimal) per [Security](Security.md) Section 9, secret-scanning CI step per Rule 10.5.

**Database implications:** None yet.

**UI implications:** Empty app shell only, no real pages.

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 0.

**Automatic validation:** CI pipeline itself running green on a trivial commit ([Testing](Testing.md) Section 7).

**Manual validation:** `npm run tauri dev` launches an empty window without errors.

**Acceptance criteria:** All lint/format/typecheck/test commands run successfully (even with zero tests) both locally and in CI.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2, adapted (no business logic exists yet, so business-rule/audit criteria are N/A for this phase only — noted explicitly in [Progress](Progress.md)).

**Regression checklist:** N/A (first phase).

**Estimated effort:** 1–2 days.

---

## Phase 1 — Database & Migration Engine Foundation

**Objective:** Establish the SQLite + SQLCipher connection layer, encryption key management, and the forward-only migration runner, with zero business tables yet.

**Description:** Implement `src-tauri/src/db/connection.rs` (SQLCipher-encrypted `rusqlite` pool, WAL mode, `foreign_keys=ON`), `src-tauri/src/db/migrator.rs` (applies `migrations/*.sql` in order, tracks `schema_migrations`), and `security/secrets.rs` (OS keychain key retrieval/generation via `keyring`).

**Scope:** Database plumbing and secrets only.

**Modules involved:** None directly (cross-cutting infrastructure consumed by every future module).

**Files affected:** `src-tauri/src/db/*`, `src-tauri/src/security/secrets.rs`, `src-tauri/migrations/0000_schema_migrations.sql`.

**Required Reading:** [Database](Database.md) (full), [Security](Security.md) Sections 1, 5.

**Documents to update after completion:** [Database](Database.md) (if any deviation from the documented connection settings is discovered), [New_files](New_files.md).

**Dependencies:** Phase 0.

**Security implications:** Encryption key never touches disk in plaintext (Rule 10.4); verified by a test that inspects the raw DB file bytes are not human-readable plaintext SQL/data.

**Database implications:** Establishes `schema_migrations` bookkeeping table ([Database](Database.md) Section 5).

**UI implications:** None.

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 1.

**Automatic validation:** Migration test (apply-to-empty-file, [Testing](Testing.md) Section 1) green; a round-trip test (write encrypted, reopen with correct key succeeds, wrong key fails) passes.

**Manual validation:** Inspect the generated `.sqlite` file with a hex viewer — confirm it is not readable as plaintext SQLite (SQLCipher header present).

**Acceptance criteria:** App boots, opens/creates an encrypted DB file, applies zero migrations without error.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Re-run Phase 0 CI checks (must still be green).

**Estimated effort:** 2–3 days.

---

## Phase 2 — Security Foundation, Authentication & Audit Modules

**Objective:** Implement session management, Argon2id hashing, rate limiting, and the Authentication + Audit modules end-to-end (backend + minimal UI), since every later module depends on both.

**Description:** Migrations `0001_init_auth.sql`, `0002_audit.sql` ([Database](Database.md) Section 6). Implement `security/hashing.rs`, `security/session.rs`, `security/rate_limit.rs`, `models/user.rs`, `repositories/user_repository.rs`, `repositories/audit_repository.rs`, `services/auth_service.rs`, `services/audit_service.rs` (including hash chain, [Audit](Audit.md) Section 4), `commands/auth_commands.rs`, `commands/audit_commands.rs`. Frontend: `modules/auth/*`, `modules/audit/*` (list/verify views), login route, root auth guard ([Routes](Routes.md) Section 2).

**Scope:** Full vertical slice for Authentication and Audit, **including first-run bootstrap and user management** — without bootstrap the application cannot be logged into at all, so this is not deferrable ([Security](Security.md) Section 9.1, [IPC](IPC.md) Section 2.1).

**Modules involved:** Authentication, Audit.

**Files affected:** Per [FolderStructure](FolderStructure.md) Sections 2–3 for these two modules; `src-tauri/migrations/0001_*.sql`, `0002_*.sql`.

**Required Reading:** [Security](Security.md) (full), [Audit](Audit.md) (full), [Database](Database.md) Sections 3.1, 3.7, [IPC](IPC.md) (auth/audit rows), [Validation](Validation.md) Section 3 (auth rows), [ErrorHandling](ErrorHandling.md) (Unauthorized/AccountLocked/Validation paths), [UI](UI.md), [Routes](Routes.md).

**Documents to update after completion:** [IPC](IPC.md) (mark commands implemented — tracked in [Progress](Progress.md), not by editing the catalog itself), [New_files](New_files.md).

**Dependencies:** Phase 1.

**Security implications:** This phase *is* the security foundation — Argon2id parameters, rate limiting/lockout escalation, session token lifecycle, audit hash chain all implemented and tested here per [Security](Security.md) and [Audit](Audit.md).

**Database implications:** First business tables (`users`, `sessions`, `audit_log`) plus the append-only trigger ([Audit](Audit.md) Section 5).

**UI implications:** Login page, top-bar user/session display, Audit log viewer page ([UI](UI.md) Section 2, [Routes](Routes.md) Section 3).

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 2.

**Automatic validation:** Security integration test suite ([Testing](Testing.md) Section 1 "Security") green: lockout escalation, session expiry, unauthorized command rejection, hash-chain verification (including a deliberately-tampered-row test).

**Manual validation:** On a fresh (deleted) database, launch the app, complete `/setup` to create the first admin, then log in, log out, trigger 5 failed logins and observe lockout countdown UI; create a second user via `/users` and log in as them; view audit log after these actions and confirm entries recorded correctly.

**Acceptance criteria:** No command outside the closed set `{auth_login, auth_bootstrap_status, auth_bootstrap_admin}` is callable without a valid session (spot-checked against every command that will exist in later phases, re-verified exhaustively at Phase 14, Task 14.2). A fresh installation can be bootstrapped exactly once and a second bootstrap attempt is rejected with `AppError::Conflict`.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2, including the security-specific coverage in [Testing](Testing.md) Section 1.

**Regression checklist:** Phases 0–1 CI still green.

**Estimated effort:** 5–7 days.

---

## Phase 3 — Frontend Application Shell & Design System

**Objective:** Build the persistent app shell (sidebar, top bar, routing skeleton, design-system primitives) so every subsequent module plugs into a consistent, already-accessible UI.

**Description:** Implement `shared/ui/*` primitives ([UI](UI.md) Section 3), `app/router.tsx` route tree skeleton for all routes in [Routes](Routes.md) Section 3 (pages initially stubbed), `shared/hooks/use-event-subscription.ts` and `shared/lib/event-query-map.ts` (empty map, populated per module going forward, [StateManagement](StateManagement.md) Section 4), `shared/errors/*` (AppError mirror + error-messages map, [ErrorHandling](ErrorHandling.md) Sections 1, 3).

**Scope:** Shell + design system + cross-cutting frontend plumbing. No module business logic.

**Modules involved:** None directly (infrastructure consumed by all).

**Files affected:** `src/app/*`, `src/shared/*`, `src/styles/globals.css`.

**Required Reading:** [UI](UI.md) (full), [Routes](Routes.md) (full), [StateManagement](StateManagement.md) (full), [ErrorHandling](ErrorHandling.md) (full).

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 2 (needs the auth guard and session slice to wire the shell's top bar/guard).

**Security implications:** Root route guard wired per [Security](Security.md) Section 4 / [Routes](Routes.md) Section 2. CSP finalized in `tauri.conf.json` per [Security](Security.md) Section 9.

**Database implications:** None.

**UI implications:** This phase *is* the UI foundation — establishes tokens, primitives, navigation, and the loading/empty/error triad pattern ([UI](UI.md) Section 6) used by every future page.

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 3.

**Automatic validation:** Component tests for every `shared/ui` primitive; `eslint-plugin-jsx-a11y` clean.

**Manual validation:** Manual accessibility pass (keyboard-only navigation through the full shell) per [UI](UI.md) Section 8.

**Acceptance criteria:** All 17 routes from [Routes](Routes.md) Section 3 render a stub page reachable via the sidebar, behind the auth guard (except the two public routes, `/login` and `/setup`), plus the catch-all `NotFoundPage`.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Phases 0–2 CI still green; login/logout flow still works through the new shell.

**Estimated effort:** 4–5 days.

---

## Phase 4 — Patients Module

**Objective:** Implement patient registration and retrieval — the root entity of the entire workflow.

**Description:** Migration `0003_patients_encounters.sql` (creates `patients` and `encounters` tables together since `encounters` is immediately needed for the workflow's next step, though only `patients` CRUD ships this phase — `encounters` write paths ship in Phase 5). Implement `models/patient.rs`, `repositories/patient_repository.rs`, `services/patient_service.rs`, `commands/patient_commands.rs`. Frontend: `modules/patients/*`, `PatientListPage`, `PatientCreatePage`, `PatientDetailPage` shell (tabs stubbed for Medical History/Beds/Billing, populated in later phases).

**Scope:** Full vertical slice for Patients (create, update, get, list).

**Modules involved:** Patients.

**Files affected:** Per [FolderStructure](FolderStructure.md); `src-tauri/migrations/0003_*.sql`.

**Required Reading:** [Database](Database.md) Section 3.2, [Validation](Validation.md) Section 3 (Patient rows), [IPC](IPC.md) (`patients_*` rows), [Rules](Rules.md) Section 17.1 (no mock data — this is the first module where the temptation to stub with fake patients is highest).

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 3.

**Security implications:** Standard session-required command validation ([Security](Security.md) Section 1.6). PHI protection is structural (encryption at rest, Phase 1) — no new security work, but the first module actually storing PHI, so a review checkpoint against [Security](Security.md) Section 4 (Defense in Depth) is included.

**Database implications:** `patients` table live; `encounters` table created but only referenced (empty) until Phase 5.

**UI implications:** First real data-bearing pages — first application of [UI](UI.md) Section 6 triad and virtualized/paginated table ([UI](UI.md) Section 5).

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 4.

**Automatic validation:** Integration tests for all `patients_*` commands against a real temp DB.

**Manual validation:** Register a patient through the UI, confirm it appears in the list and detail view, confirm MRN uniqueness is enforced with a clear error.

**Acceptance criteria:** A patient can be created, listed (with search/pagination), retrieved by id, and updated — all fully validated per [Validation](Validation.md) Section 3.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Phases 0–3 CI + manual flows still pass.

**Estimated effort:** 3–4 days.

---

## Phase 5 — Medical History Module

**Objective:** Implement encounter creation/retrieval and the append-only diagnosis/treatment/evolution registration workflow.

**Description:** Migration `0004_medical_history.sql` (`diagnoses`, `treatments`, `evolutions`) plus the `encounters` write paths (create, discharge — discharge itself is fully wired in Phase 13, but the command exists here and is exercised by tests). Implement `models/encounter.rs`, `repositories/encounter_repository.rs`, `services/medical_history_service.rs`, `commands/medical_history_commands.rs`. Frontend: `modules/medical-history/*`, wired into `PatientDetailPage`'s Medical History tab.

**Scope:** Full vertical slice for encounters + diagnoses + treatments + evolutions (append-only).

**Modules involved:** Medical History.

**Files affected:** Per [FolderStructure](FolderStructure.md); `src-tauri/migrations/0004_*.sql`.

**Required Reading:** [Database](Database.md) Sections 3.2, 7 (Permanent Medical History), [Rules](Rules.md) 9.5, [Validation](Validation.md), [IPC](IPC.md) (`medical_history_*` rows).

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 4.

**Security implications:** No new security surface beyond standard session checks; audit coverage per [Audit](Audit.md) Section 2 "Medical History" row is exercised and tested here.

**Database implications:** `diagnoses`, `treatments`, `evolutions` live; append-only enforcement verified (attempting `UPDATE`/`DELETE` in a test must fail or be structurally impossible — repository exposes no such methods, per [Audit](Audit.md) Section 5 pattern applied to clinical tables via Rule 9.5).

**UI implications:** Timeline-style rendering of an encounter's diagnoses/treatments/evolutions within the patient detail hub ([Routes](Routes.md) Section 3).

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 5.

**Automatic validation:** Integration tests proving corrections create new rows (`corrects_*_id`) rather than mutating existing ones.

**Manual validation:** Create an encounter, add a diagnosis, a treatment, an evolution note; confirm all appear in chronological order and none can be edited/deleted through the UI.

**Acceptance criteria:** Full "Diagnosis Registration -> Treatment Registration -> Medical Evolution Registration" workflow segment ([README_Project](README_Project.md) Section 2) works end-to-end.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Phases 0–4 CI + manual flows.

**Estimated effort:** 4–5 days.

---

## Phase 6 — Hospital Map Module

**Objective:** Implement the read-only, visual, interactive hospital map.

**Description:** Migration is deferred to Phase 7 (`floors`/`rooms` are created as part of `0005_hospital_map_beds.sql`, shared with Beds since both need the same physical-space tables — Hospital Map only *reads* them). This phase implements the read commands (`hospital_map_get_layout`, `hospital_map_get_room_status`) against tables created in this same migration, and the frontend SVG/canvas renderer.

**Scope:** Read-only Hospital Map backend + interactive frontend visualization, plus the Beds-owned facility-configuration write path (see note below).

**Modules involved:** Hospital Map (reads Beds/Operating Rooms data it does not own, per [Architecture](Architecture.md) Section 3).

**Files affected:** `src-tauri/migrations/0005_hospital_map_beds.sql` (shared with Phase 7 — see note below), `commands/hospital_map_commands.rs`, `services/` (a thin read-aggregation, no dedicated `hospital_map_service.rs` needed beyond a query aggregator since there's no business logic to own — read aggregation lives directly in the command via the Bed/Room repositories, an explicitly documented exception to "commands are thin" because there is no mutation, hence no service-layer business rule to encapsulate), `modules/hospital-map/*`.

**Note on migration sharing:** Because Hospital Map's data (`floors`, `rooms`) is physically owned by the Beds module ([Architecture](Architecture.md) Section 3 table), the migration `0005_hospital_map_beds.sql` is created once, in this phase, containing `floors`, `rooms`, `beds`, `bed_assignments` together — Phase 6 exercises the read side plus facility configuration; Phase 7 adds the assignment/release side. This ordering is intentional: it lets the Hospital Map's read contract be validated before Beds' assignment complexity is layered on.

**Note on facility configuration (added to this phase):** The Hospital Map cannot be built, demonstrated, or tested against an empty building, and [Rules](Rules.md) 17.1 forbids seeding one with mock data. Therefore the facility-configuration write commands — `beds_create_floor`, `beds_create_room`, `beds_create`, `beds_set_status` ([IPC](IPC.md) Section 2.1) — and the `/facility` page ship in **this** phase, not Phase 7. These commands belong to the **Beds** module, which owns `floors`/`rooms`/`beds` per [Architecture](Architecture.md) Section 3. Hospital Map itself remains strictly read-only and still exposes zero mutation commands; the automatic grep check below is unaffected, because it asserts only that no `hospital_map_*` command is a non-`get_*`.

**Required Reading:** [Database](Database.md) Section 3.3, [Architecture](Architecture.md) Section 3 (module boundary — read-only), [UI](UI.md) Section 7.

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 3 (can run in parallel with Phase 4/5 since it has no dependency on Patients/Medical History).

**Security implications:** None beyond standard session checks — no PHI exposed at the map level (room/bed status only, no patient identity shown on the map itself; patient identity requires drilling into Beds/Patients modules separately).

**Database implications:** Creates `floors`, `rooms` (and `beds`, `bed_assignments` — see note above).

**UI implications:** The interactive SVG/canvas map component — the most visually distinct component in the app ([UI](UI.md) Section 7).

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 6.

**Automatic validation:** A test asserting no `hospital_map_*` command exists that isn't a `get_*` read (grep-based CI check, enforcing [Rules](Rules.md) "never modifies data").

**Manual validation:** Create a floor, two rooms and several beds through `/facility`; then click through floors/rooms on the map and confirm the new structures appear with correct coordinates and that no edit affordance exists anywhere *within the Hospital Map UI itself*.

**Acceptance criteria:** A hospital layout can be created from an empty database through `/facility` alone (no mock data, no manual SQL); the map renders all floors/rooms with correct coordinates and live status; zero mutation paths exist in the Hospital Map module.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Phases 0–5 CI + manual flows.

**Estimated effort:** 3–4 days.

---

## Phase 7 — Beds Module

**Objective:** Implement bed assignment and release — the physical-space write side building on Phase 6's shared migration.

**Description:** Implement `models/bed.rs`, `repositories/bed_repository.rs`, `services/bed_service.rs` (availability business rule, [Database](Database.md) Section 3.3 partial-unique-index-backed), `commands/bed_commands.rs`. Frontend: `modules/beds/*`, `BedListPage`, bed assignment dialog wired from `PatientDetailPage`.

**Scope:** Full vertical slice for bed assignment/release.

**Modules involved:** Beds.

**Files affected:** Per [FolderStructure](FolderStructure.md) (migration already created in Phase 6).

**Required Reading:** [Database](Database.md) Section 3.3, [IPC](IPC.md) (`beds_*` rows), [StateManagement](StateManagement.md) Section 4 (event -> Hospital Map invalidation).

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 5 (needs `encounter_id` to exist meaningfully), Phase 6 (shared tables/migration).

**Security implications:** Standard session checks; audit coverage for `bed.assign`/`bed.release` per [Audit](Audit.md) Section 2.

**Database implications:** `bed_assignments` write path live; partial unique index enforcement tested directly (attempt double-assignment, expect `AppError::Conflict`).

**UI implications:** Bed picker respects availability (disabled state for occupied beds, [Validation](Validation.md) Section 4 UX note); Hospital Map (Phase 6) reflects bed status changes live via the event system.

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 7.

**Automatic validation:** Integration test for double-assignment rejection, release-then-reassign success, event emission assertion.

**Manual validation:** Assign a bed to a patient, observe Hospital Map update without manual refresh (cross-view propagation, [StateManagement](StateManagement.md) Section 4), release the bed, confirm it becomes assignable again.

**Acceptance criteria:** "Hospital Bed Assignment" workflow step ([README_Project](README_Project.md) Section 2) fully functional and reflected live across Beds list, Hospital Map, and Patient detail views.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Phases 0–6 CI + manual flows.

**Estimated effort:** 4 days.

---

## Phase 8 — Cross-Module Integration Checkpoint

**Objective:** Validate that Patients, Medical History, Hospital Map, and Beds together correctly implement the workflow segment from registration through bed assignment, with no state-duplication or event-propagation gaps, before building the remaining modules on top.

**Description:** No new features. This phase runs the `patient-admission-flow.spec.ts` E2E scenario (partial — through bed assignment) per [Testing](Testing.md) Section 4, performs the [StateManagement](StateManagement.md) Section 6 anti-duplication checklist across all code written so far, and fixes any gaps found. This checkpoint exists because integration defects compound silently across module boundaries if not caught before Operating Rooms/Inventory add more cross-module event dependencies.

**Scope:** Verification and remediation only, across Phases 4–7.

**Modules involved:** Patients, Medical History, Hospital Map, Beds.

**Files affected:** Any file needing a fix found during this checkpoint (logged individually in [New_files](New_files.md)/[Progress](Progress.md) as they occur, not pre-enumerated).

**Required Reading:** [StateManagement](StateManagement.md) (full), [Testing](Testing.md) Section 4, [DefinitionOfDone](DefinitionOfDone.md).

**Documents to update after completion:** [Progress](Progress.md) (checkpoint result), any document whose described behavior was found inaccurate during this checkpoint.

**Dependencies:** Phase 7.

**Security implications:** Re-run the full security integration suite ([Testing](Testing.md) Section 1) as a regression check.

**Database implications:** None new; verify referential integrity across the four modules' tables with a combined integration test.

**UI implications:** Full manual walkthrough of the registration -> bed assignment flow end to end.

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 8.

**Automatic validation:** Partial `patient-admission-flow.spec.ts` (through bed assignment) green; anti-duplication checklist ([StateManagement](StateManagement.md) Section 6) passes with zero findings.

**Manual validation:** Two-window manual test: assign a bed in window A, confirm window B's Hospital Map and Beds list update without manual refresh.

**Acceptance criteria:** Zero open findings from the anti-duplication checklist; E2E partial scenario green.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2 (adapted: "no new code" criteria are N/A; remediation code follows normal DoD).

**Regression checklist:** Full Phases 0–7 regression.

**Estimated effort:** 2–3 days.

---

## Phase 9 — Operating Rooms Module

**Objective:** Implement OR scheduling (reservation, update, cancellation) with overlap prevention.

**Description:** Migration `0006_operating_rooms.sql`. Implement `models/operating_room.rs`, `repositories/operating_room_repository.rs`, `services/operating_room_service.rs` (overlap-prevention business rule), `commands/operating_room_commands.rs`. Frontend: `modules/operating-rooms/*`, `OrSchedulePage`. Includes `operating_rooms_create` (promotes an existing `rooms` row of type `operating_room` into an `operating_rooms` row) — without it no OR can ever exist to reserve, and mock data is forbidden ([IPC](IPC.md) Section 2.1); surfaced in the `/facility` page built in Phase 6.

**Scope:** Full vertical slice for OR scheduling.

**Modules involved:** Operating Rooms.

**Files affected:** Per [FolderStructure](FolderStructure.md); `src-tauri/migrations/0006_*.sql`.

**Required Reading:** [Database](Database.md) Section 3.4, [IPC](IPC.md) (`operating_rooms_*` rows), [StateManagement](StateManagement.md) Section 4 (Hospital Map invalidation).

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 8.

**Security implications:** Standard session checks; audit coverage per [Audit](Audit.md) Section 2.

**Database implications:** `operating_rooms`, `or_reservations` live; overlap-prevention tested directly (two overlapping reservation attempts, second must fail with `AppError::Conflict`).

**UI implications:** Calendar/list schedule view; Hospital Map reflects OR-in-use status live.

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 9.

**Automatic validation:** Integration tests for overlap rejection, valid non-overlapping scheduling, cancellation freeing the slot.

**Manual validation:** Schedule two overlapping reservations, confirm the second is rejected with a clear message; schedule a valid one, confirm it appears on the Hospital Map.

**Acceptance criteria:** "Operating Room Scheduling" workflow step ([README_Project](README_Project.md) Section 2) fully functional.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Phases 0–8 CI + manual flows.

**Estimated effort:** 4 days.

---

## Phase 10 — Inventory Module

**Objective:** Implement medicine/supply/equipment inventory with transactions and maintenance scheduling, serving as the shared consultation point for Pharmacy and Laboratory.

**Description:** Migration `0007_inventory.sql`. Implement `models/inventory.rs`, `repositories/inventory_repository.rs`, `services/inventory_service.rs` (maintains the running `quantity` total per [Database](Database.md) Section 3.5, generates low-stock/expiration triggers consumed by Phase 11's Notifications), `commands/inventory_commands.rs`. Frontend: `modules/inventory/*`, `InventoryListPage`, `InventoryItemDetailPage`. Includes `inventory_create_category` / `inventory_list_categories` — `inventory_items.category_id` is `NOT NULL`, so no item can be created until at least one category exists, and mock data is forbidden ([IPC](IPC.md) Section 2.1).

**Scope:** Full vertical slice for Inventory (items, transactions, maintenance schedules).

**Modules involved:** Inventory.

**Files affected:** Per [FolderStructure](FolderStructure.md); `src-tauri/migrations/0007_*.sql`.

**Required Reading:** [Database](Database.md) Section 3.5, [IPC](IPC.md) (`inventory_*` rows), [Architecture](Architecture.md) Section 3 (Pharmacy/Lab-as-consumers note).

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 8.

**Security implications:** Standard session checks; audit coverage per [Audit](Audit.md) Section 2.

**Database implications:** `inventory_categories`, `inventory_items`, `inventory_transactions`, `maintenance_schedules` live; running-quantity denormalization ([Database](Database.md) Section 3.5) tested for consistency after concurrent-style transaction sequences.

**UI implications:** Shared list/detail view usable identically by any authenticated user (Pharmacy/Lab consultation, per the "same UI for all roles" project rule).

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 10.

**Automatic validation:** Integration tests for transaction recording, running-quantity correctness, low-stock threshold detection.

**Manual validation:** Record a consumption transaction tied to a patient encounter (via Medical History's treatment linkage, Phase 5), confirm quantity updates and, once Phase 11 lands, a low-stock notification fires.

**Acceptance criteria:** "Shared Consultation by Pharmacy and Laboratory" workflow step ([README_Project](README_Project.md) Section 2) fully functional via the single Inventory module.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Phases 0–8 CI + manual flows.

**Estimated effort:** 4–5 days.

---

## Phase 11 — Notifications Module

**Objective:** Implement system-generated notifications (medicine expiration, maintenance due, low stock) and their delivery/read-state UI.

**Description:** Migration `0008_notifications.sql`. Implement `models/notification.rs`, `repositories/notification_repository.rs`, `services/notification_service.rs` (triggered by Inventory service events — expiration date checks, low-stock threshold breaches, maintenance due-date checks — implemented as a scheduled/on-mutation check invoked from `inventory_service.rs`, not a separate background daemon, keeping the process model simple per [Architecture](Architecture.md) Section 1), `commands/notification_commands.rs`. Frontend: `modules/notifications/*`, `NotificationListPage`, top-bar bell.

**Scope:** Full vertical slice for Notifications, wired to Inventory as its primary trigger source.

**Modules involved:** Notifications (consumes Inventory events).

**Files affected:** Per [FolderStructure](FolderStructure.md); `src-tauri/migrations/0008_*.sql`.

**Required Reading:** [Database](Database.md) Section 3.6, [IPC](IPC.md) (`notifications_*` rows), [StateManagement](StateManagement.md) Section 5 (optimistic-update exception for `mark_read`).

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 9, Phase 10 (notification triggers reference OR schedule and Inventory data).

**Security implications:** Standard session checks. Notification content follows the no-PHI-in-logs spirit ([Rules](Rules.md) 12.1) applied to notification messages too — messages reference entity ids/types, not clinical detail.

**Database implications:** `notifications` live.

**UI implications:** Top-bar unread-count bell ([UI](UI.md) Section 2), list page with mark-as-read (the one documented optimistic-update exception, [StateManagement](StateManagement.md) Section 5).

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 11.

**Automatic validation:** Integration tests: create an inventory item expiring within threshold -> confirm a notification is generated; record a low-stock transaction -> confirm a notification is generated.

**Manual validation:** Trigger a low-stock transaction, observe the bell count increment live via the event system.

**Acceptance criteria:** "Notifications: Medicine expiration, Maintenance alerts" ([README_Project](README_Project.md) Section 3) fully functional.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Phases 0–10 CI + manual flows.

**Estimated effort:** 3 days.

---

## Phase 12 — Billing Module (Simulation)

**Objective:** Implement billing simulation generation, aggregating an encounter's room/treatment/inventory/OR charges — explicitly not a real payment system.

**Description:** Migration `0009_billing.sql`. Implement `models/billing.rs`, `repositories/billing_repository.rs`, `services/billing_service.rs` (aggregation logic pulling from encounter's bed assignments, treatments/inventory consumption, OR reservations), `commands/billing_commands.rs`. Frontend: `modules/billing/*`, `BillingTab` within `PatientDetailPage`.

**Scope:** Full vertical slice for Billing Simulation.

**Modules involved:** Billing (reads across Medical History, Beds, Operating Rooms, Inventory — the most cross-module-dependent service in the system).

**Files affected:** Per [FolderStructure](FolderStructure.md); `src-tauri/migrations/0009_*.sql`.

**Required Reading:** [Database](Database.md) Section 3.8, [IPC](IPC.md) (`billing_*` rows), [Architecture](Architecture.md) Section 3 (module table ownership — Billing owns none of the data it aggregates).

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 5 (treatments), Phase 9 (OR), Phase 10 (inventory) — all must be complete since Billing aggregates from all three.

**Security implications:** Standard session checks; audit coverage per [Audit](Audit.md) Section 2. Explicit UI/copy labeling ("Simulation Only — Not a real invoice") to prevent any real-world billing-accuracy assumption, a product-correctness safeguard adjacent to security.

**Database implications:** `billing_simulations`, `billing_items` live; running-total denormalization ([Database](Database.md) Section 3.8) tested for consistency.

**UI implications:** Itemized billing breakdown view within the patient detail hub.

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 12.

**Automatic validation:** Integration test generating a simulation for an encounter with known treatments/inventory/OR/bed charges, asserting the total matches the expected sum.

**Manual validation:** Generate a billing simulation for a test encounter with diverse charges, confirm the itemization and total are correct and clearly labeled as simulation-only.

**Acceptance criteria:** "Billing Generation (Simulation Only)" workflow step ([README_Project](README_Project.md) Section 2) fully functional.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2.

**Regression checklist:** Phases 0–11 CI + manual flows.

**Estimated effort:** 4 days.

---

## Phase 13 — Patient Discharge & Permanent History Preservation

**Objective:** Wire the discharge workflow step end to end: close the encounter, release any held bed, finalize billing, and confirm the medical history remains permanently queryable.

**Description:** Complete `medical_history_service::discharge_encounter` (started in Phase 5) to orchestrate: set `encounters.status = 'discharged'`, auto-release any active bed assignment for the encounter (calling `bed_service`), and surface a discharge action in the UI that checks/prompts for billing finalization. No new tables.

**Scope:** Orchestration across existing modules; no new vertical slice.

**Modules involved:** Patients, Medical History, Beds, Billing.

**Files affected:** `services/medical_history_service.rs`, `services/bed_service.rs` (cross-service call), `modules/medical-history/*` (discharge action UI).

**Required Reading:** [Database](Database.md) Section 7, [README_Project](README_Project.md) Section 2 (full workflow), [Audit](Audit.md) Section 2.

**Documents to update after completion:** [New_files](New_files.md).

**Dependencies:** Phase 12.

**Security implications:** Discharge is audited (`encounter.discharge`, already listed in [Audit](Audit.md) Section 2) with before/after state capturing the encounter closure.

**Database implications:** No schema change; verifies the cross-table transaction (encounter status + bed release) commits atomically.

**UI implications:** Discharge confirmation dialog ([UI](UI.md) Section 4 — destructive/significant action pattern), post-discharge read-only rendering of the encounter's full history.

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 13.

**Automatic validation:** Full `patient-admission-flow.spec.ts` E2E scenario ([Testing](Testing.md) Section 4) green end to end, including post-discharge history query.

**Manual validation:** Complete the full workflow manually for one patient from registration through discharge, confirm the bed is freed, billing is finalized, and the full history remains visible and correctly ordered.

**Acceptance criteria:** The entire patient-centered workflow from [README_Project](README_Project.md) Section 2 works without gaps.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 2 and Section 3 (project-level workflow criterion).

**Regression checklist:** Full Phases 0–12 regression.

**Estimated effort:** 3 days.

---

## Phase 14 — Hardening, Full Regression & Release

**Objective:** Final security/performance hardening pass, full regression suite, and first release build.

**Description:** Run `npm audit`/`cargo audit` and remediate findings; run the full E2E suite ([Testing](Testing.md) Section 4, all scenarios); perform the manual accessibility pass across every page ([UI](UI.md) Section 8); verify every command requires a valid session (Rule from Phase 2 re-checked exhaustively against the final [IPC](IPC.md) catalog); verify no mock data/dead code/unused files remain repository-wide ([Rules](Rules.md) Section 17); produce the first `tauri build` release artifact.

**Scope:** Whole repository.

**Modules involved:** all.

**Files affected:** Any file with a finding; primarily `tauri.conf.json` for release build config.

**Required Reading:** Every document in `.claude/` (full framework review).

**Documents to update after completion:** [Progress](Progress.md) (final), [New_files](New_files.md) (final).

**Dependencies:** Phase 13.

**Security implications:** This phase is the final security gate — CVE remediation, capability/permission audit, CSP verification in the release build.

**Database implications:** Verify migration history is clean and reproducible from an empty file to the final schema in one pass.

**UI implications:** Full accessibility and loading/empty/error triad audit across all pages.

**Tasks executed during this phase:** See [Tasks](Tasks.md) Phase 14.

**Automatic validation:** Full CI pipeline ([Testing](Testing.md) Section 7) green including the E2E suite, on the release build artifact.

**Manual validation:** Install and run the built release artifact on a clean machine/VM; walk through the full workflow.

**Acceptance criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 3 (Project-Level Definition of Done) fully satisfied.

**Phase Exit Criteria:** [DefinitionOfDone](DefinitionOfDone.md) Section 3.

**Regression checklist:** All prior phases' regression checklists, run once more in full.

**Estimated effort:** 4–5 days.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#planmd--project-director)
