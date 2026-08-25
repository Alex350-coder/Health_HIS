//! Server-side authoritative validation for Operating Rooms inputs (Validation.md), mirroring
//! the client Zod schemas at `modules/operating-rooms/types/or-schemas.ts` per Rule 17.4. These
//! structs double as the Tauri command input DTOs.
//!
//! FK-existence (operating room / patient / encounter) and the overlap-prevention business rule
//! are not checked here — those are service-layer concerns enforced inside a transaction
//! (`operating_room_service.rs`), mirroring `bed_validation.rs`'s `AssignBedInput` note.

use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::errors::AppError;
use crate::validation::map_validation_errors;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateOperatingRoomInput {
    #[validate(range(min = 1))]
    pub room_id: i64,
}

/// Backs `operating_rooms_reserve` (IPC.md Section 2.1). The struct-level custom validator
/// enforces `scheduled_start < scheduled_end`; overlap with other reservations is a business
/// rule checked in `operating_room_service::reserve`.
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "validate_scheduled_range"))]
pub struct CreateOrReservationInput {
    #[validate(range(min = 1))]
    pub operating_room_id: i64,
    #[validate(range(min = 1))]
    pub patient_id: i64,
    #[validate(range(min = 1))]
    pub encounter_id: i64,
    #[validate(length(min = 1, max = 2000))]
    pub procedure_description: String,
    #[validate(length(min = 1))]
    pub scheduled_start: String,
    #[validate(length(min = 1))]
    pub scheduled_end: String,
}

/// Backs `operating_rooms_update_reservation` (IPC.md Section 2.1).
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "validate_update_scheduled_range"))]
pub struct UpdateOrReservationInput {
    #[validate(range(min = 1))]
    pub id: i64,
    #[validate(length(min = 1, max = 2000))]
    pub procedure_description: String,
    #[validate(length(min = 1))]
    pub scheduled_start: String,
    #[validate(length(min = 1))]
    pub scheduled_end: String,
}

/// Backs `operating_rooms_cancel_reservation` (IPC.md Section 2.1).
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrReservationInput {
    #[validate(range(min = 1))]
    pub id: i64,
}

pub fn validate_create_operating_room(input: &CreateOperatingRoomInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_create_or_reservation(input: &CreateOrReservationInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_update_or_reservation(input: &UpdateOrReservationInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_cancel_or_reservation(input: &CancelOrReservationInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

fn validate_scheduled_range(input: &CreateOrReservationInput) -> Result<(), ValidationError> {
    if input.scheduled_start < input.scheduled_end {
        Ok(())
    } else {
        Err(ValidationError::new("scheduled_end_must_be_after_start"))
    }
}

fn validate_update_scheduled_range(
    input: &UpdateOrReservationInput,
) -> Result<(), ValidationError> {
    if input.scheduled_start < input.scheduled_end {
        Ok(())
    } else {
        Err(ValidationError::new("scheduled_end_must_be_after_start"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn valid_create_operating_room() -> CreateOperatingRoomInput {
        CreateOperatingRoomInput { room_id: 1 }
    }

    fn valid_create_or_reservation() -> CreateOrReservationInput {
        CreateOrReservationInput {
            operating_room_id: 1,
            patient_id: 1,
            encounter_id: 1,
            procedure_description: "Appendectomy".to_string(),
            scheduled_start: "2026-01-01T08:00:00".to_string(),
            scheduled_end: "2026-01-01T10:00:00".to_string(),
        }
    }

    fn valid_update_or_reservation() -> UpdateOrReservationInput {
        UpdateOrReservationInput {
            id: 1,
            procedure_description: "Appendectomy".to_string(),
            scheduled_start: "2026-01-01T08:00:00".to_string(),
            scheduled_end: "2026-01-01T10:00:00".to_string(),
        }
    }

    #[test]
    fn accepts_a_fully_valid_create_operating_room_input() {
        assert!(validate_create_operating_room(&valid_create_operating_room()).is_ok());
    }

    #[test]
    fn rejects_a_non_positive_room_id() {
        let input = CreateOperatingRoomInput { room_id: 0 };
        assert!(validate_create_operating_room(&input).is_err());
    }

    #[test]
    fn accepts_a_fully_valid_create_or_reservation_input() {
        assert!(validate_create_or_reservation(&valid_create_or_reservation()).is_ok());
    }

    #[test]
    fn rejects_an_empty_procedure_description() {
        let input = CreateOrReservationInput {
            procedure_description: String::new(),
            ..valid_create_or_reservation()
        };
        assert!(validate_create_or_reservation(&input).is_err());
    }

    #[test]
    fn rejects_a_scheduled_end_before_start() {
        let input = CreateOrReservationInput {
            scheduled_start: "2026-01-01T10:00:00".to_string(),
            scheduled_end: "2026-01-01T08:00:00".to_string(),
            ..valid_create_or_reservation()
        };
        assert!(validate_create_or_reservation(&input).is_err());
    }

    #[test]
    fn rejects_a_scheduled_end_equal_to_start() {
        let input = CreateOrReservationInput {
            scheduled_start: "2026-01-01T08:00:00".to_string(),
            scheduled_end: "2026-01-01T08:00:00".to_string(),
            ..valid_create_or_reservation()
        };
        assert!(validate_create_or_reservation(&input).is_err());
    }

    #[test]
    fn accepts_a_fully_valid_update_or_reservation_input() {
        assert!(validate_update_or_reservation(&valid_update_or_reservation()).is_ok());
    }

    #[test]
    fn rejects_an_update_with_end_before_start() {
        let input = UpdateOrReservationInput {
            scheduled_start: "2026-01-01T10:00:00".to_string(),
            scheduled_end: "2026-01-01T08:00:00".to_string(),
            ..valid_update_or_reservation()
        };
        assert!(validate_update_or_reservation(&input).is_err());
    }

    #[test]
    fn accepts_a_valid_cancel_or_reservation_input() {
        assert!(validate_cancel_or_reservation(&CancelOrReservationInput { id: 1 }).is_ok());
    }

    #[test]
    fn rejects_a_non_positive_cancel_id() {
        assert!(validate_cancel_or_reservation(&CancelOrReservationInput { id: 0 }).is_err());
    }
}
