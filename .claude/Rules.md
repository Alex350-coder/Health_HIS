# Rules.md — Highest Authority Document

## Purpose

Defines objective, measurable, non-negotiable rules governing every line of code, every file, and every decision in this repository. Never subjective recommendations.

## Dependencies

None. This is the root of the document hierarchy.

## Documents that must be read before using it

None. Read first, always.

## Referenced By

[Architecture](Architecture.md), [Audit](Audit.md), [CodingStandards](CodingStandards.md), [Database](Database.md), [DefinitionOfDone](DefinitionOfDone.md), [DevelopmentWorkflow](DevelopmentWorkflow.md), [ErrorHandling](ErrorHandling.md), [FolderStructure](FolderStructure.md), [IPC](IPC.md), [Plan](Plan.md), [README_Project](README_Project.md), [Security](Security.md), [StateManagement](StateManagement.md), [Tasks](Tasks.md), [Testing](Testing.md), [UI](UI.md), [Validation](Validation.md).

## Documents that must be updated if changes occur

Any document in the framework may need updating if a rule changes, since no document may contradict this one. Check [Architecture](Architecture.md), [CodingStandards](CodingStandards.md), [Security](Security.md), [Database](Database.md), [Testing](Testing.md), [UI](UI.md) first.

## Priority

1 (absolute — supersedes all other documents).

---

## Contents

