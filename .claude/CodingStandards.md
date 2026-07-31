# CodingStandards.md

## Purpose

Concrete, language-specific coding conventions that implement the naming/complexity rules declared in [Rules](Rules.md).

## Dependencies

[Rules](Rules.md) (Sections 3, 6, 7, 8, 20).

## Documents that must be read before using it

[Rules](Rules.md), [FolderStructure](FolderStructure.md).

## Referenced By

[Database](Database.md), [DefinitionOfDone](DefinitionOfDone.md), [ErrorHandling](ErrorHandling.md), [Plan](Plan.md), [README_Project](README_Project.md), [Routes](Routes.md), [Rules](Rules.md), [Tasks](Tasks.md).

## Documents that must be updated if changes occur

[Rules](Rules.md) if a convention changes at the rule level; [Architecture](Architecture.md) if a convention implies a structural change.

## Priority

3.

---

## Contents

- [1. TypeScript / React](#1-typescript--react)
- [2. Rust](#2-rust)
- [3. Naming Quick Reference](#3-naming-quick-reference)
- [4. No Unnecessary Abstraction](#4-no-unnecessary-abstraction)
- [5. Comments](#5-comments)

---

## 1. TypeScript / React

- Formatter: Prettier (`printWidth: 100`, `singleQuote: true`, `semi: true`, `trailingComma: 'all'`). Enforced via `lint-staged` pre-commit hook.
- Linter: ESLint with `@typescript-eslint/recommended-type-checked`, `eslint-plugin-react-hooks`, `eslint-plugin-jsx-a11y`, `import/order`, `import/no-cycle`. CI runs `eslint --max-warnings=0`.
- Components: one component per file, file name matches component name (`PatientCard.tsx` exports `PatientCard`).
- Hooks: one primary hook per file (`use-patient-query.ts` exports `usePatientQuery`). Co-located with the module that owns the data.
- Types: domain types live in `modules/<module>/types/`; a type used by more than one module moves to `shared/`.
- No default exports except for route-level page components required by the router convention (see [Routes](Routes.md)). Everything else uses named exports (improves refactor-safety and dead-code detection).
- Error boundaries: every route-level page is wrapped by a module-level `<ErrorBoundary>` (see [ErrorHandling](ErrorHandling.md)).
- Forms: `react-hook-form` + `@hookform/resolvers/zod`, schema imported from the module's `types/` (Zod schema colocated with the TS type it derives, using `z.infer`).
- Styling: Tailwind CSS utility classes; component variants via `class-variance-authority` (`cva`). No inline `style={{}}` except for computed values impossible to express as classes (e.g., dynamic map coordinates in [UI](UI.md)'s Hospital Map).

## 2. Rust

- Formatter: `rustfmt` (default settings). CI runs `cargo fmt --check`.
- Linter: `clippy` with `-D warnings`, plus `-D clippy::unwrap_used -D clippy::expect_used` outside `#[cfg(test)]`.
- Every public function in `services/` and `repositories/` has a doc comment (`///`) stating purpose, not restating the signature.
- `Result<T, AppError>` is the return type of every fallible function that crosses a module boundary. Internal, module-private helpers may use `Option`/`Result` with local error types only if never exposed.
- Repositories return domain models (`models/`), never raw `sqlx::Row`/`rusqlite::Row` — mapping happens at the repository boundary, in one place.
- Services never construct SQL. Services call repository methods only.
- Commands (`commands/*.rs`) are thin: parse/validate input, call one service method, map the result/error, return. No business logic in a command function.

## 3. Naming Quick Reference

| Element | Convention | Example |
|---|---|---|
| React component file | PascalCase.tsx | `BedAssignmentDialog.tsx` |
| React hook file | kebab-case.ts, `use-` prefix | `use-bed-assignment.ts` |
| TS type/interface | PascalCase | `BedAssignment` |
| TS constant | SCREAMING_SNAKE_CASE | `MAX_BED_CAPACITY` |
| Rust file/module | snake_case | `bed_repository.rs` |
| Rust struct/enum | PascalCase | `BedAssignment` |
| Rust function/var | snake_case | `assign_bed_to_patient` |
| DB table | snake_case, plural | `bed_assignments` |
| DB column (PK) | `id` | `id` |
| DB column (FK) | `<table_singular>_id` | `patient_id` |
| Tauri command | `<module>_<action>` | `beds_assign` |
| Tauri event | `<module>:<entity>:<action>` | `beds:assignment:created` |

## 4. No Unnecessary Abstraction

Per [Rules](Rules.md) 5.3 and the project's general principle: do not introduce a generic/abstract factory, a plugin system, or a config-driven engine for something that is currently used in one place. Three near-identical lines across three files is acceptable; a shared helper is only extracted once a third real (not hypothetical) use appears.

## 5. Comments

Default to no comments. A comment is only justified when it explains a non-obvious WHY (a hidden constraint, a workaround, a regulatory reason). Never restate WHAT the code does. Rust `///` doc comments on public service/repository functions are the one mandated exception (Section 2), because they form the API contract surface, not implementation narration.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#codingstandardsmd)
