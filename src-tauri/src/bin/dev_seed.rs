//! `dev-seed` — local-testing data generator (Rules.md 17.1 / Database.md "Migration Strategy").
//!
//! Never referenced by `main.rs`, `lib.rs`, or `invoke_handler` — this binary cannot ship with
//! the application and cannot run automatically. It opens the *real* SQLCipher database (same
//! key, same connection settings as production), applies the same embedded migrations `lib.rs`
//! runs on startup (so it also works against a brand-new, empty database file), and executes
//! plain `INSERT` statements directly against it, bypassing the command/service layers entirely,
//! per explicit user request.
//!
//! Usage: `cargo run --bin dev-seed -- <path-to-health.db> [--force]`
//! Windows dev path: `%APPDATA%\com.healthproject.his\health.db`

use std::env;
use std::path::Path;
use std::process::ExitCode;

use health_project::db::{connection, migrator};
use health_project::security::{hashing, secrets};

const DEV_SEED_PASSWORD: &str = "Dev-Seed-2026!";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let Some(db_path) = args.get(1) else {
        eprintln!("usage: cargo run --bin dev-seed -- <path-to-health.db> [--force]");
        return ExitCode::FAILURE;
    };
    let force = args.iter().any(|arg| arg == "--force");

    if let Err(error) = run(Path::new(db_path), force) {
        eprintln!("dev-seed failed: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(db_path: &Path, force: bool) -> Result<(), String> {
    let key = secrets::get_or_create_db_key().map_err(|error| error.to_string())?;
    let mut conn = connection::open(db_path, &key).map_err(|error| error.to_string())?;
    migrator::run_migrations(&conn, migrator::embedded_migrations())
        .map_err(|error| error.to_string())?;

    let existing_patients: i64 = conn
        .query_row("SELECT COUNT(*) FROM patients", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if existing_patients > 0 && !force {
        return Err(
            "database is not empty — refusing to double-seed (pass --force to override)"
                .to_string(),
        );
    }

    let tx = conn.transaction().map_err(|error| error.to_string())?;
    seed::run(&tx).map_err(|error| error.to_string())?;
    tx.commit().map_err(|error| error.to_string())?;

    println!("dev-seed: completed successfully.");
    println!("dev-seed: every seeded user's password is \"{DEV_SEED_PASSWORD}\".");
    Ok(())
}

mod seed {
    use rusqlite::{Connection as RusqliteConnection, Result as SqlResult};

    use super::{hashing, DEV_SEED_PASSWORD};

    const PHYSICIAN_NAMES: [&str; 14] = [
        "Ada Lovelace",
        "Grace Hopper",
        "Alan Turing",
        "Katherine Johnson",
        "John von Neumann",
        "Radia Perlman",
        "Edsger Dijkstra",
        "Barbara Liskov",
        "Donald Knuth",
        "Margaret Hamilton",
        "Vint Cerf",
        "Frances Allen",
        "Tim Berners-Lee",
        "Shafi Goldwasser",
    ];
    const NURSE_NAMES: [&str; 4] = [
        "Mary Seacole",
        "Clara Barton",
        "Linda Richards",
        "Dorothea Dix",
    ];
    const ADMIN_NAMES: [&str; 2] = ["Elena Rios", "Marcus Webb"];
    const PATIENT_NAMES: [&str; 20] = [
        "Liam Carter",
        "Olivia Bennett",
        "Noah Ramirez",
        "Emma Sullivan",
        "Ethan Brooks",
        "Sophia Reyes",
        "Mason Coleman",
        "Isabella Ortiz",
        "James Whitfield",
        "Mia Alvarado",
        "Benjamin Kruger",
        "Charlotte Nguyen",
        "Lucas Ferreira",
        "Amelia Osei",
        "Henry Blackwood",
        "Harper Delgado",
        "Sebastian Voss",
        "Ella Marchetti",
        "Jack Donnelly",
        "Grace Kimani",
    ];

    pub fn run(tx: &RusqliteConnection) -> SqlResult<()> {
        let physician_ids = seed_users(tx, &PHYSICIAN_NAMES, "physician", "phys")?;
        let nurse_ids = seed_users(tx, &NURSE_NAMES, "nurse", "nurse")?;
        let admin_ids = seed_users(tx, &ADMIN_NAMES, "admin", "admin")?;
        seed_users(tx, &["Priya Nair"], "pharmacy", "pharm")?;
        seed_users(tx, &["Oscar Lindqvist"], "lab", "lab")?;
        seed_users(tx, &["Nadia Farouk"], "receptionist", "front")?;

        let staff_id = physician_ids[0];
        let patient_ids = seed_patients(tx)?;
        let (bed_ids, or_room_ids) = seed_facility(tx)?;
        let encounter_ids = seed_encounters(tx, &patient_ids, &physician_ids)?;
        seed_medical_history(tx, &encounter_ids, &physician_ids, &nurse_ids)?;
        seed_bed_assignments(tx, &bed_ids, &patient_ids, &encounter_ids, &nurse_ids)?;
        let or_ids = seed_operating_rooms(tx, &or_room_ids)?;
        seed_or_reservations(tx, &or_ids, &patient_ids, &encounter_ids, &physician_ids)?;
        let item_ids = seed_inventory(tx, &admin_ids)?;
        seed_maintenance_schedules(tx, &item_ids)?;
        seed_notifications(tx, &item_ids)?;
        seed_billing(tx, &encounter_ids, &staff_id)?;

        println!(
            "dev-seed: seeded {} patients, {} encounters.",
            patient_ids.len(),
            encounter_ids.len()
        );
        Ok(())
    }

    fn seed_users(
        tx: &RusqliteConnection,
        names: &[&str],
        role: &str,
        username_prefix: &str,
    ) -> SqlResult<Vec<i64>> {
        let hash = hashing::hash_password(DEV_SEED_PASSWORD)
            .unwrap_or_else(|_| "$argon2id$invalid$".to_string());
        let mut ids = Vec::with_capacity(names.len());
        for (index, full_name) in names.iter().enumerate() {
            let username = format!("{username_prefix}.{}", index + 1);
            tx.execute(
                "INSERT INTO users (full_name, username, password_hash, role, is_active) \
                 VALUES (?1, ?2, ?3, ?4, 1)",
                rusqlite::params![full_name, username, hash, role],
            )?;
            ids.push(tx.last_insert_rowid());
        }
        Ok(ids)
    }

    fn seed_patients(tx: &RusqliteConnection) -> SqlResult<Vec<i64>> {
        let mut ids = Vec::with_capacity(PATIENT_NAMES.len());
        for (index, full_name) in PATIENT_NAMES.iter().enumerate() {
            let mrn = format!("MRN-{:04}", index + 1);
            let sex = ["male", "female", "other", "unknown"][index % 4];
            let birth_year = 1950 + (index * 3) % 70;
            let dob = format!("{birth_year}-0{}-1{}", 1 + index % 9, index % 9);
            let blood_type = if index % 5 == 0 {
                None
            } else {
                Some(["O+", "A+", "B+", "AB+", "O-"][index % 5])
            };
            let allergies = if index % 4 == 0 {
                Some("Penicillin")
            } else {
                None
            };
            tx.execute(
                "INSERT INTO patients (medical_record_number, full_name, date_of_birth, sex, \
                 phone, blood_type, allergies) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    mrn,
                    full_name,
                    dob,
                    sex,
                    format!("555-01{index:02}"),
                    blood_type,
                    allergies,
                ],
            )?;
            ids.push(tx.last_insert_rowid());
        }
        Ok(ids)
    }

    /// Returns (`ward_bed_ids`, `operating_room_room_ids`).
    fn seed_facility(tx: &RusqliteConnection) -> SqlResult<(Vec<i64>, Vec<i64>)> {
        let floor_names = [
            "Ground Floor",
            "First Floor",
            "Second Floor",
            "Third Floor",
            "Fourth Floor",
        ];
        let mut floor_ids = Vec::with_capacity(floor_names.len());
        for (index, name) in floor_names.iter().enumerate() {
            tx.execute(
                "INSERT INTO floors (name, level_order) VALUES (?1, ?2)",
                rusqlite::params![name, index as i64],
            )?;
            floor_ids.push(tx.last_insert_rowid());
        }

        let mut bed_ids = Vec::new();
        let mut or_room_ids = Vec::new();
        let mut room_counter = 0;
        for (floor_index, floor_id) in floor_ids.iter().enumerate() {
            for slot in 0..6 {
                room_counter += 1;
                let room_type = if floor_index == 0 && slot < 2 {
                    "operating_room"
                } else if floor_index == 0 && slot == 2 {
                    "pharmacy"
                } else if floor_index == 0 && slot == 3 {
                    "laboratory"
                } else {
                    "ward"
                };
                // Rooms are laid out on a 3-column x 2-row grid within each floor's own
                // 0..1 map, with margins so room markers near the edges aren't clipped.
                let col = f64::from(slot % 3);
                let row = f64::from(slot / 3);
                let map_x = 0.2 + col * 0.3;
                let map_y = 0.25 + row * 0.5;
                tx.execute(
                    "INSERT INTO rooms (floor_id, name, room_type, map_x, map_y) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        floor_id,
                        format!("Room {room_counter}"),
                        room_type,
                        map_x,
                        map_y
                    ],
                )?;
                let room_id = tx.last_insert_rowid();

                if room_type == "operating_room" {
                    or_room_ids.push(room_id);
                } else if room_type == "ward" {
                    for bed_slot in 0..2 {
                        let label = format!("Bed {room_counter}{}", (b'A' + bed_slot) as char);
                        let status = if bed_ids.len() % 6 == 0 {
                            "maintenance"
                        } else {
                            "available"
                        };
                        tx.execute(
                            "INSERT INTO beds (room_id, label, status) VALUES (?1, ?2, ?3)",
                            rusqlite::params![room_id, label, status],
                        )?;
                        bed_ids.push(tx.last_insert_rowid());
                    }
                }
            }
        }
        Ok((bed_ids, or_room_ids))
    }

    fn seed_encounters(
        tx: &RusqliteConnection,
        patient_ids: &[i64],
        physician_ids: &[i64],
    ) -> SqlResult<Vec<i64>> {
        let mut ids = Vec::with_capacity(patient_ids.len());
        for (index, patient_id) in patient_ids.iter().enumerate() {
            let physician_id = physician_ids[index % physician_ids.len()];
            let discharged = index % 5 == 0;
            if discharged {
                tx.execute(
                    "INSERT INTO encounters (patient_id, status, discharged_at, discharge_summary, \
                     created_by_user_id) VALUES (?1, 'discharged', datetime('now'), ?2, ?3)",
                    rusqlite::params![
                        patient_id,
                        "Recovered without complications; follow-up in two weeks.",
                        physician_id,
                    ],
                )?;
            } else {
                tx.execute(
                    "INSERT INTO encounters (patient_id, status, created_by_user_id) \
                     VALUES (?1, 'open', ?2)",
                    rusqlite::params![patient_id, physician_id],
                )?;
            }
            ids.push(tx.last_insert_rowid());
        }
        Ok(ids)
    }

    fn seed_medical_history(
        tx: &RusqliteConnection,
        encounter_ids: &[i64],
        physician_ids: &[i64],
        nurse_ids: &[i64],
    ) -> SqlResult<()> {
        let descriptions = [
            "Acute bronchitis",
            "Type 2 diabetes mellitus",
            "Hypertension",
            "Appendicitis",
            "Fractured radius",
        ];
        for (index, encounter_id) in encounter_ids.iter().enumerate() {
            let physician_id = physician_ids[index % physician_ids.len()];
            let nurse_id = nurse_ids[index % nurse_ids.len()];
            let description = descriptions[index % descriptions.len()];

            tx.execute(
                "INSERT INTO diagnoses (encounter_id, description, icd_code, registered_by_user_id) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![encounter_id, description, format!("ICD-{:03}", 100 + index), physician_id],
            )?;
            let diagnosis_id = tx.last_insert_rowid();

            tx.execute(
                "INSERT INTO treatments (encounter_id, diagnosis_id, description, dosage, \
                 registered_by_user_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    encounter_id,
                    diagnosis_id,
                    format!("Treatment plan for {description}"),
                    "500mg every 8 hours",
                    physician_id,
                ],
            )?;

            tx.execute(
                "INSERT INTO evolutions (encounter_id, note, registered_by_user_id) \
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    encounter_id,
                    "Patient stable, responding well to treatment.",
                    nurse_id,
                ],
            )?;
        }
        Ok(())
    }

    fn seed_bed_assignments(
        tx: &RusqliteConnection,
        bed_ids: &[i64],
        patient_ids: &[i64],
        encounter_ids: &[i64],
        nurse_ids: &[i64],
    ) -> SqlResult<()> {
        let mut bed_index = 0;
        for (index, encounter_id) in encounter_ids.iter().enumerate() {
            let is_open: i64 = tx.query_row(
                "SELECT CASE WHEN status = 'open' THEN 1 ELSE 0 END FROM encounters WHERE id = ?1",
                [encounter_id],
                |row| row.get(0),
            )?;
            if is_open == 0 || bed_index >= bed_ids.len() {
                continue;
            }
            let bed_id = bed_ids[bed_index];
            bed_index += 1;
            let nurse_id = nurse_ids[index % nurse_ids.len()];
            tx.execute(
                "INSERT INTO bed_assignments (bed_id, patient_id, encounter_id, assigned_by_user_id) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![bed_id, patient_ids[index], encounter_id, nurse_id],
            )?;
            tx.execute(
                "UPDATE beds SET status = 'occupied' WHERE id = ?1",
                [bed_id],
            )?;
        }
        Ok(())
    }

    fn seed_operating_rooms(tx: &RusqliteConnection, or_room_ids: &[i64]) -> SqlResult<Vec<i64>> {
        let mut ids = Vec::with_capacity(or_room_ids.len());
        for (index, room_id) in or_room_ids.iter().enumerate() {
            tx.execute(
                "INSERT INTO operating_rooms (room_id, name) VALUES (?1, ?2)",
                rusqlite::params![room_id, format!("OR {}", index + 1)],
            )?;
            ids.push(tx.last_insert_rowid());
        }
        Ok(ids)
    }

    fn seed_or_reservations(
        tx: &RusqliteConnection,
        or_ids: &[i64],
        patient_ids: &[i64],
        encounter_ids: &[i64],
        physician_ids: &[i64],
    ) -> SqlResult<()> {
        for index in 0..20usize {
            let or_id = or_ids[index % or_ids.len()];
            let patient_id = patient_ids[index % patient_ids.len()];
            let encounter_id = encounter_ids[index % encounter_ids.len()];
            let physician_id = physician_ids[index % physician_ids.len()];
            // Stagger reservations two hours apart per OR so same-OR slots never overlap.
            let slot = index / or_ids.len();
            let day = 1 + slot / 12;
            let hour = (slot % 12) * 2;
            let start = format!("2026-10-{day:02} {hour:02}:00:00");
            let end = format!("2026-10-{day:02} {:02}:00:00", hour + 1);
            let status = ["scheduled", "completed", "cancelled"][index % 3];
            tx.execute(
                "INSERT INTO or_reservations (operating_room_id, patient_id, encounter_id, \
                 procedure_description, scheduled_start, scheduled_end, status, \
                 scheduled_by_user_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    or_id,
                    patient_id,
                    encounter_id,
                    "Exploratory procedure",
                    start,
                    end,
                    status,
                    physician_id,
                ],
            )?;
        }
        Ok(())
    }

    fn seed_inventory(tx: &RusqliteConnection, admin_ids: &[i64]) -> SqlResult<Vec<i64>> {
        let categories = [
            ("Medicines", "medicine"),
            ("Supplies", "supply"),
            ("Equipment", "equipment"),
        ];
        let mut category_ids = Vec::with_capacity(categories.len());
        for (name, kind) in categories {
            tx.execute(
                "INSERT INTO inventory_categories (name, kind) VALUES (?1, ?2)",
                rusqlite::params![name, kind],
            )?;
            category_ids.push(tx.last_insert_rowid());
        }

        let item_names = [
            "Amoxicillin 500mg",
            "Ibuprofen 200mg",
            "Saline Solution 1L",
            "Surgical Gloves (box)",
            "Syringe 5ml",
            "Gauze Roll",
            "Insulin 10ml",
            "Paracetamol 500mg",
            "Suture Kit",
            "IV Catheter",
            "Blood Pressure Cuff",
            "Pulse Oximeter",
            "Defibrillator Pad",
            "Surgical Mask (box)",
            "Bandage Roll",
            "Antiseptic Wipes (box)",
            "Oxygen Mask",
            "Thermometer",
            "Wheelchair",
            "Infusion Pump",
        ];
        let mut item_ids = Vec::with_capacity(item_names.len());
        for (index, name) in item_names.iter().enumerate() {
            let category_id = category_ids[index % category_ids.len()];
            let quantity = (10 + (index * 7) % 200) as i64;
            let reorder_threshold: i64 = 15;
            let expiration = if index % 6 == 0 {
                Some("2026-11-01")
            } else if index % 5 == 0 {
                None
            } else {
                Some("2027-06-01")
            };
            tx.execute(
                "INSERT INTO inventory_items (category_id, name, quantity, unit, \
                 reorder_threshold, expiration_date, location) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    category_id,
                    name,
                    quantity,
                    "units",
                    reorder_threshold,
                    expiration,
                    format!("Shelf {}", (index % 6) + 1),
                ],
            )?;
            let item_id = tx.last_insert_rowid();
            item_ids.push(item_id);

            let admin_id = admin_ids[index % admin_ids.len()];
            tx.execute(
                "INSERT INTO inventory_transactions (item_id, quantity_delta, reason, \
                 performed_by_user_id) VALUES (?1, ?2, 'restock', ?3)",
                rusqlite::params![item_id, quantity, admin_id],
            )?;
        }
        Ok(item_ids)
    }

    fn seed_maintenance_schedules(tx: &RusqliteConnection, item_ids: &[i64]) -> SqlResult<()> {
        for (index, item_id) in item_ids.iter().enumerate().take(5) {
            tx.execute(
                "INSERT INTO maintenance_schedules (inventory_item_id, scheduled_date, notes) \
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    item_id,
                    format!("2026-11-{:02}", 10 + index),
                    "Routine calibration check",
                ],
            )?;
        }
        Ok(())
    }

    fn seed_notifications(tx: &RusqliteConnection, item_ids: &[i64]) -> SqlResult<()> {
        let types = [
            "medicine_expiration",
            "maintenance_due",
            "low_stock",
            "or_schedule",
            "other",
        ];
        for index in 0..20usize {
            let notif_type = types[index % types.len()];
            let item_id = item_ids[index % item_ids.len()];
            let is_read = i64::from(index % 3 == 0);
            tx.execute(
                "INSERT INTO notifications (type, message, related_entity_type, \
                 related_entity_id, is_read) VALUES (?1, ?2, 'inventory_item', ?3, ?4)",
                rusqlite::params![
                    notif_type,
                    format!("{notif_type} alert for item #{item_id}"),
                    item_id,
                    is_read,
                ],
            )?;
        }
        Ok(())
    }

    fn seed_billing(
        tx: &RusqliteConnection,
        encounter_ids: &[i64],
        staff_id: &i64,
    ) -> SqlResult<()> {
        for encounter_id in encounter_ids {
            let is_discharged: i64 = tx.query_row(
                "SELECT CASE WHEN status = 'discharged' THEN 1 ELSE 0 END FROM encounters WHERE id = ?1",
                [encounter_id],
                |row| row.get(0),
            )?;
            if is_discharged == 0 {
                continue;
            }

            tx.execute(
                "INSERT INTO billing_simulations (encounter_id, status, generated_by_user_id) \
                 VALUES (?1, 'draft', ?2)",
                rusqlite::params![encounter_id, staff_id],
            )?;
            let simulation_id = tx.last_insert_rowid();

            let items: [(&str, &str, f64); 3] = [
                ("Ward room charge", "room", 150.0),
                ("Treatment plan", "treatment", 80.0),
                ("Consumed supplies", "inventory", 25.5),
            ];
            let mut total = 0.0;
            for (description, source, amount) in items {
                tx.execute(
                    "INSERT INTO billing_items (billing_simulation_id, description, source, amount) \
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![simulation_id, description, source, amount],
                )?;
                total += amount;
            }

            tx.execute(
                "UPDATE billing_simulations SET total_amount = ?1 WHERE id = ?2",
                rusqlite::params![total, simulation_id],
            )?;
        }
        Ok(())
    }
}
