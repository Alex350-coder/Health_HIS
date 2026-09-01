//! Pure SQL data access for `billing_simulations`/`billing_items` (Architecture.md —
//! repositories hold no business rules and make no cross-repository calls). Aggregation across
//! other modules' data lives in `billing_service.rs`; this module only persists the resulting
//! simulation/items (Database.md Section 3.8).

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::DbError;
use crate::models::{BillingItem, BillingSimulation};

pub struct NewBillingItem<'a> {
    pub description: &'a str,
    pub source: &'a str,
    pub source_entity_id: Option<i64>,
    pub amount: f64,
}

const BILLING_SIMULATION_COLUMNS: &str = "id, encounter_id, status, total_amount, \
     generated_by_user_id, created_at, updated_at";
const BILLING_ITEM_COLUMNS: &str =
    "id, billing_simulation_id, description, source, source_entity_id, amount, created_at";

pub fn insert_simulation(
    conn: &Connection,
    encounter_id: i64,
    generated_by_user_id: i64,
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO billing_simulations (encounter_id, generated_by_user_id) VALUES (?1, ?2)",
        params![encounter_id, generated_by_user_id],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_item(
    conn: &Connection,
    billing_simulation_id: i64,
    new_item: &NewBillingItem,
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO billing_items \
         (billing_simulation_id, description, source, source_entity_id, amount) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            billing_simulation_id,
            new_item.description,
            new_item.source,
            new_item.source_entity_id,
            new_item.amount,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_simulation_by_id(
    conn: &Connection,
    id: i64,
) -> Result<Option<BillingSimulation>, DbError> {
    conn.query_row(
        &format!("SELECT {BILLING_SIMULATION_COLUMNS} FROM billing_simulations WHERE id = ?1"),
        params![id],
        map_row_to_simulation,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_simulation_by_encounter_id(
    conn: &Connection,
    encounter_id: i64,
) -> Result<Option<BillingSimulation>, DbError> {
    conn.query_row(
        &format!(
            "SELECT {BILLING_SIMULATION_COLUMNS} FROM billing_simulations \
             WHERE encounter_id = ?1"
        ),
        params![encounter_id],
        map_row_to_simulation,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_items_for_simulation(
    conn: &Connection,
    billing_simulation_id: i64,
) -> Result<Vec<BillingItem>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {BILLING_ITEM_COLUMNS} FROM billing_items \
         WHERE billing_simulation_id = ?1 ORDER BY id"
    ))?;
    let rows = stmt.query_map(params![billing_simulation_id], map_row_to_item)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn update_simulation_status(
    conn: &Connection,
    id: i64,
    status: &str,
) -> Result<Option<BillingSimulation>, DbError> {
    conn.execute(
        "UPDATE billing_simulations SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status, id],
    )?;
    find_simulation_by_id(conn, id)
}

/// Overwrites the maintained-sum `total_amount` (Database.md Section 3.8 normalization notes)
/// after every `billing_items` row for a simulation has been inserted.
pub fn update_simulation_total(
    conn: &Connection,
    id: i64,
    total_amount: f64,
) -> Result<Option<BillingSimulation>, DbError> {
    conn.execute(
        "UPDATE billing_simulations SET total_amount = ?1, updated_at = datetime('now') \
         WHERE id = ?2",
        params![total_amount, id],
    )?;
    find_simulation_by_id(conn, id)
}

fn map_row_to_simulation(row: &Row) -> rusqlite::Result<BillingSimulation> {
    Ok(BillingSimulation {
        id: row.get(0)?,
        encounter_id: row.get(1)?,
        status: row.get(2)?,
        total_amount: row.get(3)?,
        generated_by_user_id: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn map_row_to_item(row: &Row) -> rusqlite::Result<BillingItem> {
    Ok(BillingItem {
        id: row.get(0)?,
        billing_simulation_id: row.get(1)?,
        description: row.get(2)?,
        source: row.get(3)?,
        source_entity_id: row.get(4)?,
        amount: row.get(5)?,
        created_at: row.get(6)?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [22u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("billing-repo-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn
    }

    fn seed_user(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO users (full_name, username, password_hash, role) \
             VALUES ('Test Admin', 'test-admin', 'argon2id$dummy', 'admin')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn seed_patient(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO patients (medical_record_number, full_name, date_of_birth, sex) \
             VALUES ('MRN-1', 'Jane Doe', '1990-01-01', 'female')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn seed_encounter(conn: &Connection, patient_id: i64, created_by_user_id: i64) -> i64 {
        conn.execute(
            "INSERT INTO encounters (patient_id, created_by_user_id) VALUES (?1, ?2)",
            params![patient_id, created_by_user_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn inserts_and_finds_a_simulation_by_encounter_id() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);

        let simulation_id = insert_simulation(&conn, encounter_id, user_id).unwrap();
        let found = find_simulation_by_encounter_id(&conn, encounter_id)
            .unwrap()
            .unwrap();

        assert_eq!(found.id, simulation_id);
        assert_eq!(found.status, "draft");
        assert_eq!(found.total_amount, 0.0);
    }

    #[test]
    fn returns_none_for_an_encounter_with_no_simulation() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        assert!(find_simulation_by_encounter_id(&conn, 999)
            .unwrap()
            .is_none());
    }

    #[test]
    fn inserts_and_lists_items_for_a_simulation() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let simulation_id = insert_simulation(&conn, encounter_id, user_id).unwrap();

        insert_item(
            &conn,
            simulation_id,
            &NewBillingItem {
                description: "Room charge",
                source: "room",
                source_entity_id: Some(1),
                amount: 150.0,
            },
        )
        .unwrap();
        insert_item(
            &conn,
            simulation_id,
            &NewBillingItem {
                description: "Treatment",
                source: "treatment",
                source_entity_id: None,
                amount: 75.0,
            },
        )
        .unwrap();

        let items = list_items_for_simulation(&conn, simulation_id).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].amount, 150.0);
        assert_eq!(items[1].source, "treatment");
    }

    #[test]
    fn updates_simulation_status_and_total() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let simulation_id = insert_simulation(&conn, encounter_id, user_id).unwrap();

        let updated = update_simulation_total(&conn, simulation_id, 225.0)
            .unwrap()
            .unwrap();
        assert_eq!(updated.total_amount, 225.0);

        let finalized = update_simulation_status(&conn, simulation_id, "finalized")
            .unwrap()
            .unwrap();
        assert_eq!(finalized.status, "finalized");
    }
}
