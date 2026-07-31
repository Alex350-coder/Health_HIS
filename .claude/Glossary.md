# Glossary.md

## Purpose

Defines clinical and technical terms used throughout the documentation, so implementers (human or LLM) share one unambiguous vocabulary. Added because the domain mixes hospital/clinical terminology with Tauri-specific technical terminology, both of which are easy to misinterpret.

## Dependencies

None (reference document).

## Documents that must be read before using it

None — consult on demand.

## Referenced By

[README_Project](README_Project.md).

## Documents that must be updated if changes occur

Any document introducing a new domain term should add it here in the same PR.

## Priority

Reference (no ordering position in the hierarchy).

---

## Clinical / Domain Terms

- **Patient**: A person registered in the system as a care recipient. Root entity of the workflow. See [Database](Database.md) Section 3.2.
- **Encounter**: One hospital stay/episode for a patient, from admission to discharge. Referred to informally as "Medical Record" in the workflow diagram — implemented as the `encounters` table.
- **Medical History**: The permanent, aggregate record of all of a patient's encounters, diagnoses, treatments, and evolutions across time. Not a single table — a query/view composed from `encounters` + `diagnoses` + `treatments` + `evolutions`.
- **Diagnosis**: A clinical determination recorded against an encounter.
- **Treatment**: A clinical intervention (may reference an inventory item, e.g., a medicine) recorded against an encounter/diagnosis.
- **Evolution**: A clinical progress note recorded against an encounter over time (the "how is the patient doing today" record).
- **Bed Assignment**: The act of associating a patient/encounter with a specific physical bed, and the record of that association's active/released lifecycle.
- **Operating Room (OR) Reservation**: A scheduled block of time reserving an operating room for a patient's procedure.
- **Inventory Item**: A medicine, medical supply, or piece of equipment tracked by quantity/location/expiration or maintenance schedule.
- **Pharmacy / Laboratory consultation**: Not separate data domains — both are consumers of the shared `Inventory` module and (for lab results tied to a patient) the `Medical History` module. There are no dedicated `pharmacy`/`laboratory` tables (see [Architecture](Architecture.md) Section 3).
- **Billing Simulation**: A generated, non-real invoice-like aggregation of an encounter's charges (room, treatments, inventory consumption, OR usage). Explicitly not connected to any real payment processor.
- **Discharge**: The workflow step closing an encounter (`encounters.status = 'discharged'`), after which the encounter's clinical content is permanently preserved and read-only for editing (append-only corrections still possible per [Database](Database.md) Section 7).
- **MRN (Medical Record Number)**: Hospital-assigned unique patient identifier, distinct from the internal database `id`.
- **Facility Configuration**: The structural reference data describing the hospital building itself — floors, rooms, beds, operating rooms, inventory categories. Owned by the Beds module (and OR/Inventory for their own tables), created through the `/facility` page. Distinct from clinical data: it describes *where* care happens, not *what* care happened. See [IPC](IPC.md) Section 2.1.

## Process Terms

- **Bootstrap (first-run)**: The one-time, unauthenticated creation of the first admin account on an installation whose `users` table is empty. Closes permanently once any user exists. See [Security](Security.md) Section 9.1.

## Technical Terms

- **Tauri Core (process)**: The Rust-based OS process hosting business logic, DB access, and the WebView window. See [Architecture](Architecture.md) Section 1.
- **WebView (process)**: The OS-native browser engine rendering the React UI, sandboxed from direct filesystem/DB access.
- **`invoke`**: The Tauri JS API used by the frontend to call a Rust `#[command]` function (request/response IPC).
- **Tauri Event**: A one-way, publish/subscribe IPC message (`emit`/`listen`) used for real-time cross-view state propagation. See [IPC](IPC.md) Section 3.
- **Capability**: A Tauri 2.x permission manifest scoping exactly which commands/plugins a given window may use. See [Security](Security.md) Section 1.
- **AppError**: The single serializable Rust error enum returned by every fallible command. See [ErrorHandling](ErrorHandling.md) Section 1.
- **SQLCipher**: A SQLite extension providing transparent, page-level AES-256 encryption at rest. See [Database](Database.md) Section 1.
- **Argon2id**: The memory-hard password hashing algorithm used for all stored credentials. See [Security](Security.md) Section 2.
- **Hash Chain**: The tamper-evidence mechanism on `audit_log`, where each row's hash incorporates the previous row's hash. See [Audit](Audit.md) Section 4.
- **Query Key**: The array-based cache identifier used by TanStack Query to address a specific piece of cached server-state. See [StateManagement](StateManagement.md) Section 2.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#glossarymd)
