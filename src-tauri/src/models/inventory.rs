//! `inventory_categories`/`inventory_items`/`inventory_transactions`/`maintenance_schedules`
//! domain models (Database.md Section 3.5). Inventory owns all four tables; Pharmacy and Lab are
//! consumers, not separate modules with their own schema.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryCategory {
    pub id: i64,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItem {
    pub id: i64,
    pub category_id: i64,
    pub name: String,
    pub quantity: i64,
    pub unit: String,
    pub reorder_threshold: i64,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// A single stock movement. `quantity_delta` is negative for consumption, positive for restock;
/// `inventory_items.quantity` is the maintained running total kept in sync with these rows
/// (Database.md Section 3.5, Rule 9.4).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryTransaction {
    pub id: i64,
    pub item_id: i64,
    pub quantity_delta: i64,
    pub reason: String,
    pub encounter_id: Option<i64>,
    pub treatment_id: Option<i64>,
    pub performed_by_user_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceSchedule {
    pub id: i64,
    pub inventory_item_id: i64,
    pub scheduled_date: String,
    pub completed_date: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn inventory_category_serializes_field_names_as_camel_case() {
        let category = InventoryCategory {
            id: 1,
            name: "Analgesics".to_string(),
            kind: "medicine".to_string(),
        };
        let json = serde_json::to_string(&category).expect("serializes");
        assert!(json.contains("\"kind\""));
    }

    #[test]
    fn inventory_item_serializes_field_names_as_camel_case() {
        let item = InventoryItem {
            id: 1,
            category_id: 2,
            name: "Ibuprofen 400mg".to_string(),
            quantity: 100,
            unit: "box".to_string(),
            reorder_threshold: 10,
            expiration_date: Some("2027-01-01".to_string()),
            location: Some("Pharmacy Shelf B2".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
        };
        let json = serde_json::to_string(&item).expect("serializes");
        assert!(json.contains("\"categoryId\""));
        assert!(json.contains("\"reorderThreshold\""));
        assert!(json.contains("\"expirationDate\""));
        assert!(json.contains("\"createdAt\""));
        assert!(json.contains("\"updatedAt\""));
    }

    #[test]
    fn inventory_transaction_serializes_field_names_as_camel_case() {
        let transaction = InventoryTransaction {
            id: 1,
            item_id: 2,
            quantity_delta: -5,
            reason: "consumption".to_string(),
            encounter_id: Some(3),
            treatment_id: Some(4),
            performed_by_user_id: 5,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&transaction).expect("serializes");
        assert!(json.contains("\"itemId\""));
        assert!(json.contains("\"quantityDelta\""));
        assert!(json.contains("\"encounterId\""));
        assert!(json.contains("\"treatmentId\""));
        assert!(json.contains("\"performedByUserId\""));
    }

    #[test]
    fn maintenance_schedule_serializes_field_names_as_camel_case() {
        let schedule = MaintenanceSchedule {
            id: 1,
            inventory_item_id: 2,
            scheduled_date: "2026-02-01".to_string(),
            completed_date: None,
            notes: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&schedule).expect("serializes");
        assert!(json.contains("\"inventoryItemId\""));
        assert!(json.contains("\"scheduledDate\""));
        assert!(json.contains("\"completedDate\""));
    }
}
