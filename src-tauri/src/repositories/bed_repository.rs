//! Pure SQL data access for `floors`/`rooms`/`beds` (Architecture.md — repositories hold no
//! business rules and make no cross-repository calls). Read helpers here (`list_floors_with_rooms`,
//! `get_room_status`) back the Hospital Map's read-only commands directly, with no service layer
//! in between (Tasks.md Phase 6 deviation note — Hospital Map has no dedicated service).

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::DbError;
use crate::models::{
    ActiveAssignmentRef, Bed, BedAssignment, BedSummary, Floor, FloorLayout, Room, RoomStatus,
};

pub struct NewFloor<'a> {
    pub name: &'a str,
    pub level_order: i64,
}

pub struct NewRoom<'a> {
    pub floor_id: i64,
    pub name: &'a str,
    pub room_type: &'a str,
    pub map_x: f64,
    pub map_y: f64,
}

pub struct NewBed<'a> {
    pub room_id: i64,
    pub label: &'a str,
}

pub struct NewAssignment {
    pub bed_id: i64,
    pub patient_id: i64,
    pub encounter_id: i64,
    pub assigned_by_user_id: i64,
}

const FLOOR_COLUMNS: &str = "id, name, level_order, created_at";
const ROOM_COLUMNS: &str = "id, floor_id, name, room_type, map_x, map_y, created_at";
const BED_COLUMNS: &str = "id, room_id, label, status, created_at, updated_at";
const ASSIGNMENT_COLUMNS: &str = "id, bed_id, patient_id, encounter_id, assigned_at, released_at";

pub fn insert_floor(conn: &Connection, new_floor: &NewFloor) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO floors (name, level_order) VALUES (?1, ?2)",
        params![new_floor.name, new_floor.level_order],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_room(conn: &Connection, new_room: &NewRoom) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO rooms (floor_id, name, room_type, map_x, map_y) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            new_room.floor_id,
            new_room.name,
            new_room.room_type,
            new_room.map_x,
            new_room.map_y,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_bed(conn: &Connection, new_bed: &NewBed) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO beds (room_id, label) VALUES (?1, ?2)",
        params![new_bed.room_id, new_bed.label],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_bed_status(conn: &Connection, id: i64, status: &str) -> Result<Option<Bed>, DbError> {
    conn.execute(
        "UPDATE beds SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status, id],
    )?;
    find_bed_by_id(conn, id)
}

