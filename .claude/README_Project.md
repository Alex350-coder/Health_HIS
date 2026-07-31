# README_Project.md — Hospital Information System (HIS)

> A secure, offline-first desktop Hospital Information System MVP.
> **Tauri 2.x** · **React 18 + TypeScript** · **Rust** · **SQLite + SQLCipher**

## Purpose

Entry point to the planning framework. Orients any reader (human or LLM) before touching code or any other document.

## Dependencies

None (entry point).

## Documents that must be read before using it

None. Read this first.

## Referenced By

[DefinitionOfDone](DefinitionOfDone.md), [Plan](Plan.md), [Testing](Testing.md), [UI](UI.md).

## Documents that must be updated if changes occur

This file is updated only when the document set itself changes (new document added/removed) or the project summary changes.

## Priority

0 (orientation layer — read first, authority rests with [Rules](Rules.md)).

---

## Contents

- [1. What This Project Is](#1-what-this-project-is)
- [2. Patient-Centered Workflow](#2-patient-centered-workflow)
- [3. Modules](#3-modules)
- [4. Document Map](#4-document-map)
- [5. Document Hierarchy (authority order)](#5-document-hierarchy-authority-order)
- [6. How To Use This Framework](#6-how-to-use-this-framework)

---

## 1. What This Project Is

A secure desktop **Hospital Information System (HIS)** MVP built with **Tauri + React + TypeScript** (frontend) and **Rust** (Tauri core / backend), backed by an embedded **SQLite + SQLCipher** database. See [Database](Database.md) for the database decision and justification.

This is **not** a full commercial HIS. It is a realistic MVP demonstrating: software architecture, clean code, desktop development, relational database design, cross-module state synchronization, and secure software engineering. Security is a first-class citizen, not an afterthought.

### At a glance

| | |
|---|---|
| **Desktop shell** | Tauri 2.x (Rust core process + sandboxed WebView) |
| **Frontend** | React 18, TypeScript (strict), Vite |
| **Backend** | Rust — command / service / repository layers |
| **Database** | SQLite encrypted at rest with SQLCipher (AES-256) |
| **State** | TanStack Query (server state) + Zustand (UI state only) |
| **Phases** | 15 (Phase 0 – Phase 14), see [Plan](Plan.md) |
| **Modules** | 10, see Section 3 |
| **Documents** | 22 in `.claude/`, plus [CLAUDE.md](../CLAUDE.md) at the repository root — see Section 4 |

> [!IMPORTANT]
> Three constraints are non-negotiable and shape every other decision in this framework:
>
> 1. **No mock data, ever.** No temporary arrays, no hardcoded entities. Every feature reads and writes the real database from its first commit — see [Rules](Rules.md) Section 17.
> 2. **The Hospital Map never modifies data.** It is a pure read-only visualization over tables owned by other modules — see [Architecture](Architecture.md) Section 3.
> 3. **Authentication gates everything**, and every authenticated user sees the same application — see [Security](Security.md).

## 2. Patient-Centered Workflow

```
Patient Registration
  -> Medical Record Creation / Retrieval
    -> Hospital Bed Assignment
      -> Diagnosis Registration
        -> Treatment Registration
          -> Medical Evolution Registration
            -> Operating Room Scheduling
              -> Shared Consultation (Pharmacy + Laboratory)
                -> Billing Generation (Simulation Only)
                  -> Patient Discharge
                    -> Permanent Medical History Preservation
```

Every module in the system exists to serve exactly one step (or several) of this workflow. No module exists in isolation.

## 3. Modules

Authentication, Patients, Medical History, Hospital Map (visual/read-only), Beds, Operating Rooms, Inventory, Notifications, Audit, Billing (Simulation).

## 4. Document Map

| Document | Purpose |
|---|---|
| [Rules](Rules.md) | Highest authority. Objective, measurable rules for the entire codebase. |
| [DefinitionOfDone](DefinitionOfDone.md) | Gate: when is a task/phase actually complete. |
| [Architecture](Architecture.md) | Layered architecture, module boundaries, data flow, dependency graph. |
| [FolderStructure](FolderStructure.md) | Canonical repository tree. |
| [CodingStandards](CodingStandards.md) | Naming, formatting, language-specific idioms. |
| [Database](Database.md) | Full relational schema, constraints, indexes, migrations. |
| [Security](Security.md) | AuthN/AuthZ, hashing, rate limiting, secrets, secure defaults. |
| [Validation](Validation.md) | Input validation strategy (client + backend, double validation). |
| [ErrorHandling](ErrorHandling.md) | Error taxonomy and handling strategy per category. |
| [Audit](Audit.md) | What is audited, audit log integrity, tamper evidence. |
| [Testing](Testing.md) | Test strategy and coverage gates across all layers. |
| [UI](UI.md) | Design system, components, accessibility. |
| [Routes](Routes.md) | Frontend route map and guards. |
| [StateManagement](StateManagement.md) | Single source of truth, cache/state sync strategy. |
| [IPC](IPC.md) | Tauri command contract catalog (frontend <-> backend boundary). |
| [Glossary](Glossary.md) | Clinical and technical term definitions. |
| [DevelopmentWorkflow](DevelopmentWorkflow.md) | Branching, commits, PRs, local dev, release process. |
| [Plan](Plan.md) | Phase-by-phase project director. |
| [Tasks](Tasks.md) | Atomic implementation tasks per phase. |
| [Progress](Progress.md) | Live status tracker per phase/task. |
| [New_files](New_files.md) | Log of every new file created and its purpose. |

Outside this directory, at the repository root:

| Document | Purpose |
|---|---|
| [CLAUDE.md](../CLAUDE.md) | Single-file project context loaded automatically by Claude Code. A summary and map of the framework above — it holds no authority and is updated whenever a document here changes. |

## 5. Document Hierarchy (authority order)

```
Rules
  -> Architecture
    -> Security
      -> Database
        -> Validation / ErrorHandling / Audit / StateManagement / IPC
          -> UI / Routes / Testing
            -> Plan
              -> Tasks
                -> Progress / New_files
```

Documents with higher priority always prevail. No document may contradict [Rules](Rules.md). See [Rules](Rules.md) Section 1 for the enforcement statement.

## 6. How To Use This Framework

1. Before writing any code, read [Rules](Rules.md) in full.
2. Before starting a phase, read every document listed in that phase's "Required Reading" section in [Plan](Plan.md).
3. Implement only what is described in [Tasks](Tasks.md) for the active phase.
4. A phase is not finished until every criterion in [DefinitionOfDone](DefinitionOfDone.md) and the phase's own Exit Criteria (in [Plan](Plan.md)) pass.
5. Update [Progress](Progress.md) and [New_files](New_files.md) as part of finishing any task — this is not optional, it is part of Definition of Done.
6. Never invent requirements. If something is ambiguous, it is a documentation gap — resolve it by extending the relevant document, not by improvising in code.

---

<!-- nav-footer -->
[↑ Back to top](#readme_projectmd--hospital-information-system-his)
