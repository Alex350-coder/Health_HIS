# IPC.md

## Purpose

Catalog of the Tauri command (request/response) and event (real-time propagation) contracts that form the boundary between the React frontend and the Rust backend. This is an addition to the mandatory document set because the IPC boundary is the single most critical seam in a Tauri app's architecture and security model — it needs one authoritative catalog, not scattered per-module notes.

## Dependencies

[Rules](Rules.md) Sections 3.5–3.6, 8; [Architecture](Architecture.md) Section 2.3; [ErrorHandling](ErrorHandling.md) Section 1; [StateManagement](StateManagement.md) Section 4.

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md), [ErrorHandling](ErrorHandling.md), [StateManagement](StateManagement.md).

## Referenced By

[Architecture](Architecture.md), [DefinitionOfDone](DefinitionOfDone.md), [ErrorHandling](ErrorHandling.md), [FolderStructure](FolderStructure.md), [Glossary](Glossary.md), [Plan](Plan.md), [README_Project](README_Project.md), [Rules](Rules.md), [StateManagement](StateManagement.md), [Tasks](Tasks.md), [Validation](Validation.md).

## Documents that must be updated if changes occur

[StateManagement](StateManagement.md) (event-to-query-key map), [Tasks](Tasks.md) (each command is one implementation task), [Testing](Testing.md) (each command needs an integration test).

## Priority

6.

---

## Contents

