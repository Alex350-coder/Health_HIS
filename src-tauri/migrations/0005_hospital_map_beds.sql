-- Database.md Section 3.3 — Hospital Map, Beds. All four tables are created here in one shared
-- migration (Plan.md Phase 6 "Note on migration sharing"): Hospital Map (Phase 6) only reads
-- `floors`/`rooms`, Beds facility-configuration (Phase 6) writes all three structural tables, and
-- `bed_assignments` is created now so its schema exists ahead of the Phase 7 assign/release
-- write path, even though nothing writes to it yet.
CREATE TABLE floors (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  level_order INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE rooms (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  floor_id INTEGER NOT NULL REFERENCES floors(id),
  name TEXT NOT NULL,
  room_type TEXT NOT NULL CHECK (room_type IN ('ward','operating_room','pharmacy','laboratory','admin','other')),
  map_x REAL NOT NULL,
  map_y REAL NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_rooms_floor_id ON rooms(floor_id);

CREATE TABLE beds (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  room_id INTEGER NOT NULL REFERENCES rooms(id),
  label TEXT NOT NULL,
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
  released_at TEXT,
  assigned_by_user_id INTEGER NOT NULL REFERENCES users(id)
);
CREATE INDEX idx_bed_assignments_bed_id ON bed_assignments(bed_id);
CREATE INDEX idx_bed_assignments_encounter_id ON bed_assignments(encounter_id);
CREATE UNIQUE INDEX idx_bed_assignments_active ON bed_assignments(bed_id) WHERE released_at IS NULL;
