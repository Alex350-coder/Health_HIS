-- Database.md Section 3.2 — Medical History write path (Phase 5). `encounters` already exists
-- (migration 0003); this migration adds the three append-only clinical tables (Rule 9.5,
-- CLAUDE.md Section 3.5): a correction is a new row referencing the row it corrects via
-- `corrects_<entity>_id`, never an UPDATE/DELETE of the original.

CREATE TABLE diagnoses (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL REFERENCES encounters(id),
  description TEXT NOT NULL,
  icd_code TEXT,
  registered_by_user_id INTEGER NOT NULL REFERENCES users(id),
  corrects_diagnosis_id INTEGER REFERENCES diagnoses(id),
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
