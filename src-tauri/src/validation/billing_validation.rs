//! Server-side authoritative validation for Billing inputs (Validation.md), mirroring the
//! client Zod schema at `modules/billing/types/billing-schemas.ts` per Rule 17.4.

use serde::Deserialize;
use validator::Validate;

use crate::errors::AppError;
use crate::validation::map_validation_errors;

/// Backs `billing_generate_simulation` (IPC.md). FK-existence (the encounter actually exists,
/// and has no simulation yet) is a service-layer concern, not checked here, mirroring every
/// other module's convention.
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GenerateBillingSimulationInput {
    #[validate(range(min = 1))]
    pub encounter_id: i64,
}

pub fn validate_generate_billing_simulation(
    input: &GenerateBillingSimulationInput,
) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

/// Backs `billing_finalize_simulation` (IPC.md).
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct FinalizeBillingSimulationInput {
    #[validate(range(min = 1))]
    pub id: i64,
}

pub fn validate_finalize_billing_simulation(
    input: &FinalizeBillingSimulationInput,
) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_positive_encounter_id() {
        assert!(
            validate_generate_billing_simulation(&GenerateBillingSimulationInput {
                encounter_id: 1
            })
            .is_ok()
        );
    }

    #[test]
    fn rejects_a_non_positive_encounter_id() {
        assert!(
            validate_generate_billing_simulation(&GenerateBillingSimulationInput {
                encounter_id: 0
            })
            .is_err()
        );
    }

    #[test]
    fn accepts_a_positive_simulation_id() {
        assert!(
            validate_finalize_billing_simulation(&FinalizeBillingSimulationInput { id: 1 }).is_ok()
        );
    }

    #[test]
    fn rejects_a_non_positive_simulation_id() {
        assert!(
            validate_finalize_billing_simulation(&FinalizeBillingSimulationInput { id: 0 })
                .is_err()
        );
    }
}
