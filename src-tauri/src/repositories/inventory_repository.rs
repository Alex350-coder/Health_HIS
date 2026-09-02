//! Pure SQL data access for `inventory_categories`/`inventory_items`/`inventory_transactions`/
//! `maintenance_schedules` (Architecture.md — repositories hold no business rules and make no
//! cross-repository calls). Sufficient-stock and FK-existence checks live in
//! `inventory_service.rs`; this module only exposes reads and writes (Database.md Section 3.5).

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::DbError;
use crate::models::{InventoryCategory, InventoryItem, InventoryTransaction, MaintenanceSchedule};

pub struct NewInventoryCategory<'a> {
    pub name: &'a str,
    pub kind: &'a str,
}

pub struct NewInventoryItem<'a> {
    pub category_id: i64,
    pub name: &'a str,
    pub unit: &'a str,
    pub reorder_threshold: i64,
    pub expiration_date: Option<&'a str>,
    pub location: Option<&'a str>,
}

pub struct NewInventoryTransaction<'a> {
    pub item_id: i64,
    pub quantity_delta: i64,
    pub reason: &'a str,
    pub encounter_id: Option<i64>,
    pub treatment_id: Option<i64>,
    pub performed_by_user_id: i64,
}

pub struct NewMaintenanceSchedule<'a> {
    pub inventory_item_id: i64,
    pub scheduled_date: &'a str,
    pub notes: Option<&'a str>,
}

const CATEGORY_COLUMNS: &str = "id, name, kind";
const ITEM_COLUMNS: &str = "id, category_id, name, quantity, unit, reorder_threshold, \
     expiration_date, location, created_at, updated_at";
const TRANSACTION_COLUMNS: &str = "id, item_id, quantity_delta, reason, encounter_id, \
     treatment_id, performed_by_user_id, created_at";
const MAINTENANCE_COLUMNS: &str =
    "id, inventory_item_id, scheduled_date, completed_date, notes, created_at";

pub fn insert_category(
    conn: &Connection,
    new_category: &NewInventoryCategory,
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO inventory_categories (name, kind) VALUES (?1, ?2)",
        params![new_category.name, new_category.kind],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_category_by_id(
    conn: &Connection,
    id: i64,
) -> Result<Option<InventoryCategory>, DbError> {
    conn.query_row(
        &format!("SELECT {CATEGORY_COLUMNS} FROM inventory_categories WHERE id = ?1"),
        params![id],
        map_row_to_category,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_categories(conn: &Connection) -> Result<Vec<InventoryCategory>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {CATEGORY_COLUMNS} FROM inventory_categories ORDER BY name"
    ))?;
    let rows = stmt.query_map([], map_row_to_category)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn insert_item(conn: &Connection, new_item: &NewInventoryItem) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO inventory_items \
         (category_id, name, unit, reorder_threshold, expiration_date, location) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            new_item.category_id,
            new_item.name,
            new_item.unit,
            new_item.reorder_threshold,
            new_item.expiration_date,
            new_item.location,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_item_by_id(conn: &Connection, id: i64) -> Result<Option<InventoryItem>, DbError> {
    conn.query_row(
        &format!("SELECT {ITEM_COLUMNS} FROM inventory_items WHERE id = ?1"),
        params![id],
        map_row_to_item,
    )
    .optional()
    .map_err(DbError::from)
}

/// Backs `inventory_list_items` (IPC.md Section 2), optionally scoped to one category and/or
/// filtered to items at or below their reorder threshold.
pub fn list_items(
    conn: &Connection,
    category_id: Option<i64>,
    low_stock_only: bool,
) -> Result<Vec<InventoryItem>, DbError> {
    let mut conditions: Vec<&str> = Vec::new();
    if category_id.is_some() {
        conditions.push("category_id = ?1");
    }
    if low_stock_only {
        conditions.push("quantity <= reorder_threshold");
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let sql = format!("SELECT {ITEM_COLUMNS} FROM inventory_items {where_clause} ORDER BY name");
    let mut stmt = conn.prepare(&sql)?;
    let rows = match category_id {
        Some(id) => stmt.query_map(params![id], map_row_to_item)?,
        None => stmt.query_map([], map_row_to_item)?,
    };
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// Every item at or below its reorder threshold — the read-only detection Phase 11
/// (Notifications) calls to decide when to raise a low-stock alert.
pub fn find_low_stock_items(conn: &Connection) -> Result<Vec<InventoryItem>, DbError> {
    list_items(conn, None, true)
}

/// Every item expiring on or before `before_date` (inclusive, `YYYY-MM-DD`) — the read-only
/// detection Phase 11 (Notifications) calls to decide when to raise an expiration alert.
pub fn find_expiring_items(
    conn: &Connection,
    before_date: &str,
) -> Result<Vec<InventoryItem>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ITEM_COLUMNS} FROM inventory_items \
         WHERE expiration_date IS NOT NULL AND expiration_date <= ?1 \
         ORDER BY expiration_date"
    ))?;
    let rows = stmt.query_map(params![before_date], map_row_to_item)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn update_item_quantity(
    conn: &Connection,
    item_id: i64,
    new_quantity: i64,
) -> Result<Option<InventoryItem>, DbError> {
    conn.execute(
        "UPDATE inventory_items SET quantity = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![new_quantity, item_id],
    )?;
    find_item_by_id(conn, item_id)
}

