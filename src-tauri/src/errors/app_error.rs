//! The single application-wide error type crossing the IPC boundary (ErrorHandling.md Section 1).

use rand::rngs::OsRng;
use rand::TryRngCore;
use serde::Serialize;
use thiserror::Error;

use crate::db::DbError;
use crate::security::hashing::HashingError;
use crate::security::SecretsError;

/// Every fallible command returns `Result<T, AppError>`. The three technical variants carry a
/// `correlation_id` so a user-reported error can be matched to the exact server-side log entry
/// without ever exposing raw error detail to the frontend (ErrorHandling.md Section 5).
#[derive(Debug, Error, Serialize)]
#[serde(tag = "type")]
pub enum AppError {
    #[error("validation error on {field}: {message}")]
    Validation { field: String, message: String },

    #[error("{entity} {id} not found")]
    NotFound { entity: String, id: i64 },

    #[error("conflict: {message}")]
    Conflict { message: String },

    #[error("unauthorized")]
    Unauthorized,

    #[error("account locked, retry after {retry_after_secs}s")]
    AccountLocked { retry_after_secs: i64 },

    #[error("database error (correlation_id={correlation_id})")]
    Database {
        message: String,
        correlation_id: String,
    },

    #[error("filesystem error (correlation_id={correlation_id})")]
    Filesystem {
        message: String,
        correlation_id: String,
    },

    #[error("unexpected error (correlation_id={correlation_id})")]
    Unexpected {
        message: String,
        correlation_id: String,
    },
}

/// Short, opaque, non-sequential id (e.g. `a1b2c3`) — carries no PHI and no technical detail of
/// its own, only enough entropy to disambiguate concurrent log entries (ErrorHandling.md Section 5).
///
/// `pub(crate)` so other technical-error sites (e.g. `security::session`) can build an
/// `AppError::Unexpected` with the same correlation-id convention without duplicating it.
pub(crate) fn correlation_id() -> String {
    let mut bytes = [0u8; 3];
    // A CSPRNG failure here is not fatal to error reporting: fall back to a fixed marker rather
    // than propagating a second error while already handling the first.
    if OsRng.try_fill_bytes(&mut bytes).is_err() {
        return "000000".to_string();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl From<DbError> for AppError {
    fn from(error: DbError) -> Self {
        let correlation_id = correlation_id();
        tracing::error!(correlation_id, error = %error, "database operation failed");
        AppError::Database {
            message: "a database error occurred".to_string(),
            correlation_id,
        }
    }
}

impl From<SecretsError> for AppError {
    fn from(error: SecretsError) -> Self {
        let correlation_id = correlation_id();
        tracing::error!(correlation_id, error = %error, "secrets operation failed");
        AppError::Unexpected {
            message: "a secrets error occurred".to_string(),
            correlation_id,
        }
    }
}

impl From<HashingError> for AppError {
    fn from(error: HashingError) -> Self {
        let correlation_id = correlation_id();
        tracing::error!(correlation_id, error = %error, "password hashing operation failed");
        AppError::Unexpected {
            message: "a password hashing error occurred".to_string(),
            correlation_id,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn database_variant_never_leaks_the_raw_driver_message() {
        let db_error = DbError::Io(std::io::Error::other("disk full, path=/secret/patients.db"));
        let app_error: AppError = db_error.into();

        match app_error {
            AppError::Database { message, .. } => {
                assert!(!message.contains("/secret"));
            }
            other => panic!("expected Database variant, got {other:?}"),
        }
    }

    #[test]
    fn correlation_ids_are_six_hex_characters() {
        let id = correlation_id();
        assert_eq!(id.len(), 6);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn serializes_with_a_type_tag() {
        let error = AppError::Unauthorized;
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, r#"{"type":"Unauthorized"}"#);
    }

    #[test]
    fn validation_variant_serializes_its_fields() {
        let error = AppError::Validation {
            field: "username".to_string(),
            message: "too short".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(
            json,
            r#"{"type":"Validation","field":"username","message":"too short"}"#
        );
    }
}
