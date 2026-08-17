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
}