pub fn insert_transaction(
    conn: &Connection,
    new_transaction: &NewInventoryTransaction,
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO inventory_transactions \
         (item_id, quantity_delta, reason, encounter_id, treatment_id, performed_by_user_id) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            new_transaction.item_id,
            new_transaction.quantity_delta,
            new_transaction.reason,
            new_transaction.encounter_id,
            new_transaction.treatment_id,
            new_transaction.performed_by_user_id,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_transaction_by_id(
    conn: &Connection,
    id: i64,
) -> Result<Option<InventoryTransaction>, DbError> {
    conn.query_row(
        &format!("SELECT {TRANSACTION_COLUMNS} FROM inventory_transactions WHERE id = ?1"),
        params![id],
        map_row_to_transaction,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_transactions_for_item(
    conn: &Connection,
    item_id: i64,
) -> Result<Vec<InventoryTransaction>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {TRANSACTION_COLUMNS} FROM inventory_transactions \
         WHERE item_id = ?1 ORDER BY created_at"
    ))?;
    let rows = stmt.query_map(params![item_id], map_row_to_transaction)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// Backs Billing's inventory-charge aggregation (Plan.md Phase 12) — every transaction tied to
/// one encounter, regardless of `reason`; the caller decides which reasons are billable.
pub fn list_transactions_by_encounter_id(
    conn: &Connection,
    encounter_id: i64,
) -> Result<Vec<InventoryTransaction>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {TRANSACTION_COLUMNS} FROM inventory_transactions \
         WHERE encounter_id = ?1 ORDER BY created_at"
    ))?;
    let rows = stmt.query_map(params![encounter_id], map_row_to_transaction)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn insert_maintenance_schedule(
    conn: &Connection,
    new_schedule: &NewMaintenanceSchedule,
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO maintenance_schedules (inventory_item_id, scheduled_date, notes) \
         VALUES (?1, ?2, ?3)",
        params![
            new_schedule.inventory_item_id,
            new_schedule.scheduled_date,
            new_schedule.notes,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_maintenance_schedule_by_id(
    conn: &Connection,
    id: i64,
) -> Result<Option<MaintenanceSchedule>, DbError> {
    conn.query_row(
        &format!("SELECT {MAINTENANCE_COLUMNS} FROM maintenance_schedules WHERE id = ?1"),
        params![id],
        map_row_to_maintenance_schedule,
    )
    .optional()
    .map_err(DbError::from)
}

/// Every incomplete maintenance schedule due on or before `before_date` (inclusive,
/// `YYYY-MM-DD`) — the read-only detection Phase 11 (Notifications) calls to decide when to
/// raise a maintenance-due alert. Mirrors `find_expiring_items`.
pub fn find_due_maintenance_schedules(
    conn: &Connection,
    before_date: &str,
) -> Result<Vec<MaintenanceSchedule>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {MAINTENANCE_COLUMNS} FROM maintenance_schedules \
         WHERE completed_date IS NULL AND scheduled_date <= ?1 \
         ORDER BY scheduled_date"
    ))?;
    let rows = stmt.query_map(params![before_date], map_row_to_maintenance_schedule)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn list_maintenance_schedules_for_item(
    conn: &Connection,
    inventory_item_id: i64,
) -> Result<Vec<MaintenanceSchedule>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {MAINTENANCE_COLUMNS} FROM maintenance_schedules \
         WHERE inventory_item_id = ?1 ORDER BY scheduled_date"
    ))?;
    let rows = stmt.query_map(params![inventory_item_id], map_row_to_maintenance_schedule)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

