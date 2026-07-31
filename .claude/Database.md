# Database.md

## Purpose

Defines the database technology decision, the complete relational schema, constraints, indexes, normalization rationale, and migration strategy.

## Dependencies

[Rules](Rules.md) Section 9, [Architecture](Architecture.md) Section 6.

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md).

## Referenced By

[Architecture](Architecture.md), [Audit](Audit.md), [DefinitionOfDone](DefinitionOfDone.md), [FolderStructure](FolderStructure.md), [Glossary](Glossary.md), [New_files](New_files.md), [Plan](Plan.md), [README_Project](README_Project.md), [Rules](Rules.md), [Security](Security.md), [Tasks](Tasks.md), [Testing](Testing.md), [UI](UI.md).

## Documents that must be updated if changes occur

[Security](Security.md) (encryption/key handling), [Audit](Audit.md) (audit_log schema), [ErrorHandling](ErrorHandling.md) (DB error mapping), [Plan](Plan.md)/[Tasks](Tasks.md) (migration tasks), [New_files](New_files.md) (new migration files).

## Priority

5.

---

## Contents

- [1. Database Technology Decision](#1-database-technology-decision)
- [2. Connection & Mode Settings](#2-connection--mode-settings)
- [3. Schema](#3-schema)
- [4. Normalization](#4-normalization)
- [5. Migration Strategy](#5-migration-strategy)
- [6. Migration-to-Phase Mapping](#6-migration-to-phase-mapping)
- [7. Permanent Medical History](#7-permanent-medical-history)
- [8. Scalability](#8-scalability)

---

## 1. Database Technology Decision

**Choice: SQLite, accessed exclusively from the Rust core process, encrypted at rest with SQLCipher (AES-256), via the `rusqlite` crate compiled with the `bundled-sqlcipher` feature.**

### Alternatives considered and rejected

| Option | Rejected because |
|---|---|
| IndexedDB (WebView-side) | Runs in the untrusted WebView process, violates the process-boundary security model in [Architecture](Architecture.md) Section 1; no relational integrity/joins; no native encryption at rest. |
| A key-value embedded store (e.g., `sled`) | No relational query capability; joins/constraints (foreign keys, uniqueness) would have to be hand-rolled in application code — directly conflicts with normalization requirements in [Rules](Rules.md) 9.4 and the highly relational domain (patients ↔ encounters ↔ beds ↔ rooms ↔ ORs). |
| DuckDB | Optimized for analytical (OLAP) workloads, not row-level transactional OLTP writes typical of a HIS (frequent single-row inserts/updates). |
| Client-server DB (Postgres/MySQL) requiring a local server process | Adds an operational dependency (a running DB server) inappropriate for a single-user desktop MVP; increases attack surface (network-facing service) for no benefit in a single-machine deployment. |
| Turso/LiteFS (replicated SQLite) | Adds distributed-systems complexity (replication, sync conflicts) not needed — this is a single-device desktop app, not multi-node. |

### Why SQLite + SQLCipher fits this project

1. **Embedded, zero-server**: matches "desktop application," no network attack surface for the DB itself.
2. **Relational + ACID**: the domain (patients, encounters, beds, rooms, ORs, inventory, billing) is inherently relational with strong referential-integrity needs — foreign keys, transactions across multiple tables (e.g., bed assignment updates both `beds` and `bed_assignments` atomically).
3. **SQLCipher transparent encryption**: PHI (Protected Health Information) must be encrypted at rest (Rule 10.4). SQLCipher provides page-level AES-256 encryption transparent to all SQL above it — no application-level encrypt/decrypt code needed per field.
4. **Process-boundary security**: because only the Rust core process links against the database driver, the WebView can never reach the DB even if compromised (Rule 1.3, [Security](Security.md)).
5. **Maturity**: SQLite is the most deployed and audited embedded database in existence; well-understood failure modes.

## 2. Connection & Mode Settings

- Journal mode: `WAL` (better concurrent read/write behavior for a desktop app with a single writer, multiple readers within the same process).
- `foreign_keys = ON` (enforced every connection open — SQLite disables this by default).
- `busy_timeout` set to 5000ms to avoid `SQLITE_BUSY` errors under transient contention.
- Encryption key: 256-bit, derived and retrieved via OS keychain (`keyring` crate) — never stored in the SQLite file, config file, or source. See [Security](Security.md) Section "Secrets Management."

> [!CAUTION]
> Lose the keychain entry and the database is unrecoverable by design — there is no escrow, no recovery key, and no plaintext fallback. Any feature that touches key storage must be reviewed against [Security](Security.md) before merge.

## 3. Schema

All tables use `id INTEGER PRIMARY KEY AUTOINCREMENT` unless noted. All tables have `created_at TEXT NOT NULL DEFAULT (datetime('now'))`; mutable tables also have `updated_at TEXT`.

### 3.1 Authentication

```sql
CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  full_name TEXT NOT NULL,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,           -- Argon2id encoded hash string
  role TEXT NOT NULL CHECK (role IN ('physician','nurse','admin','pharmacy','lab','receptionist')),
  is_active INTEGER NOT NULL DEFAULT 1,  -- boolean 0/1
  failed_login_attempts INTEGER NOT NULL DEFAULT 0,
  locked_until TEXT,                      -- NULL when not locked
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT
);

CREATE TABLE sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL REFERENCES users(id),
  token_hash TEXT NOT NULL UNIQUE,        -- SHA-256 of the opaque session token
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  expires_at TEXT NOT NULL,
  last_active_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_token_hash ON sessions(token_hash);
```

Note: single `role` column, no separate `roles`/`user_roles` tables. Justification: per the project spec, every authenticated user sees the same application UI regardless of role — role exists only for audit/traceability attribution, not for access-control branching. Full RBAC tables would be unused complexity (see [CodingStandards](CodingStandards.md) "No Unnecessary Abstraction").

### 3.2 Patients & Medical History

```sql
CREATE TABLE patients (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  medical_record_number TEXT NOT NULL UNIQUE,  -- hospital-assigned MRN
  full_name TEXT NOT NULL,
  date_of_birth TEXT NOT NULL,
  sex TEXT NOT NULL CHECK (sex IN ('male','female','other','unknown')),
  national_id TEXT UNIQUE,
  phone TEXT,
  address TEXT,
  emergency_contact_name TEXT,
  emergency_contact_phone TEXT,
  blood_type TEXT,
  allergies TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT
);
CREATE INDEX idx_patients_mrn ON patients(medical_record_number);
CREATE INDEX idx_patients_full_name ON patients(full_name);

-- One encounter = one hospital stay/episode. This is the "Medical Record" workflow step.
CREATE TABLE encounters (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  patient_id INTEGER NOT NULL REFERENCES patients(id),
  status TEXT NOT NULL CHECK (status IN ('open','discharged')) DEFAULT 'open',
  admitted_at TEXT NOT NULL DEFAULT (datetime('now')),
  discharged_at TEXT,
  discharge_summary TEXT,
  created_by_user_id INTEGER NOT NULL REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT
);
CREATE INDEX idx_encounters_patient_id ON encounters(patient_id);
CREATE INDEX idx_encounters_status ON encounters(status);

-- Append-only: corrections are new rows referencing corrected_row_id, never UPDATE/DELETE of clinical content.
CREATE TABLE diagnoses (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL REFERENCES encounters(id),
  description TEXT NOT NULL,
  icd_code TEXT,
  registered_by_user_id INTEGER NOT NULL REFERENCES users(id),
  corrects_diagnosis_id INTEGER REFERENCES diagnoses(id),  -- NULL unless this is a correction
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_diagnoses_encounter_id ON diagnoses(encounter_id);

CREATE TABLE treatments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL REFERENCES encounters(id),
  diagnosis_id INTEGER REFERENCES diagnoses(id),
  description TEXT NOT NULL,
  dosage TEXT,
  registered_by_user_id INTEGER NOT NULL REFERENCES users(id),
  corrects_treatment_id INTEGER REFERENCES treatments(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_treatments_encounter_id ON treatments(encounter_id);

CREATE TABLE evolutions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL REFERENCES encounters(id),
  note TEXT NOT NULL,
  registered_by_user_id INTEGER NOT NULL REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_evolutions_encounter_id ON evolutions(encounter_id);
```

**Why `treatments` has no `inventory_item_id` column.** A treatment's material consumption is recorded on `inventory_transactions.treatment_id` (Section 3.5), not as a foreign key on `treatments`. Two reasons:

1. **Migration ordering (correctness).** `treatments` is created in `0004`, `inventory_items` in `0007`. SQLite permits declaring a foreign key to a not-yet-existing table at `CREATE TABLE` time, but with `foreign_keys = ON` (Section 2) the first `INSERT` into `treatments` would fail at runtime with `no such table: main.inventory_items`. A forward-referencing FK is therefore not merely untidy here — it would break Phase 5 outright, three phases before Inventory ships.
2. **Cardinality (modelling).** One treatment may consume several distinct items (e.g., a drug plus a syringe plus saline). A single nullable FK on `treatments` cannot express that; a transaction row per consumed item can.

The clinical link is fully preserved and queryable in both directions: a treatment's consumed items are `SELECT * FROM inventory_transactions WHERE treatment_id = ?`.

### 3.3 Hospital Map, Beds

```sql
CREATE TABLE floors (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  level_order INTEGER NOT NULL,          -- display ordering
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE rooms (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  floor_id INTEGER NOT NULL REFERENCES floors(id),
  name TEXT NOT NULL,
  room_type TEXT NOT NULL CHECK (room_type IN ('ward','operating_room','pharmacy','laboratory','admin','other')),
  map_x REAL NOT NULL,                   -- normalized 0..1 coordinate for Hospital Map rendering
  map_y REAL NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_rooms_floor_id ON rooms(floor_id);

CREATE TABLE beds (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  room_id INTEGER NOT NULL REFERENCES rooms(id),
  label TEXT NOT NULL,                   -- e.g. "Bed 3A"
  status TEXT NOT NULL CHECK (status IN ('available','occupied','maintenance')) DEFAULT 'available',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT
);
CREATE INDEX idx_beds_room_id ON beds(room_id);
CREATE INDEX idx_beds_status ON beds(status);

CREATE TABLE bed_assignments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bed_id INTEGER NOT NULL REFERENCES beds(id),
  patient_id INTEGER NOT NULL REFERENCES patients(id),
  encounter_id INTEGER NOT NULL REFERENCES encounters(id),
  assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
  released_at TEXT,                      -- NULL while active
  assigned_by_user_id INTEGER NOT NULL REFERENCES users(id)
);
CREATE INDEX idx_bed_assignments_bed_id ON bed_assignments(bed_id);
CREATE INDEX idx_bed_assignments_encounter_id ON bed_assignments(encounter_id);
CREATE UNIQUE INDEX idx_bed_assignments_active ON bed_assignments(bed_id) WHERE released_at IS NULL;
```

The partial unique index `idx_bed_assignments_active` enforces at the database level that a bed can have at most one active (unreleased) assignment — a business rule enforced redundantly in `bed_service.rs` and the database, per defense-in-depth.

### 3.4 Operating Rooms

```sql
CREATE TABLE operating_rooms (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  room_id INTEGER NOT NULL UNIQUE REFERENCES rooms(id),
  name TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE or_reservations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  operating_room_id INTEGER NOT NULL REFERENCES operating_rooms(id),
  patient_id INTEGER NOT NULL REFERENCES patients(id),
  encounter_id INTEGER NOT NULL REFERENCES encounters(id),
  procedure_description TEXT NOT NULL,
  scheduled_start TEXT NOT NULL,
  scheduled_end TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('scheduled','in_progress','completed','cancelled')) DEFAULT 'scheduled',
  scheduled_by_user_id INTEGER NOT NULL REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT
);
CREATE INDEX idx_or_reservations_or_id ON or_reservations(operating_room_id);
CREATE INDEX idx_or_reservations_encounter_id ON or_reservations(encounter_id);
CREATE INDEX idx_or_reservations_scheduled_start ON or_reservations(scheduled_start);
```

Overlap prevention (no two `scheduled`/`in_progress` reservations for the same OR overlapping in time) is a business rule enforced in `operating_room_service.rs` (SQLite lacks native exclusion constraints); covered by an integration test per [Testing](Testing.md).

### 3.5 Inventory (Pharmacy + Lab share this)

```sql
CREATE TABLE inventory_categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  kind TEXT NOT NULL CHECK (kind IN ('medicine','supply','equipment'))
);

CREATE TABLE inventory_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  category_id INTEGER NOT NULL REFERENCES inventory_categories(id),
  name TEXT NOT NULL,
  quantity INTEGER NOT NULL DEFAULT 0,
  unit TEXT NOT NULL,                    -- e.g. "mg", "units", "box"
  reorder_threshold INTEGER NOT NULL DEFAULT 0,
  expiration_date TEXT,                  -- NULL for durable equipment
  location TEXT,                          -- e.g. "Pharmacy Shelf B2"
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT
);
CREATE INDEX idx_inventory_items_category_id ON inventory_items(category_id);
CREATE INDEX idx_inventory_items_expiration_date ON inventory_items(expiration_date);

CREATE TABLE inventory_transactions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  item_id INTEGER NOT NULL REFERENCES inventory_items(id),
  quantity_delta INTEGER NOT NULL,       -- negative = consumption, positive = restock
  reason TEXT NOT NULL CHECK (reason IN ('restock','consumption','adjustment','disposal')),
  encounter_id INTEGER REFERENCES encounters(id),  -- set when consumption tied to a patient encounter
  treatment_id INTEGER REFERENCES treatments(id),  -- set when consumption tied to a specific treatment
  performed_by_user_id INTEGER NOT NULL REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_inventory_transactions_item_id ON inventory_transactions(item_id);
CREATE INDEX idx_inventory_transactions_encounter_id ON inventory_transactions(encounter_id);
CREATE INDEX idx_inventory_transactions_treatment_id ON inventory_transactions(treatment_id);

CREATE TABLE maintenance_schedules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  inventory_item_id INTEGER NOT NULL REFERENCES inventory_items(id),
  scheduled_date TEXT NOT NULL,
  completed_date TEXT,
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_maintenance_schedules_item_id ON maintenance_schedules(inventory_item_id);
CREATE INDEX idx_maintenance_schedules_date ON maintenance_schedules(scheduled_date);
```

`inventory_items.quantity` is a maintained running total, updated transactionally alongside each `inventory_transactions` insert within `inventory_service.rs` — this is the one documented, justified denormalization (Rule 9.4): recomputing SUM(quantity_delta) on every read would be needlessly expensive for a value read constantly (stock displays, low-stock checks).

### 3.6 Notifications

```sql
CREATE TABLE notifications (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  type TEXT NOT NULL CHECK (type IN ('medicine_expiration','maintenance_due','low_stock','or_schedule','other')),
  target_role TEXT,                       -- NULL = all roles
  message TEXT NOT NULL,
  related_entity_type TEXT,               -- e.g. 'inventory_item'
  related_entity_id INTEGER,
  is_read INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_notifications_is_read ON notifications(is_read);
CREATE INDEX idx_notifications_target_role ON notifications(target_role);
```

### 3.7 Audit (see [Audit](Audit.md) for full behavioral spec)

```sql
CREATE TABLE audit_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp TEXT NOT NULL DEFAULT (datetime('now')),
  user_id INTEGER REFERENCES users(id),   -- NULL only for pre-auth events (e.g., failed login of unknown username)
  action TEXT NOT NULL,                    -- e.g. "patient.create", "bed.assign"
  entity_type TEXT NOT NULL,
  entity_id INTEGER,
  before_state TEXT,                       -- JSON snapshot, nullable
  after_state TEXT,                        -- JSON snapshot, nullable
  result TEXT NOT NULL CHECK (result IN ('success','failure')),
  prev_hash TEXT NOT NULL,                 -- hash of previous audit_log row (hash chain)
  row_hash TEXT NOT NULL                   -- hash of this row's content + prev_hash
);
CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);
```

A database trigger denies `UPDATE` and `DELETE` on `audit_log` at the SQL level (defense in depth alongside the Rust-layer restriction — see [Audit](Audit.md)).

### 3.8 Billing (Simulation Only)

```sql
CREATE TABLE billing_simulations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL UNIQUE REFERENCES encounters(id),
  status TEXT NOT NULL CHECK (status IN ('draft','finalized')) DEFAULT 'draft',
  total_amount REAL NOT NULL DEFAULT 0,
  generated_by_user_id INTEGER NOT NULL REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT
);

CREATE TABLE billing_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  billing_simulation_id INTEGER NOT NULL REFERENCES billing_simulations(id),
  description TEXT NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('room','treatment','inventory','operating_room','other')),
  source_entity_id INTEGER,
  amount REAL NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_billing_items_simulation_id ON billing_items(billing_simulation_id);
```

`total_amount` on `billing_simulations` is a maintained sum of its `billing_items.amount` — same justified-denormalization pattern as inventory quantity (Section 3.5).

## 4. Normalization

All tables are in 3NF. Every non-key column is fully functionally dependent on its table's primary key and nothing else. The two documented exceptions (`inventory_items.quantity`, `billing_simulations.total_amount`) are maintained running totals justified above, not raw duplication of another table's row.

## 5. Migration Strategy

- Tool: `sqlx-cli` migration format (plain, timestamped `.sql` files) even though runtime uses `rusqlite` — migrations are applied by a small first-party `migrator.rs` reading the same files, avoiding a heavy dependency for the runtime.
- File naming: `NNNN_description.sql` (e.g., `0001_init_auth.sql`, `0002_patients_and_encounters.sql`), strictly forward-only, sequential.
- Migrations never edited post-merge (Rule 9.2). A mistake is corrected by a new migration.
- Each migration file is one logical schema unit (see Section 6 for the phase-aligned migration plan referenced by [Plan](Plan.md)).
- Migrations run automatically on app startup (`db/migrator.rs`, called from `main.rs`) against the current DB file, tracked via a `schema_migrations` bookkeeping table.

```sql
CREATE TABLE schema_migrations (
  version TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

## 6. Migration-to-Phase Mapping

| Migration | Tables | Phase (see [Plan](Plan.md)) |
|---|---|---|
| `0000_schema_migrations.sql` | `schema_migrations` (bookkeeping only, Section 5) | Phase 1 |
| `0001_init_auth.sql` | `users`, `sessions` | Phase 2 |
| `0002_audit.sql` | `audit_log` (+ triggers) | Phase 2 |
| `0003_patients_encounters.sql` | `patients`, `encounters` | Phase 4 |
| `0004_medical_history.sql` | `diagnoses`, `treatments`, `evolutions` | Phase 5 |
| `0005_hospital_map_beds.sql` | `floors`, `rooms`, `beds`, `bed_assignments` | Phase 6/7 |
| `0006_operating_rooms.sql` | `operating_rooms`, `or_reservations` | Phase 9 |
| `0007_inventory.sql` | `inventory_categories`, `inventory_items`, `inventory_transactions`, `maintenance_schedules` | Phase 10 |
| `0008_notifications.sql` | `notifications` | Phase 11 |
| `0009_billing.sql` | `billing_simulations`, `billing_items` | Phase 12 |

## 7. Permanent Medical History

Per project spec and Rule 9.5: `diagnoses`, `treatments`, and `evolutions` rows are never `UPDATE`d or `DELETE`d after creation. A correction is a new row with `corrects_<entity>_id` pointing at the row it amends. `encounters.discharge_summary`/`status` are the only mutable fields on the medical-history side, representing the encounter lifecycle itself (open -> discharged), not clinical content.

## 8. Scalability

A single-hospital desktop MVP is not expected to exceed low tens of thousands of rows per table over realistic usage. SQLite with WAL mode and the indexes above comfortably serves this. If multi-device sync is ever required (explicitly out of scope for this MVP), that would be a new architecture decision revisiting Section 1, not an incremental change.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#databasemd)
