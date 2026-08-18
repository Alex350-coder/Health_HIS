//! `floors`/`rooms`/`beds` domain models (Database.md Section 3.3). Hospital Map reads these
//! read-only; Beds owns the facility-configuration write path (Architecture.md Section 3).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Floor {
    pub id: i64,
    pub name: String,
    pub level_order: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub id: i64,
    pub floor_id: i64,
    pub name: String,
    pub room_type: String,
    pub map_x: f64,
    pub map_y: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bed {
    pub id: i64,
    pub room_id: i64,
    pub label: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// A floor with its rooms nested — the shape `hospital_map_get_layout` returns (IPC.md Section 2).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FloorLayout {
    pub id: i64,
    pub name: String,
    pub level_order: i64,
    pub rooms: Vec<Room>,
}

/// A room's occupancy summary — the shape `hospital_map_get_room_status` returns.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomStatus {
    pub room: Room,
    pub beds: Vec<Bed>,
    pub available_count: i64,
    pub occupied_count: i64,
    pub maintenance_count: i64,
}

/// A patient's occupancy of a bed for the duration of one encounter (Database.md Section 3.3).
/// Append-only in spirit: a release sets `released_at` rather than deleting the row.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BedAssignment {
    pub id: i64,
    pub bed_id: i64,
    pub patient_id: i64,
    pub encounter_id: i64,
    pub assigned_at: String,
    pub released_at: Option<String>,
}

/// The minimal linkage `beds_list` exposes for a bed's current occupant, without pulling in
/// clinical content (Rule 12.1 — no PHI beyond entity IDs).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAssignmentRef {
    pub id: i64,
    pub patient_id: i64,
    pub encounter_id: i64,
}

/// Backs `beds_list` (IPC.md Section 2.1) — every bed plus its active assignment, if any.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BedSummary {
    #[serde(flatten)]
    pub bed: Bed,
    pub active_assignment: Option<ActiveAssignmentRef>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn floor_serializes_field_names_as_camel_case() {
        let floor = Floor {
            id: 1,
            name: "Ground Floor".to_string(),
            level_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&floor).expect("serializes");
        assert!(json.contains("\"levelOrder\""));
        assert!(json.contains("\"createdAt\""));
    }

    #[test]
    fn room_serializes_field_names_as_camel_case() {
        let room = Room {
            id: 1,
            floor_id: 1,
            name: "Ward A".to_string(),
            room_type: "ward".to_string(),
            map_x: 0.5,
            map_y: 0.25,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&room).expect("serializes");
        assert!(json.contains("\"floorId\""));
        assert!(json.contains("\"roomType\""));
        assert!(json.contains("\"mapX\""));
        assert!(json.contains("\"mapY\""));
    }

    #[test]
    fn bed_serializes_field_names_as_camel_case() {
        let bed = Bed {
            id: 1,
            room_id: 1,
            label: "Bed 3A".to_string(),
            status: "available".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
        };
        let json = serde_json::to_string(&bed).expect("serializes");
        assert!(json.contains("\"roomId\""));
        assert!(json.contains("\"updatedAt\""));
    }

    #[test]
    fn bed_assignment_serializes_field_names_as_camel_case() {
        let assignment = BedAssignment {
            id: 1,
            bed_id: 2,
            patient_id: 3,
            encounter_id: 4,
            assigned_at: "2026-01-01T00:00:00Z".to_string(),
            released_at: None,
        };
        let json = serde_json::to_string(&assignment).expect("serializes");
        assert!(json.contains("\"bedId\""));
        assert!(json.contains("\"patientId\""));
        assert!(json.contains("\"encounterId\""));
        assert!(json.contains("\"releasedAt\""));
    }

    #[test]
    fn bed_summary_flattens_the_bed_fields_alongside_active_assignment() {
        let summary = BedSummary {
            bed: Bed {
                id: 1,
                room_id: 1,
                label: "Bed 3A".to_string(),
                status: "occupied".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: None,
            },
            active_assignment: Some(ActiveAssignmentRef {
                id: 9,
                patient_id: 3,
                encounter_id: 4,
            }),
        };
        let json = serde_json::to_string(&summary).expect("serializes");
        assert!(json.contains("\"label\":\"Bed 3A\""));
        assert!(json.contains("\"activeAssignment\""));
        assert!(json.contains("\"patientId\":3"));
    }
}
