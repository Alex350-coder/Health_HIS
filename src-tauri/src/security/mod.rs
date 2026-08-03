pub mod secrets;

use thiserror::Error;

/// Errors raised by the security layer. Not yet unified with `AppError` — see the equivalent
/// note on `db::DbError`.
#[derive(Debug, Error)]
pub enum SecretsError {
    #[error("keychain access failed: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("stored key was not valid hex: {0}")]
    InvalidHex(String),
    #[error("stored key had an unexpected length: expected 32 bytes, got {0}")]
    InvalidLength(usize),
}