- [1. Command Contract Convention](#1-command-contract-convention)
- [2. Command Catalog](#2-command-catalog)
- [2.1 Why Bootstrap and Facility-Configuration Commands Exist](#21-why-bootstrap-and-facility-configuration-commands-exist)
- [3. Events](#3-events)
- [4. Adding a New Command or Event](#4-adding-a-new-command-or-event)

---

## 1. Command Contract Convention

Every command:
- Is named `<module>_<action>` (Rule 3.5).
- Takes a single struct parameter (even for one field) — consistent shape simplifies validation and future extension.
- Returns `Result<T, AppError>` (see [ErrorHandling](ErrorHandling.md)).
- Requires a valid session (checked via `security::session::require_session`) except exactly three commands, which are the complete and closed set of unauthenticated entry points: `auth_login`, `auth_bootstrap_status`, `auth_bootstrap_admin`. Any fourth unauthenticated command is a rule violation; the exhaustive re-check is [Tasks](Tasks.md) Task 14.2.
- Has a corresponding Zod schema client-side and Rust validator server-side per [Validation](Validation.md) Section 3.

## 2. Command Catalog

| Command | Module | Input | Output | Notes |
|---|---|---|---|---|
| `auth_bootstrap_status` | Authentication | `{}` | `{ needsBootstrap }` | **No session required.** Returns `true` iff `users` is empty. Read-only; leaks no data. See [Security](Security.md) Section 9.1. |
| `auth_bootstrap_admin` | Authentication | `CreateUserInput` | `{ token, user }` | **No session required, but hard-gated: rejects with `AppError::Conflict` unless `COUNT(*) FROM users = 0`, checked inside the same transaction as the insert.** Creates the first `admin` and logs them in. See [Security](Security.md) Section 9.1. |
| `auth_login` | Authentication | `{ username, password }` | `{ token, user }` | No session required. Rate-limited, see [Security](Security.md). |
| `auth_logout` | Authentication | `{}` | `{}` | Deletes session row. |
| `auth_current_user` | Authentication | `{}` | `{ user }` | Used on app start to restore session context if token still in memory. |
| `auth_create_user` | Authentication | `CreateUserInput` | `User` | Creates a further user. Audited as `user.create` ([Audit](Audit.md) Section 2). |
| `auth_list_users` | Authentication | `{}` | `User[]` | Never returns `password_hash`. |
| `auth_deactivate_user` | Authentication | `{ id }` | `User` | Soft state change (`is_active = 0`), never a delete. Audited as `user.deactivate`. |
| `patients_create` | Patients | `CreatePatientInput` | `Patient` | |
| `patients_update` | Patients | `UpdatePatientInput` | `Patient` | |
| `patients_get` | Patients | `{ id }` | `Patient` | |
| `patients_list` | Patients | `{ search?, limit, offset }` | `Patient[]` | |
| `medical_history_create_encounter` | Medical History | `CreateEncounterInput` | `Encounter` | |
| `medical_history_discharge_encounter` | Medical History | `{ encounterId, dischargeSummary }` | `Encounter` | Triggers permanent-history finalization. |
| `medical_history_get_by_patient` | Medical History | `{ patientId }` | `MedicalHistoryBundle` | Aggregates encounters+diagnoses+treatments+evolutions. |
| `medical_history_create_diagnosis` | Medical History | `CreateDiagnosisInput` | `Diagnosis` | |
| `medical_history_create_treatment` | Medical History | `CreateTreatmentInput` | `Treatment` | May reference an `inventory_item_id`; triggers an `inventory_transactions` row via `inventory_service`. |
| `medical_history_create_evolution` | Medical History | `CreateEvolutionInput` | `Evolution` | |
| `hospital_map_get_layout` | Hospital Map | `{}` | `Floor[]` (with nested `Room[]`) | Read-only. |
| `hospital_map_get_room_status` | Hospital Map | `{ roomId }` | `RoomStatus` | Read-only; aggregates bed/OR occupancy for display. |
| `beds_list` | Beds | `{ roomId? }` | `Bed[]` | |
| `beds_assign` | Beds | `{ bedId, patientId, encounterId }` | `BedAssignment` | Enforces bed availability business rule. |
| `beds_release` | Beds | `{ bedAssignmentId }` | `BedAssignment` | |
| `beds_create_floor` | Beds | `CreateFloorInput` | `Floor` | Facility configuration — see Section 2.1. |
| `beds_create_room` | Beds | `CreateRoomInput` | `Room` | Facility configuration. Sets `map_x`/`map_y` consumed by Hospital Map. |
| `beds_create` | Beds | `CreateBedInput` | `Bed` | Facility configuration. |
| `beds_set_status` | Beds | `{ bedId, status }` | `Bed` | `available`/`maintenance` only; occupancy is set by assign/release, never directly. Audited as `bed.status_change`. |
| `operating_rooms_create` | Operating Rooms | `CreateOperatingRoomInput` | `OperatingRoom` | Facility configuration — promotes an existing `room` of type `operating_room`. |
| `operating_rooms_list` | Operating Rooms | `{}` | `OperatingRoom[]` | |
| `operating_rooms_reserve` | Operating Rooms | `CreateOrReservationInput` | `OrReservation` | Enforces overlap-prevention business rule. |
| `operating_rooms_update_reservation` | Operating Rooms | `UpdateOrReservationInput` | `OrReservation` | |
| `operating_rooms_cancel_reservation` | Operating Rooms | `{ id }` | `OrReservation` | |
| `inventory_list_categories` | Inventory | `{}` | `InventoryCategory[]` | |
| `inventory_create_category` | Inventory | `CreateInventoryCategoryInput` | `InventoryCategory` | Facility configuration — required before any item can be created. |
| `inventory_list_items` | Inventory | `{ categoryId?, lowStockOnly? }` | `InventoryItem[]` | Shared by Pharmacy/Lab consultation views. |
| `inventory_create_item` | Inventory | `CreateInventoryItemInput` | `InventoryItem` | |
| `inventory_record_transaction` | Inventory | `CreateInventoryTransactionInput` | `InventoryTransaction` | |
| `inventory_schedule_maintenance` | Inventory | `CreateMaintenanceScheduleInput` | `MaintenanceSchedule` | |
| `notifications_list` | Notifications | `{ unreadOnly? }` | `Notification[]` | |
| `notifications_mark_read` | Notifications | `{ id }` | `Notification` | Optimistic-update exception, see [StateManagement](StateManagement.md) Section 5. |
| `audit_list` | Audit | `{ entityType?, userId?, from?, to?, limit, offset }` | `AuditLogEntry[]` | |
| `audit_verify_integrity` | Audit | `{}` | `{ isValid, brokenAtId? }` | See [Audit](Audit.md) Section 4. |
| `billing_generate_simulation` | Billing | `{ encounterId }` | `BillingSimulation` | Aggregates treatments/inventory/room/OR charges — simulation only, no real payment. |
| `billing_get_simulation` | Billing | `{ encounterId }` | `BillingSimulation` | |
| `billing_finalize_simulation` | Billing | `{ id }` | `BillingSimulation` | |

Every row above corresponds to exactly one Rust file location per [FolderStructure](FolderStructure.md) (`commands/<module>_commands.rs`) and one frontend hook per module `api/` folder.

## 2.1 Why Bootstrap and Facility-Configuration Commands Exist

[Rules](Rules.md) 17.1 forbids mock data, hardcoded entities, and seeded fake rows. That prohibition has a direct consequence that is easy to miss until implementation stalls: **every row in this system must be creatable through a real command, or it can never exist at all.**

Without the commands added above, the documented system is unreachable in two ways:

1. **No first user.** [Security](Security.md) Section 9 permits user creation only by an already-authenticated user, and there is no self-registration screen. With an empty `users` table and no seed data permitted, `auth_login` can never succeed for anybody, so the application can never be entered. Resolved by `auth_bootstrap_status` + `auth_bootstrap_admin` (see [Security](Security.md) Section 9.1 for the security analysis of these two unauthenticated commands).
2. **No hospital.** `floors`, `rooms`, `beds`, `operating_rooms`, and `inventory_categories` are structural reference data that every clinical workflow depends on, but nothing wrote them. `beds_assign` cannot assign a bed that no command can create; the Hospital Map would render an empty building forever. Resolved by the facility-configuration commands, which are owned by the module that owns the underlying tables per [Architecture](Architecture.md) Section 3 — **Beds owns `floors`/`rooms`/`beds`, so the write path is `beds_*`, not `hospital_map_*`.** Hospital Map remains strictly read-only and still has zero mutation commands, exactly as [Plan](Plan.md) Phase 6's automatic grep check asserts.

Facility-configuration commands are ordinary authenticated commands: same session check, same validation, same audit coverage. They are called out as a group only because they are the entry point that makes the rest of the system non-empty.

## 3. Events

Convention: `<module>:<entity>:<action>` (Rule 3.6). Emitted by the Service layer after a successful, committed mutation. Consumed centrally per [StateManagement](StateManagement.md) Section 4.

| Event | Emitted by | Frontend invalidates |
|---|---|---|
| `patients:record:created` / `:updated` | `patient_service.rs` | `['patients', 'list']`, `['patients', 'detail', id]` |
| `medical-history:encounter:created` / `:discharged` | `medical_history_service.rs` | `['medical-history', patientId]`, `['patients', 'detail', patientId]` |
| `medical-history:diagnosis:created` / `:treatment:created` / `:evolution:created` | `medical_history_service.rs` | `['medical-history', patientId]` |
| `beds:assignment:created` / `:released` | `bed_service.rs` | `['beds', 'list']`, `['hospital-map', 'layout']`, `['medical-history', patientId]` |
| `beds:facility:changed` | `bed_service.rs` (floor/room/bed created, bed status set) | `['beds', 'list']`, `['hospital-map', 'layout']` |
| `auth:user:created` / `:deactivated` | `auth_service.rs` | `['auth', 'users']` |
| `operating-rooms:reservation:created` / `:updated` / `:cancelled` | `operating_room_service.rs` | `['operating-rooms', 'list']`, `['hospital-map', 'layout']` |
| `operating-rooms:room:created` | `operating_room_service.rs` | `['operating-rooms', 'list']`, `['hospital-map', 'layout']` |
| `inventory:item:created` / `:transaction:created` / `:category:created` | `inventory_service.rs` | `['inventory', 'items']`, `['inventory', 'categories']`, may also trigger `['notifications', 'list']` invalidation if a low-stock notification was generated |
| `notifications:notification:created` | any service that generates a notification | `['notifications', 'list']` |
| `billing:simulation:created` / `:finalized` | `billing_service.rs` | `['billing', encounterId]` |

`['hospital-map', 'layout']` is invalidated by both Beds and Operating Rooms events because the Hospital Map visualizes both, despite owning neither (see [Architecture](Architecture.md) Section 3).

## 4. Adding a New Command or Event

1. Add a row to Section 2 or 3 of this document.
2. Add the corresponding row to [Validation](Validation.md) Section 3 if it takes user input.
3. Add the corresponding entry to `shared/lib/event-query-map.ts` if it emits an event ([StateManagement](StateManagement.md) Section 4).
4. Implement per [Tasks](Tasks.md)' atomic task template.
5. Add an integration test per [Testing](Testing.md).

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#ipcmd)
