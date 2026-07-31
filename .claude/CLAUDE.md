# CLAUDE.md — Project Context

> **Hospital Information System (HIS)** — a secure, offline-first desktop MVP.
> **Tauri 2.x** · **React 18 + TypeScript** · **Rust** · **SQLite + SQLCipher**

## Purpose

Single-file orientation for anyone (human or LLM) starting work in this repository. It states what the project is, how it is built, the constraints that cannot be violated, and where the authoritative detail lives. It is a **map, not an authority** — every statement here is a summary of a document in `.claude/`, and where this file and a `.claude/` document disagree, the `.claude/` document wins.

## Dependencies

The 22-document planning framework in [`.claude/`](.claude/README_Project.md). This file duplicates none of it in full and cannot override any of it.

## Priority

0 (orientation layer — read first, authority rests with [Rules](.claude/Rules.md)).

## Documents that must be updated if changes occur

None. This file is *downstream* of the framework: it is updated when `.claude/` changes, never the reverse.

---

## Contents

- [1. Current Status](#1-current-status)
- [2. What This Project Is](#2-what-this-project-is)
- [3. Non-Negotiable Constraints](#3-non-negotiable-constraints)
- [4. The Documentation Framework](#4-the-documentation-framework)
- [5. Architecture](#5-architecture)
- [6. Repository Layout](#6-repository-layout)
- [7. Domain Model](#7-domain-model)
- [8. Conventions Quick Reference](#8-conventions-quick-reference)
- [9. Commands](#9-commands)
- [10. Phase Roadmap](#10-phase-roadmap)
- [11. How To Work In This Repository](#11-how-to-work-in-this-repository)

---

## 1. Current Status

> [!NOTE]
> **No implementation exists yet.** The repository currently contains only the `.claude/` planning framework. Every phase in [Progress](.claude/Progress.md) is `Not Started`, including Phase 0 (project bootstrap and tooling).
>
> The first work item is **Task 0.1** in [Tasks](.claude/Tasks.md). Do not create `src/`, `src-tauri/`, or any config file outside the tree defined in [FolderStructure](.claude/FolderStructure.md).

## 2. What This Project Is

A secure desktop **Hospital Information System** MVP demonstrating software architecture, clean code, relational database design, cross-module state synchronization, and security engineering. Security is a first-class citizen, not an afterthought.

It is deliberately **not** a full commercial HIS. Billing is a simulation with no payment processor. There is no networking, no multi-tenancy, and no cloud component — it is a single-user, single-device desktop application.

### Stack

| Concern | Choice |
|---|---|
| Desktop shell | Tauri 2.x — Rust core process + sandboxed native WebView |
| Frontend | React 18, TypeScript (`strict: true`), Vite |
| Backend | Rust — command / service / repository layers |
| Database | SQLite, encrypted at rest with SQLCipher (AES-256), via `rusqlite` (`bundled-sqlcipher`) |
| Server state | TanStack Query (the only server-state cache) |
| UI state | Zustand (ephemeral, no DB representation) |
| Routing | TanStack Router |
| Forms / validation | react-hook-form + Zod (client) · `validator` crate (server, authoritative) |
| UI primitives | Radix UI + Tailwind CSS (shadcn/ui pattern) |
| Passwords | Argon2id |
| Secrets | OS keychain via the `keyring` crate |
| Testing | Vitest + RTL (TS) · `cargo test` + `rstest` (Rust) · `tauri-driver` + WebdriverIO (E2E) |

Full decision record with rejected alternatives: [Architecture](.claude/Architecture.md) Section 6 and [Database](.claude/Database.md) Section 1. **No dependency may be added without a justification entry in that table** (Rule 5.1).

## 3. Non-Negotiable Constraints

These shape every other decision. Violating one is a defect regardless of whether tests pass.

1. **No mock data — ever.** No hardcoded arrays of fake entities, no `faker` data in application code, no temporary placeholder state. Every feature reads and writes the real database from its first commit. Dev seed data, if any, lives in a clearly named `dev-seed` script never imported by production paths. (Rule 17.1)
2. **The frontend never touches the database.** All persistence goes through Tauri `invoke` commands. The absence of any SQLite driver in `package.json` is the mechanical proof. (Rule 1.3)
3. **The Hospital Map never modifies data.** It is a pure read-only visualization over tables owned by Beds and Operating Rooms. Its commands are all `get_*`. (Architecture Section 3)
4. **Authentication gates everything**, and every authenticated user sees the same application — auth exists for audit, traceability, and access protection, not for role-gated UI. Exactly three commands are reachable without a session: `auth_login`, `auth_bootstrap_status`, `auth_bootstrap_admin`. A fourth is a rule violation. ([IPC](.claude/IPC.md) Section 1)
5. **Clinical records are append-only.** Encounters, diagnoses, treatments, and evolutions are never physically deleted. A correction is a new row referencing the row it corrects. (Rule 9.5)
6. **One writable source of truth: the database.** Frontend state derived from it is a cache that is *invalidated*, never manually patched. No `useState` mirror of server data. (Rule 13.1)
7. **Every input crossing IPC is validated twice** — Zod client-side for UX, Rust server-side as the authoritative check. (Rule 11.1)
8. **No PHI in logs.** Application logs reference entity IDs, never clinical content. Audit logging is a separate mechanism governed by [Audit](.claude/Audit.md). (Rule 12.1)
9. **No `any` (TS), no `unwrap()`/`expect()` (Rust) outside tests.** Every command returns `Result<T, AppError>`. (Rules 7.2, 8.3)
10. **Docs and code never diverge.** A PR changing architecture, schema, security behavior, or contracts updates the corresponding `.claude/` document in the same PR. (Rule 16.3)

## 4. The Documentation Framework

22 documents in [`.claude/`](.claude/README_Project.md). Each declares its Purpose, Dependencies, prerequisite reading, Referenced By, downstream documents, and Priority.

### Authority order

```
Rules
  -> Architecture
    -> Security
      -> Database
        -> Validation / ErrorHandling / Audit / StateManagement / IPC
          -> UI / Routes / Testing
            -> Plan
              -> Tasks
                -> Progress / New_files
```

Higher authority always prevails. **No document may contradict [Rules](.claude/Rules.md).**

### Index

| Document | What it answers |
|---|---|
| [Rules](.claude/Rules.md) | The 21 groups of objective, verifiable rules. Highest authority. |
| [DefinitionOfDone](.claude/DefinitionOfDone.md) | When is a task / phase / the project actually complete. |
| [Architecture](.claude/Architecture.md) | Layers, module boundaries, data flow, tech-stack decisions. |
| [FolderStructure](.claude/FolderStructure.md) | The canonical repository tree. Nothing exists outside it. |
| [CodingStandards](.claude/CodingStandards.md) | Formatting, naming, idioms, comment policy. |
| [Database](.claude/Database.md) | Full schema, constraints, indexes, migration strategy. |
| [Security](.claude/Security.md) | AuthN/AuthZ, hashing, sessions, rate limiting, secrets, bootstrap. |
| [Validation](.claude/Validation.md) | The two-layer validation contract and its duplication table. |
| [ErrorHandling](.claude/ErrorHandling.md) | The `AppError` taxonomy and per-category strategy. |
| [Audit](.claude/Audit.md) | What is audited, hash-chain tamper evidence, immutability. |
| [Testing](.claude/Testing.md) | Test strategy, coverage gates, CI pipeline. |
| [UI](.claude/UI.md) | Design system, states, accessibility. |
| [Routes](.claude/Routes.md) | Route map and guards. |
| [StateManagement](.claude/StateManagement.md) | Query keys, invalidation, event propagation, anti-duplication. |
| [IPC](.claude/IPC.md) | The complete Tauri command and event catalog. |
| [Glossary](.claude/Glossary.md) | Clinical and technical vocabulary. |
| [DevelopmentWorkflow](.claude/DevelopmentWorkflow.md) | Branching, commits, PRs, local commands, release. |
| [Plan](.claude/Plan.md) | The 15 phases, each with objective, scope, exit criteria. |
| [Tasks](.claude/Tasks.md) | Atomic implementation tasks per phase. |
| [Progress](.claude/Progress.md) | Live status per phase and task. |
| [New_files](.claude/New_files.md) | Log of every file created and why. |
| [README_Project](.claude/README_Project.md) | Framework entry point and orientation. |

## 5. Architecture

### Process model

Two OS processes. This split is the primary security boundary — a compromised WebView still cannot reach the database or filesystem without passing through a validated, allowlisted command.

- **Core process (Rust)** — owns the SQLCipher connection, all business logic, all filesystem/OS access, session state, and the encryption key.
- **WebView process (React/TS, sandboxed)** — renders UI only. Talks to the core exclusively via `invoke` (request/response) and `listen`/`emit` (events).

### Layers

```
Presentation        React components/pages          src/modules/*/components
Application State   TanStack Query + Zustand        src/modules/*/hooks, api
══════════════════ IPC BOUNDARY ═══════════════════
Command             Tauri #[command] fns            src-tauri/src/commands
Service             Business logic / orchestration  src-tauri/src/services
Repository          Data access (SQL only)          src-tauri/src/repositories
Database            SQLite + SQLCipher              src-tauri/migrations, db/
```

A layer calls **only** the layer directly beneath it (Rule 1.2). Commands are thin adapters — validate, call one service method, map the result. Services hold all business rules and own transactions, audit writes, and event emission. Repositories are pure SQL with no rules and no cross-repository calls.

### Cross-module communication

Only two channels are permitted (Rule 1.5):
1. The database, as single source of truth.
2. Tauri events (`<module>:<entity>:<action>`) for real-time propagation.

Direct imports between two modules' component/hook trees are forbidden — shared code lives in `src/shared/`. Events map to query-key invalidations in exactly one place: `shared/lib/event-query-map.ts`.

## 6. Repository Layout

```
Health_Project/
├── .claude/          # Planning framework (22 documents)
├── CLAUDE.md         # This file
├── src/              # React + TypeScript frontend
│   ├── app/          #   App.tsx, router.tsx, providers/
│   ├── modules/      #   10 modules, each: components/ hooks/ api/ types/
│   ├── shared/       #   ui/ components/ hooks/ lib/ schemas/ errors/
│   └── styles/
├── src-tauri/        # Rust backend
│   ├── capabilities/ #   Least-privilege permission manifests
│   ├── migrations/   #   Forward-only numbered SQL
│   └── src/          #   main.rs, db/, models/, repositories/, services/,
│                     #   commands/, security/, errors/, validation/, events/
├── tests/e2e/        # Cross-cutting E2E specs
└── .github/workflows/
```

Every module folder — frontend and backend — has the **same internal shape**. That uniformity is itself a rule: it removes all ambiguity about where new code goes. Full tree: [FolderStructure](.claude/FolderStructure.md).

## 7. Domain Model

### Patient-centered workflow

Every module exists to serve one or more steps of this chain. No module exists in isolation.

```
Patient Registration
  -> Medical Record Creation / Retrieval
    -> Hospital Bed Assignment
      -> Diagnosis Registration
        -> Treatment Registration
          -> Medical Evolution Registration
            -> Operating Room Scheduling
              -> Shared Consultation (Pharmacy + Laboratory)
                -> Billing Generation (Simulation Only)
                  -> Patient Discharge
                    -> Permanent Medical History Preservation
```

### Modules and ownership

| Module | Owns | Note |
|---|---|---|
| Authentication | `users`, `sessions` | Gate for everything; owns user management and first-run bootstrap. |
| Patients | `patients` | Root entity of the workflow. |
| Medical History | `encounters`, `diagnoses`, `treatments`, `evolutions` | Append-only. |
| Hospital Map | — (reads `floors`, `rooms`) | **Never writes.** |
| Beds | `floors`, `rooms`, `beds`, `bed_assignments` | Owns the facility-configuration write path. |
| Operating Rooms | `operating_rooms`, `or_reservations` | Scheduling only. |
| Inventory | `inventory_categories`, `inventory_items`, `inventory_transactions`, `maintenance_schedules` | Pharmacy and Lab are *consumers*, not separate tables. |
| Notifications | `notifications` | Generated by other services. |
| Audit | `audit_log` | Hash-chained, append-only, trigger-enforced. |
| Billing (Simulation) | `billing_simulations`, `billing_items` | No real payment processing. |

22 tables total (including `schema_migrations`). Full schema, indexes, and migration-to-phase mapping: [Database](.claude/Database.md).

## 8. Conventions Quick Reference

| Element | Convention | Example |
|---|---|---|
| React component file | `PascalCase.tsx` | `BedAssignmentDialog.tsx` |
| Hook file | `kebab-case.ts`, `use-` prefix | `use-bed-assignment.ts` |
| TS type / interface | `PascalCase` | `BedAssignment` |
| TS constant | `SCREAMING_SNAKE_CASE` | `MAX_BED_CAPACITY` |
| Rust file / module | `snake_case` | `bed_repository.rs` |
| Rust struct / enum | `PascalCase` | `BedAssignment` |
| DB table | `snake_case`, plural | `bed_assignments` |
| DB primary key | always `id` | `id` |
| DB foreign key | `<table_singular>_id` | `patient_id` |
| Tauri command | `<module>_<action>` | `beds_assign` |
| Tauri event | `<module>:<entity>:<action>` | `beds:assignment:created` |
| Query key | `[module, resource, ...params]` | `['beds', 'detail', bedId]` |
| Branch | `phase/<n>-<name>` | `phase/4-patients-module` |
| Commit | `<type>(<module>): <description>` | `feat(beds): add assignment command` |

### Hard limits (Rules 20.x)

Function ≤ 40 executable lines · cyclomatic complexity ≤ 10 · ≤ 4 positional params · nesting depth ≤ 3 · React component file ≤ 300 lines · Rust service/repo file ≤ 400 · hooks/utils ≤ 200.

### Coverage gates (Rule 15.1)

Rust services/repositories 80% · TypeScript business logic 80% · React components 70%. CI fails below.

## 9. Commands

| Command | Purpose |
|---|---|
| `npm install` | Install frontend dependencies. |
| `npm run tauri dev` | Run the full app in dev mode. |
| `npm run lint` | ESLint (`--max-warnings=0` in CI). |
| `npm run typecheck` | `tsc --noEmit`. |
| `npm run test` | Vitest. |
| `npm run test:e2e` | `tauri-driver` + WebdriverIO. |
| `npm run tauri build` | Platform-native installer. |
| `cargo build` / `cargo test` | Rust build and tests (in `src-tauri/`). |
| `cargo fmt --check` / `cargo clippy -D warnings` | Rust formatting and lints. |

> [!NOTE]
> None of these work yet — the toolchain is stood up in Phase 0. Prerequisites: Node.js LTS, Rust stable, and platform Tauri prerequisites (WebView2 on Windows).

## 10. Phase Roadmap

15 phases. Each has an objective, scope, required reading, exit criteria, and a regression checklist in [Plan](.claude/Plan.md); atomic tasks in [Tasks](.claude/Tasks.md).

| # | Phase | Depends on |
|---|---|---|
| 0 | Project Bootstrap & Tooling | — |
| 1 | Database & Migration Engine Foundation | 0 |
| 2 | Security Foundation, Authentication, Audit | 1 |
| 3 | Frontend Application Shell & Design System | 2 |
| 4 | Patients Module | 3 |
| 5 | Medical History Module | 4 |
| 6 | Hospital Map Module | 3 |
| 7 | Beds Module | 5, 6 |
| 8 | Cross-Module Integration Checkpoint | 7 |
| 9 | Operating Rooms Module | 8 |
| 10 | Inventory Module | 8 |
| 11 | Notifications Module | 9, 10 |
| 12 | Billing Module (Simulation) | 5, 9, 10 |
| 13 | Patient Discharge & Permanent History Preservation | 12 |
| 14 | Hardening, Full Regression & Release | 13 |

## 11. How To Work In This Repository

1. **Read [Rules](.claude/Rules.md) in full before writing any code.** It is short, objective, and binding.
2. **Work one phase at a time.** Before starting a phase, read every document in that phase's *Required Reading* list in [Plan](.claude/Plan.md).
3. **Implement only what [Tasks](.claude/Tasks.md) describes** for the active phase — nothing more. No speculative abstraction; a shared helper is extracted on the third *real* use, not the first hypothetical one.
4. **Finish a task properly.** Task-level Definition of Done requires all of: build succeeds, `fmt`/`clippy`/`eslint`/`tsc` clean, tests written and coverage met, `cargo test` and `vitest` green, applicable rules satisfied, audit writes present for critical actions, and — **in the same commit** — [New_files](.claude/New_files.md) and [Progress](.claude/Progress.md) updated. Full checklist: [DefinitionOfDone](.claude/DefinitionOfDone.md).
5. **Never invent requirements.** Ambiguity is a documentation gap: resolve it by extending the relevant `.claude/` document, not by improvising in code.
6. **Never bypass a gate.** No `--no-verify`, no disabled lint rule, no lowered coverage threshold without an explicit, justified instruction from the user.

---

<!-- nav-footer -->
[Documentation Index](.claude/README_Project.md) · [↑ Back to top](#claudemd--project-context)
