-- Database.md Section 3.8 — Billing (Simulation Only). Aggregates room/treatment/inventory/OR
-- charges for one encounter into an itemized simulation. No real payment processing.
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
