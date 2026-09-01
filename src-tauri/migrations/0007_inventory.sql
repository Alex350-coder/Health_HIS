-- Database.md Section 3.5 — Inventory (Pharmacy + Lab share this). Pharmacy and Lab are pure
-- consumers of these tables, not separate modules with their own schema. `inventory_items.quantity`
-- is a maintained running total, updated transactionally alongside each `inventory_transactions`
-- insert within `inventory_service.rs` — the one documented, justified denormalization (Rule 9.4):
-- recomputing SUM(quantity_delta) on every read would be needlessly expensive for a value read
-- constantly (stock displays, low-stock checks).
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
