//! Server-side authoritative validation for Medical History inputs (Validation.md), mirroring the
//! client Zod schemas at `modules/medical-history/types/medical-history-schemas.ts` per Rule
//! 17.4. These structs double as the Tauri command input DTOs.

use serde::Deserialize;
use validator::{Validate, ValidationError};

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

/// Backs `medical_history_create_treatment` (IPC.md Section 2.1). `inventory_item_id` and
/// `quantity` are both-or-neither and, when present, drive an `inventory_transactions`
/// consumption row inside the same DB transaction as the treatment insert
/// (`medical_history_service::create_treatment` calls
/// `inventory_service::record_transaction_core`) — insufficient stock rolls back the whole
/// treatment insert. FK-existence of `inventory_item_id` is a service-layer concern, mirroring
/// every other FK in this file.
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "validate_inventory_linkage"))]
pub struct CreateTreatmentInput {
    pub encounter_id: i64,
    pub diagnosis_id: Option<i64>,
    #[validate(length(min = 1, max = 2000))]
    pub description: String,
    #[validate(length(max = 200))]
    pub dosage: Option<String>,
    pub corrects_treatment_id: Option<i64>,
    pub inventory_item_id: Option<i64>,
    pub quantity: Option<i64>,
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

/// Enforces that `inventory_item_id`/`quantity` are supplied together (or not at all) and, when
/// present, that `quantity` is positive.
fn validate_inventory_linkage(input: &CreateTreatmentInput) -> Result<(), ValidationError> {
    match (input.inventory_item_id, input.quantity) {
        (None, None) => Ok(()),
        (Some(_), Some(quantity)) if quantity > 0 => Ok(()),
        (Some(_), Some(_)) => Err(ValidationError::new("quantity_must_be_positive")),
        _ => Err(ValidationError::new(
            "inventory_item_id_and_quantity_must_be_supplied_together",
        )),
    }
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
            inventory_item_id: None,
            quantity: None,
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
    fn accepts_a_treatment_input_with_a_valid_inventory_linkage() {
        let input = CreateTreatmentInput {
            inventory_item_id: Some(1),
            quantity: Some(2),
            ..valid_treatment_input()
        };
        assert!(validate_create_treatment(&input).is_ok());
    }

    #[test]
    fn rejects_an_inventory_item_id_without_a_quantity() {
        let input = CreateTreatmentInput {
            inventory_item_id: Some(1),
            quantity: None,
            ..valid_treatment_input()
        };
        assert!(validate_create_treatment(&input).is_err());
    }

    #[test]
    fn rejects_a_quantity_without_an_inventory_item_id() {
        let input = CreateTreatmentInput {
            inventory_item_id: None,
            quantity: Some(2),
            ..valid_treatment_input()
        };
        assert!(validate_create_treatment(&input).is_err());
    }

    #[test]
    fn rejects_a_non_positive_quantity() {
        let input = CreateTreatmentInput {
            inventory_item_id: Some(1),
            quantity: Some(0),
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