fn map_row_to_category(row: &Row) -> rusqlite::Result<InventoryCategory> {
    Ok(InventoryCategory {
        id: row.get(0)?,
        name: row.get(1)?,
        kind: row.get(2)?,
    })
}

fn map_row_to_item(row: &Row) -> rusqlite::Result<InventoryItem> {
    Ok(InventoryItem {
        id: row.get(0)?,
        category_id: row.get(1)?,
        name: row.get(2)?,
        quantity: row.get(3)?,
        unit: row.get(4)?,
        reorder_threshold: row.get(5)?,
        expiration_date: row.get(6)?,
        location: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn map_row_to_transaction(row: &Row) -> rusqlite::Result<InventoryTransaction> {
    Ok(InventoryTransaction {
        id: row.get(0)?,
        item_id: row.get(1)?,
        quantity_delta: row.get(2)?,
        reason: row.get(3)?,
        encounter_id: row.get(4)?,
        treatment_id: row.get(5)?,
        performed_by_user_id: row.get(6)?,
        created_at: row.get(7)?,
    })
}

fn map_row_to_maintenance_schedule(row: &Row) -> rusqlite::Result<MaintenanceSchedule> {
    Ok(MaintenanceSchedule {
        id: row.get(0)?,
        inventory_item_id: row.get(1)?,
        scheduled_date: row.get(2)?,
        completed_date: row.get(3)?,
        notes: row.get(4)?,
        created_at: row.get(5)?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [41u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("inventory-repo-test.sqlite"), &TEST_KEY).unwrap();
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

    fn seed_category(conn: &Connection) -> i64 {
        insert_category(
            conn,
            &NewInventoryCategory {
                name: "Analgesics",
                kind: "medicine",
            },
        )
        .unwrap()
    }

    fn seed_item(conn: &Connection, category_id: i64) -> i64 {
        insert_item(
            conn,
            &NewInventoryItem {
                category_id,
                name: "Ibuprofen 400mg",
                unit: "box",
                reorder_threshold: 10,
                expiration_date: Some("2027-01-01"),
                location: Some("Pharmacy Shelf B2"),
            },
        )
        .unwrap()
    }

    #[test]
    fn insert_then_find_category_by_id_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        let id = seed_category(&conn);
        let found = find_category_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.name, "Analgesics");
        assert_eq!(found.kind, "medicine");
    }

    #[test]
    fn insert_category_rejects_a_duplicate_name() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        seed_category(&conn);

        let result = insert_category(
            &conn,
            &NewInventoryCategory {
                name: "Analgesics",
                kind: "supply",
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn insert_then_find_item_by_id_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let category_id = seed_category(&conn);

        let id = seed_item(&conn, category_id);
        let found = find_item_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.name, "Ibuprofen 400mg");
        assert_eq!(found.quantity, 0);
    }

    #[test]
    fn list_items_filters_by_low_stock_only() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let category_id = seed_category(&conn);
        let item_id = seed_item(&conn, category_id);
        update_item_quantity(&conn, item_id, 5).unwrap();

        let low_stock = list_items(&conn, None, true).unwrap();

        assert_eq!(low_stock.len(), 1);
        assert_eq!(low_stock[0].id, item_id);
    }

    #[test]
    fn find_low_stock_items_excludes_items_above_threshold() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let category_id = seed_category(&conn);
        let item_id = seed_item(&conn, category_id);
        update_item_quantity(&conn, item_id, 50).unwrap();

        assert!(find_low_stock_items(&conn).unwrap().is_empty());
    }

    #[test]
    fn find_expiring_items_returns_items_on_or_before_the_cutoff() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let category_id = seed_category(&conn);
        seed_item(&conn, category_id);

        let expiring = find_expiring_items(&conn, "2027-06-01").unwrap();

        assert_eq!(expiring.len(), 1);
    }

    #[test]
    fn update_item_quantity_persists_the_new_total() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let category_id = seed_category(&conn);
        let item_id = seed_item(&conn, category_id);

        let updated = update_item_quantity(&conn, item_id, 42).unwrap().unwrap();

        assert_eq!(updated.quantity, 42);
        assert!(updated.updated_at.is_some());
    }

    #[test]
    fn insert_then_list_transactions_for_item_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let category_id = seed_category(&conn);
        let item_id = seed_item(&conn, category_id);
        let user_id = seed_user(&conn);

        insert_transaction(
            &conn,
            &NewInventoryTransaction {
                item_id,
                quantity_delta: 100,
                reason: "restock",
                encounter_id: None,
                treatment_id: None,
                performed_by_user_id: user_id,
            },
        )
        .unwrap();

        let transactions = list_transactions_for_item(&conn, item_id).unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].quantity_delta, 100);
        assert_eq!(transactions[0].reason, "restock");
    }

    #[test]
    fn list_transactions_by_encounter_id_returns_only_that_encounters_rows() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let category_id = seed_category(&conn);
        let item_id = seed_item(&conn, category_id);
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);

        insert_transaction(
            &conn,
            &NewInventoryTransaction {
                item_id,
                quantity_delta: -2,
                reason: "consumption",
                encounter_id: Some(encounter_id),
                treatment_id: None,
                performed_by_user_id: user_id,
            },
        )
        .unwrap();
        insert_transaction(
            &conn,
            &NewInventoryTransaction {
                item_id,
                quantity_delta: 100,
                reason: "restock",
                encounter_id: None,
                treatment_id: None,
                performed_by_user_id: user_id,
            },
        )
        .unwrap();

        let transactions = list_transactions_by_encounter_id(&conn, encounter_id).unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].reason, "consumption");
    }

    #[test]
    fn find_due_maintenance_schedules_excludes_completed_and_far_future_schedules() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let category_id = seed_category(&conn);
        let item_id = seed_item(&conn, category_id);

        let due_id = insert_maintenance_schedule(
            &conn,
            &NewMaintenanceSchedule {
                inventory_item_id: item_id,
                scheduled_date: "2026-01-05",
                notes: None,
            },
        )
        .unwrap();
        insert_maintenance_schedule(
            &conn,
            &NewMaintenanceSchedule {
                inventory_item_id: item_id,
                scheduled_date: "2099-01-01",
                notes: None,
            },
        )
        .unwrap();
        let completed_id = insert_maintenance_schedule(
            &conn,
            &NewMaintenanceSchedule {
                inventory_item_id: item_id,
                scheduled_date: "2026-01-01",
                notes: None,
            },
        )
        .unwrap();
        conn.execute(
            "UPDATE maintenance_schedules SET completed_date = '2026-01-02' WHERE id = ?1",
            params![completed_id],
        )
        .unwrap();

        let due = find_due_maintenance_schedules(&conn, "2026-01-10").unwrap();

        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, due_id);
    }

    #[test]
    fn insert_then_list_maintenance_schedules_for_item_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let category_id = seed_category(&conn);
        let item_id = seed_item(&conn, category_id);

        insert_maintenance_schedule(
            &conn,
            &NewMaintenanceSchedule {
                inventory_item_id: item_id,
                scheduled_date: "2026-02-01",
                notes: Some("Routine calibration"),
            },
        )
        .unwrap();

        let schedules = list_maintenance_schedules_for_item(&conn, item_id).unwrap();

        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].notes.as_deref(), Some("Routine calibration"));
    }
}
