use rusqlite::{Connection, Result, params};
use bcrypt::{hash, DEFAULT_COST};
use log::info;

pub fn initialize(db_path: &str, admin_email: &str, admin_password: &str) -> Result<Connection> {
    // Ensure data directory exists
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        std::fs::create_dir_all(parent).expect("Failed to create data directory");
    }

    let conn = Connection::open(db_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    create_tables(&conn)?;
    migrate_existing_db(&conn)?;
    seed_settings(&conn)?;
    seed_faqs(&conn)?;
    seed_reviews(&conn)?;
    seed_admin(&conn, admin_email, admin_password)?;

    info!("Database initialized at {}", db_path);
    Ok(conn)
}

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS customers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            first_name TEXT NOT NULL DEFAULT '',
            last_name TEXT NOT NULL DEFAULT '',
            phone TEXT NOT NULL DEFAULT '',
            street TEXT NOT NULL DEFAULT '',
            zip_code TEXT NOT NULL DEFAULT '',
            city TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            is_admin INTEGER NOT NULL DEFAULT 0,
            reset_token TEXT,
            reset_token_expires TEXT,
            email_verified INTEGER NOT NULL DEFAULT 0,
            verification_token TEXT,
            verification_token_expires TEXT,
            calendar_token TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS appointments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_id INTEGER NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'confirmed',
            appointment_type TEXT NOT NULL DEFAULT 'single',
            is_home_visit INTEGER NOT NULL DEFAULT 0,
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (customer_id) REFERENCES customers(id)
        );

        CREATE TABLE IF NOT EXISTS payments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_id INTEGER NOT NULL,
            appointment_id INTEGER,
            amount REAL NOT NULL,
            payment_type TEXT NOT NULL DEFAULT 'single',
            status TEXT NOT NULL DEFAULT 'pending',
            notes TEXT NOT NULL DEFAULT '',
            paid_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (customer_id) REFERENCES customers(id),
            FOREIGN KEY (appointment_id) REFERENCES appointments(id)
        );

        CREATE TABLE IF NOT EXISTS credit_packages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_id INTEGER NOT NULL,
            total_sessions INTEGER NOT NULL DEFAULT 10,
            used_sessions INTEGER NOT NULL DEFAULT 0,
            price_per_session REAL NOT NULL,
            valid_until TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (customer_id) REFERENCES customers(id)
        );

        CREATE TABLE IF NOT EXISTS faqs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question TEXT NOT NULL,
            answer_html TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS reviews (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            author_name TEXT NOT NULL,
            author_location TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL,
            stars INTEGER NOT NULL DEFAULT 5,
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS site_settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            setting_key TEXT NOT NULL UNIQUE,
            setting_value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
    ")?;
    Ok(())
}

/// Migrate existing databases that were created before new columns were added.
fn migrate_existing_db(conn: &Connection) -> Result<()> {
    // SQLite does not support ALTER TABLE ADD COLUMN IF NOT EXISTS,
    // so we attempt each ALTER and ignore the error if column already exists.
    let migrations = vec![
        "ALTER TABLE customers ADD COLUMN email_verified INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE customers ADD COLUMN verification_token TEXT",
        "ALTER TABLE customers ADD COLUMN verification_token_expires TEXT",
        "ALTER TABLE customers ADD COLUMN calendar_token TEXT",
    ];
    for sql in migrations {
        let _ = conn.execute_batch(sql); // ignore "duplicate column" error
    }
    Ok(())
}

fn seed_settings(conn: &Connection) -> Result<()> {
    let defaults = vec![
        ("price_single", "195"),
        ("price_pack", "169,90"),
        ("price_pack_count", "10"),
        ("home_visit_surcharge", "15"),
        ("appointment_duration_min", "90"),
        ("appointment_break_min", "0"),
        ("hours_weekday", "Mo–Fr 16:00–22:00 Uhr"),
        ("hours_saturday", "Sa 9:00–19:00 Uhr"),
        ("hours_stripe", "Mo–Fr 16:00–22:00 Uhr · Sa 9:00–19:00 Uhr"),
        ("hours_stripe_short", "Mo–Fr 16–22 Uhr · Sa 9–19 Uhr"),
        ("phone", "+49 152 34 00 72 25"),
        ("phone_raw", "+4915234007225"),
        ("email_contact", "termin@faszienbehandlung.jetzt"),
        ("address_street", "Sulgauer Straße 24"),
        ("address_zip", "78713"),
        ("address_city", "Sulgen"),
        ("cancellation_hours", "24"),
        ("nextcloud_caldav_url", ""),
        ("nextcloud_caldav_username", ""),
        ("nextcloud_caldav_password", ""),
    ];
    for (key, value) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO site_settings (setting_key, setting_value) VALUES (?1, ?2)",
            params![key, value],
        )?;
    }
    Ok(())
}

