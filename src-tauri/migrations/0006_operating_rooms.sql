-- Database.md Section 3.4 — Operating Rooms. `operating_rooms` promotes an existing `rooms` row
-- (owned by Beds, `room_type = 'operating_room'`) into a schedulable resource; `or_reservations`
-- is the scheduling record. Overlap prevention (no two `scheduled`/`in_progress` reservations for
-- the same OR overlapping in time) is enforced in `operating_room_service.rs`, not here — SQLite
-- lacks native exclusion constraints.
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
