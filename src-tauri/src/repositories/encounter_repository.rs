//! Pure SQL data access for `encounters`, `diagnoses`, `treatments`, `evolutions`
//! (Architecture.md — repositories hold no business rules and make no cross-repository calls).
//! Callers pass either a bare `Connection` or an open `Transaction` (deref-compatible), so
//! `medical_history_service` can run a write + audit record in one transaction.
//!
//! No update/delete functions exist for `diagnoses`, `treatments`, or `evolutions` — append-only
//! (Rule 9.5) is enforced structurally by the absence of the function, the same pattern already
//! used for `audit_repository`. A correction is a new row referencing the row it corrects.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::DbError;
use crate::models::{Diagnosis, Encounter, Evolution, Treatment};

pub struct NewEncounter {
    pub patient_id: i64,
    pub created_by_user_id: i64,
}

pub struct DischargeEncounter<'a> {
    pub discharge_summary: Option<&'a str>,
}

pub struct NewDiagnosis<'a> {
    pub encounter_id: i64,
    pub description: &'a str,
    pub icd_code: Option<&'a str>,
    pub registered_by_user_id: i64,
    pub corrects_diagnosis_id: Option<i64>,
}

pub struct NewTreatment<'a> {
    pub encounter_id: i64,
    pub diagnosis_id: Option<i64>,
    pub description: &'a str,
    pub dosage: Option<&'a str>,
    pub registered_by_user_id: i64,
    pub corrects_treatment_id: Option<i64>,
}

pub struct NewEvolution<'a> {
    pub encounter_id: i64,
    pub note: &'a str,
    pub registered_by_user_id: i64,
}

const ENCOUNTER_COLUMNS: &str = "id, patient_id, status, admitted_at, discharged_at, \
     discharge_summary, created_by_user_id, created_at, updated_at";
const DIAGNOSIS_COLUMNS: &str = "id, encounter_id, description, icd_code, \
     registered_by_user_id, corrects_diagnosis_id, created_at";
const TREATMENT_COLUMNS: &str = "id, encounter_id, diagnosis_id, description, dosage, \
     registered_by_user_id, corrects_treatment_id, created_at";
const EVOLUTION_COLUMNS: &str = "id, encounter_id, note, registered_by_user_id, created_at";

pub fn insert_encounter(conn: &Connection, new_encounter: &NewEncounter) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO encounters (patient_id, created_by_user_id) VALUES (?1, ?2)",
        params![new_encounter.patient_id, new_encounter.created_by_user_id],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn discharge_encounter(
    conn: &Connection,
    id: i64,
    patch: &DischargeEncounter,
) -> Result<Option<Encounter>, DbError> {
    conn.execute(
        "UPDATE encounters SET status = 'discharged', discharged_at = datetime('now'), \
         discharge_summary = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![patch.discharge_summary, id],
    )?;
    find_encounter_by_id(conn, id)
}

