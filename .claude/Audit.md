# Audit.md

## Purpose

Defines exactly what is audited, how audit integrity (tamper-evidence) is guaranteed, and how the audit log is exposed for review.

## Dependencies

[Database](Database.md) Section 3.7 (`audit_log` schema), [Security](Security.md) Section 8 (audit integrity), [Rules](Rules.md) Section 12.3 (audit logging is distinct from application logging).

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md), [Database](Database.md), [Security](Security.md).

## Referenced By

[Architecture](Architecture.md), [Database](Database.md), [DefinitionOfDone](DefinitionOfDone.md), [ErrorHandling](ErrorHandling.md), [Glossary](Glossary.md), [IPC](IPC.md), [Plan](Plan.md), [README_Project](README_Project.md), [Rules](Rules.md), [Security](Security.md), [Tasks](Tasks.md).

## Documents that must be updated if changes occur

[Database](Database.md) (schema), [ErrorHandling](ErrorHandling.md) (audit-write failure handling), [UI](UI.md) (audit viewer), [Testing](Testing.md).

## Priority

6.

---

## Contents

- [1. Principle](#1-principle)
- [2. What Is Audited (exhaustive list)](#2-what-is-audited-exhaustive-list)
- [3. Audit Write Semantics](#3-audit-write-semantics)
- [4. Tamper Evidence — Hash Chain](#4-tamper-evidence--hash-chain)
- [5. Immutability Enforcement](#5-immutability-enforcement)
- [6. Audit Viewer (MVP Scope Note)](#6-audit-viewer-mvp-scope-note)
- [7. Retention](#7-retention)

---

## 1. Principle

Every critical action in the system is recorded with: **who** (`user_id`), **when** (`timestamp`), **where** (`entity_type` + `entity_id`), **what** (`action`, `before_state`/`after_state`), and **result** (`success`/`failure`). Schema: [Database](Database.md) Section 3.7.

## 2. What Is Audited (exhaustive list)

| Domain | Actions |
|---|---|
| Authentication | `auth.login` (success + failure + locked), `auth.logout`, `auth.session_expired` |
| User management | `user.create` (including the bootstrap admin, which is the audit chain's genesis entry — [Security](Security.md) Section 9.1), `user.deactivate`, `user.role_change` |
| Facility configuration | `floor.create`, `room.create`, `bed.create`, `operating_room.create`, `inventory_category.create` — structural changes to the hospital itself are low-frequency and high-impact, so they are audited like any other critical action |
| Patients | `patient.create`, `patient.update` |
| Medical History | `encounter.create`, `encounter.discharge`, `diagnosis.create`, `diagnosis.correct`, `treatment.create`, `treatment.correct`, `evolution.create` |
| Beds | `bed.assign`, `bed.release`, `bed.status_change` (e.g., to maintenance) |
| Operating Rooms | `or_reservation.create`, `or_reservation.update`, `or_reservation.cancel` |
| Inventory | `inventory_item.create`, `inventory_transaction.create` (covers both pharmacy and lab consumption), `maintenance_schedule.create`, `maintenance_schedule.complete` |
| Billing | `billing_simulation.create`, `billing_simulation.finalize` |
| Notifications | Not audited individually (system-generated, non-critical); only the triggering action (e.g., `inventory_transaction.create` causing a low-stock notification) is audited. |
| Hospital Map | Not audited — read-only module, no state change (Section 3, [Architecture](Architecture.md)). |

Every action above is written by the owning **Service** (never by a Repository or Command directly, Rule: single responsibility) via a shared `AuditService::record(...)` call, ensuring one code path writes audit rows (no duplicated logic, Rule 17.4).

## 3. Audit Write Semantics

- Audit writes happen **inside the same database transaction** as the action they describe (e.g., `bed_service::assign` writes to `bed_assignments`, `beds`, and `audit_log` in one transaction). If the transaction rolls back, no orphaned audit row is left claiming success for something that didn't happen.
- If the action itself fails validation before reaching the transaction (e.g., a `AppError::Validation`), a `result: 'failure'` audit row is still written (outside any transaction, since nothing to roll back) — failed attempts are audit-worthy events, especially for authentication (Section 2).
- `before_state`/`after_state` are JSON snapshots of the relevant row(s), captured by the service immediately before/after the mutation. For create actions, `before_state` is `NULL`; for the audit's own append-only design, no `delete` action exists in the clinical domain (Rule 9.5) except administrative ones (e.g., deactivating a user, which is a soft state change, not a deletion).

## 4. Tamper Evidence — Hash Chain

- Each `audit_log` row stores `prev_hash` (the `row_hash` of the immediately preceding row, or a fixed genesis constant for the first row) and `row_hash` = `SHA-256(prev_hash || timestamp || user_id || action || entity_type || entity_id || before_state || after_state || result)`.
- Computed in `audit_service.rs`, never client-side.
- A verification routine (`audit_service::verify_chain()`) recomputes the chain and flags any break — exposed via a `audit_verify_integrity` command, callable from the Audit module's viewer UI (see [UI](UI.md)), restricted to authenticated users (all users can view per the "same UI for all roles" project rule, but see Section 6 for an MVP-scoped exception).

## 5. Immutability Enforcement

Two independent layers (defense in depth, mirroring [Security](Security.md) Section 1):

1. **Database trigger** (`0002_audit.sql`, see [Database](Database.md) Section 6): `CREATE TRIGGER audit_log_no_update BEFORE UPDATE ON audit_log BEGIN SELECT RAISE(ABORT, 'audit_log is append-only'); END;` and an equivalent `BEFORE DELETE` trigger.
2. **Application layer**: `audit_repository.rs` exposes only an `insert` function — no `update`/`delete` function exists in that module at all (nothing to accidentally call).

## 6. Audit Viewer (MVP Scope Note)

Per the project's authentication model (every authenticated user sees the same application), the Audit module's viewer is visible to all authenticated users in this MVP, consistent with "Authentication exists mainly for audit, security, traceability, access protection" rather than role-gated UI. If a future requirement restricts audit-log visibility to specific roles, that is a documented change to this section and to [Security](Security.md) Section 7 (Least Privilege) — not an ad hoc frontend check.

## 7. Retention

No automatic purging. Medical/audit records are permanent per the project's "Permanent Medical History Preservation" workflow step and Rule 9.5. Storage growth is acceptable at the scale described in [Database](Database.md) Section 8.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#auditmd)
