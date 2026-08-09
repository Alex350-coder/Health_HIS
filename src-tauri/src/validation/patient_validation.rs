//! Server-side authoritative validation for patient inputs (Validation.md), mirroring the client
//! Zod schemas at `modules/patients/types/patient-schemas.ts` per Rule 17.4. These structs double
//! as the Tauri command input DTOs.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::errors::AppError;
use crate::validation::map_validation_errors;

const VALID_SEXES: [&str; 4] = ["male", "female", "other", "unknown"];

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreatePatientInput {
    #[validate(length(min = 1))]
    pub medical_record_number: String,
    #[validate(length(min = 1, max = 200))]
    pub full_name: String,
    #[validate(custom(function = "validate_date_of_birth"))]
    pub date_of_birth: String,
    #[validate(custom(function = "validate_sex"))]
    pub sex: String,
    pub national_id: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub blood_type: Option<String>,
    pub allergies: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePatientInput {
    pub id: i64,
    #[validate(length(min = 1))]
    pub medical_record_number: String,
    #[validate(length(min = 1, max = 200))]
    pub full_name: String,
    #[validate(custom(function = "validate_date_of_birth"))]
    pub date_of_birth: String,
    #[validate(custom(function = "validate_sex"))]
    pub sex: String,
    pub national_id: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub blood_type: Option<String>,
    pub allergies: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetPatientInput {
    pub id: i64,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListPatientsInput {
    pub search: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

pub fn validate_create_patient(input: &CreatePatientInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_update_patient(input: &UpdatePatientInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

/// Database.md 3.2's `sex` CHECK enum — validated here too for an actionable error message
/// instead of surfacing a raw SQLite constraint failure.
fn validate_sex(sex: &str) -> Result<(), ValidationError> {
    if VALID_SEXES.contains(&sex) {
        Ok(())
    } else {
        Err(ValidationError::new("sex_invalid"))
    }
}

/// Rejects malformed dates and dates in the future. Compared lexically against today's ISO-8601
/// date string — sound because `YYYY-MM-DD` sorts identically as text and as a calendar date.
fn validate_date_of_birth(date_of_birth: &str) -> Result<(), ValidationError> {
    if !is_well_formed_iso_date(date_of_birth) {
        return Err(ValidationError::new("date_of_birth_format"));
    }
    if date_of_birth > today_iso_date().as_str() {
        return Err(ValidationError::new("date_of_birth_future"));
    }
    Ok(())
}

fn is_well_formed_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

/// Today's date as `YYYY-MM-DD`, derived from `SystemTime` with no date/time crate dependency
/// (Decision 5). Uses Howard Hinnant's `civil_from_days` algorithm for the Gregorian conversion.
fn today_iso_date() -> String {
    let days_since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    let (year, month, day) = civil_from_days(days_since_epoch);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if month <= 2 { y + 1 } else { y };
    (year, month, day)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn valid_input() -> CreatePatientInput {
        CreatePatientInput {
            medical_record_number: "MRN-0001".to_string(),
            full_name: "Ada Lovelace".to_string(),
            date_of_birth: "1990-01-01".to_string(),
            sex: "female".to_string(),
            national_id: None,
            phone: None,
            address: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            blood_type: None,
            allergies: None,
        }
    }

    #[test]
    fn accepts_a_fully_valid_create_patient_input() {
        assert!(validate_create_patient(&valid_input()).is_ok());
    }

    #[test]
    fn rejects_an_empty_full_name() {
        let input = CreatePatientInput {
            full_name: String::new(),
            ..valid_input()
        };
        assert!(validate_create_patient(&input).is_err());
    }

    #[test]
    fn rejects_a_full_name_over_two_hundred_characters() {
        let input = CreatePatientInput {
            full_name: "a".repeat(201),
            ..valid_input()
        };
        assert!(validate_create_patient(&input).is_err());
    }

    #[test]
    fn rejects_an_empty_medical_record_number() {
        let input = CreatePatientInput {
            medical_record_number: String::new(),
            ..valid_input()
        };
        assert!(validate_create_patient(&input).is_err());
    }

    #[test]
    fn rejects_an_invalid_sex_value() {
        let input = CreatePatientInput {
            sex: "unspecified".to_string(),
            ..valid_input()
        };
        assert!(validate_create_patient(&input).is_err());
    }

    #[test]
    fn rejects_a_malformed_date_of_birth() {
        let input = CreatePatientInput {
            date_of_birth: "01-01-1990".to_string(),
            ..valid_input()
        };
        assert!(validate_create_patient(&input).is_err());
    }

    #[test]
    fn rejects_a_date_of_birth_in_the_future() {
        let input = CreatePatientInput {
            date_of_birth: "2999-01-01".to_string(),
            ..valid_input()
        };
        assert!(validate_create_patient(&input).is_err());
    }

    #[test]
    fn accepts_todays_date_of_birth() {
        let input = CreatePatientInput {
            date_of_birth: today_iso_date(),
            ..valid_input()
        };
        assert!(validate_create_patient(&input).is_ok());
    }
}
