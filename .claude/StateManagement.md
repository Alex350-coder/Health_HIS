# StateManagement.md

## Purpose

Defines the single-source-of-truth model and how state stays synchronized across every module and window, so no duplicated or stale state ever exists (per project mandate: "There must never be duplicated state").

## Dependencies

[Rules](Rules.md) Section 13, [Architecture](Architecture.md) Section 2.2, [IPC](IPC.md) Section "Events."

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md).

## Referenced By

[Architecture](Architecture.md), [Glossary](Glossary.md), [IPC](IPC.md), [Plan](Plan.md), [README_Project](README_Project.md), [Rules](Rules.md), [Tasks](Tasks.md), [Testing](Testing.md).

## Documents that must be updated if changes occur

[IPC](IPC.md) (event catalog), [Testing](Testing.md) (state-sync tests), [Plan](Plan.md)/[Tasks](Tasks.md) (per-module query key definitions).

## Priority

5.

---

## Contents

- [1. The One Writable Source of Truth](#1-the-one-writable-source-of-truth)
- [2. TanStack Query — Server-State Cache](#2-tanstack-query--server-state-cache)
- [3. Zustand — Ephemeral UI State](#3-zustand--ephemeral-ui-state)
- [4. Cross-Window / Cross-Component Real-Time Propagation](#4-cross-window--cross-component-real-time-propagation)
- [5. Optimistic Updates Policy](#5-optimistic-updates-policy)
- [6. Anti-Duplication Checklist (used in code review)](#6-anti-duplication-checklist-used-in-code-review)

---

## 1. The One Writable Source of Truth

The SQLite database (via the Rust core) is the only writable source of truth in the system. Every other piece of state in the frontend is either:

- **A cache** of a subset of the database (TanStack Query), or
- **Ephemeral, non-persistent UI state** with no database representation (Zustand).

No component ever holds a local `useState` copy of server data that outlives a single render cycle of derived/computed values.

## 2. TanStack Query — Server-State Cache

- Every module's `api/` folder defines: (a) query key factories, (b) query hooks (`useXQuery`), (c) mutation hooks (`useXMutation`).
- Query key convention: `[moduleName, resourceName, ...params]`, e.g., `['beds', 'list']`, `['beds', 'detail', bedId]`, `['patients', 'medical-history', patientId]`.
- **Mutations never manually patch the cache with `setQueryData` for correctness-critical clinical data.** They call `queryClient.invalidateQueries` for every affected key after a successful mutation, forcing a refetch from the database via the relevant Tauri command. Optimistic updates are restricted to non-clinical UI preference state only (see [Rules](Rules.md) and Section 5 below).
- Stale time: `0` for clinical/workflow data (beds, patients, encounters, OR schedule, inventory) — always considered stale, refetched on window focus and on invalidation, because correctness matters more than avoiding a refetch in a low-traffic desktop app. Longer stale times (5 min) are permitted only for near-static reference data (e.g., `floors`, `inventory_categories`).

## 3. Zustand — Ephemeral UI State

Used only for state with no DB representation:
- Active module/tab, open dialog state, multi-step form/wizard progress (before submission), authenticated session summary (user id, name, role, token — in memory only, see [Security](Security.md) Section 4).

Zustand stores never hold a copy of an entity fetched from the DB (e.g., never `useStore(s => s.patients)`. Patients always come from `usePatientsQuery()`).

## 4. Cross-Window / Cross-Component Real-Time Propagation

Because TanStack Query cache invalidation alone only affects the window/component tree that triggered the mutation, cross-window sync requires Tauri events:

1. After a successful mutation, the Service layer (Rust) emits a typed event via `events/emitter.rs` (e.g., `beds:assignment:created`). Full catalog in [IPC](IPC.md) Section "Events."
2. The frontend's `shared/hooks/use-event-subscription.ts` subscribes to relevant event names at the `App.tsx` root and calls `queryClient.invalidateQueries` for the affected query keys — this mapping (event name -> query keys to invalidate) is defined once, centrally, in `shared/lib/event-query-map.ts`, not duplicated per component.
3. This guarantees Rule 13.2: any mutation visible in more than one open view propagates immediately everywhere, without any component needing to know about other components.

## 5. Optimistic Updates Policy

Disallowed by default for anything touching clinical/workflow-critical state (patients, encounters, diagnoses, treatments, evolutions, bed assignments, OR reservations, inventory transactions, billing). These always wait for the Rust-confirmed result before the UI reflects the change — a HIS must never show a clinician state that didn't actually persist.

Optimistic updates are permitted only for pure UI preferences with no correctness impact (e.g., toggling a sidebar collapsed state, marking a notification as locally dismissed before the `is_read` mutation round-trip completes) — and must be explicitly listed as an exception in the relevant module's `api/` file with a one-line comment citing this section.

## 6. Anti-Duplication Checklist (used in code review)

- [ ] No component-level `useState`/`useReducer` mirrors data obtainable from a TanStack Query hook.
- [ ] No manual `setQueryData` outside the documented optimistic-update exceptions (Section 5).
- [ ] Every mutation that changes cross-view-visible data has a corresponding event emission (verified against [IPC](IPC.md) event catalog) and a corresponding entry in `event-query-map.ts`.
- [ ] No two modules define separate query hooks for the same underlying table/resource.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#statemanagementmd)
