//! `billing_simulations`/`billing_items` domain models (Database.md Section 3.8). Billing owns
//! both tables; it is a simulation only — never a real payment record (CLAUDE.md Section 2).

use serde::Serialize;

/// One encounter's billing simulation. `status` transitions `draft -> finalized` only; rows are
/// never deleted. `total_amount` is a maintained-sum denormalization over `billing_items`
/// (Database.md Section 3.8 normalization notes).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingSimulation {
    pub id: i64,
    pub encounter_id: i64,
    pub status: String,
    pub total_amount: f64,
    pub generated_by_user_id: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// One itemized charge belonging to a `BillingSimulation`. `source` identifies which module's
/// data the charge was aggregated from; `source_entity_id` references the originating row
/// (e.g. a `bed_assignments.id`) when applicable.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingItem {
    pub id: i64,
    pub billing_simulation_id: i64,
    pub description: String,
    pub source: String,
    pub source_entity_id: Option<i64>,
    pub amount: f64,
    pub created_at: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn billing_simulation_serializes_field_names_as_camel_case() {
        let simulation = BillingSimulation {
            id: 1,
            encounter_id: 2,
            status: "draft".to_string(),
            total_amount: 150.0,
            generated_by_user_id: 3,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
        };
        let json = serde_json::to_string(&simulation).expect("serializes");
        assert!(json.contains("\"encounterId\""));
        assert!(json.contains("\"totalAmount\""));
        assert!(json.contains("\"generatedByUserId\""));
        assert!(json.contains("\"createdAt\""));
        assert!(json.contains("\"updatedAt\""));
    }

    #[test]
    fn billing_item_serializes_field_names_as_camel_case() {
        let item = BillingItem {
            id: 1,
            billing_simulation_id: 2,
            description: "Room charge".to_string(),
            source: "room".to_string(),
            source_entity_id: Some(3),
            amount: 150.0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&item).expect("serializes");
        assert!(json.contains("\"billingSimulationId\""));
        assert!(json.contains("\"sourceEntityId\""));
        assert!(json.contains("\"createdAt\""));
    }
}