pub fn find_floor_by_id(conn: &Connection, id: i64) -> Result<Option<Floor>, DbError> {
    conn.query_row(
        &format!("SELECT {FLOOR_COLUMNS} FROM floors WHERE id = ?1"),
        params![id],
        map_row_to_floor,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_room_by_id(conn: &Connection, id: i64) -> Result<Option<Room>, DbError> {
    conn.query_row(
        &format!("SELECT {ROOM_COLUMNS} FROM rooms WHERE id = ?1"),
        params![id],
        map_row_to_room,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_bed_by_id(conn: &Connection, id: i64) -> Result<Option<Bed>, DbError> {
    conn.query_row(
        &format!("SELECT {BED_COLUMNS} FROM beds WHERE id = ?1"),
        params![id],
        map_row_to_bed,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn insert_assignment(
    conn: &Connection,
    new_assignment: &NewAssignment,
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO bed_assignments (bed_id, patient_id, encounter_id, assigned_by_user_id) \
         VALUES (?1, ?2, ?3, ?4)",
        params![
            new_assignment.bed_id,
            new_assignment.patient_id,
            new_assignment.encounter_id,
            new_assignment.assigned_by_user_id,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn find_assignment_by_id(conn: &Connection, id: i64) -> Result<Option<BedAssignment>, DbError> {
    conn.query_row(
        &format!("SELECT {ASSIGNMENT_COLUMNS} FROM bed_assignments WHERE id = ?1"),
        params![id],
        map_row_to_assignment,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn find_active_assignment_by_bed_id(
    conn: &Connection,
    bed_id: i64,
) -> Result<Option<BedAssignment>, DbError> {
    conn.query_row(
        &format!(
            "SELECT {ASSIGNMENT_COLUMNS} FROM bed_assignments \
             WHERE bed_id = ?1 AND released_at IS NULL"
        ),
        params![bed_id],
        map_row_to_assignment,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn release_assignment(conn: &Connection, id: i64) -> Result<Option<BedAssignment>, DbError> {
    conn.execute(
        "UPDATE bed_assignments SET released_at = datetime('now') \
         WHERE id = ?1 AND released_at IS NULL",
        params![id],
    )?;
    find_assignment_by_id(conn, id)
}

fn list_rooms_by_floor(conn: &Connection, floor_id: i64) -> Result<Vec<Room>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ROOM_COLUMNS} FROM rooms WHERE floor_id = ?1 ORDER BY name"
    ))?;
    let rows = stmt.query_map(params![floor_id], map_row_to_room)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// Backs `hospital_map_get_layout` (IPC.md Section 2) — every floor with its rooms nested.
pub fn list_floors_with_rooms(conn: &Connection) -> Result<Vec<FloorLayout>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {FLOOR_COLUMNS} FROM floors ORDER BY level_order"
    ))?;
    let floors = stmt
        .query_map([], map_row_to_floor)?
        .collect::<Result<Vec<_>, _>>()?;

    floors
        .into_iter()
        .map(|floor| {
            let rooms = list_rooms_by_floor(conn, floor.id)?;
            Ok(FloorLayout {
                id: floor.id,
                name: floor.name,
                level_order: floor.level_order,
                rooms,
            })
        })
        .collect()
}

pub fn list_beds_by_room(conn: &Connection, room_id: i64) -> Result<Vec<Bed>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {BED_COLUMNS} FROM beds WHERE room_id = ?1 ORDER BY label"
    ))?;
    let rows = stmt.query_map(params![room_id], map_row_to_bed)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// Backs `hospital_map_get_room_status` (IPC.md Section 2) — a room's beds plus an occupancy
/// summary. Returns `None` if the room does not exist.
pub fn get_room_status(conn: &Connection, room_id: i64) -> Result<Option<RoomStatus>, DbError> {
    let Some(room) = find_room_by_id(conn, room_id)? else {
        return Ok(None);
    };
    let beds = list_beds_by_room(conn, room_id)?;
    let available_count = beds.iter().filter(|bed| bed.status == "available").count() as i64;
    let occupied_count = beds.iter().filter(|bed| bed.status == "occupied").count() as i64;
    let maintenance_count = beds
        .iter()
        .filter(|bed| bed.status == "maintenance")
        .count() as i64;

    Ok(Some(RoomStatus {
        room,
        beds,
        available_count,
        occupied_count,
        maintenance_count,
    }))
}

/// Backs `beds_list` (IPC.md Section 2.1) — every bed (optionally scoped to one room) with its
/// active assignment linkage, via a `LEFT JOIN` on the partial-unique-index-backed active row.
pub fn list_beds_with_active_assignment(
    conn: &Connection,
    room_id: Option<i64>,
) -> Result<Vec<BedSummary>, DbError> {
    let sql = format!(
        "SELECT b.id, b.room_id, b.label, b.status, b.created_at, b.updated_at, \
                a.id, a.patient_id, a.encounter_id \
         FROM beds b \
         LEFT JOIN bed_assignments a ON a.bed_id = b.id AND a.released_at IS NULL \
         {} \
         ORDER BY b.label",
        if room_id.is_some() {
            "WHERE b.room_id = ?1"
        } else {
            ""
        }
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = match room_id {
        Some(id) => stmt.query_map(params![id], map_row_to_bed_summary)?,
        None => stmt.query_map([], map_row_to_bed_summary)?,
    };
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

fn map_row_to_bed_summary(row: &Row) -> rusqlite::Result<BedSummary> {
    let assignment_id: Option<i64> = row.get(6)?;
    let active_assignment = assignment_id.map(|id| -> rusqlite::Result<ActiveAssignmentRef> {
        Ok(ActiveAssignmentRef {
            id,
            patient_id: row.get(7)?,
            encounter_id: row.get(8)?,
        })
    });
    let active_assignment = active_assignment.transpose()?;

    Ok(BedSummary {
        bed: Bed {
            id: row.get(0)?,
            room_id: row.get(1)?,
            label: row.get(2)?,
            status: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        },
        active_assignment,
    })
}

fn map_row_to_assignment(row: &Row) -> rusqlite::Result<BedAssignment> {
    Ok(BedAssignment {
        id: row.get(0)?,
        bed_id: row.get(1)?,
        patient_id: row.get(2)?,
        encounter_id: row.get(3)?,
        assigned_at: row.get(4)?,
        released_at: row.get(5)?,
    })
}

fn map_row_to_floor(row: &Row) -> rusqlite::Result<Floor> {
    Ok(Floor {
        id: row.get(0)?,
        name: row.get(1)?,
        level_order: row.get(2)?,
        created_at: row.get(3)?,
    })
}

fn map_row_to_room(row: &Row) -> rusqlite::Result<Room> {
    Ok(Room {
        id: row.get(0)?,
        floor_id: row.get(1)?,
        name: row.get(2)?,
        room_type: row.get(3)?,
        map_x: row.get(4)?,
        map_y: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn map_row_to_bed(row: &Row) -> rusqlite::Result<Bed> {
    Ok(Bed {
        id: row.get(0)?,
        room_id: row.get(1)?,
        label: row.get(2)?,
        status: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
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
        let conn = connection::open(&dir.join("bed-repo-test.sqlite"), &TEST_KEY).unwrap();
        migrator::run_migrations(&conn, migrator::embedded_migrations()).unwrap();
        conn
    }

    fn sample_new_floor() -> NewFloor<'static> {
        NewFloor {
            name: "Ground Floor",
            level_order: 0,
        }
    }

    fn sample_new_room(floor_id: i64) -> NewRoom<'static> {
        NewRoom {
            floor_id,
            name: "Ward A",
            room_type: "ward",
            map_x: 0.25,
            map_y: 0.5,
        }
    }

    #[test]
    fn insert_then_find_floor_by_id_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        let id = insert_floor(&conn, &sample_new_floor()).unwrap();
        let found = find_floor_by_id(&conn, id).unwrap().unwrap();

        assert_eq!(found.name, "Ground Floor");
        assert_eq!(found.level_order, 0);
    }

    #[test]
    fn find_floor_by_id_returns_none_when_absent() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        assert!(find_floor_by_id(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn insert_room_rejects_a_nonexistent_floor() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());

        let result = insert_room(&conn, &sample_new_room(999));

        assert!(result.is_err());
    }

    #[test]
    fn insert_room_rejects_an_invalid_room_type() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let floor_id = insert_floor(&conn, &sample_new_floor()).unwrap();

        let result = insert_room(
            &conn,
            &NewRoom {
                room_type: "not-a-real-type",
                ..sample_new_room(floor_id)
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn insert_then_find_bed_by_id_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let floor_id = insert_floor(&conn, &sample_new_floor()).unwrap();
        let room_id = insert_room(&conn, &sample_new_room(floor_id)).unwrap();

        let bed_id = insert_bed(
            &conn,
            &NewBed {
                room_id,
                label: "Bed 3A",
            },
        )
        .unwrap();
        let found = find_bed_by_id(&conn, bed_id).unwrap().unwrap();

        assert_eq!(found.label, "Bed 3A");
        assert_eq!(found.status, "available");
    }

    #[test]
    fn update_bed_status_persists_the_new_status() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let floor_id = insert_floor(&conn, &sample_new_floor()).unwrap();
        let room_id = insert_room(&conn, &sample_new_room(floor_id)).unwrap();
        let bed_id = insert_bed(
            &conn,
            &NewBed {
                room_id,
                label: "Bed 3A",
            },
        )
        .unwrap();

        let updated = update_bed_status(&conn, bed_id, "maintenance")
            .unwrap()
            .unwrap();

        assert_eq!(updated.status, "maintenance");
        assert!(updated.updated_at.is_some());
    }

    #[test]
    fn update_bed_status_rejects_an_invalid_status() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let floor_id = insert_floor(&conn, &sample_new_floor()).unwrap();
        let room_id = insert_room(&conn, &sample_new_room(floor_id)).unwrap();
        let bed_id = insert_bed(
            &conn,
            &NewBed {
                room_id,
                label: "Bed 3A",
            },
        )
        .unwrap();

        let result = update_bed_status(&conn, bed_id, "on-fire");

        assert!(result.is_err());
    }

    #[test]
    fn list_floors_with_rooms_nests_each_floors_rooms() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let floor_id = insert_floor(&conn, &sample_new_floor()).unwrap();
        insert_room(&conn, &sample_new_room(floor_id)).unwrap();
        insert_room(
            &conn,
            &NewRoom {
                name: "Ward B",
                ..sample_new_room(floor_id)
            },
        )
        .unwrap();

        let layout = list_floors_with_rooms(&conn).unwrap();

        assert_eq!(layout.len(), 1);
        assert_eq!(layout[0].rooms.len(), 2);
    }

    #[test]
    fn get_room_status_returns_none_when_room_absent() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        assert!(get_room_status(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn get_room_status_aggregates_bed_occupancy() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let floor_id = insert_floor(&conn, &sample_new_floor()).unwrap();
        let room_id = insert_room(&conn, &sample_new_room(floor_id)).unwrap();
        let bed_id = insert_bed(
            &conn,
            &NewBed {
                room_id,
                label: "Bed 1",
            },
        )
        .unwrap();
        insert_bed(
            &conn,
            &NewBed {
                room_id,
                label: "Bed 2",
            },
        )
        .unwrap();
        update_bed_status(&conn, bed_id, "maintenance").unwrap();

        let status = get_room_status(&conn, room_id).unwrap().unwrap();

        assert_eq!(status.beds.len(), 2);
        assert_eq!(status.available_count, 1);
        assert_eq!(status.maintenance_count, 1);
        assert_eq!(status.occupied_count, 0);
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
    fn insert_then_find_active_assignment_round_trips() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let floor_id = insert_floor(&conn, &sample_new_floor()).unwrap();
        let room_id = insert_room(&conn, &sample_new_room(floor_id)).unwrap();
        let bed_id = insert_bed(
            &conn,
            &NewBed {
                room_id,
                label: "Bed 1",
            },
        )
        .unwrap();

        let assignment_id = insert_assignment(
            &conn,
            &NewAssignment {
                bed_id,
                patient_id,
                encounter_id,
                assigned_by_user_id: user_id,
            },
        )
        .unwrap();

        let active = find_active_assignment_by_bed_id(&conn, bed_id)
            .unwrap()
            .unwrap();
        assert_eq!(active.id, assignment_id);
        assert!(active.released_at.is_none());
    }

    #[test]
    fn release_assignment_sets_released_at() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let floor_id = insert_floor(&conn, &sample_new_floor()).unwrap();
        let room_id = insert_room(&conn, &sample_new_room(floor_id)).unwrap();
        let bed_id = insert_bed(
            &conn,
            &NewBed {
                room_id,
                label: "Bed 1",
            },
        )
        .unwrap();
        let assignment_id = insert_assignment(
            &conn,
            &NewAssignment {
                bed_id,
                patient_id,
                encounter_id,
                assigned_by_user_id: user_id,
            },
        )
        .unwrap();

        let released = release_assignment(&conn, assignment_id).unwrap().unwrap();

        assert!(released.released_at.is_some());
        assert!(find_active_assignment_by_bed_id(&conn, bed_id)
            .unwrap()
            .is_none());
    }

    #[test]
    fn list_beds_with_active_assignment_reflects_the_active_row_only() {
        let dir = tempdir().unwrap();
        let conn = open_migrated(dir.path());
        let user_id = seed_user(&conn);
        let patient_id = seed_patient(&conn);
        let encounter_id = seed_encounter(&conn, patient_id, user_id);
        let floor_id = insert_floor(&conn, &sample_new_floor()).unwrap();
        let room_id = insert_room(&conn, &sample_new_room(floor_id)).unwrap();
        let occupied_bed_id = insert_bed(
            &conn,
            &NewBed {
                room_id,
                label: "Bed 1",
            },
        )
        .unwrap();
        insert_bed(
            &conn,
            &NewBed {
                room_id,
                label: "Bed 2",
            },
        )
        .unwrap();
        insert_assignment(
            &conn,
            &NewAssignment {
                bed_id: occupied_bed_id,
                patient_id,
                encounter_id,
                assigned_by_user_id: user_id,
            },
        )
        .unwrap();

        let summaries = list_beds_with_active_assignment(&conn, Some(room_id)).unwrap();

        assert_eq!(summaries.len(), 2);
        let occupied = summaries
            .iter()
            .find(|summary| summary.bed.id == occupied_bed_id)
            .unwrap();
        assert!(occupied.active_assignment.is_some());
        let free = summaries
            .iter()
            .find(|summary| summary.bed.id != occupied_bed_id)
            .unwrap();
        assert!(free.active_assignment.is_none());
    }
}
