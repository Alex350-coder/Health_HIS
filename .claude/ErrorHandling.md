# ErrorHandling.md

## Purpose

Defines the error taxonomy and the handling strategy for every category of error the system can encounter, end to end (Rust -> IPC -> React).

## Dependencies

[Rules](Rules.md) Section 14, [Architecture](Architecture.md) Sections 2.3–2.5.

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md), [Validation](Validation.md), [Security](Security.md).

## Referenced By

[Architecture](Architecture.md), [Audit](Audit.md), [CodingStandards](CodingStandards.md), [Database](Database.md), [FolderStructure](FolderStructure.md), [Glossary](Glossary.md), [IPC](IPC.md), [Plan](Plan.md), [README_Project](README_Project.md), [Rules](Rules.md), [Security](Security.md), [Tasks](Tasks.md), [Testing](Testing.md), [UI](UI.md), [Validation](Validation.md).

## Documents that must be updated if changes occur

[IPC](IPC.md) (error payload contract), [Audit](Audit.md) (failure events), [UI](UI.md) (error state rendering), [Testing](Testing.md).

## Priority

6.

---

## Contents

- [1. The Single `AppError` Type](#1-the-single-apperror-type)
- [2. Category-by-Category Strategy](#2-category-by-category-strategy)
- [3. Frontend Error Surfacing Rules](#3-frontend-error-surfacing-rules)
- [4. Empty Catch Blocks](#4-empty-catch-blocks)
- [5. Logging vs. User-Facing Messages](#5-logging-vs-user-facing-messages)
- [6. Testing Error Paths](#6-testing-error-paths)

---

## 1. The Single `AppError` Type

All fallible operations that cross the IPC boundary return `Result<T, AppError>`, where `AppError` (`src-tauri/src/errors/app_error.rs`) is a single `enum` implementing `serde::Serialize` so it round-trips to the frontend intact.

```rust
pub enum AppError {
    Validation { field: String, message: String },
    NotFound { entity: String, id: i64 },
    Conflict { message: String },           // e.g., bed already occupied
    Unauthorized,                            // no/invalid/expired session
    AccountLocked { retry_after_secs: i64 },
    Database { message: String, correlation_id: String },   // sanitized — never raw SQL/driver text
    Filesystem { message: String, correlation_id: String },
    Unexpected { message: String, correlation_id: String }, // last-resort catch-all
}
```

The three technical variants carry a `correlation_id` (an opaque, randomly generated short id, e.g. `a1b2c3`) so a user-reported error can be matched to the exact server-side log entry — see Section 5. The clinical/user-facing variants (`Validation`, `NotFound`, `Conflict`, `Unauthorized`, `AccountLocked`) do not: they are expected, actionable outcomes rather than faults to be diagnosed from logs, and the id would be noise in the UI.

The frontend has a mirrored TypeScript discriminated union in `shared/errors/app-error.ts`, generated/kept in sync manually per the Section 3 table in [Validation](Validation.md)'s pattern (documented, tracked duplication, Rule 17.4 exception).

## 2. Category-by-Category Strategy

| Category | Source | Handling |
|---|---|---|
| **Validation** | Rust `validation/*.rs` (authoritative) or Zod (client pre-check) | Rust: return `AppError::Validation{field, message}` before touching the DB. React: `react-hook-form` surfaces the message inline on the field; if a validation error arrives from the server despite client-side checks passing (schema drift bug), it is surfaced as a form-level banner, and this scenario itself is logged as a `warn` (indicates a Section 3 table entry has drifted). |
| **Database** | `repositories/*.rs` — `rusqlite::Error` / `sqlx::Error` | Caught at the repository boundary, mapped to `AppError::Database` with a generic, non-leaking message (no raw SQL, no column names in the user-facing text — Rule 14.1). The real error detail is logged server-side (`tracing::error!`) with full context for debugging, never sent to the frontend. |
| **Unexpected** | Any panic-adjacent condition, unforeseen `Err` variant | Rust: `unwrap()`/`expect()` are forbidden (Rule 8.3), so genuinely "unexpected" errors are those bubbling from a dependency; caught at the command boundary via a top-level `catch_unwind`-free `Result` chain and mapped to `AppError::Unexpected`. React: caught by the nearest `<ErrorBoundary>` (per module, see [CodingStandards](CodingStandards.md) Section 1), rendering a generic "Something went wrong" state per [UI](UI.md) Section "Error States," with a "reload module" action. |
| **Filesystem** | Backup/export features, migration file reads | Mapped to `AppError::Filesystem`. Clinical workflow commands do not touch the filesystem directly (only the DB), so this category is scoped to admin/maintenance features (e.g., future DB backup command). |
| **Authentication** | `security/session.rs`, `auth_service.rs` | Invalid credentials -> `AppError::Validation` (deliberately generic "invalid username or password," not "user not found" vs "wrong password," to avoid username enumeration). Locked account -> `AppError::AccountLocked`. |
| **Authorization** | `security/session.rs::require_session` | Missing/expired/invalid session token on any protected command -> `AppError::Unauthorized`. React: global Tauri event/response interceptor catches this centrally (one place, `shared/lib/api-client.ts`) and redirects to the login route, clearing the Zustand session slice — never handled ad hoc per component. |

## 3. Frontend Error Surfacing Rules

- Every `AppError` variant maps to exactly one presentation pattern in [UI](UI.md): inline field error (Validation), toast (Conflict, transient Database), full error state (Unexpected, Filesystem), redirect (Unauthorized), modal with countdown (AccountLocked).
- No raw error object, stack trace, or `AppError` debug representation is ever rendered directly to a user. Mapping from `AppError` to a human-readable message lives in one place: `shared/errors/error-messages.ts`.
- Toasts/banners never include PHI (Rule 12.1) — messages reference the action, not the clinical content (e.g., "Could not save diagnosis" not "Could not save diagnosis 'Type 2 Diabetes' for John Doe").

## 4. Empty Catch Blocks

Forbidden (Rule 14.2). Every `catch`/`.catch()`/`Result::Err(_)` arm either: (a) maps to a user-visible state, (b) logs at an appropriate level and rethrows/propagates, or (c) is a deliberate, commented no-op with a one-line justification (rare; requires code review sign-off).

## 5. Logging vs. User-Facing Messages

Server-side logs (`tracing` crate, JSON lines per Rule 12.2) capture full technical detail for every `AppError::Database`/`Filesystem`/`Unexpected` occurrence, tagged with the `correlation_id` carried on those variants (Section 1) — an opaque value with no PHI and no technical detail of its own. The frontend displays it in the `ErrorState`/toast ("Error reference: a1b2c3"), so a support session can correlate a user's report with the exact log entry without ever exposing the underlying error to the user.

## 6. Testing Error Paths

See [Testing](Testing.md) Section "Error Path Coverage" — every `AppError` variant has at least one integration test proving it is produced under the right condition and correctly serialized across the IPC boundary.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#errorhandlingmd)
