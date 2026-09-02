-- Database.md Section 3.6 — Notifications. System-generated alerts surfaced in the top-bar bell.
-- `target_role` is always NULL in this MVP (Constraint 4 / CLAUDE.md Section 3.4: every
-- authenticated user sees the same application), retained for forward compatibility only.
-- Notifications are not individually audited (Audit.md) — only the triggering action is.
CREATE TABLE notifications (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  type TEXT NOT NULL CHECK (
    type IN ('medicine_expiration', 'maintenance_due', 'low_stock', 'or_schedule', 'other')
  ),
  target_role TEXT,
  message TEXT NOT NULL,
  related_entity_type TEXT,
  related_entity_id INTEGER,
  is_read INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_notifications_is_read ON notifications(is_read);
CREATE INDEX idx_notifications_target_role ON notifications(target_role);
