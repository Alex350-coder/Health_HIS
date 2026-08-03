-- Database.md Section 3.1 — authentication tables. `users` is the single-role-column design
-- (no separate roles table): role exists for audit/traceability attribution only, since every
-- authenticated user sees the same application UI (Rules.md, CLAUDE.md Section 3.4).
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