pub fn find_encounter_by_id(conn: &Connection, id: i64) -> Result<Option<Encounter>, DbError> {
    conn.query_row(
        &format!("SELECT {ENCOUNTER_COLUMNS} FROM encounters WHERE id = ?1"),
        params![id],
        map_row_to_encounter,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_encounters_by_patient(
    conn: &Connection,
    patient_id: i64,
) -> Result<Vec<Encounter>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ENCOUNTER_COLUMNS} FROM encounters WHERE patient_id = ?1 ORDER BY admitted_at"
    ))?;
    let rows = stmt.query_map(params![patient_id], map_row_to_encounter)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn insert_diagnosis(conn: &Connection, new_diagnosis: &NewDiagnosis) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO diagnoses \
         (encounter_id, description, icd_code, registered_by_user_id, corrects_diagnosis_id) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            new_diagnosis.encounter_id,
            new_diagnosis.description,
            new_diagnosis.icd_code,
            new_diagnosis.registered_by_user_id,
            new_diagnosis.corrects_diagnosis_id,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_diagnosis_by_id(conn: &Connection, id: i64) -> Result<Option<Diagnosis>, DbError> {
    conn.query_row(
        &format!("SELECT {DIAGNOSIS_COLUMNS} FROM diagnoses WHERE id = ?1"),
        params![id],
        map_row_to_diagnosis,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_diagnoses_by_encounter(
    conn: &Connection,
    encounter_id: i64,
) -> Result<Vec<Diagnosis>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {DIAGNOSIS_COLUMNS} FROM diagnoses WHERE encounter_id = ?1 ORDER BY created_at"
    ))?;
    let rows = stmt.query_map(params![encounter_id], map_row_to_diagnosis)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn insert_treatment(conn: &Connection, new_treatment: &NewTreatment) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO treatments \
         (encounter_id, diagnosis_id, description, dosage, registered_by_user_id, \
          corrects_treatment_id) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            new_treatment.encounter_id,
            new_treatment.diagnosis_id,
            new_treatment.description,
            new_treatment.dosage,
            new_treatment.registered_by_user_id,
            new_treatment.corrects_treatment_id,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_treatment_by_id(conn: &Connection, id: i64) -> Result<Option<Treatment>, DbError> {
    conn.query_row(
        &format!("SELECT {TREATMENT_COLUMNS} FROM treatments WHERE id = ?1"),
        params![id],
        map_row_to_treatment,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_treatments_by_encounter(
    conn: &Connection,
    encounter_id: i64,
) -> Result<Vec<Treatment>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {TREATMENT_COLUMNS} FROM treatments WHERE encounter_id = ?1 ORDER BY created_at"
    ))?;
    let rows = stmt.query_map(params![encounter_id], map_row_to_treatment)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn insert_evolution(conn: &Connection, new_evolution: &NewEvolution) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO evolutions (encounter_id, note, registered_by_user_id) VALUES (?1, ?2, ?3)",
        params![
            new_evolution.encounter_id,
            new_evolution.note,
            new_evolution.registered_by_user_id,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_evolution_by_id(conn: &Connection, id: i64) -> Result<Option<Evolution>, DbError> {
    conn.query_row(
        &format!("SELECT {EVOLUTION_COLUMNS} FROM evolutions WHERE id = ?1"),
        params![id],
        map_row_to_evolution,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_evolutions_by_encounter(
    conn: &Connection,
    encounter_id: i64,
) -> Result<Vec<Evolution>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {EVOLUTION_COLUMNS} FROM evolutions WHERE encounter_id = ?1 ORDER BY created_at"
    ))?;
    let rows = stmt.query_map(params![encounter_id], map_row_to_evolution)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

fn map_row_to_encounter(row: &Row) -> rusqlite::Result<Encounter> {
    Ok(Encounter {
        id: row.get(0)?,
        patient_id: row.get(1)?,
        status: row.get(2)?,
        admitted_at: row.get(3)?,
        discharged_at: row.get(4)?,
        discharge_summary: row.get(5)?,
        created_by_user_id: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn map_row_to_diagnosis(row: &Row) -> rusqlite::Result<Diagnosis> {
    Ok(Diagnosis {
        id: row.get(0)?,
        encounter_id: row.get(1)?,
        description: row.get(2)?,
        icd_code: row.get(3)?,
        registered_by_user_id: row.get(4)?,
        corrects_diagnosis_id: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn map_row_to_treatment(row: &Row) -> rusqlite::Result<Treatment> {
    Ok(Treatment {
        id: row.get(0)?,
        encounter_id: row.get(1)?,
        diagnosis_id: row.get(2)?,
        description: row.get(3)?,
        dosage: row.get(4)?,
        registered_by_user_id: row.get(5)?,
        corrects_treatment_id: row.get(6)?,
        created_at: row.get(7)?,
    })
}

fn map_row_to_evolution(row: &Row) -> rusqlite::Result<Evolution> {
    Ok(Evolution {
        id: row.get(0)?,
        encounter_id: row.get(1)?,
        note: row.get(2)?,
        registered_by_user_id: row.get(3)?,
        created_at: row.get(4)?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [7u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("encounter-repo-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn.execute(
            "INSERT INTO users (full_name, username, password_hash, role) \
             VALUES ('Test Admin', 'test-admin', 'argon2id$dummy', 'admin')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO patients (medical_record_number, full_name, date_of_birth, sex) \
             VALUES ('MRN-0001', 'Ada Lovelace', '1990-01-01', 'female')",
            [],
        )
        .unwrap();
        conn
    }

    fn insert_sample_encounter(conn: &Connection) -> i64 {
        insert_encounter(
            conn,
            &NewEncounter {
                patient_id: 1,
                created_by_user_id: 1,
            },
        )
        .unwrap()
    }

    #[test]
    fn insert_then_find_encounter_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        let id = insert_sample_encounter(&conn);
        let found = find_encounter_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.patient_id, 1);
        assert_eq!(found.status, "open");
    }

    #[test]
    fn discharge_encounter_sets_status_and_summary() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let id = insert_sample_encounter(&conn);

        let discharged = discharge_encounter(
            &conn,
            id,
            &DischargeEncounter {
                discharge_summary: Some("Recovered well."),
            },
        )
        .unwrap()
        .unwrap();

        assert_eq!(discharged.status, "discharged");
        assert_eq!(
            discharged.discharge_summary.as_deref(),
            Some("Recovered well.")
        );
        assert!(discharged.discharged_at.is_some());
    }

    #[test]
    fn find_encounters_by_patient_orders_by_admitted_at() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        insert_sample_encounter(&conn);
        insert_sample_encounter(&conn);

        let encounters = find_encounters_by_patient(&conn, 1).unwrap();

        assert_eq!(encounters.len(), 2);
    }

    #[test]
    fn insert_then_find_diagnosis_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let encounter_id = insert_sample_encounter(&conn);

        let id = insert_diagnosis(
            &conn,
            &NewDiagnosis {
                encounter_id,
                description: "Acute appendicitis",
                icd_code: Some("K35.80"),
                registered_by_user_id: 1,
                corrects_diagnosis_id: None,
            },
        )
        .unwrap();
        let found = find_diagnosis_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.description, "Acute appendicitis");
        assert_eq!(found.icd_code.as_deref(), Some("K35.80"));
    }

    #[test]
    fn a_correction_diagnosis_references_the_original() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let encounter_id = insert_sample_encounter(&conn);
        let original_id = insert_diagnosis(
            &conn,
            &NewDiagnosis {
                encounter_id,
                description: "Acute appendicitis",
                icd_code: None,
                registered_by_user_id: 1,
                corrects_diagnosis_id: None,
            },
        )
        .unwrap();

        let correction_id = insert_diagnosis(
            &conn,
            &NewDiagnosis {
                encounter_id,
                description: "Acute cholecystitis",
                icd_code: None,
                registered_by_user_id: 1,
                corrects_diagnosis_id: Some(original_id),
            },
        )
        .unwrap();
        let correction = find_diagnosis_by_id(&conn, correction_id).unwrap().unwrap();

        assert_eq!(correction.corrects_diagnosis_id, Some(original_id));
        let all = find_diagnoses_by_encounter(&conn, encounter_id).unwrap();
        assert_eq!(
            all.len(),
            2,
            "the original row must still exist — append-only"
        );
    }

    #[test]
    fn insert_then_find_treatment_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let encounter_id = insert_sample_encounter(&conn);

        let id = insert_treatment(
            &conn,
            &NewTreatment {
                encounter_id,
                diagnosis_id: None,
                description: "Appendectomy",
                dosage: None,
                registered_by_user_id: 1,
                corrects_treatment_id: None,
            },
        )
        .unwrap();
        let found = find_treatment_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.description, "Appendectomy");
    }

    #[test]
    fn find_treatments_by_encounter_returns_only_that_encounters_rows() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let encounter_id = insert_sample_encounter(&conn);
        let other_encounter_id = insert_sample_encounter(&conn);
        insert_treatment(
            &conn,
            &NewTreatment {
                encounter_id,
                diagnosis_id: None,
                description: "Appendectomy",
                dosage: None,
                registered_by_user_id: 1,
                corrects_treatment_id: None,
            },
        )
        .unwrap();
        insert_treatment(
            &conn,
            &NewTreatment {
                encounter_id: other_encounter_id,
                diagnosis_id: None,
                description: "IV fluids",
                dosage: Some("1L NS"),
                registered_by_user_id: 1,
                corrects_treatment_id: None,
            },
        )
        .unwrap();

        let results = find_treatments_by_encounter(&conn, encounter_id).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "Appendectomy");
    }

    #[test]
    fn insert_then_find_evolution_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let encounter_id = insert_sample_encounter(&conn);

        insert_evolution(
            &conn,
            &NewEvolution {
                encounter_id,
                note: "Patient stable post-op.",
                registered_by_user_id: 1,
            },
        )
        .unwrap();
        let results = find_evolutions_by_encounter(&conn, encounter_id).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].note, "Patient stable post-op.");
    }
}
