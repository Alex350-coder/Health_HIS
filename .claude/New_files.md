# New_files.md — New File Log

## Purpose

Chronological log of every new file created in the repository and its purpose. Written to in the same commit that creates the file (Rule 16.1).

## Dependencies

[FolderStructure](FolderStructure.md) (every new file must fit the declared tree, or amend it first).

## Documents that must be read before using it

[FolderStructure](FolderStructure.md) before adding a file outside the existing tree.

## Referenced By

[Database](Database.md), [DefinitionOfDone](DefinitionOfDone.md), [FolderStructure](FolderStructure.md), [Plan](Plan.md), [README_Project](README_Project.md), [Rules](Rules.md), [Tasks](Tasks.md).

## Documents that must be updated if changes occur

None (terminal log).

## Priority

11.

---

## Contents

- [How To Update This Document](#how-to-update-this-document)
- [Phase 0 — Project Bootstrap & Tooling](#phase-0--project-bootstrap--tooling)
- [Phase 1 — Database & Migration Engine Foundation](#phase-1--database--migration-engine-foundation)
- [Phase 2 — Security Foundation, Authentication & Audit](#phase-2--security-foundation-authentication--audit)
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

## How To Update This Document

Add one row per new file, in the table for the phase during which it was created. Do not remove rows when files are later deleted — instead add a `Removed` note in the Notes column referencing why (keeps history auditable, mirrors the append-only philosophy applied elsewhere in this project, see [Database](Database.md) Section 7).

## Phase 0 — Project Bootstrap & Tooling

| File | Purpose | Notes |
|---|---|---|
| `.gitignore` | Excludes `node_modules/`, `dist/`, `coverage/` and editor/OS noise. | Task 0.1. |
| `README.md` | Public-facing readme: prerequisites, commands, quality gates. | Task 0.1. |
| `index.html` | Vite HTML entry point; loads `src/main.tsx` and nothing else. | Task 0.1. |
| `package.json` | Frontend manifest and script surface. | Task 0.1. Contains **no** SQLite driver — the mechanical proof of Rule 1.3. |
| `package-lock.json` | Locked dependency graph. | Task 0.1. Committed so CI can use `npm ci`. |
| `tsconfig.json` | TypeScript config: `strict` plus `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noUnusedLocals`/`Parameters`; `@app`/`@modules`/`@shared` aliases. | Task 0.1, Rules 7.1 / 4.3. |
| `vite.config.ts` | Vite build/dev config; port 1420 `strictPort` for Tauri; ignores `src-tauri/` in the watcher. | Task 0.1. |
| `src/vite-env.d.ts` | Ambient Vite client types. | Task 0.1. |
| `src/main.tsx` | React bootstrap. Throws a descriptive error if `#root` is absent rather than using a non-null assertion. | Task 0.1, Rule 7.4. |
| `src/app/App.tsx` | Placeholder root component; replaced by the real shell in Phase 3. | Task 0.1. |
| `src/app/App.test.tsx` | Smoke test proving the Vitest + RTL + jsdom pipeline works end to end. | Task 0.4. |
| `eslint.config.js` | ESLint 9 flat config implementing CodingStandards Section 1 and the Rules 4/7/20 limits. | Task 0.2. Replaces the `.eslintrc.cjs` named in FolderStructure. |
| `.prettierrc` | Formatter settings fixed by CodingStandards Section 1. | Task 0.2. |
| `.prettierignore` | Keeps generated output out of formatting. | Task 0.2. |
| `.lintstagedrc.json` | Runs Prettier then ESLint on staged TS/TSX. | Task 0.2. |
| `.husky/pre-commit` | Runs `lint-staged`; additionally runs `cargo fmt --check` and `cargo clippy` when Rust files are staged **and** cargo is on PATH, otherwise warns and defers to CI. | Task 0.2, DevelopmentWorkflow Section 5. |
| `vitest.config.ts` | Extends `vite.config.ts`; jsdom environment; per-glob coverage thresholds (80% logic / 70% components). | Task 0.4, Rules 15.1. |
| `vitest.setup.ts` | Registers `@testing-library/jest-dom` matchers. | Task 0.4. |
| `src-tauri/Cargo.toml` | Rust manifest. Denies `clippy::unwrap_used` / `expect_used` crate-wide; release profile strips symbols. | Task 0.3, Rule 8.3. |
| `src-tauri/build.rs` | Invokes `tauri-build` codegen. | Task 0.1. |
| `src-tauri/rustfmt.toml` | Rust formatting config. | Task 0.3, CodingStandards Section 2. |
| `src-tauri/.gitignore` | Ignores `/target` and `/gen`. | Task 0.1. |
| `src-tauri/src/main.rs` | Tauri bootstrap. Uses `unwrap_or_else` + explicit exit instead of `.unwrap()`. | Task 0.1, Rule 8.3. |
| `src-tauri/tauri.conf.json` | Window/bundle config and the explicit CSP (no `unsafe-inline`/`unsafe-eval` for scripts); `withGlobalTauri: false`; `freezePrototype: true`. | Task 0.1, Security Section 9. |
| `src-tauri/capabilities/default.json` | Least-privilege capability manifest — `core:default` only, no wildcards. | Task 0.1, Rule 8.1. |
| `src-tauri/icons/*` | Application icon set referenced by the bundle config. | Task 0.1. |
| `.github/workflows/ci.yml` | Quality gate implementing the Testing Section 7 order exactly, plus a `main`/`release` E2E job. | Task 0.5, Rule 21. |

## Phase 1 — Database & Migration Engine Foundation

| File | Purpose | Notes |
|---|---|---|

## Phase 2 — Security Foundation, Authentication & Audit

| File | Purpose | Notes |
|---|---|---|

## Phase 3 — Frontend Application Shell & Design System

| File | Purpose | Notes |
|---|---|---|

## Phase 4 — Patients Module

| File | Purpose | Notes |
|---|---|---|

## Phase 5 — Medical History Module

| File | Purpose | Notes |
|---|---|---|

## Phase 6 — Hospital Map Module

| File | Purpose | Notes |
|---|---|---|

## Phase 7 — Beds Module

| File | Purpose | Notes |
|---|---|---|

## Phase 8 — Cross-Module Integration Checkpoint

| File | Purpose | Notes |
|---|---|---|

## Phase 9 — Operating Rooms Module

| File | Purpose | Notes |
|---|---|---|

## Phase 10 — Inventory Module

| File | Purpose | Notes |
|---|---|---|

## Phase 11 — Notifications Module

| File | Purpose | Notes |
|---|---|---|

## Phase 12 — Billing Module (Simulation)

| File | Purpose | Notes |
|---|---|---|

## Phase 13 — Patient Discharge & Permanent History Preservation

| File | Purpose | Notes |
|---|---|---|

## Phase 14 — Hardening, Full Regression & Release

| File | Purpose | Notes |
|---|---|---|

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#new_filesmd--new-file-log)

# Phase 0
- .gitignore
- package.json
- tsconfig.json
- vite.config.ts
- vitest.config.ts
- vitest.setup.ts
- .eslintrc.cjs
- .prettierrc
- .prettierignore
- .lintstagedrc.json
- .husky/pre-commit
- index.html
- src/vite-env.d.ts
- src/app/App.tsx
- src/app/App.test.tsx
- src/main.tsx
- src-tauri/Cargo.toml
- src-tauri/build.rs
- src-tauri/src/main.rs
- src-tauri/rustfmt.toml
- src-tauri/tauri.conf.json
- src-tauri/capabilities/default.json
- src-tauri/.gitignore
- .github/workflows/ci.yml
- README.md
- eslint.config.js
