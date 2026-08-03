-- Database.md Section 3.7 / Audit.md — append-only, hash-chained audit log. The two triggers
-- below are the database-level half of the append-only guarantee (defense in depth alongside
-- audit_repository.rs exposing no update/delete method at all).
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

CREATE TRIGGER audit_log_no_update
BEFORE UPDATE ON audit_log
BEGIN
  SELECT RAISE(ABORT, 'audit_log is append-only');
END;

CREATE TRIGGER audit_log_no_delete
BEFORE DELETE ON audit_log
BEGIN
  SELECT RAISE(ABORT, 'audit_log is append-only');
END;
