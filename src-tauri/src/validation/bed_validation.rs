//! Server-side authoritative validation for Beds facility-configuration inputs (Validation.md),
//! mirroring the client Zod schemas at `modules/beds/types/bed-schemas.ts` per Rule 17.4. These
//! structs double as the Tauri command input DTOs.

use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::errors::AppError;
use crate::validation::map_validation_errors;

const VALID_ROOM_TYPES: [&str; 6] = [
    "ward",
    "operating_room",
    "pharmacy",
    "laboratory",
    "admin",
    "other",
];

/// `beds_set_status` is limited to `available`/`maintenance` — occupancy is set only by the
/// Phase 7 assign/release business rule, never by a free-form status write (IPC.md Section 2).
const SETTABLE_BED_STATUSES: [&str; 2] = ["available", "maintenance"];

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFloorInput {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(range(min = 0))]
    pub level_order: i64,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomInput {
    pub floor_id: i64,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(custom(function = "validate_room_type"))]
    pub room_type: String,
    #[validate(range(min = 0.0, max = 1.0))]
    pub map_x: f64,
    #[validate(range(min = 0.0, max = 1.0))]
    pub map_y: f64,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateBedInput {
    pub room_id: i64,
    #[validate(length(min = 1, max = 50))]
    pub label: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SetBedStatusInput {
    pub bed_id: i64,
    #[validate(custom(function = "validate_settable_status"))]
    pub status: String,
}

pub fn validate_create_floor(input: &CreateFloorInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_create_room(input: &CreateRoomInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_create_bed(input: &CreateBedInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

pub fn validate_set_bed_status(input: &SetBedStatusInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

/// Database.md 3.3's `room_type` CHECK enum — validated here too for an actionable error message
/// instead of surfacing a raw SQLite constraint failure.
fn validate_room_type(room_type: &str) -> Result<(), ValidationError> {
    if VALID_ROOM_TYPES.contains(&room_type) {
        Ok(())
    } else {
        Err(ValidationError::new("room_type_invalid"))
    }
}

fn validate_settable_status(status: &str) -> Result<(), ValidationError> {
    if SETTABLE_BED_STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(ValidationError::new("bed_status_not_settable"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn valid_create_floor() -> CreateFloorInput {
        CreateFloorInput {
            name: "Ground Floor".to_string(),
            level_order: 0,
        }
    }

    fn valid_create_room() -> CreateRoomInput {
        CreateRoomInput {
            floor_id: 1,
            name: "Ward A".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.5,
            map_y: 0.5,
        }
    }

    fn valid_create_bed() -> CreateBedInput {
        CreateBedInput {
            room_id: 1,
            label: "Bed 3A".to_string(),
        }
    }

    fn valid_set_bed_status() -> SetBedStatusInput {
        SetBedStatusInput {
            bed_id: 1,
            status: "maintenance".to_string(),
        }
    }

    #[test]
    fn accepts_a_fully_valid_create_floor_input() {
        assert!(validate_create_floor(&valid_create_floor()).is_ok());
    }

    #[test]
    fn rejects_an_empty_floor_name() {
        let input = CreateFloorInput {
            name: String::new(),
            ..valid_create_floor()
        };
        assert!(validate_create_floor(&input).is_err());
    }

    #[test]
    fn accepts_a_fully_valid_create_room_input() {
        assert!(validate_create_room(&valid_create_room()).is_ok());
    }

    #[test]
    fn rejects_an_invalid_room_type() {
        let input = CreateRoomInput {
            room_type: "broom-closet".to_string(),
            ..valid_create_room()
        };
        assert!(validate_create_room(&input).is_err());
    }

    #[test]
    fn rejects_a_map_x_above_one() {
        let input = CreateRoomInput {
            map_x: 1.5,
            ..valid_create_room()
        };
        assert!(validate_create_room(&input).is_err());
    }

    #[test]
    fn rejects_a_map_y_below_zero() {
        let input = CreateRoomInput {
            map_y: -0.1,
            ..valid_create_room()
        };
        assert!(validate_create_room(&input).is_err());
    }

    #[test]
    fn accepts_a_fully_valid_create_bed_input() {
        assert!(validate_create_bed(&valid_create_bed()).is_ok());
    }

    #[test]
    fn rejects_an_empty_bed_label() {
        let input = CreateBedInput {
            label: String::new(),
            ..valid_create_bed()
        };
        assert!(validate_create_bed(&input).is_err());
    }

    #[test]
    fn accepts_a_settable_bed_status() {
        assert!(validate_set_bed_status(&valid_set_bed_status()).is_ok());
    }

    #[test]
    fn rejects_occupied_as_a_directly_settable_status() {
        let input = SetBedStatusInput {
            status: "occupied".to_string(),
            ..valid_set_bed_status()
        };
        assert!(validate_set_bed_status(&input).is_err());
    }

    #[test]
    fn rejects_an_unknown_bed_status() {
        let input = SetBedStatusInput {
            status: "on-fire".to_string(),
            ..valid_set_bed_status()
        };
        assert!(validate_set_bed_status(&input).is_err());
    }
}
