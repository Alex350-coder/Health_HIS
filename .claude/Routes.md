# Routes.md

## Purpose

Defines the frontend route map, route guards, and code-splitting strategy.

## Dependencies

[Architecture](Architecture.md) Section 6 (TanStack Router decision), [Security](Security.md) Section 4 (session/auth guard).

## Documents that must be read before using it

[Architecture](Architecture.md), [UI](UI.md), [Security](Security.md).

## Referenced By

[Architecture](Architecture.md), [CodingStandards](CodingStandards.md), [DefinitionOfDone](DefinitionOfDone.md), [FolderStructure](FolderStructure.md), [Plan](Plan.md), [README_Project](README_Project.md), [Tasks](Tasks.md), [UI](UI.md).

## Documents that must be updated if changes occur

[UI](UI.md) (if navigation structure changes), [Tasks](Tasks.md) (new routes are implementation tasks), [FolderStructure](FolderStructure.md).

## Priority

7.

---

## Contents

- [1. Router](#1-router)
- [2. Route Guard](#2-route-guard)
- [3. Route Map](#3-route-map)
- [4. Route Parameter Validation](#4-route-parameter-validation)
- [5. Code Splitting](#5-code-splitting)

---

## 1. Router

TanStack Router, file-adjacent route tree defined in `src/app/router.tsx`, referencing lazy-loaded page components per module (`src/modules/<module>/components/<Module>Page.tsx` as the route-level entry — the one documented exception to the "no default export" rule, see [CodingStandards](CodingStandards.md) Section 1).

## 2. Route Guard

A root-level `beforeLoad` guard checks the Zustand auth slice (session token present and not expired client-side estimate) before allowing navigation to any route except `/login` and `/setup`. The authoritative check is always server-side (every command re-validates the session, [Security](Security.md) Section 4) — this guard is a UX convenience to avoid flashing protected UI before an inevitable `Unauthorized` error.

## 3. Route Map

| Path | Module | Page | Notes |
|---|---|---|---|
| `/setup` | Authentication | `BootstrapPage` | Public, but reachable **only** while `auth_bootstrap_status` returns `needsBootstrap: true`; otherwise redirects to `/login`. Creates the first admin — see [Security](Security.md) Section 9.1. |
| `/login` | Authentication | `LoginPage` | Public. Redirects to `/` if already authenticated; redirects to `/setup` if the installation is un-bootstrapped. |
| `/users` | Authentication | `UserListPage` | Create/list/deactivate users (`auth_create_user`, `auth_list_users`, `auth_deactivate_user`). |
| `/facility` | Beds | `FacilityConfigPage` | Create floors, rooms, beds; set bed `available`/`maintenance`; promote a room to an operating room. The only write path for structural data — see [IPC](IPC.md) Section 2.1. |
| `/` | — | `DashboardPage` | Overview: bed occupancy summary, notifications, quick actions. |
| `/patients` | Patients | `PatientListPage` | |
| `/patients/new` | Patients | `PatientCreatePage` | |
| `/patients/:patientId` | Patients | `PatientDetailPage` | Hub page — tabs into Medical History, Beds, Billing for this patient (patient-centered workflow). |
| `/patients/:patientId/medical-history` | Medical History | `MedicalHistoryTab` (rendered within `PatientDetailPage`) | Encounters, diagnoses, treatments, evolutions. |
| `/patients/:patientId/billing` | Billing | `BillingTab` (rendered within `PatientDetailPage`) | |
| `/hospital-map` | Hospital Map | `HospitalMapPage` | Read-only. |
| `/beds` | Beds | `BedListPage` | Cross-patient bed board (all beds, all statuses). |
| `/operating-rooms` | Operating Rooms | `OrSchedulePage` | Calendar/list of reservations. |
| `/inventory` | Inventory | `InventoryListPage` | Shared Pharmacy/Lab consultation view. |
| `/inventory/:itemId` | Inventory | `InventoryItemDetailPage` | Transaction history, maintenance schedule. |
| `/notifications` | Notifications | `NotificationListPage` | |
| `/audit` | Audit | `AuditLogPage` | |
| `*` (catch-all) | — | `NotFoundPage` | |

## 4. Route Parameter Validation

Route params (e.g., `:patientId`) are parsed and validated (positive integer) by TanStack Router's `params.parse` hook, using the same primitive Zod validators as the corresponding entity schema where applicable — rejecting malformed params before any query fires (avoids a wasted/invalid `invoke` call).

## 5. Code Splitting

Every page component is lazy-loaded (`React.lazy` via TanStack Router's built-in code-splitting) at the module boundary — matches the module folder boundary in [FolderStructure](FolderStructure.md), so each module's JS is only loaded when a user navigates into it.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#routesmd)
