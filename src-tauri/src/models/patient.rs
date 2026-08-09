//! `patients` domain model (Database.md Section 3.2). Root entity of the clinical workflow
//! (CLAUDE.md Section 7).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    pub id: i64,
    pub medical_record_number: String,
    pub full_name: String,
    pub date_of_birth: String,
    pub sex: String,
    pub national_id: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub blood_type: Option<String>,
    pub allergies: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_patient() -> Patient {
        Patient {
            id: 1,
            medical_record_number: "MRN-0001".to_string(),
            full_name: "Ada Lovelace".to_string(),
            date_of_birth: "1990-01-01".to_string(),
            sex: "female".to_string(),
            national_id: None,
            phone: None,
            address: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            blood_type: None,
            allergies: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
        }
    }

    #[test]
    fn serializes_field_names_as_camel_case() {
        let json = serde_json::to_string(&sample_patient()).expect("serializes");

        assert!(json.contains("\"medicalRecordNumber\""));
        assert!(json.contains("\"fullName\""));
        assert!(json.contains("\"dateOfBirth\""));
        assert!(json.contains("\"nationalId\""));
        assert!(json.contains("\"emergencyContactName\""));
        assert!(json.contains("\"emergencyContactPhone\""));
        assert!(json.contains("\"bloodType\""));
        assert!(json.contains("\"createdAt\""));
        assert!(json.contains("\"updatedAt\""));
    }
}
