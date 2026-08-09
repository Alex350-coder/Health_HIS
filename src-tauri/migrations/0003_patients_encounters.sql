-- Database.md Section 3.2 — Patients & Medical History. `patients` is the root entity of the
-- clinical workflow (CLAUDE.md Section 7). `encounters` is created here alongside it because the
-- next workflow step needs the table to exist, but only the `patients` CRUD ships in Phase 4 —
-- the `encounters` write path (diagnoses/treatments/evolutions too) ships in Phase 5.
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
