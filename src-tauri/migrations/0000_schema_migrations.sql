-- Database.md Section 5 — bookkeeping table tracking which forward-only migrations have
-- been applied to this database file. `IF NOT EXISTS` lets `migrator::run_migrations` bootstrap
-- this same table before iterating the migration list, without conflicting with this file
-- being applied as migration 0000 in the normal course.
CREATE TABLE IF NOT EXISTS schema_migrations (
  version TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
