//! SQLCipher-encrypted `rusqlite` connection setup (Database.md Section 2).

use std::path::Path;

use rusqlite::Connection;

use super::DbError;

const BUSY_TIMEOUT_MS: u32 = 5000;

/// Opens (creating if absent) a SQLCipher-encrypted SQLite database at `path`, keyed with
/// `key`, and applies the connection settings required by every connection in this
/// application: WAL journaling, foreign-key enforcement, and a busy timeout.
///
/// `foreign_keys = ON` is a per-connection setting in SQLite (Task 1.1) — it is applied here,
/// on every call, rather than once at database-creation time.
pub fn open(path: &Path, key: &[u8; 32]) -> Result<Connection, DbError> {
    let conn = Connection::open(path)?;

    // SQLCipher's key pragma cannot be bound as a query parameter — PRAGMA statements do not
    // accept bind parameters in SQLite. The value here is a locally generated random key, not
    // external input, so building this one statement via `format!` does not violate Rules.md
    // 9.1 (that rule targets user-supplied data reaching SQL text).
    conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", to_hex(key)))?;

    // Forces SQLCipher to actually validate the key against the file header/first page.
    // `Connection::open` + `PRAGMA key` alone succeed even with a wrong key — SQLCipher only
    // reports the mismatch on the first real read.
    verify_key(&conn)?;

    conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", BUSY_TIMEOUT_MS)?;

    Ok(conn)
}

fn verify_key(conn: &Connection) -> Result<(), DbError> {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
        row.get::<_, i64>(0)
    })?;
    Ok(())
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    const TEST_KEY_A: [u8; 32] = [0x11; 32];
    const TEST_KEY_B: [u8; 32] = [0x22; 32];

    #[test]
    fn opens_and_applies_expected_pragmas() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pragmas.sqlite");

        let conn = open(&path, &TEST_KEY_A).unwrap();

        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        let foreign_keys: i64 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        let busy_timeout: i64 = conn
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .unwrap();

        assert_eq!(journal_mode.to_lowercase(), "wal");
        assert_eq!(foreign_keys, 1);
        assert_eq!(busy_timeout, i64::from(BUSY_TIMEOUT_MS));
    }

    #[test]
    fn data_survives_close_and_reopen_with_same_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roundtrip.sqlite");

        {
            let conn = open(&path, &TEST_KEY_A).unwrap();
            conn.execute_batch("CREATE TABLE probe (value TEXT NOT NULL);")
                .unwrap();
            conn.execute("INSERT INTO probe (value) VALUES (?1)", ["hello"])
                .unwrap();
        }

        let conn = open(&path, &TEST_KEY_A).unwrap();
        let value: String = conn
            .query_row("SELECT value FROM probe LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "hello");
    }

    #[test]
    fn reopening_with_wrong_key_is_rejected() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("wrongkey.sqlite");

        {
            let conn = open(&path, &TEST_KEY_A).unwrap();
            conn.execute_batch("CREATE TABLE probe (value TEXT NOT NULL);")
                .unwrap();
        }

        let result = open(&path, &TEST_KEY_B);
        assert!(result.is_err());
    }

    #[test]
    fn database_file_is_not_plaintext_on_disk() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("plaintext-check.sqlite");

        {
            let conn = open(&path, &TEST_KEY_A).unwrap();
            conn.execute_batch("CREATE TABLE probe (secret TEXT NOT NULL);")
                .unwrap();
            conn.execute("INSERT INTO probe (secret) VALUES (?1)", ["needle-value"])
                .unwrap();
        }

        let raw_bytes = fs::read(&path).unwrap();
        assert!(!raw_bytes.starts_with(b"SQLite format 3\0"));

        let needle = b"needle-value";
        let contains_plaintext_needle = raw_bytes
            .windows(needle.len())
            .any(|window| window == needle);
        assert!(!contains_plaintext_needle);
    }
}
