//! Server-side authoritative validation for Medical History inputs (Validation.md), mirroring the
//! client Zod schemas at `modules/medical-history/types/medical-history-schemas.ts` per Rule
//! 17.4. These structs double as the Tauri command input DTOs.

use serde::Deserialize;
use validator::Validate;

use crate::errors::AppError;
use crate::validation::map_validation_errors;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateEncounterInput {
    pub patient_id: i64,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DischargeEncounterInput {
    pub encounter_id: i64,
    #[validate(length(max = 4000))]
    pub discharge_summary: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetByPatientInput {
    pub patient_id: i64,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiagnosisInput {
    pub encounter_id: i64,
    #[validate(length(min = 1, max = 2000))]
    pub description: String,
    #[validate(length(max = 20))]
    pub icd_code: Option<String>,
    pub corrects_diagnosis_id: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateTreatmentInput {
    pub encounter_id: i64,
    pub diagnosis_id: Option<i64>,
    #[validate(length(min = 1, max = 2000))]
    pub description: String,
    #[validate(length(max = 200))]
    pub dosage: Option<String>,
    pub corrects_treatment_id: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateEvolutionInput {
    pub encounter_id: i64,
    #[validate(length(min = 1, max = 4000))]
    pub note: String,
}

pub fn validate_create_encounter(input: &CreateEncounterInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_discharge_encounter(input: &DischargeEncounterInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_create_diagnosis(input: &CreateDiagnosisInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_create_treatment(input: &CreateTreatmentInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_create_evolution(input: &CreateEvolutionInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_valid_create_encounter_input() {
        assert!(validate_create_encounter(&CreateEncounterInput { patient_id: 1 }).is_ok());
    }

    #[test]
    fn accepts_a_valid_discharge_input_with_no_summary() {
        let input = DischargeEncounterInput {
            encounter_id: 1,
            discharge_summary: None,
        };
        assert!(validate_discharge_encounter(&input).is_ok());
    }

    #[test]
    fn rejects_a_discharge_summary_over_four_thousand_characters() {
        let input = DischargeEncounterInput {
            encounter_id: 1,
            discharge_summary: Some("a".repeat(4001)),
        };
        assert!(validate_discharge_encounter(&input).is_err());
    }

    fn valid_diagnosis_input() -> CreateDiagnosisInput {
        CreateDiagnosisInput {
            encounter_id: 1,
            description: "Acute appendicitis".to_string(),
            icd_code: Some("K35.80".to_string()),
            corrects_diagnosis_id: None,
        }
    }

    #[test]
    fn accepts_a_fully_valid_create_diagnosis_input() {
        assert!(validate_create_diagnosis(&valid_diagnosis_input()).is_ok());
    }

    #[test]
    fn rejects_an_empty_diagnosis_description() {
        let input = CreateDiagnosisInput {
            description: String::new(),
            ..valid_diagnosis_input()
        };
        assert!(validate_create_diagnosis(&input).is_err());
    }

    fn valid_treatment_input() -> CreateTreatmentInput {
        CreateTreatmentInput {
            encounter_id: 1,
            diagnosis_id: Some(1),
            description: "Appendectomy".to_string(),
            dosage: None,
            corrects_treatment_id: None,
        }
    }

    #[test]
    fn accepts_a_fully_valid_create_treatment_input() {
        assert!(validate_create_treatment(&valid_treatment_input()).is_ok());
    }

    #[test]
    fn rejects_an_empty_treatment_description() {
        let input = CreateTreatmentInput {
            description: String::new(),
            ..valid_treatment_input()
        };
        assert!(validate_create_treatment(&input).is_err());
    }

    #[test]
    fn accepts_a_valid_create_evolution_input() {
        let input = CreateEvolutionInput {
            encounter_id: 1,
            note: "Patient stable post-op.".to_string(),
        };
        assert!(validate_create_evolution(&input).is_ok());
    }

    #[test]
    fn rejects_an_empty_evolution_note() {
        let input = CreateEvolutionInput {
            encounter_id: 1,
            note: String::new(),
        };
        assert!(validate_create_evolution(&input).is_err());
    }
}
