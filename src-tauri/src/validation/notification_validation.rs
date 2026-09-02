//! Server-side authoritative validation for Notifications inputs (Validation.md), mirroring the
//! client Zod schema at `modules/notifications/types/notification-schemas.ts` per Rule 17.4.

use serde::Deserialize;
use validator::Validate;

use crate::errors::AppError;
use crate::validation::map_validation_errors;

/// Backs `notifications_mark_read` (IPC.md). FK-existence (the notification actually exists) is
/// a service-layer concern, not checked here, mirroring every other module's convention.
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct MarkNotificationReadInput {
    #[validate(range(min = 1))]
    pub id: i64,
}

pub fn validate_mark_notification_read(input: &MarkNotificationReadInput) -> Result<(), AppError> {
    input.validate().map_err(map_validation_errors)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_positive_id() {
        assert!(validate_mark_notification_read(&MarkNotificationReadInput { id: 1 }).is_ok());
    }

    #[test]
    fn rejects_a_non_positive_id() {
        assert!(validate_mark_notification_read(&MarkNotificationReadInput { id: 0 }).is_err());
    }
}
