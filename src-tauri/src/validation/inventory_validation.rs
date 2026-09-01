//! Server-side authoritative validation for Inventory inputs (Validation.md), mirroring the
//! client Zod schemas at `modules/inventory/types/inventory-schemas.ts` per Rule 17.4. These
//! structs double as the Tauri command input DTOs.
//!
//! FK-existence (category / item exists) and the sufficient-stock business rule are not checked
//! here — those are service-layer concerns enforced inside a transaction
//! (`inventory_service.rs`), mirroring `bed_validation.rs`'s `AssignBedInput` note.

use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::errors::AppError;
use crate::validation::map_validation_errors;

const VALID_CATEGORY_KINDS: [&str; 3] = ["medicine", "supply", "equipment"];
const VALID_TRANSACTION_REASONS: [&str; 4] = ["restock", "consumption", "adjustment", "disposal"];

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateInventoryCategoryInput {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(custom(function = "validate_category_kind"))]
    pub kind: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateInventoryItemInput {
    #[validate(range(min = 1))]
    pub category_id: i64,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1, max = 20))]
    pub unit: String,
    #[validate(range(min = 0))]
    pub reorder_threshold: i64,
    pub expiration_date: Option<String>,
    #[validate(length(max = 200))]
    pub location: Option<String>,
}

/// Backs `inventory_record_transaction` (IPC.md Section 2.1). Sufficient-stock (rejecting a
/// consumption that would drive `quantity` negative) is a business rule checked in
/// `inventory_service::record_transaction_core`, not here.
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateInventoryTransactionInput {
    #[validate(range(min = 1))]
    pub item_id: i64,
    #[validate(custom(function = "validate_nonzero_delta"))]
    pub quantity_delta: i64,
    #[validate(custom(function = "validate_transaction_reason"))]
    pub reason: String,
    pub encounter_id: Option<i64>,
    pub treatment_id: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleMaintenanceInput {
    #[validate(range(min = 1))]
    pub inventory_item_id: i64,
    #[validate(length(min = 1))]
    pub scheduled_date: String,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

pub fn validate_create_inventory_category(
    input: &CreateInventoryCategoryInput,
) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_create_inventory_item(input: &CreateInventoryItemInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_create_inventory_transaction(
    input: &CreateInventoryTransactionInput,
) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_schedule_maintenance(input: &ScheduleMaintenanceInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

/// Database.md 3.5's `inventory_categories.kind` CHECK enum — validated here too for an
/// actionable error message instead of surfacing a raw SQLite constraint failure.
fn validate_category_kind(kind: &str) -> Result<(), ValidationError> {
    if VALID_CATEGORY_KINDS.contains(&kind) {
        Ok(())
    } else {
        Err(ValidationError::new("category_kind_invalid"))
    }
}

/// Database.md 3.5's `inventory_transactions.reason` CHECK enum.
fn validate_transaction_reason(reason: &str) -> Result<(), ValidationError> {
    if VALID_TRANSACTION_REASONS.contains(&reason) {
        Ok(())
    } else {
        Err(ValidationError::new("transaction_reason_invalid"))
    }
}

fn validate_nonzero_delta(quantity_delta: i64) -> Result<(), ValidationError> {
    if quantity_delta == 0 {
        Err(ValidationError::new("quantity_delta_must_not_be_zero"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_create_inventory_category() -> CreateInventoryCategoryInput {
        CreateInventoryCategoryInput {
            name: "Analgesics".to_string(),
            kind: "medicine".to_string(),
        }
    }

    fn valid_create_inventory_item() -> CreateInventoryItemInput {
        CreateInventoryItemInput {
            category_id: 1,
            name: "Ibuprofen 400mg".to_string(),
            unit: "box".to_string(),
            reorder_threshold: 10,
            expiration_date: Some("2027-01-01".to_string()),
            location: Some("Pharmacy Shelf B2".to_string()),
        }
    }

    fn valid_create_inventory_transaction() -> CreateInventoryTransactionInput {
        CreateInventoryTransactionInput {
            item_id: 1,
            quantity_delta: 100,
            reason: "restock".to_string(),
            encounter_id: None,
            treatment_id: None,
        }
    }

    fn valid_schedule_maintenance() -> ScheduleMaintenanceInput {
        ScheduleMaintenanceInput {
            inventory_item_id: 1,
            scheduled_date: "2026-02-01".to_string(),
            notes: Some("Routine calibration".to_string()),
        }
    }

    #[test]
    fn accepts_a_fully_valid_create_inventory_category_input() {
        assert!(validate_create_inventory_category(&valid_create_inventory_category()).is_ok());
    }

    #[test]
    fn rejects_an_empty_category_name() {
        let input = CreateInventoryCategoryInput {
            name: String::new(),
            ..valid_create_inventory_category()
        };
        assert!(validate_create_inventory_category(&input).is_err());
    }

    #[test]
    fn rejects_an_invalid_category_kind() {
        let input = CreateInventoryCategoryInput {
            kind: "furniture".to_string(),
            ..valid_create_inventory_category()
        };
        assert!(validate_create_inventory_category(&input).is_err());
    }

    #[test]
    fn accepts_a_fully_valid_create_inventory_item_input() {
        assert!(validate_create_inventory_item(&valid_create_inventory_item()).is_ok());
    }

    #[test]
    fn rejects_a_non_positive_category_id() {
        let input = CreateInventoryItemInput {
            category_id: 0,
            ..valid_create_inventory_item()
        };
        assert!(validate_create_inventory_item(&input).is_err());
    }

    #[test]
    fn rejects_a_negative_reorder_threshold() {
        let input = CreateInventoryItemInput {
            reorder_threshold: -1,
            ..valid_create_inventory_item()
        };
        assert!(validate_create_inventory_item(&input).is_err());
    }

    #[test]
    fn accepts_a_fully_valid_create_inventory_transaction_input() {
        assert!(
            validate_create_inventory_transaction(&valid_create_inventory_transaction()).is_ok()
        );
    }

    #[test]
    fn rejects_a_zero_quantity_delta() {
        let input = CreateInventoryTransactionInput {
            quantity_delta: 0,
            ..valid_create_inventory_transaction()
        };
        assert!(validate_create_inventory_transaction(&input).is_err());
    }

    #[test]
    fn rejects_an_invalid_transaction_reason() {
        let input = CreateInventoryTransactionInput {
            reason: "theft".to_string(),
            ..valid_create_inventory_transaction()
        };
        assert!(validate_create_inventory_transaction(&input).is_err());
    }

    #[test]
    fn accepts_a_fully_valid_schedule_maintenance_input() {
        assert!(validate_schedule_maintenance(&valid_schedule_maintenance()).is_ok());
    }

    #[test]
    fn rejects_an_empty_scheduled_date() {
        let input = ScheduleMaintenanceInput {
            scheduled_date: String::new(),
            ..valid_schedule_maintenance()
        };
        assert!(validate_schedule_maintenance(&input).is_err());
    }
}
