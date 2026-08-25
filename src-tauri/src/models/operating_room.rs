//! `operating_rooms`/`or_reservations` domain models (Database.md Section 3.4). Operating Rooms
//! owns both tables; `operating_rooms` promotes an existing `rooms` row owned by Beds.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatingRoom {
    pub id: i64,
    pub room_id: i64,
    pub name: String,
    pub created_at: String,
}

/// A scheduled use of an operating room for one patient encounter. `status` transitions
/// `scheduled -> in_progress -> completed`, or `scheduled/in_progress -> cancelled`; rows are
/// never deleted (Database.md Section 3.4).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrReservation {
    pub id: i64,
    pub operating_room_id: i64,
    pub patient_id: i64,
    pub encounter_id: i64,
    pub procedure_description: String,
    pub scheduled_start: String,
    pub scheduled_end: String,
    pub status: String,
    pub scheduled_by_user_id: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn operating_room_serializes_field_names_as_camel_case() {
        let operating_room = OperatingRoom {
            id: 1,
            room_id: 2,
            name: "OR 1".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&operating_room).expect("serializes");
        assert!(json.contains("\"roomId\""));
        assert!(json.contains("\"createdAt\""));
    }

    #[test]
    fn or_reservation_serializes_field_names_as_camel_case() {
        let reservation = OrReservation {
            id: 1,
            operating_room_id: 2,
            patient_id: 3,
            encounter_id: 4,
            procedure_description: "Appendectomy".to_string(),
            scheduled_start: "2026-01-01T08:00:00Z".to_string(),
            scheduled_end: "2026-01-01T10:00:00Z".to_string(),
            status: "scheduled".to_string(),
            scheduled_by_user_id: 5,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
        };
        let json = serde_json::to_string(&reservation).expect("serializes");
        assert!(json.contains("\"operatingRoomId\""));
        assert!(json.contains("\"patientId\""));
        assert!(json.contains("\"encounterId\""));
        assert!(json.contains("\"procedureDescription\""));
        assert!(json.contains("\"scheduledStart\""));
        assert!(json.contains("\"scheduledEnd\""));
        assert!(json.contains("\"scheduledByUserId\""));
        assert!(json.contains("\"updatedAt\""));
    }
}
