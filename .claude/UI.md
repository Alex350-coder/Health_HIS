# UI.md

## Purpose

Defines the design system: tokens, component behavior, states, and accessibility requirements, so every module produces visually and behaviorally consistent UI.

## Dependencies

[Rules](Rules.md) Sections 18–19, [Architecture](Architecture.md) Section 6 (Radix + Tailwind decision).

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md), [ErrorHandling](ErrorHandling.md) (error state mapping).

## Referenced By

[Architecture](Architecture.md), [Audit](Audit.md), [CodingStandards](CodingStandards.md), [DefinitionOfDone](DefinitionOfDone.md), [ErrorHandling](ErrorHandling.md), [FolderStructure](FolderStructure.md), [Plan](Plan.md), [README_Project](README_Project.md), [Routes](Routes.md), [Rules](Rules.md), [Tasks](Tasks.md), [Testing](Testing.md).

## Documents that must be updated if changes occur

[Routes](Routes.md) (page-level layout conventions), [Testing](Testing.md) (component test expectations), [FolderStructure](FolderStructure.md) (`shared/ui` inventory).

## Priority

7.

---

## Contents

- [1. Design Tokens](#1-design-tokens)
- [2. Navigation](#2-navigation)
- [3. Component Inventory (`shared/ui`)](#3-component-inventory-sharedui)
- [4. Forms](#4-forms)
- [5. Tables](#5-tables)
- [6. Loading / Empty / Error States (mandatory for every list/detail view)](#6-loading--empty--error-states-mandatory-for-every-listdetail-view)
- [7. Hospital Map (Special Component)](#7-hospital-map-special-component)
- [8. Accessibility](#8-accessibility)

---

## 1. Design Tokens

- **Spacing scale**: 4px base unit — `4, 8, 12, 16, 24, 32, 48, 64` (Tailwind default spacing scale used as-is, no custom overrides, to avoid an undocumented parallel scale).
- **Typography scale**: `xs 12px / sm 14px / base 16px / lg 18px / xl 20px / 2xl 24px / 3xl 30px`. Body text defaults to `base`. Clinical data tables use `sm` for density.
- **Color tokens** (semantic, not raw hex in components): `bg-surface`, `bg-surface-raised`, `text-primary`, `text-secondary`, `border-default`, `accent`, `success`, `warning`, `danger`, `info`. All token pairs (light/dark, if dark mode is added later) meet WCAG AA contrast (4.5:1 for text) — verified via an automated contrast check in the design-token definition file (`styles/globals.css`).
- Status colors always pair with a non-color indicator (icon/text) per Rule 19.2 — e.g., bed status: green dot + "Available" text, not color alone.

## 2. Navigation

- Persistent left sidebar listing all 10 modules (no role-based hiding, per the "same UI for all users" project rule), plus a visually separated administration group at the bottom containing **Facility** (`/facility`) and **Users** (`/users`) — configuration surfaces rather than clinical workflow, so they are grouped apart but not hidden or role-gated.
- `/setup` ([Routes](Routes.md) Section 3) renders standalone with no sidebar or top bar: at that point no user is authenticated and no navigation target is meaningful.
- Top bar: current user name/role (display only), logout action, notification bell (unread count from `notifications_list`).
- Patient-centric modules (Medical History, Beds assignment, Billing) are reachable both from the sidebar (list views) and contextually from a patient's detail page (per the patient-centered workflow — see [README_Project](README_Project.md) Section 2).

## 3. Component Inventory (`shared/ui`)

Built on Radix UI primitives + Tailwind (`cva` for variants), per [Architecture](Architecture.md) Section 6:

- `Button` (variants: primary, secondary, danger, ghost; sizes: sm/md/lg)
- `Card` (header/content/footer slots)
- `Dialog` (Radix Dialog wrapper — used for all create/edit/confirm flows, never a full page navigation for simple CRUD)
- `Form` primitives: `FormField`, `FormLabel`, `FormError` (wired to react-hook-form context)
- `Table` (with a virtualized variant — `VirtualizedTable` — for lists > 100 rows, Rule 18.1)
- `Badge` (status indicators — always icon/text + color, Rule 19.2)
- `Toast` (transient notifications, e.g., successful save, non-blocking `AppError::Conflict`)
- `Skeleton` (loading state placeholders, matches the shape of the content it replaces)
- `EmptyState` (icon + message + optional primary action, used whenever a list query returns zero rows)
- `ErrorState` (used inside `<ErrorBoundary>` fallback and for `AppError::Unexpected`/`Filesystem`, per [ErrorHandling](ErrorHandling.md) Section 3)

## 4. Forms

- Every form uses `react-hook-form` + the module's Zod schema (see [Validation](Validation.md)).
- Field-level errors render via `FormError` directly under the field, using `AppError::Validation.message` when the error originates server-side, or the Zod issue message when client-side — same visual treatment either way.
- Destructive actions (e.g., cancelling an OR reservation, deactivating a user) require a `Dialog`-based confirmation step. No native `window.confirm` (violates the "no browser dialogs" constraint in Tauri/webview automation contexts and is inconsistent with the design system).

## 5. Tables

- Default page size 25 for paginated list views (`patients_list`, `audit_list`); virtualization (Section 3) for views expected to render the full result set at once (e.g., a floor's bed grid).
- Sortable columns where the underlying query supports server-side sorting (avoids fetching then re-sorting client-side, which would create a second source of truth for ordering).

## 6. Loading / Empty / Error States (mandatory for every list/detail view)

Every data-bearing component built on a TanStack Query hook must explicitly render all three:
1. **Loading** — `Skeleton` matching the eventual layout, shown while `isLoading`.
2. **Empty** — `EmptyState`, shown when the query succeeds with zero results.
3. **Error** — `ErrorState` or inline error per [ErrorHandling](ErrorHandling.md) Section 3, shown when the query fails.

This triad is a checklist item in [DefinitionOfDone](DefinitionOfDone.md) for any new list/detail component.

## 7. Hospital Map (Special Component)

- Renders `floors`/`rooms` (via `hospital_map_get_layout`) as an interactive but strictly read-only SVG/canvas map, using `map_x`/`map_y` normalized coordinates ([Database](Database.md) Section 3.3).
- Clicking a room shows a read-only detail panel (bed/OR occupancy summary) — no edit affordances exist anywhere in this module's UI, enforcing the "never modifies data" project requirement at the UI layer in addition to the backend layer ([Architecture](Architecture.md) Section 3).

## 8. Accessibility

- WCAG 2.1 AA minimum (Rule 19.1). Radix primitives provide correct ARIA roles/keyboard interaction out of the box; custom components must preserve this (e.g., custom-styled `Dialog` still uses Radix's `Dialog.Root`, never a hand-rolled modal div).
- All interactive elements reachable and operable via keyboard (`Tab`/`Shift+Tab`/`Enter`/`Escape`/arrow keys where applicable). Focus is trapped within open dialogs and returned to the triggering element on close (Radix default behavior — must not be overridden).
- Every form input has an associated, visible `<label>` (via `FormLabel`) — no placeholder-only labeling.
- Automated check: `eslint-plugin-jsx-a11y` in CI (Rule 19, verification mechanism per Rules Section 21) plus a manual accessibility pass listed in [DefinitionOfDone](DefinitionOfDone.md).

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#uimd)
