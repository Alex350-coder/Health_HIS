# DevelopmentWorkflow.md

## Purpose

Defines how work actually gets done day to day: branching, commits, PRs, local dev commands, and the release process.

## Dependencies

[Rules](Rules.md), [DefinitionOfDone](DefinitionOfDone.md), [Testing](Testing.md).

## Documents that must be read before using it

[Rules](Rules.md), [DefinitionOfDone](DefinitionOfDone.md).

## Referenced By

[README_Project](README_Project.md), [Tasks](Tasks.md).

## Documents that must be updated if changes occur

None typically; this document is largely self-contained tooling/process guidance.

## Priority

8.

---

## Contents

- [1. Branching Strategy](#1-branching-strategy)
- [2. Commit Conventions](#2-commit-conventions)
- [3. Pull Request Checklist](#3-pull-request-checklist)
- [4. Local Development Commands](#4-local-development-commands)
- [5. Pre-Commit Hooks](#5-pre-commit-hooks)
- [6. Release / Build Process](#6-release--build-process)
- [7. Environment Setup](#7-environment-setup)

---

## 1. Branching Strategy

- `main` — always green (CI passing), always deployable/buildable.
- `phase/<n>-<short-name>` — one branch per [Plan](Plan.md) phase (e.g., `phase/4-patients-module`).
- `task/<phase>-<task-id>-<short-name>` branches off the phase branch for individual [Tasks](Tasks.md) items when parallelizing within a phase; merged back into the phase branch via PR.
- The phase branch merges into `main` only when the full Phase-Level Definition of Done ([DefinitionOfDone](DefinitionOfDone.md) Section 2) passes.

## 2. Commit Conventions

Conventional Commits format: `<type>(<module>): <description>`. Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `security`. Example: `feat(beds): add bed assignment command and service`. Documentation-only commits touching `.claude/` use `docs(<doc-name>): <description>`.

## 3. Pull Request Checklist

Every PR description includes:
- Which [Plan](Plan.md) phase / [Tasks](Tasks.md) item it implements.
- Which documents were updated (per Rule 16.3) — or an explicit statement that none needed updating and why.
- Confirmation that [DefinitionOfDone](DefinitionOfDone.md) Section 1 (task-level) is satisfied.
- Test evidence (CI link/output).

## 4. Local Development Commands

| Command | Purpose |
|---|---|
| `npm install` | Install frontend dependencies. |
| `cargo build` (in `src-tauri/`) | Build Rust backend. |
| `npm run tauri dev` | Run the full app in dev mode (hot-reload frontend, debug Rust backend). |
| `npm run lint` | ESLint. |
| `npm run typecheck` | `tsc --noEmit`. |
| `npm run test` | Vitest. |
| `cargo fmt` / `cargo clippy` | Rust formatting/linting. |
| `cargo test` | Rust unit + integration tests. |
| `npm run test:e2e` | `tauri-driver` + WebdriverIO E2E suite. |

## 5. Pre-Commit Hooks

`husky` + `lint-staged` (frontend): runs Prettier + ESLint on staged `.ts(x)` files. A Rust-side `pre-commit` git hook (or CI-only enforcement, since Rust formatting hooks are less standard) runs `cargo fmt --check` and `cargo clippy -D warnings` on staged Rust files before allowing commit. Hooks are never bypassed with `--no-verify` except by explicit, individually justified user instruction (project-wide safety principle, not specific to this doc).

## 6. Release / Build Process

1. `npm run tauri build` produces the platform-native installer/bundle (via `tauri.conf.json` bundle config).
2. Release builds disable devtools and enforce the production CSP ([Security](Security.md) Section 9).
3. A release is only cut from `main` after Project-Level Definition of Done ([DefinitionOfDone](DefinitionOfDone.md) Section 3) passes.
4. Versioning: SemVer, tagged in git (`vMAJOR.MINOR.PATCH`), tag message references the [Plan](Plan.md) phases included.

## 7. Environment Setup

Prerequisites: Node.js LTS, Rust stable toolchain, platform-specific Tauri prerequisites (WebView2 on Windows, etc., per official Tauri docs). No `.env` file ships secrets (Rule 10.5) — the only environment-dependent config is non-secret (e.g., dev vs. release build flags), documented in `tauri.conf.json`, not a separate secrets file.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#developmentworkflowmd)
