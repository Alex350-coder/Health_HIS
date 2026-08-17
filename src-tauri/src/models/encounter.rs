//! Medical History domain models (Database.md Section 3.2): one encounter per hospital
//! stay/episode, with append-only diagnoses/treatments/evolutions attached to it (Rule 9.5 —
//! corrections are new rows referencing the row they correct via `corrects_*_id`, never an
//! UPDATE/DELETE).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub id: i64,
    pub patient_id: i64,
    pub status: String,
    pub admitted_at: String,
    pub discharged_at: Option<String>,
    pub discharge_summary: Option<String>,
    pub created_by_user_id: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnosis {
    pub id: i64,
    pub encounter_id: i64,
    pub description: String,
    pub icd_code: Option<String>,
    pub registered_by_user_id: i64,
    pub corrects_diagnosis_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Treatment {
    pub id: i64,
    pub encounter_id: i64,
    pub diagnosis_id: Option<i64>,
    pub description: String,
    pub dosage: Option<String>,
    pub registered_by_user_id: i64,
    pub corrects_treatment_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evolution {
    pub id: i64,
    pub encounter_id: i64,
    pub note: String,
    pub registered_by_user_id: i64,
    pub created_at: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_encounter() -> Encounter {
        Encounter {
            id: 1,
            patient_id: 1,
            status: "open".to_string(),
            admitted_at: "2026-01-01T00:00:00Z".to_string(),
            discharged_at: None,
            discharge_summary: None,
            created_by_user_id: 1,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
        }
    }

    fn sample_diagnosis() -> Diagnosis {
        Diagnosis {
            id: 1,
            encounter_id: 1,
            description: "Acute appendicitis".to_string(),
            icd_code: Some("K35.80".to_string()),
            registered_by_user_id: 1,
            corrects_diagnosis_id: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn sample_treatment() -> Treatment {
        Treatment {
            id: 1,
            encounter_id: 1,
            diagnosis_id: Some(1),
            description: "Appendectomy".to_string(),
            dosage: None,
            registered_by_user_id: 1,
            corrects_treatment_id: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn sample_evolution() -> Evolution {
        Evolution {
            id: 1,
            encounter_id: 1,
            note: "Patient stable post-op.".to_string(),
            registered_by_user_id: 1,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn encounter_serializes_field_names_as_camel_case() {
        let json = serde_json::to_string(&sample_encounter()).expect("serializes");

        assert!(json.contains("\"patientId\""));
        assert!(json.contains("\"admittedAt\""));
        assert!(json.contains("\"dischargedAt\""));
        assert!(json.contains("\"dischargeSummary\""));
        assert!(json.contains("\"createdByUserId\""));
        assert!(json.contains("\"createdAt\""));
        assert!(json.contains("\"updatedAt\""));
    }

    #[test]
    fn diagnosis_serializes_field_names_as_camel_case() {
        let json = serde_json::to_string(&sample_diagnosis()).expect("serializes");

        assert!(json.contains("\"encounterId\""));
        assert!(json.contains("\"icdCode\""));
        assert!(json.contains("\"registeredByUserId\""));
        assert!(json.contains("\"correctsDiagnosisId\""));
    }

    #[test]
    fn treatment_serializes_field_names_as_camel_case() {
        let json = serde_json::to_string(&sample_treatment()).expect("serializes");

        assert!(json.contains("\"encounterId\""));
        assert!(json.contains("\"diagnosisId\""));
        assert!(json.contains("\"correctsTreatmentId\""));
    }

    #[test]
    fn evolution_serializes_field_names_as_camel_case() {
        let json = serde_json::to_string(&sample_evolution()).expect("serializes");

        assert!(json.contains("\"encounterId\""));
        assert!(json.contains("\"registeredByUserId\""));
        assert!(json.contains("\"createdAt\""));
    }
}