fn seed_faqs(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM faqs", [], |row| row.get(0))?;
    if count > 0 { return Ok(()); }

    let faqs = vec![
        (1, "Was ist die Gantke® Faszienbehandlung genau?",
         "Die Gantke® Methode ist eine sanfte, manuelle Faszientherapie, entwickelt von Robert Gantke. Durch präzise Grifftechniken werden Verklebungen und Fehlspannungen im Bindegewebe (Faszien) gelöst. Vollständig schmerzfrei, ca. 90 Minuten, ohne Geräte – ganzheitlich vom Fuß bis zum Kopf."),
        (2, "Für wen ist die Faszienbehandlung geeignet?",
         "Für alle Altersgruppen. Besonders wirksam bei chronischen Rückenschmerzen, Nackenverspannungen, Migräne, Gelenkbeschwerden und nach Verletzungen.<br><br><strong>Nicht geeignet bei:</strong> akuten Entzündungen, Fieber, frischen Operationswunden oder Thrombosen."),
        (3, "Ist die Behandlung schmerzhaft?",
         "In der Regel vollständig schmerzfrei. Du kannst ein angenehmes Ziehen oder Druckgefühl spüren – viele empfinden die Sitzung als tief entspannend. Sollte etwas unangenehm sein, passe ich den Druck sofort an."),
        (4, "Was sollte ich vor der Behandlung beachten?",
         "Bitte trinke ausreichend Wasser (mind. 1,5 Liter am Behandlungstag) und komme in bequemer Kleidung – Leggings oder lockere Hose mit T-Shirt. Schwere Mahlzeiten direkt vorher vermeiden."),
        (5, "Wie viele Termine sind nötig?",
         "Sehr individuell. Viele spüren nach der ersten Sitzung bereits Verbesserung. Bei chronischen Beschwerden empfehle ich 3–6 Behandlungen im Abstand von 1–3 Wochen."),
        (6, "Übernimmt die Krankenkasse die Kosten?",
         "Gesetzliche Krankenkassen übernehmen die Kosten in der Regel nicht. Private Zusatzversicherungen erstatten häufig einen Teil. Eine Rechnung stelle ich gerne aus."),
        (7, "Wo findet die Behandlung statt – und wie komme ich hin?",
         "<strong>Praxis:</strong> Sulgauer Straße 24, 78713 Sulgen. Kostenlose Parkplätze direkt vor dem Haus. Auto: Dunningen ca. 3 Min., Schramberg ca. 8 Min., Rottweil ca. 15 Min.<br><br>Hausbesuche in Sulgen, Dunningen, Schramberg und Villingendorf nach Absprache – Fahrtpauschale +15 €."),
        (8, "Was ist der Unterschied zur klassischen Massage?",
         "Klassische Massage behandelt primär die Muskulatur. Die Gantke® Methode zielt gezielt auf das fasziale Bindegewebsnetz und korrigiert strukturelle Fehlspannungen im gesamten Körper – nicht nur lokal. Die Wirkung ist oft nachhaltiger und langfristiger."),
    ];
    for (order, question, answer) in faqs {
        conn.execute(
            "INSERT INTO faqs (question, answer_html, sort_order) VALUES (?1, ?2, ?3)",
            params![question, answer, order],
        )?;
    }
    Ok(())
}

fn seed_reviews(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM reviews", [], |row| row.get(0))?;
    if count > 0 { return Ok(()); }

    let reviews = vec![
        (1, "Sandra M.", "Schramberg", "Nach nur drei Behandlungen sind meine chronischen Nackenschmerzen, die mich jahrelang gequält haben, fast vollständig verschwunden. Einfach unglaublich!", 5),
        (2, "Klaus W.", "Sulgen", "Thilo nimmt sich wirklich Zeit und erklärt genau, was er macht. Ich fühle mich nach jeder Sitzung wie neu – entspannt und so viel beweglicher.", 5),
        (3, "Maria K.", "Dunningen", "Meine Knieschmerzen haben sich nach der Behandlungsserie deutlich gebessert. Ich kann wieder Sport machen – sehr empfehlenswert!", 5),
    ];
    for (order, name, location, content, stars) in reviews {
        conn.execute(
            "INSERT INTO reviews (author_name, author_location, content, stars, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, location, content, stars, order],
        )?;
    }
    Ok(())
}

fn seed_admin(conn: &Connection, email: &str, password: &str) -> Result<()> {
    let existing: Option<(i64, String, String)> = conn.prepare(
        "SELECT id, email, password_hash FROM customers WHERE is_admin = 1"
    )?.query_row([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
    .ok();

    match existing {
        Some((id, existing_email, existing_hash)) => {
            if existing_email != email {
                conn.execute(
                    "UPDATE customers SET email = ?1, updated_at = datetime('now') WHERE id = ?2",
                    params![email, id],
                )?;
                info!("Admin email updated to: {}", email);
            }
            if !bcrypt::verify(password, &existing_hash).unwrap_or(false) {
                let new_hash = hash(password, DEFAULT_COST).expect("Failed to hash admin password");
                conn.execute(
                    "UPDATE customers SET password_hash = ?1, updated_at = datetime('now') WHERE id = ?2",
                    params![new_hash, id],
                )?;
                info!("Admin password updated for: {}", email);
            }
            // Ensure admin is verified and has a calendar token
            conn.execute(
                "UPDATE customers SET email_verified = 1, calendar_token = COALESCE(calendar_token, lower(hex(randomblob(16)))) WHERE id = ?1",
                params![id],
            )?;
        }
        None => {
            let password_hash = hash(password, DEFAULT_COST).expect("Failed to hash admin password");
            conn.execute(
                "INSERT INTO customers (email, password_hash, first_name, last_name, is_admin, email_verified, calendar_token) VALUES (?1, ?2, 'Admin', 'Admin', 1, 1, lower(hex(randomblob(16))))",
                params![email, password_hash],
            )?;
            info!("Admin account created with email: {}", email);
        }
    }
    Ok(())
}
