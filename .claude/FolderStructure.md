# FolderStructure.md

## Purpose

Canonical, exhaustive repository tree. No file or folder may exist outside this structure without a documented amendment here.

## Dependencies

[Rules](Rules.md) Section 2 (Folder Organization Rules).

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md).

## Referenced By

[Architecture](Architecture.md), [CodingStandards](CodingStandards.md), [IPC](IPC.md), [New_files](New_files.md), [Plan](Plan.md), [README_Project](README_Project.md), [Routes](Routes.md), [Rules](Rules.md), [Tasks](Tasks.md), [Testing](Testing.md), [UI](UI.md).

## Documents that must be updated if changes occur

[Architecture](Architecture.md) (if a layer changes), [Rules](Rules.md) (if a naming/organization rule changes), [New_files](New_files.md) (log every new file).

## Priority

2.

---

## Contents

- [1. Repository Root](#1-repository-root)
- [2. Frontend — `src/`](#2-frontend--src)
- [3. Backend — `src-tauri/`](#3-backend--src-tauri)
- [4. Cross-Cutting Tests — `tests/`](#4-cross-cutting-tests--tests)
- [5. Rules](#5-rules)

---

## 1. Repository Root

```
Health_Project/
├── .claude/                     # Planning framework (this directory)
├── .husky/
│   └── pre-commit               # lint-staged + conditional Rust gate — DevelopmentWorkflow Section 5
├── src/                         # React + TypeScript frontend
├── src-tauri/                   # Rust backend (Tauri core)
├── tests/                       # Cross-cutting E2E tests
├── .github/workflows/           # CI pipelines
├── CLAUDE.md                    # Orientation layer for humans and LLMs
├── index.html                   # Vite HTML entry point; loads src/main.tsx only
├── package.json
├── package-lock.json            # Committed — CI uses `npm ci` for reproducible installs
├── tsconfig.json
├── vite.config.ts
├── vitest.config.ts             # Extends vite.config.ts; owns the coverage thresholds
├── vitest.setup.ts              # Registers jest-dom matchers for every test file
├── eslint.config.js             # ESLint 9 flat config (replaces the former .eslintrc.cjs)
├── .prettierrc
├── .prettierignore
├── .lintstagedrc.json
├── .gitignore
└── README.md                    # Public-facing project readme (not the planning framework)
```

> [!NOTE]
> **Amendment (Phase 0).** The original tree predated the toolchain decisions taken in Phase 0.
> `.eslintrc.cjs` is replaced by `eslint.config.js` because ESLint 9 uses flat config; the
> remaining additions (`index.html`, `vitest.*`, `.prettierignore`, `.lintstagedrc.json`,
> `.gitignore`, `.husky/`, `package-lock.json`) are the mandatory entry points and tool
> configuration files implied by Tasks 0.1–0.5. See [New_files](New_files.md) for the per-file
> rationale.

## 2. Frontend — `src/`

```
src/
├── app/
│   ├── App.tsx                  # Root component, providers (QueryClient, Router, Auth)
│   ├── router.tsx               # Route tree — see [Routes](Routes.md)
│   └── providers/                # AuthProvider, QueryProvider, ThemeProvider
├── modules/
│   ├── auth/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── api/                 # TanStack Query hooks wrapping Tauri invoke calls
│   │   └── types/
│   ├── patients/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── api/
│   │   └── types/
│   ├── medical-history/
│   │   ├── components/          # Diagnoses, Treatments, Evolutions sub-views
│   │   ├── hooks/
│   │   ├── api/
│   │   └── types/
│   ├── hospital-map/
│   │   ├── components/          # Read-only visual floor/room renderer
│   │   ├── hooks/
│   │   ├── api/
│   │   └── types/
│   ├── beds/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── api/
│   │   └── types/
│   ├── operating-rooms/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── api/
│   │   └── types/
│   ├── inventory/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── api/
│   │   └── types/
│   ├── notifications/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── api/
│   │   └── types/
│   ├── audit/
│   │   ├── components/          # Audit log viewer (read-only)
│   │   ├── hooks/
│   │   ├── api/
│   │   └── types/
│   └── billing/
│       ├── components/
│       ├── hooks/
│       ├── api/
│       └── types/
├── shared/
│   ├── ui/                      # Design-system primitives — see [UI](UI.md) (Button, Dialog, Card, Table, ...)
│   ├── components/              # Composed shared components (e.g., EmptyState, PageHeader)
│   ├── hooks/                   # Cross-module hooks (useSession, useEventSubscription)
│   ├── lib/                     # Pure utility functions (formatting, date helpers)
│   ├── schemas/                 # Zod schemas shared across modules (if any)
│   └── errors/                  # Frontend AppError type + mapping helpers
├── styles/
│   └── globals.css              # Tailwind base + design tokens
├── main.tsx                     # Vite/React entry point
└── vite-env.d.ts                # Vite client type reference (ambient declarations only)
```

> [!NOTE]
> **Amendment (Phase 0).** `vite-env.d.ts` is added — Vite requires it for `import.meta.env`
> typing. Colocated test files (`*.test.tsx` next to the unit under test, e.g.
> `src/app/App.test.tsx`) are permitted throughout `src/` per [Testing](Testing.md); they are
> not listed individually in this tree.

## 3. Backend — `src-tauri/`

```
src-tauri/
├── Cargo.toml
├── build.rs                     # Tauri build script (tauri-build codegen) — required by Cargo
├── rustfmt.toml                 # Rust formatting config — CodingStandards Section 2
├── .gitignore                   # Ignores /target and /gen build artefacts
├── icons/                       # Application icon set consumed by tauri.conf.json bundle
├── tauri.conf.json
├── capabilities/                # Least-privilege permission manifests — see [Security](Security.md)
├── migrations/                  # Versioned, forward-only SQL migrations — see [Database](Database.md) Section 6
│   └── 0000_schema_migrations.sql, 0001_init_auth.sql, 0002_audit.sql, ...
├── src/
│   ├── main.rs                  # Tauri app bootstrap, plugin registration
│   ├── db/
│   │   ├── mod.rs
│   │   ├── connection.rs        # SQLCipher-encrypted connection pool setup
│   │   └── migrator.rs
│   ├── models/                  # Rust structs mirroring DB rows (one file per aggregate)
│   │   ├── patient.rs
│   │   ├── encounter.rs
│   │   ├── bed.rs
│   │   ├── operating_room.rs
│   │   ├── inventory.rs
│   │   ├── notification.rs
│   │   ├── audit.rs
│   │   ├── billing.rs
│   │   └── user.rs
│   ├── repositories/            # Data access only — no business logic
│   │   ├── patient_repository.rs
│   │   ├── encounter_repository.rs
│   │   ├── bed_repository.rs
│   │   ├── operating_room_repository.rs
│   │   ├── inventory_repository.rs
│   │   ├── notification_repository.rs
│   │   ├── audit_repository.rs
│   │   ├── billing_repository.rs
│   │   └── user_repository.rs
│   ├── services/                # Business logic, orchestration, validation
│   │   ├── auth_service.rs
│   │   ├── patient_service.rs
│   │   ├── medical_history_service.rs
│   │   ├── bed_service.rs
│   │   ├── operating_room_service.rs
│   │   ├── inventory_service.rs
│   │   ├── notification_service.rs
│   │   ├── audit_service.rs
│   │   └── billing_service.rs
│   ├── commands/                 # Tauri #[command] entry points — see [IPC](IPC.md)
│   │   ├── auth_commands.rs
│   │   ├── patient_commands.rs
│   │   ├── medical_history_commands.rs
│   │   ├── hospital_map_commands.rs
│   │   ├── bed_commands.rs
│   │   ├── operating_room_commands.rs
│   │   ├── inventory_commands.rs
│   │   ├── notification_commands.rs
│   │   ├── audit_commands.rs
│   │   └── billing_commands.rs
│   ├── security/
│   │   ├── hashing.rs            # Argon2id wrapper
│   │   ├── session.rs            # Session token generation/validation
│   │   ├── rate_limit.rs
│   │   └── secrets.rs            # OS keychain integration
│   ├── errors/
│   │   └── app_error.rs          # Single AppError enum — see [ErrorHandling](ErrorHandling.md)
│   ├── validation/                # Server-side validators — see [Validation](Validation.md)
│   └── events/
│       └── emitter.rs             # Typed Tauri event emission helpers — see [IPC](IPC.md)
└── tests/
    ├── commands/                 # Integration tests per command, against temp SQLite
    └── services/                 # Unit tests per service
```

## 4. Cross-Cutting Tests — `tests/`

```
tests/
└── e2e/
    ├── auth.spec.ts
    ├── patient-admission-flow.spec.ts   # Full workflow: register -> admit -> discharge
    └── ...
```

## 5. Rules

- No file is created outside this tree without updating this document in the same commit (see [Rules](Rules.md) 2.1).
- Every module folder (frontend and backend) follows the exact same internal shape. This uniformity is itself a rule: it removes ambiguity about where new code goes.
- `shared/` (frontend) and `security/`/`errors/`/`events/` (backend) are the only places cross-module code may live.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#folderstructuremd)
