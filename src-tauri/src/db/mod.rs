pub mod connection;
pub mod migrator;

use thiserror::Error;

/// Errors raised by the database connection/migration layer. Not yet unified with the
/// application-wide `AppError` enum — that mapping is introduced in Phase 2 once commands
/// exist to surface it over IPC (see ErrorHandling.md).
#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
