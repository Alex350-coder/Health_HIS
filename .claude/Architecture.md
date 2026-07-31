# Architecture.md

## Purpose

Defines the complete system architecture: layers, responsibilities, module boundaries, communication rules, data flow, and the tech stack decision record.

## Dependencies

[Rules](Rules.md) Section 1.

## Documents that must be read before using it

[Rules](Rules.md).

## Referenced By

[Audit](Audit.md), [CodingStandards](CodingStandards.md), [Database](Database.md), [ErrorHandling](ErrorHandling.md), [FolderStructure](FolderStructure.md), [Glossary](Glossary.md), [IPC](IPC.md), [Plan](Plan.md), [README_Project](README_Project.md), [Routes](Routes.md), [Rules](Rules.md), [Security](Security.md), [StateManagement](StateManagement.md), [Testing](Testing.md), [UI](UI.md), [Validation](Validation.md).

## Documents that must be updated if changes occur

[FolderStructure](FolderStructure.md), [Database](Database.md), [Security](Security.md), [StateManagement](StateManagement.md), [IPC](IPC.md), [Plan](Plan.md).

## Priority

4.

---

## Contents

- [1. Process Model (Tauri)](#1-process-model-tauri)
- [2. Layers](#2-layers)
- [3. Module Boundaries](#3-module-boundaries)
- [4. Data Flow (example: Bed Assignment)](#4-data-flow-example-bed-assignment)
- [5. Dependency Graph (module/layer level)](#5-dependency-graph-modulelayer-level)
- [6. Tech Stack (Decision Record)](#6-tech-stack-decision-record)

---

## 1. Process Model (Tauri)

Two OS processes:

1. **Core process (Rust)** — owns the SQLite/SQLCipher connection pool, all business logic, all filesystem/OS access, session state, the encryption key (via OS keychain). This is the only process with database access.
2. **WebView process (React/TS, sandboxed)** — renders UI only. Cannot touch the filesystem or database directly. Communicates with the Core process exclusively via Tauri's IPC bridge (`invoke` for request/response, `listen`/`emit` for events).

This split is the primary security boundary: even a compromised/XSS'd WebView cannot read the database or filesystem without going through a validated, allowlisted Tauri command (see [Security](Security.md) Section "Defense in Depth").

## 2. Layers

```
┌─────────────────────────────────────────────┐
│ Presentation        React components/pages   │  src/modules/*/components
├─────────────────────────────────────────────┤
│ Application State   TanStack Query + Zustand │  src/modules/*/hooks, api
├─────────────────────═══ IPC BOUNDARY ═══─────┤
│ Command             Tauri #[command] fns     │  src-tauri/src/commands
├─────────────────────────────────────────────┤
│ Service             Business logic/orchestr. │  src-tauri/src/services
├─────────────────────────────────────────────┤
│ Repository          Data access (SQL)        │  src-tauri/src/repositories
├─────────────────────────────────────────────┤
│ Database            SQLite + SQLCipher       │  src-tauri/migrations, db/
└─────────────────────────────────────────────┘
```

Layer call rule (see [Rules](Rules.md) 1.2): a layer calls only the layer directly beneath it. Presentation never calls Command directly — it calls Application State hooks, which call `invoke`, which crosses into Command.

### 2.1 Presentation Layer

React function components. No business logic, no direct data fetching (Rule 6.3). Consumes hooks from the module's `api/` folder. Responsible for layout, user interaction, and delegating to `shared/ui` design-system primitives (see [UI](UI.md)).

### 2.2 Application State Layer

- **TanStack Query**: server-state cache. Every entity (patients, beds, encounters, ...) has query keys and hooks defined once per module in `api/`. This is the *only* place `invoke()` is called for data reads/writes. See [StateManagement](StateManagement.md).
- **Zustand**: ephemeral, client-only UI state that has no DB representation (active module tab, dialog open/closed, current wizard step, current authenticated user session summary). Zustand never stores a copy of server data.

### 2.3 Command Layer (Rust, `#[tauri::command]`)

Thin adapters. Deserialize input (already Zod-validated client-side, but re-validated here per Rule 8.2/11.1), call exactly one service method, serialize `Result<T, AppError>` back. Full catalog in [IPC](IPC.md).

### 2.4 Service Layer

All business rules live here: workflow rules (e.g., "a bed cannot be assigned if already occupied," "an encounter must exist before a diagnosis can be registered"), orchestration across multiple repositories within a transaction, audit log writing (via `audit_service`), and Tauri event emission after a successful mutation.

### 2.5 Repository Layer

Pure data access. One repository per aggregate root. Parameterized queries only (Rule 9.1). Maps SQL rows to `models/` structs. No validation, no business rules, no cross-repository calls (that belongs in services).

### 2.6 Database Layer

See [Database](Database.md) for full schema. SQLite file encrypted at rest via SQLCipher; WAL journal mode for concurrent read performance; migrations are the only way schema changes.

## 3. Module Boundaries

Ten modules, each a vertical slice spanning all layers: **Authentication, Patients, Medical History, Hospital Map, Beds, Operating Rooms, Inventory, Notifications, Audit, Billing (Simulation)**.

| Module | Owns (DB tables) | Special notes |
|---|---|---|
| Authentication | `users`, `sessions` | Gate for every other module; see [Security](Security.md). Also owns user management and the first-run bootstrap path ([Security](Security.md) Section 9.1). |
| Patients | `patients` | Root entity of the workflow. |
| Medical History | `encounters`, `diagnoses`, `treatments`, `evolutions` | Append-only; see [Database](Database.md) "Permanent Medical History." |
| Hospital Map | (reads `floors`, `rooms` — read-only) | **Never writes.** Pure visualization layer over data owned by Beds/Operating Rooms. |
| Beds | `floors`, `rooms`, `beds`, `bed_assignments` | Owns physical space + bed state, and therefore owns the **facility-configuration write path** (`beds_create_floor`/`_room`/`beds_create`/`beds_set_status`) that Hospital Map reads but must never provide — see [IPC](IPC.md) Section 2.1. |
| Operating Rooms | `operating_rooms`, `or_reservations` | Scheduling only; no clinical data duplication (references `encounter_id`). |
| Inventory | `inventory_categories`, `inventory_items`, `inventory_transactions`, `maintenance_schedules` | Consulted (read + transact) by Pharmacy/Lab workflows — no separate Pharmacy/Lab tables; they are consumers of Inventory + Medical History. |
| Notifications | `notifications` | Derived/generated by other services (expiry, maintenance) — see [Audit](Audit.md)/[ErrorHandling](ErrorHandling.md) for triggers. |
| Audit | `audit_log` | Written to by every service via `audit_service`; never written to directly by repositories. |
| Billing (Simulation) | `billing_simulations`, `billing_items` | Simulation only — no real payment processing, no external gateway integration. |

> [!WARNING]
> **Cross-module rule:** `Hospital Map` owns no mutation path — it is explicitly read-only per the project spec ("It never modifies data"). Its backend commands are all `get_*`, never `create/update/delete`. This is verified mechanically in Phase 6 (see [Plan](Plan.md)) by grepping the module's command surface for write verbs.

## 4. Data Flow (example: Bed Assignment)

```
User clicks "Assign Bed" (Presentation: BedAssignmentDialog)
  -> useAssignBedMutation() (Application State: beds/api/use-assign-bed.ts)
    -> invoke('beds_assign', { patientId, encounterId, bedId })  [IPC boundary]
      -> beds_assign command (Command: bed_commands.rs) — re-validates input
        -> BedService::assign(...) (Service: bed_service.rs)
           - checks bed.status == Available (business rule)
           - opens DB transaction
           - BedRepository::insert_assignment(...)
           - BedRepository::update_bed_status(...)
           - AuditService::record("bed.assign", ...)
           - commits transaction
           - EventEmitter::emit("beds:assignment:created", payload)
        <- Ok(BedAssignment)
      <- Ok(BedAssignment) serialized
    <- TanStack Query invalidates ['beds'], ['bed-assignments', patientId]
  -> All open windows/components subscribed to 'beds:assignment:created' also invalidate (see [StateManagement](StateManagement.md))
-> UI re-renders with updated bed status everywhere, instantly, without duplicated local state.
```

## 5. Dependency Graph (module/layer level)

```
Presentation ──depends on──▶ Application State ──depends on──▶ (IPC) ──▶ Command
Command ──depends on──▶ Service ──depends on──▶ Repository ──depends on──▶ Database
Service ──depends on──▶ Audit Service, Event Emitter (cross-cutting)
All Services ──depend on──▶ Security (session validation on every command, see [Security](Security.md))
```

No reverse dependencies are permitted (a Repository never calls a Service; Database never calls Rust code).

## 6. Tech Stack (Decision Record)

| Concern | Choice | Justification |
|---|---|---|
| Desktop shell | Tauri 2.x | Small binary, Rust-secured core process, native OS webview (no bundled Chromium), fine-grained capability permissions — see [Security](Security.md). |
| Frontend framework | React 18 + TypeScript (strict) | Team requirement; large ecosystem for accessible component primitives. |
| Bundler | Vite | Native Tauri integration, fast dev server. |
| Server-state | TanStack Query | Cache invalidation model maps directly to the "single source of truth" requirement (Rule 13). |
| Client UI state | Zustand | Minimal boilerplate, no Provider-hell, easy to keep strictly separate from server-state per Rule 13.1. |
| Forms | react-hook-form + Zod | Schema-first validation reusable between form UX and API-boundary parsing. |
| Routing | TanStack Router | Type-safe routes, built-in loader/guard support — see [Routes](Routes.md). |
| UI primitives | Radix UI + Tailwind CSS (shadcn/ui pattern) | Accessible (WAI-ARIA compliant) unstyled primitives — satisfies [Rules](Rules.md) 19 without hand-rolling ARIA. |
| Backend language | Rust | Tauri's native core language; memory safety; strong `Result`-based error handling fits Rule 8.3/14. |
| Database | SQLite + SQLCipher (via `rusqlite` + `sqlcipher` feature, or `sqlx` with a SQLCipher-enabled libsqlite3) | See [Database](Database.md) Section 1 for full justification. |
| Password hashing | `argon2` crate (Argon2id) | Industry-standard, memory-hard, resistant to GPU cracking — see [Security](Security.md). |
| Secrets/key storage | `keyring` crate (OS keychain: Credential Manager / Keychain / Secret Service) | The SQLCipher key must never live in a plaintext file — see [Security](Security.md). |
| Testing (TS) | Vitest + React Testing Library | Vite-native, fast. |
| Testing (Rust) | Built-in `cargo test` + `rstest` for parameterized cases | Standard toolchain. |
| E2E | `tauri-driver` + WebdriverIO | Only mature, actively maintained E2E stack for Tauri apps as of this writing. |

Any addition to this table requires an update to this document in the same PR (Rule 5.1).

### 6.1 Toolchain Dependencies (added in Phase 0)

These are development-only dependencies. None is bundled into the shipped application: the
production bundle contains React, React DOM and `@tauri-apps/api` only.

| Package | Justification |
|---|---|
| `@tauri-apps/cli` | Drives `tauri dev` / `tauri build`; the counterpart of the `tauri` crate. |
| `@vitejs/plugin-react` | React Fast Refresh and JSX transform for Vite. |
| `typescript` | Pinned to `^5.9`, **not** 7.x, because `typescript-eslint@8` declares a peer range of `>=4.8.4 <6.1.0`. Revisit when typescript-eslint 9 ships. |
| `typescript-eslint` | The v8 unified package replacing the separate `@typescript-eslint/{parser,eslint-plugin}` entries; required for `recommendedTypeChecked` (CodingStandards Section 1). |
| `eslint` | Pinned to `^9`, **not** 10, because `eslint-plugin-jsx-a11y@6.10.2` caps its peer range at `eslint ^9`. See the Phase 0 blocker in [Progress](Progress.md). |
| `eslint-plugin-import` + `eslint-import-resolver-typescript` | Supplies `import/order` and `import/no-cycle`, mandated by Rules 4.1–4.3; the resolver teaches them the `@app`/`@modules`/`@shared` path aliases. |
| `eslint-plugin-jsx-a11y` | Mandated by CodingStandards Section 1; enforces Rules 19 (accessibility) at lint time. |
| `eslint-plugin-react-hooks` | Mandated by CodingStandards Section 1; enforces the Rules of Hooks. |
| `eslint-plugin-react-refresh` | Guards the Fast Refresh contract (component files export components only). |
| `globals` | Supplies the browser/node global sets to the ESLint flat config. |
| `prettier` | The formatter fixed by CodingStandards Section 1. |
| `husky` + `lint-staged` | Pre-commit enforcement per [DevelopmentWorkflow](DevelopmentWorkflow.md) Section 5. |
| `vitest` + `@vitest/coverage-v8` + `jsdom` | Test runner, coverage provider and DOM environment for the Rules 15.1 thresholds. |
| `@testing-library/{react,dom,jest-dom,user-event}` | Behaviour-focused component testing per [Testing](Testing.md). |
| `@types/node` | Types for `node:path` and `import.meta.dirname`, used by `vite.config.ts`. |
| `@types/react`, `@types/react-dom` | React 18 type definitions. |

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#architecturemd)
