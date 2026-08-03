//! Server-side authoritative validation for auth inputs (Validation.md Section 3), mirroring the
//! client Zod schemas at `modules/auth/types/auth-schemas.ts` — the one intentionally duplicated
//! rule set per Rule 17.4. These structs double as the Tauri command input DTOs.

use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::errors::AppError;

const VALID_ROLES: [&str; 6] = [
    "physician",
    "nurse",
    "admin",
    "pharmacy",
    "lab",
    "receptionist",
];

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserInput {
    #[validate(length(min = 1, max = 200))]
    pub full_name: String,
    #[validate(custom(function = "validate_username"))]
    pub username: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,
    #[validate(custom(function = "validate_role"))]
    pub role: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}

pub fn validate_create_user(input: &CreateUserInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_login(input: &LoginInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

/// Validation.md Section 3 — `username` 3-50 chars, alphanumeric plus `._-`.
fn validate_username(username: &str) -> Result<(), ValidationError> {
    let length = username.chars().count();
    if !(3..=50).contains(&length) {
        return Err(ValidationError::new("username_length"));
    }
    let valid_charset = username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if !valid_charset {
        return Err(ValidationError::new("username_charset"));
    }
    Ok(())
}

/// Validation.md Section 5 — 12+ chars, at least one upper/lower/digit/symbol.
fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.chars().count() < 12 {
        return Err(ValidationError::new("password_length"));
    }

    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_ascii_alphanumeric());

    if has_upper && has_lower && has_digit && has_symbol {
        Ok(())
    } else {
        Err(ValidationError::new("password_complexity"))
    }
}

/// Database.md 3.1's `role` CHECK enum — validated here too for an actionable error message
/// instead of surfacing a raw SQLite constraint failure.
fn validate_role(role: &str) -> Result<(), ValidationError> {
    if VALID_ROLES.contains(&role) {
        Ok(())
    } else {
        Err(ValidationError::new("role_invalid"))
    }
}

fn map_validation_errors(errors: validator::ValidationErrors) -> AppError {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> CreateUserInput {
        CreateUserInput {
            full_name: "Ada Lovelace".to_string(),
            username: "ada.lovelace".to_string(),
            password: "Correct-Horse-9".to_string(),
            role: "admin".to_string(),
        }
    }

    #[test]
    fn accepts_a_fully_valid_create_user_input() {
        assert!(validate_create_user(&valid_input()).is_ok());
    }

    #[test]
    fn rejects_a_username_shorter_than_three_characters() {
        let input = CreateUserInput {
            username: "ab".to_string(),
            ..valid_input()
        };
        assert!(validate_create_user(&input).is_err());
    }

    #[test]
    fn rejects_a_username_with_a_disallowed_character() {
        let input = CreateUserInput {
            username: "ada lovelace".to_string(),
            ..valid_input()
        };
        assert!(validate_create_user(&input).is_err());
    }

    #[test]
    fn rejects_a_password_shorter_than_twelve_characters() {
        let input = CreateUserInput {
            password: "Short-1a".to_string(),
            ..valid_input()
        };
        assert!(validate_create_user(&input).is_err());
    }

    #[test]
    fn rejects_a_password_missing_a_symbol() {
        let input = CreateUserInput {
            password: "NoSymbolHere1".to_string(),
            ..valid_input()
        };
        assert!(validate_create_user(&input).is_err());
    }

    #[test]
    fn rejects_an_unknown_role() {
        let input = CreateUserInput {
            role: "superuser".to_string(),
            ..valid_input()
        };
        assert!(validate_create_user(&input).is_err());
    }

    #[test]
    fn rejects_an_empty_login_username() {
        let input = LoginInput {
            username: String::new(),
            password: "whatever".to_string(),
        };
        assert!(validate_login(&input).is_err());
    }

    #[test]
    fn accepts_a_well_formed_login_input() {
        let input = LoginInput {
            username: "ada.lovelace".to_string(),
            password: "whatever".to_string(),
        };
        assert!(validate_login(&input).is_ok());
    }
}
