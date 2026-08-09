pub mod auth_validation;
pub mod patient_validation;

use crate::errors::AppError;

/// Maps the first `validator`-crate field error into the single-field `AppError::Validation`
/// shape (ErrorHandling.md Section 1) — shared by every `validate_x` wrapper across modules so
/// the mapping rule lives in exactly one place.
pub(crate) fn map_validation_errors(errors: validator::ValidationErrors) -> AppError {
    let field_errors = errors.field_errors();
    let first = field_errors.iter().next();

    match first {
        Some((field, errs)) => {
            let message = errs
                .first()
                .map(|error| error.code.to_string())
                .unwrap_or_else(|| "invalid".to_string());
            AppError::Validation {
                field: (*field).to_string(),
                message,
            }
        }
        None => AppError::Validation {
            field: "unknown".to_string(),
            message: "invalid".to_string(),
        },
    }
}