- [0. Enforcement Statement](#0-enforcement-statement)
- [1. Architecture Rules](#1-architecture-rules)
- [2. Folder Organization Rules](#2-folder-organization-rules)
- [3. Naming Rules](#3-naming-rules)
- [4. Import Rules](#4-import-rules)
- [5. Dependency Rules](#5-dependency-rules)
- [6. React Rules](#6-react-rules)
- [7. TypeScript Rules](#7-typescript-rules)
- [8. Tauri Rules](#8-tauri-rules)
- [9. Database Rules](#9-database-rules)
- [10. Security Rules](#10-security-rules)
- [11. Validation Rules](#11-validation-rules)
- [12. Logging Rules](#12-logging-rules)
- [13. State Synchronization Rules](#13-state-synchronization-rules)
- [14. Error Handling Rules](#14-error-handling-rules)
- [15. Testing Rules](#15-testing-rules)
- [16. Documentation Rules](#16-documentation-rules)
- [17. Forbidden Practices](#17-forbidden-practices)
- [18. Performance Rules](#18-performance-rules)
- [19. Accessibility Rules](#19-accessibility-rules)
- [20. Code Complexity Rules](#20-code-complexity-rules)
- [21. Verification](#21-verification)

---

## 0. Enforcement Statement

> [!IMPORTANT]
> **This is the highest-authority document in the framework (Priority 1).** Read it in full before writing any code.

No document, plan, task, or piece of code may contradict this file. If a conflict is found between this file and any other document, this file wins and the other document must be corrected before work continues. Every rule below is written to be objectively verifiable (by a linter, a script, a test, or a binary yes/no code review check) — if a "rule" cannot be verified objectively, it does not belong in this file.

---

## 1. Architecture Rules

1.1. The system has exactly 5 layers (frontend: Presentation, State; backend: Command/IPC, Service, Repository) plus the Database. See [Architecture](Architecture.md) for full definitions.

1.2. A layer may only call the layer directly beneath it. No layer skipping (e.g., a React component must never issue raw SQL; a Repository must never contain business/validation rules).

1.3. The frontend (WebView/JS) NEVER accesses the database directly. All persistence happens through Tauri `invoke` commands. This is enforced by the fact that no SQLite driver/binding exists in `package.json` — its presence is a rule violation.

1.4. Every Tauri command is defined once, in `src-tauri/src/commands/`, and documented in [IPC](IPC.md) before implementation.

1.5. Cross-module communication happens only through: (a) the database as single source of truth, (b) Tauri events for real-time propagation (see [StateManagement](StateManagement.md)). Direct imports between two modules' component/hook trees (e.g., `modules/beds` importing a component from `modules/patients`) are forbidden — shared code must live in `src/shared/`.

## 2. Folder Organization Rules

2.1. The repository MUST match the tree defined in [FolderStructure](FolderStructure.md) exactly. Any new top-level folder requires a [FolderStructure](FolderStructure.md) update in the same commit.

2.2. Each frontend module under `src/modules/<module>/` MUST contain only: `components/`, `hooks/`, `api/`, `types/`. No other subfolder is permitted without a [FolderStructure](FolderStructure.md) amendment.

2.3. Each backend module under `src-tauri/src/` MUST separate `commands/`, `services/`, `repositories/`, `models/` per [FolderStructure](FolderStructure.md).

## 3. Naming Rules

3.1. TypeScript/React files: components use `PascalCase.tsx` (e.g., `PatientCard.tsx`); non-component files use `kebab-case.ts` (e.g., `use-patient-query.ts`, `patient-api.ts`).

3.2. React components, hooks, and types: components `PascalCase`; hooks `useCamelCase` (prefix `use`); types/interfaces `PascalCase`; constants `SCREAMING_SNAKE_CASE`.

3.3. Rust files and modules: `snake_case`. Structs/Enums: `PascalCase`. Functions/variables: `snake_case`. Constants: `SCREAMING_SNAKE_CASE`.

3.4. Database identifiers (tables, columns): `snake_case`, singular concept but plural table names (e.g., `patients`, `bed_assignments`). Primary key column is always `id`. Foreign keys are always `<referenced_table_singular>_id` (e.g., `patient_id`).

3.5. Tauri command names: `<module>_<action>` snake_case (e.g., `patients_create`, `beds_assign`). Defined and enumerated in [IPC](IPC.md).

3.6. Tauri event names: `<module>:<entity>:<action>` (e.g., `patients:record:created`). Defined and enumerated in [IPC](IPC.md).

## 4. Import Rules

4.1. No circular imports (enforced via ESLint `import/no-cycle` and `cargo` module graph review).

4.2. Import order (ESLint `import/order`, enforced): (1) external packages, (2) `src/shared/*`, (3) same-module relative imports, (4) types (type-only imports last, using `import type`).

4.3. No deep relative imports beyond one directory up (`../../../x`). Use path aliases (`@shared/*`, `@modules/*`) configured in `tsconfig.json` and `vite.config.ts`.

## 5. Dependency Rules

5.1. No new dependency (npm or cargo) may be added without being recorded, with justification, in [Architecture](Architecture.md) Section "Tech Stack". Undocumented dependencies fail code review.

5.2. No dependency with a known unpatched critical/high CVE (checked via `npm audit` / `cargo audit` in CI) may be introduced or remain.

5.3. Direct dependencies only where possible; avoid adding a package to solve a problem coverable in under 30 lines of first-party code — see [CodingStandards](CodingStandards.md) "No unnecessary abstraction."

## 6. React Rules

6.1. Function components only. No class components.

6.2. All data mutation goes through TanStack Query mutations that invalidate/refetch via the query keys defined in [StateManagement](StateManagement.md) — never local component state acting as a cache of server data.

6.3. No `useEffect` for data fetching. Data fetching is done exclusively through TanStack Query hooks defined in each module's `api/` folder.

6.4. Props for any component exceeding 3 primitive props MUST be passed as a single typed object.

6.5. No inline function definitions passed as props to list-rendered children (must be memoized with `useCallback` or hoisted) when the list can exceed 20 items.

## 7. TypeScript Rules

7.1. `strict: true` in `tsconfig.json`, non-negotiable. `noImplicitAny`, `strictNullChecks`, `noUncheckedIndexedAccess` all enabled.

7.2. The `any` type is forbidden. `unknown` + narrowing is the only escape hatch, and only at true external boundaries (e.g., parsing a Tauri event payload before Zod validation).

7.3. Every exported function has an explicit return type.

7.4. No non-null assertion operator (`!`) except in test files.

## 8. Tauri Rules

8.1. Tauri capabilities/permissions (`src-tauri/capabilities/*.json`) follow least privilege: only the specific commands and APIs each window actually needs are allowlisted. Wildcard permissions are forbidden.

8.2. All Tauri commands validate their input with a `validator`-derived or manual check in Rust BEFORE touching the database, independent of any client-side (Zod) validation already performed. See [Validation](Validation.md).

8.3. Every Tauri command returns `Result<T, AppError>` where `AppError` is the single serializable error enum defined in [ErrorHandling](ErrorHandling.md). No command may panic; `unwrap()`/`expect()` are forbidden outside of `#[test]` code and one-time startup code proven infallible (e.g., static regex compilation checked by a startup test).

## 9. Database Rules

9.1. All queries are parameterized (`sqlx` compile-time-checked queries or `rusqlite` bound parameters). String concatenation to build SQL is forbidden — zero exceptions.

9.2. Every schema change is a new, timestamped, forward-only migration file. Migrations are never edited after being merged. See [Database](Database.md) Section "Migration Strategy".

9.3. Every table has a primary key. Every foreign key has a corresponding index. Every table involved in workflow-critical lookups has the indexes specified in [Database](Database.md).

9.4. No table stores duplicated/derivable data (normalization to 3NF) except explicitly justified denormalization documented inline in the migration file's comment and cross-referenced in [Database](Database.md).

9.5. Medical history records (diagnoses, treatments, evolutions, encounters) are never physically deleted. Corrections are new rows referencing the corrected row (append-only). See [Database](Database.md) Section "Permanent Medical History".

9.6. No migration may declare a foreign key referencing a table created in a *later* migration. SQLite accepts such a declaration at `CREATE TABLE` time but fails at first `INSERT` with `foreign_keys = ON`. Verified by the migration test ([Testing](Testing.md) Section 1 "Database"), which applies all migrations to an empty file and then inserts one row into every table in migration order.

## 10. Security Rules

10.1. Passwords are hashed with Argon2id only, using the parameters specified in [Security](Security.md) Section "Password Hashing." Plaintext passwords never touch the database or logs.

10.2. Every authentication attempt (success or failure) is rate-limited per account per [Security](Security.md) Section "Rate Limiting."

10.3. Session tokens are opaque, cryptographically random (≥256 bits), stored server-side (in the DB) as a hash, never as plaintext.

10.4. All PHI (Protected Health Information) at rest is encrypted via SQLCipher (see [Database](Database.md)). The encryption key is never hardcoded and never committed — see [Security](Security.md) Section "Secrets Management."

10.5. No secret, key, token, or credential appears in source code, git history, or logs. CI includes a secret-scanning step.

## 11. Validation Rules

11.1. Every user input crossing the IPC boundary is validated twice: once client-side (Zod schema, for UX) and once server-side (Rust, for security — the authoritative check). See [Validation](Validation.md).

11.2. Validation schemas are the single source of truth for a given entity's shape; the schema is defined once and referenced/mirrored, not redefined ad hoc per form.

## 12. Logging Rules

12.1. No PHI (patient name, diagnosis, treatment detail, etc.) is ever written to application logs. Logs reference entity IDs, not clinical content.

12.2. Every log entry has a severity level (`error`, `warn`, `info`, `debug`) and a structured, machine-parsable format (JSON lines).

12.3. Audit logging (who/what/when/where/result) is distinct from application/debug logging and follows [Audit](Audit.md), not this section.

## 13. State Synchronization Rules

13.1. There is exactly one writable source of truth: the SQLite database. All frontend state derived from it is a cache (TanStack Query) that is invalidated, not manually patched, after every mutation. See [StateManagement](StateManagement.md).

13.2. Any mutation that affects data visible in more than one open window/view MUST emit a Tauri event so all subscribed views invalidate their cache. See [IPC](IPC.md) Section "Events."

## 14. Error Handling Rules

14.1. Every error surfaced to a clinical user has a human-readable message. Raw stack traces/DB errors never reach the UI. See [ErrorHandling](ErrorHandling.md).

14.2. Every `catch`/`Result::Err` branch either handles the error meaningfully or explicitly re-throws/propagates it — silent empty catch blocks are forbidden.

## 15. Testing Rules

15.1. Minimum coverage thresholds (enforced in CI, build fails below): 80% line coverage for Rust services/repositories, 80% for TypeScript business logic (hooks, utils), 70% for React components. See [Testing](Testing.md).

15.2. Every Tauri command has at least one integration test that exercises it against a real (temporary, file-based) SQLite database — never a mock DB.

15.3. Every bug fix ships with a regression test reproducing the bug before the fix.

## 16. Documentation Rules

16.1. Every new file created is logged in [New_files](New_files.md) with its purpose, in the same commit that creates it.

16.2. Every phase completion updates [Progress](Progress.md) in the same commit/PR that completes it.

16.3. A PR that changes architecture, schema, security behavior, or module boundaries MUST update the corresponding document ([Architecture](Architecture.md), [Database](Database.md), [Security](Security.md), etc.) in the same PR. Code and docs are never allowed to diverge.

## 17. Forbidden Practices

17.1. **No mock data.** No hardcoded arrays of fake entities, no `faker`-generated placeholder data shipped in application code. Seed data for local development, if any, lives in a clearly named `dev-seed` migration/script, is never imported by production code paths, and is documented in [Database](Database.md).

17.2. **No commented-out code.** Delete it; git history preserves it.

17.3. **No dead code.** No unused exports, unused files, unreachable branches. CI runs `ts-prune`/`knip` (TS) and `cargo clippy` (`dead_code` lint denied) to catch this.

17.4. **No duplicated logic.** The same validation rule, formatting function, or business calculation must exist in exactly one place. If a rule must exist in two languages (TS + Rust) because of the client/server boundary (see Rule 11.1), it is documented as an intentional, tracked duplication in [Validation](Validation.md) — not silent drift.

17.5. **No unused files.** Any file not imported/referenced by the build graph or explicitly listed as a documentation/config file is deleted.

17.6. No direct DOM manipulation in React (no `document.querySelector` etc.) outside of a small, documented set of accessibility-focused exceptions in [UI](UI.md).

17.7. No global mutable singletons in Rust outside of the managed Tauri `State<T>` mechanism.

## 18. Performance Rules

18.1. Any list rendering more than 100 rows (e.g., patient lists, inventory) MUST use virtualization (see [UI](UI.md)).

18.2. Any Tauri command expected to run over 100ms (bulk queries, report generation) MUST run asynchronously and not block the UI thread; the frontend shows a loading state per [UI](UI.md).

18.3. Database queries on tables expected to exceed 10,000 rows must use the indexes defined in [Database](Database.md) — verified via `EXPLAIN QUERY PLAN` in the relevant integration test.

## 19. Accessibility Rules

19.1. WCAG 2.1 AA is the minimum bar. All interactive elements are keyboard-operable and have visible focus states. See [UI](UI.md).

19.2. All color-based status indicators (e.g., bed availability, expiration alerts) also carry a non-color indicator (icon or text label).

## 20. Code Complexity Rules

20.1. **Maximum function length:** 40 executable lines (TS and Rust), excluding blank lines, comments, and closing braces. Enforced via ESLint `max-lines-per-function` and a Rust `clippy` custom check / code review.

20.2. **Maximum file size:** 300 lines for React components, 400 lines for Rust service/repository files, 200 lines for hooks/utility files. Files exceeding this must be split by responsibility.

20.3. **Maximum cyclomatic complexity:** 10 per function (ESLint `complexity` rule; `clippy::cognitive_complexity` for Rust).

20.4. **Maximum function parameters:** 4 positional parameters. Beyond that, use a single object/struct parameter.

20.5. **Maximum nesting depth:** 3 levels of block nesting per function.

---

## 21. Verification

Every rule above is checked by one of: ESLint, `tsc --noEmit`, `clippy -D warnings`, `cargo audit`, `npm audit`, a Vitest/cargo test, or a documented manual code-review checklist item in [DefinitionOfDone](DefinitionOfDone.md). A rule that cannot be mapped to one of these verification mechanisms must be rewritten or removed from this document.

## 22. Git Commits

- Every commit must use the local Git identity configured for the repository.
- Do not include AI attribution.
- Do not include Co-authored-by trailers.
- Do not include Generated by messages.
- Use conventional commit messages.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#rulesmd--highest-authority-document)
