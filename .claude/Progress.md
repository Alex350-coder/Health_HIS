# Progress.md — Live Status Tracker

## Purpose

Single live record of what phase/task status the project is actually in. Updated as part of Definition of Done for every task and phase — not optional.

## Dependencies

[Plan](Plan.md), [Tasks](Tasks.md), [DefinitionOfDone](DefinitionOfDone.md).

## Documents that must be read before using it

[Plan](Plan.md) (to know what "done" means for the phase being updated).

## Referenced By

[DefinitionOfDone](DefinitionOfDone.md), [Plan](Plan.md), [README_Project](README_Project.md), [Rules](Rules.md), [Tasks](Tasks.md).

## Documents that must be updated if changes occur

None (this document is the terminal node — nothing depends on it being read before use, except as a status source for [Plan](Plan.md)/[Tasks](Tasks.md) continuation).

## Priority

11 (tracking layer — always reflects, never dictates).

---

## Contents

- [How To Update This Document](#how-to-update-this-document)
- [Phase Status](#phase-status)
- [Task-Level Detail](#task-level-detail)
- [Open Blockers](#open-blockers)
- [Deviations From Plan](#deviations-from-plan)

---

## How To Update This Document

- Status values: `Not Started`, `In Progress`, `Blocked`, `Done`.
- Update the relevant row the moment a task's status changes — not in a batch at the end of a session.
- A phase's status is `Done` only when every task in it is `Done` AND [DefinitionOfDone](DefinitionOfDone.md) Section 2 has been checked off.
- If `Blocked`, add a one-line reason and link the blocking task/phase.

## Phase Status

| # | Phase | Status | Notes |
|---|---|---|---|
| 0 | Project Bootstrap & Tooling | Blocked | All five tasks implemented; every runnable gate is green. Blocked on the two items in Open Blockers below. |
| 1 | Database & Migration Engine Foundation | Not Started | |
| 2 | Security Foundation, Authentication & Audit | Not Started | |
| 3 | Frontend Application Shell & Design System | Not Started | |
| 4 | Patients Module | Not Started | |
| 5 | Medical History Module | Not Started | |
| 6 | Hospital Map Module | Not Started | |
| 7 | Beds Module | Not Started | |
| 8 | Cross-Module Integration Checkpoint | Not Started | |
| 9 | Operating Rooms Module | Not Started | |
| 10 | Inventory Module | Not Started | |
| 11 | Notifications Module | Not Started | |
| 12 | Billing Module (Simulation) | Not Started | |
| 13 | Patient Discharge & Permanent History Preservation | Not Started | |
| 14 | Hardening, Full Regression & Release | Not Started | |

## Task-Level Detail

Populate one subsection per phase as work begins, listing each task from [Tasks](Tasks.md) with its status. Example shape (replicate per phase when that phase starts):

### Phase 0 — Project Bootstrap & Tooling

| Task | Status | Notes |
|---|---|---|
| 0.1 Initialize Tauri + React + TS project | Partially Done | Scaffold reshaped to FolderStructure; `tsc --noEmit` and `vite build` pass. The `npm run tauri dev` window check needs the Rust toolchain (blocker 1). |
| 0.2 Configure ESLint/Prettier/lint-staged/husky | Done | `npm run lint` and `npm run format:check` clean. Pre-commit hook verified to reject a deliberately malformed file (6 errors incl. `no-explicit-any`); probe then removed. |
| 0.3 Configure Rust toolchain | Blocked | `rustfmt.toml`, clippy deny attributes and the Cargo lint tables are written, but `cargo fmt --check` / `cargo clippy` cannot be executed (blocker 1). |
| 0.4 Set up Vitest and cargo test scaffolding | Partially Done | `npm run test:coverage` passes (1/1); per-glob thresholds proven to fail correctly under a probe. `cargo test` not runnable (blocker 1). |
| 0.5 Stand up CI pipeline | Done (unverified) | Workflow implements the Testing Section 7 gate order exactly. Not yet observed running — requires a push to a remote. |

*(Add the equivalent table for each phase as it becomes active. Do not pre-populate future phases' task tables — add them when that phase's work actually starts, to keep this document reflecting reality rather than aspiration.)*

## Open Blockers

### Blocker 1 — No Rust toolchain on the development machine (Phase 0)

`rustc` and `cargo` are not installed (no `~/.cargo`, no `~/.rustup`). Every Rust source and
config file for Tasks 0.1/0.3/0.4 is written, but nothing Rust-side can be **verified**:

- `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo build` — not run.
- The Task 0.1 acceptance check "`npm run tauri dev` opens an empty window" — not run.
- `Cargo.lock` therefore does not exist yet and is not committed.

Installing `rustup` is a machine-global change, so it needs the user's go-ahead.
**Until this clears, Phase 0 cannot satisfy [DefinitionOfDone](DefinitionOfDone.md) Section 1**
("build succeeds", "`cargo test` green"). CI gates 1–3 will exercise these on the first push
regardless, since the GitHub runner installs its own toolchain.

### Blocker 2 — Unfixable high-severity advisory in the ESLint dependency tree (Phase 0)

CI Gate 7 (`npm audit --audit-level=high`) reports 7 high findings, all one root cause:
GHSA-mh99-v99m-4gvg, a DoS in `brace-expansion`, reached transitively through `minimatch@3`.

This has **no upstream fix today**, and it is not a case of an un-run `npm audit fix`:

| Fact | Consequence |
|---|---|
| Only `brace-expansion` ≥ 5.0.8 is patched (verified against every published major). | 1.x / 2.x / 3.x / 4.x are all vulnerable. |
| Patched versions export a namespace object (`{ expand }`); versions ≤ 2.x export a bare callable. | An `overrides` pin to 5.x resolves, but crashes ESLint with `TypeError: expand is not a function`. Verified, then reverted. |
| Only `minimatch` ≥ 10.0.3 consumes a patched `brace-expansion`, and its CommonJS build is not callable. | Overriding `minimatch` breaks `eslint-plugin-jsx-a11y`, which does `interopRequireDefault(require('minimatch'))` and calls the result. Verified. |
| `eslint-plugin-jsx-a11y@6.10.2` (latest) declares `minimatch: ^3.1.2` and caps its peer range at `eslint ^9`. | The `eslint@10` upgrade npm suggests is a peer-dependency conflict, and dropping to a clean tree means dropping the plugin. |

This surfaces a genuine conflict between two binding documents:

- **Rule 5.2** — "No dependency with a known unpatched critical/high CVE … may be introduced or remain." Absolute, no exception clause.
- **[CodingStandards](CodingStandards.md) Section 1** — mandates `eslint-plugin-jsx-a11y`, which is the sole remaining blocker and the mechanism enforcing Rules 19 (accessibility).

[Tasks](Tasks.md) Task 0.5 anticipates "document any accepted exception explicitly, do not
silently ignore", but Rules outranks Tasks, so an exception cannot be self-granted. The gate in
`ci.yml` has deliberately **not** been weakened. Resolving this needs a user decision between:

1. Grant an explicit, time-boxed exception for this dev-only advisory (nothing here ships — the production bundle is React, React DOM and `@tauri-apps/api`), scoping the audit gate to production dependencies and keeping a report-only full audit; amend Rule 5.2 with the exception mechanism.
2. Drop `eslint-plugin-jsx-a11y` and amend CodingStandards Section 1, accepting that Rules 19 loses its automated enforcement.
3. Leave CI red until `eslint-plugin-jsx-a11y` releases a version on `minimatch` ≥ 10.

Option 1 is the recommendation: the advisory is a ReDoS reachable only through glob patterns
that this repository itself authors, at lint time, on a developer machine or CI runner.

## Deviations From Plan

### Phase 0

| Deviation | Reason | Document amended |
|---|---|---|
| `eslint.config.js` (flat config) instead of the `.eslintrc.cjs` named in Tasks 0.2 and FolderStructure Section 1. | ESLint 9 uses flat config; `.eslintrc.cjs` is legacy and would need `ESLINT_USE_FLAT_CONFIG=false`. | [FolderStructure](FolderStructure.md) Section 1. |
| Rust formatting/lint moved out of `lint-staged` into `.husky/pre-commit`, guarded by `command -v cargo`. | In `lint-staged` it hard-fails on any machine without a Rust toolchain (see blocker 1). [DevelopmentWorkflow](DevelopmentWorkflow.md) Section 5 explicitly permits "CI-only enforcement, since Rust formatting hooks are less standard". The Rust gates remain unconditional in CI. | None required. |
| Root tree gained `index.html`, `vitest.config.ts`, `vitest.setup.ts`, `.prettierignore`, `.lintstagedrc.json`, `.gitignore`, `.husky/`, `package-lock.json`; `src/` gained `vite-env.d.ts`; `src-tauri/` gained `build.rs`, `rustfmt.toml`, `.gitignore`, `icons/`. | Mandatory entry points and tool configuration implied by Tasks 0.1–0.5 but absent from the original tree. | [FolderStructure](FolderStructure.md) Sections 1–3, [New_files](New_files.md). |
| Toolchain dependency justifications recorded in a new Architecture Section 6.1 rather than the main table. | Keeps the architectural decision record readable by separating shipped choices from dev-only tooling; satisfies Rule 5.1. | [Architecture](Architecture.md) Section 6.1. |
| `typescript` pinned to `^5.9`, not the latest 7.x. | `typescript-eslint@8` declares peer `typescript >=4.8.4 <6.1.0`. | [Architecture](Architecture.md) Section 6.1. |
| `src/main.tsx` excluded from coverage measurement. | It is a bootstrap entry point, not a React component; Rules 15.1's 70% component gate does not describe it. All other `src/**/*.tsx` remain gated. | None required. |

Any further deviation from [Plan](Plan.md) or [Tasks](Tasks.md) discovered during implementation must be recorded here with a reference to which document was amended as a result (per Rule 16.3 — code and docs never diverge silently).

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#progressmd--live-status-tracker)
