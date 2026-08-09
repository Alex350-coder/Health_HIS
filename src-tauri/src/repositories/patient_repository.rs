//! Pure SQL data access for `patients` (Architecture.md — repositories hold no business rules
//! and make no cross-repository calls). Callers pass either a bare `Connection` or an open
//! `Transaction` (deref-compatible), so `patient_service` can run insert/update + audit write in
//! one transaction.

use rusqlite::{params, Connection, OptionalExtension, Row, ToSql};

use crate::db::DbError;
use crate::models::Patient;

pub struct NewPatient<'a> {
    pub medical_record_number: &'a str,
    pub full_name: &'a str,
    pub date_of_birth: &'a str,
    pub sex: &'a str,
    pub national_id: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub address: Option<&'a str>,
    pub emergency_contact_name: Option<&'a str>,
    pub emergency_contact_phone: Option<&'a str>,
    pub blood_type: Option<&'a str>,
    pub allergies: Option<&'a str>,
}

pub struct UpdatePatient<'a> {
    pub medical_record_number: &'a str,
    pub full_name: &'a str,
    pub date_of_birth: &'a str,
    pub sex: &'a str,
    pub national_id: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub address: Option<&'a str>,
    pub emergency_contact_name: Option<&'a str>,
    pub emergency_contact_phone: Option<&'a str>,
    pub blood_type: Option<&'a str>,
    pub allergies: Option<&'a str>,
}

#[derive(Default)]
pub struct ListFilter<'a> {
    /// Caller wraps the raw search term with `%...%` — the repository stays pure SQL.
    pub search: Option<&'a str>,
    pub limit: i64,
    pub offset: i64,
}

const SELECT_COLUMNS: &str = "id, medical_record_number, full_name, date_of_birth, sex, \
     national_id, phone, address, emergency_contact_name, emergency_contact_phone, \
     blood_type, allergies, created_at, updated_at";

pub fn insert(conn: &Connection, new_patient: &NewPatient) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO patients \
         (medical_record_number, full_name, date_of_birth, sex, national_id, phone, address, \
          emergency_contact_name, emergency_contact_phone, blood_type, allergies) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            new_patient.medical_record_number,
            new_patient.full_name,
            new_patient.date_of_birth,
            new_patient.sex,
            new_patient.national_id,
            new_patient.phone,
            new_patient.address,
            new_patient.emergency_contact_name,
            new_patient.emergency_contact_phone,
            new_patient.blood_type,
            new_patient.allergies,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(
    conn: &Connection,
    id: i64,
    patch: &UpdatePatient,
) -> Result<Option<Patient>, DbError> {
    conn.execute(
        "UPDATE patients SET \
         medical_record_number = ?1, full_name = ?2, date_of_birth = ?3, sex = ?4, \
         national_id = ?5, phone = ?6, address = ?7, emergency_contact_name = ?8, \
         emergency_contact_phone = ?9, blood_type = ?10, allergies = ?11, \
         updated_at = datetime('now') \
         WHERE id = ?12",
        params![
            patch.medical_record_number,
            patch.full_name,
            patch.date_of_birth,
            patch.sex,
            patch.national_id,
            patch.phone,
            patch.address,
            patch.emergency_contact_name,
            patch.emergency_contact_phone,
            patch.blood_type,
            patch.allergies,
            id,
        ],
    )?;
    find_by_id(conn, id)
}

pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<Patient>, DbError> {
    conn.query_row(
        &format!("SELECT {SELECT_COLUMNS} FROM patients WHERE id = ?1"),
        params![id],
        map_row_to_patient,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list(conn: &Connection, filter: &ListFilter) -> Result<Vec<Patient>, DbError> {
    let mut sql = format!("SELECT {SELECT_COLUMNS} FROM patients WHERE 1 = 1");
    let mut bindings: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(search) = filter.search {
        sql.push_str(" AND (full_name LIKE ? OR medical_record_number LIKE ?)");
        bindings.push(Box::new(search.to_string()));
        bindings.push(Box::new(search.to_string()));
    }
    sql.push_str(" ORDER BY full_name LIMIT ? OFFSET ?");
    bindings.push(Box::new(filter.limit));
    bindings.push(Box::new(filter.offset));

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn ToSql> = bindings.iter().map(AsRef::as_ref).collect();
    let rows = stmt.query_map(param_refs.as_slice(), map_row_to_patient)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

fn map_row_to_patient(row: &Row) -> rusqlite::Result<Patient> {
    Ok(Patient {
        id: row.get(0)?,
        medical_record_number: row.get(1)?,
        full_name: row.get(2)?,
        date_of_birth: row.get(3)?,
        sex: row.get(4)?,
        national_id: row.get(5)?,
        phone: row.get(6)?,
        address: row.get(7)?,
        emergency_contact_name: row.get(8)?,
        emergency_contact_phone: row.get(9)?,
        blood_type: row.get(10)?,
        allergies: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [5u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("patient-repo-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn
    }

    fn sample_new_patient() -> NewPatient<'static> {
        NewPatient {
            medical_record_number: "MRN-0001",
            full_name: "Ada Lovelace",
            date_of_birth: "1990-01-01",
            sex: "female",
            national_id: None,
            phone: None,
            address: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            blood_type: None,
            allergies: None,
        }
    }

    fn sample_update() -> UpdatePatient<'static> {
        UpdatePatient {
            medical_record_number: "MRN-0001",
            full_name: "Ada Byron",
            date_of_birth: "1990-01-01",
            sex: "female",
            national_id: None,
            phone: Some("555-0100"),
            address: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            blood_type: None,
            allergies: None,
        }
    }

    #[test]
    fn insert_then_find_by_id_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        let id = insert(&conn, &sample_new_patient()).unwrap();
        let found = find_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.medical_record_number, "MRN-0001");
        assert_eq!(found.full_name, "Ada Lovelace");
    }

    #[test]
    fn find_by_id_returns_none_when_absent() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        assert!(find_by_id(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn update_persists_changes_and_returns_the_patient() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let id = insert(&conn, &sample_new_patient()).unwrap();

        let updated = update(&conn, id, &sample_update()).unwrap().unwrap();

        assert_eq!(updated.full_name, "Ada Byron");
        assert_eq!(updated.phone, Some("555-0100".to_string()));
    }

    #[test]
    fn update_returns_none_when_id_absent() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        assert!(update(&conn, 999, &sample_update()).unwrap().is_none());
    }

    #[test]
    fn list_filters_by_search_term_across_name_and_mrn() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        insert(&conn, &sample_new_patient()).unwrap();
        insert(
            &conn,
            &NewPatient {
                medical_record_number: "MRN-0002",
                full_name: "Grace Hopper",
                ..sample_new_patient()
            },
        )
        .unwrap();

        let results = list(
            &conn,
            &ListFilter {
                search: Some("%Hopper%"),
                limit: 10,
                offset: 0,
            },
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].full_name, "Grace Hopper");
    }

    #[test]
    fn list_respects_limit_and_offset() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        insert(&conn, &sample_new_patient()).unwrap();
        insert(
            &conn,
            &NewPatient {
                medical_record_number: "MRN-0002",
                full_name: "Bertha Lovelace",
                ..sample_new_patient()
            },
        )
        .unwrap();
        insert(
            &conn,
            &NewPatient {
                medical_record_number: "MRN-0003",
                full_name: "Grace Hopper",
                ..sample_new_patient()
            },
        )
        .unwrap();

        let page = list(
            &conn,
            &ListFilter {
                search: None,
                limit: 1,
                offset: 1,
            },
        )
        .unwrap();

        assert_eq!(page.len(), 1);
        assert_eq!(page[0].full_name, "Bertha Lovelace");
    }

    #[test]
    fn insert_rejects_duplicate_medical_record_number() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        insert(&conn, &sample_new_patient()).unwrap();

        let result = insert(&conn, &sample_new_patient());
        assert!(result.is_err());
    }
}
