//! Pure SQL data access for `operating_rooms`/`or_reservations` (Architecture.md — repositories
//! hold no business rules and make no cross-repository calls). The overlap-prevention business
//! rule itself lives in `operating_room_service.rs`; this module only exposes the range-overlap
//! query it needs (Database.md Section 3.4).

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::DbError;
use crate::models::{OperatingRoom, OrReservation};

pub struct NewOperatingRoom<'a> {
    pub room_id: i64,
    pub name: &'a str,
}

pub struct NewOrReservation<'a> {
    pub operating_room_id: i64,
    pub patient_id: i64,
    pub encounter_id: i64,
    pub procedure_description: &'a str,
    pub scheduled_start: &'a str,
    pub scheduled_end: &'a str,
    pub scheduled_by_user_id: i64,
}

const OPERATING_ROOM_COLUMNS: &str = "id, room_id, name, created_at";
const OR_RESERVATION_COLUMNS: &str = "id, operating_room_id, patient_id, encounter_id, \
     procedure_description, scheduled_start, scheduled_end, status, scheduled_by_user_id, \
     created_at, updated_at";

pub fn insert_operating_room(
    conn: &Connection,
    new_operating_room: &NewOperatingRoom,
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO operating_rooms (room_id, name) VALUES (?1, ?2)",
        params![new_operating_room.room_id, new_operating_room.name],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_operating_room_by_id(
    conn: &Connection,
    id: i64,
) -> Result<Option<OperatingRoom>, DbError> {
    conn.query_row(
        &format!("SELECT {OPERATING_ROOM_COLUMNS} FROM operating_rooms WHERE id = ?1"),
        params![id],
        map_row_to_operating_room,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_operating_room_by_room_id(
    conn: &Connection,
    room_id: i64,
) -> Result<Option<OperatingRoom>, DbError> {
    conn.query_row(
        &format!("SELECT {OPERATING_ROOM_COLUMNS} FROM operating_rooms WHERE room_id = ?1"),
        params![room_id],
        map_row_to_operating_room,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_operating_rooms(conn: &Connection) -> Result<Vec<OperatingRoom>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {OPERATING_ROOM_COLUMNS} FROM operating_rooms ORDER BY name"
    ))?;
    let rows = stmt.query_map([], map_row_to_operating_room)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn insert_reservation(
    conn: &Connection,
    new_reservation: &NewOrReservation,
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO or_reservations \
         (operating_room_id, patient_id, encounter_id, procedure_description, \
          scheduled_start, scheduled_end, scheduled_by_user_id) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            new_reservation.operating_room_id,
            new_reservation.patient_id,
            new_reservation.encounter_id,
            new_reservation.procedure_description,
            new_reservation.scheduled_start,
            new_reservation.scheduled_end,
            new_reservation.scheduled_by_user_id,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_reservation_by_id(
    conn: &Connection,
    id: i64,
) -> Result<Option<OrReservation>, DbError> {
    conn.query_row(
        &format!("SELECT {OR_RESERVATION_COLUMNS} FROM or_reservations WHERE id = ?1"),
        params![id],
        map_row_to_reservation,
    )
    .optional()
    .map_err(DbError::from)
}

/// Backs `operating_rooms_list_reservations`-style reads (IPC.md Section 2), optionally scoped to
/// one operating room.
pub fn list_reservations(
    conn: &Connection,
    operating_room_id: Option<i64>,
) -> Result<Vec<OrReservation>, DbError> {
    let sql = format!(
        "SELECT {OR_RESERVATION_COLUMNS} FROM or_reservations {} ORDER BY scheduled_start",
        if operating_room_id.is_some() {
            "WHERE operating_room_id = ?1"
        } else {
            ""
        }
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = match operating_room_id {
        Some(id) => stmt.query_map(params![id], map_row_to_reservation)?,
        None => stmt.query_map([], map_row_to_reservation)?,
    };
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// The overlap-prevention query (Database.md Section 3.4) — every `scheduled`/`in_progress`
/// reservation for `operating_room_id` whose `[scheduled_start, scheduled_end)` range intersects
/// `[start, end)`. `exclude_reservation_id` lets an update check overlap against every *other*
/// reservation without conflicting with itself.
pub fn find_overlapping_reservations(
    conn: &Connection,
    operating_room_id: i64,
    start: &str,
    end: &str,
    exclude_reservation_id: Option<i64>,
) -> Result<Vec<OrReservation>, DbError> {
    let sql = format!(
        "SELECT {OR_RESERVATION_COLUMNS} FROM or_reservations \
         WHERE operating_room_id = ?1 \
         AND status IN ('scheduled', 'in_progress') \
         AND NOT (scheduled_end <= ?2 OR scheduled_start >= ?3) \
         {}",
        if exclude_reservation_id.is_some() {
            "AND id != ?4"
        } else {
            ""
        }
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = match exclude_reservation_id {
        Some(exclude_id) => stmt.query_map(
            params![operating_room_id, start, end, exclude_id],
            map_row_to_reservation,
        )?,
        None => stmt.query_map(
            params![operating_room_id, start, end],
            map_row_to_reservation,
        )?,
    };
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn update_reservation_status(
    conn: &Connection,
    id: i64,
    status: &str,
) -> Result<Option<OrReservation>, DbError> {
    conn.execute(
        "UPDATE or_reservations SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status, id],
    )?;
    find_reservation_by_id(conn, id)
}

pub struct ReservationDetails<'a> {
    pub procedure_description: &'a str,
    pub scheduled_start: &'a str,
    pub scheduled_end: &'a str,
}

pub fn update_reservation_details(
    conn: &Connection,
    id: i64,
    details: &ReservationDetails,
) -> Result<Option<OrReservation>, DbError> {
    conn.execute(
        "UPDATE or_reservations \
         SET procedure_description = ?1, scheduled_start = ?2, scheduled_end = ?3, \
             updated_at = datetime('now') \
         WHERE id = ?4",
        params![
            details.procedure_description,
            details.scheduled_start,
            details.scheduled_end,
            id,
        ],
    )?;
    find_reservation_by_id(conn, id)
}

fn map_row_to_operating_room(row: &Row) -> rusqlite::Result<OperatingRoom> {
    Ok(OperatingRoom {
        id: row.get(0)?,
        room_id: row.get(1)?,
        name: row.get(2)?,
        created_at: row.get(3)?,
    })
}

fn map_row_to_reservation(row: &Row) -> rusqlite::Result<OrReservation> {
    Ok(OrReservation {
        id: row.get(0)?,
        operating_room_id: row.get(1)?,
        patient_id: row.get(2)?,
        encounter_id: row.get(3)?,
        procedure_description: row.get(4)?,
        scheduled_start: row.get(5)?,
        scheduled_end: row.get(6)?,
        status: row.get(7)?,
        scheduled_by_user_id: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::db::{connection, migrator};

    const TEST_KEY: [u8; 32] = [21u8; 32];

    fn open_migrated(dir: &std::path::Path) -> Connection {
        let conn = connection::open(&dir.join("or-repo-test.sqlite"), &TEST_KEY).unwrap();
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

    fn seed_room(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO floors (name, level_order) VALUES ('Ground Floor', 0)",
            [],
        )
        .unwrap();
        let floor_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO rooms (floor_id, name, room_type, map_x, map_y) \
             VALUES (?1, 'OR Suite', 'operating_room', 0.5, 0.5)",
            params![floor_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn seed_operating_room(conn: &Connection) -> i64 {
        let room_id = seed_room(conn);
        insert_operating_room(
            conn,
            &NewOperatingRoom {
                room_id,
                name: "OR 1",
            },
        )
        .unwrap()
    }

    fn sample_reservation(
        operating_room_id: i64,
        patient_id: i64,
        encounter_id: i64,
        user_id: i64,
    ) -> NewOrReservation<'static> {
        NewOrReservation {
            operating_room_id,
            patient_id,
            encounter_id,
            procedure_description: "Appendectomy",
            scheduled_start: "2026-01-01T08:00:00",
            scheduled_end: "2026-01-01T10:00:00",
            scheduled_by_user_id: user_id,
        }
    }

    #[test]
    fn insert_then_find_operating_room_by_id_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let room_id = seed_room(&conn);

        let id = insert_operating_room(
            &conn,
            &NewOperatingRoom {
                room_id,
                name: "OR 1",
            },
        )
        .unwrap();
        let found = find_operating_room_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.name, "OR 1");
        assert_eq!(found.room_id, room_id);
    }

    #[test]
    fn find_operating_room_by_room_id_returns_none_when_absent() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        assert!(find_operating_room_by_room_id(&conn, 999)
            .unwrap()
            .is_none());
    }

    #[test]
    fn insert_operating_room_rejects_a_duplicate_room_id() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let room_id = seed_room(&conn);
        insert_operating_room(
            &conn,
            &NewOperatingRoom {
                room_id,
                name: "OR 1",
            },
        )
        .unwrap();

        let result = insert_operating_room(
            &conn,
            &NewOperatingRoom {
                room_id,
                name: "OR 2",
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn insert_then_find_reservation_by_id_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let operating_room_id = seed_operating_room(&conn);

        let id = insert_reservation(
            &conn,
            &sample_reservation(operating_room_id, patient_id, encounter_id, user_id),
        )
        .unwrap();
        let found = find_reservation_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.status, "scheduled");
        assert_eq!(found.procedure_description, "Appendectomy");
    }

    #[test]
    fn find_overlapping_reservations_detects_a_partial_overlap() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let operating_room_id = seed_operating_room(&conn);
        insert_reservation(
            &conn,
            &sample_reservation(operating_room_id, patient_id, encounter_id, user_id),
        )
        .unwrap();

        let overlaps = find_overlapping_reservations(
            &conn,
            operating_room_id,
            "2026-01-01T09:00:00",
            "2026-01-01T11:00:00",
            None,
        )
        .unwrap();

        assert_eq!(overlaps.len(), 1);
    }

    #[test]
    fn find_overlapping_reservations_ignores_an_adjacent_non_overlapping_slot() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let operating_room_id = seed_operating_room(&conn);
        insert_reservation(
            &conn,
            &sample_reservation(operating_room_id, patient_id, encounter_id, user_id),
        )
        .unwrap();

        let overlaps = find_overlapping_reservations(
            &conn,
            operating_room_id,
            "2026-01-01T10:00:00",
            "2026-01-01T12:00:00",
            None,
        )
        .unwrap();

        assert!(overlaps.is_empty());
    }

    #[test]
    fn find_overlapping_reservations_excludes_the_given_reservation_id() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let operating_room_id = seed_operating_room(&conn);
        let reservation_id = insert_reservation(
            &conn,
            &sample_reservation(operating_room_id, patient_id, encounter_id, user_id),
        )
        .unwrap();

        let overlaps = find_overlapping_reservations(
            &conn,
            operating_room_id,
            "2026-01-01T08:00:00",
            "2026-01-01T10:00:00",
            Some(reservation_id),
        )
        .unwrap();

        assert!(overlaps.is_empty());
    }

    #[test]
    fn update_reservation_status_persists_the_new_status() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let operating_room_id = seed_operating_room(&conn);
        let reservation_id = insert_reservation(
            &conn,
            &sample_reservation(operating_room_id, patient_id, encounter_id, user_id),
        )
        .unwrap();

        let updated = update_reservation_status(&conn, reservation_id, "cancelled")
            .unwrap()
            .unwrap();

        assert_eq!(updated.status, "cancelled");
        assert!(updated.updated_at.is_some());
    }

    #[test]
    fn list_reservations_filters_by_operating_room_when_given() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let operating_room_id = seed_operating_room(&conn);
        insert_reservation(
            &conn,
            &sample_reservation(operating_room_id, patient_id, encounter_id, user_id),
        )
        .unwrap();

        let reservations = list_reservations(&conn, Some(operating_room_id)).unwrap();

        assert_eq!(reservations.len(), 1);
    }
}
