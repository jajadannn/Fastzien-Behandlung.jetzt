use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result, params, Row};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Appointment {
    pub id: i64,
    pub customer_id: i64,
    pub start_time: String,
    pub end_time: String,
    pub status: String,        // confirmed, cancelled, completed
    pub appointment_type: String, // single, pack, blocked
    pub is_home_visit: bool,
    pub notes: String,
    pub therapist_notes: String,
    pub reminder_sent: bool,
    pub review_reminder_sent: bool,
    pub created_at: String,
    // joined fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookingForm {
    pub date: String,
    pub time: String,
    pub is_home_visit: Option<bool>,
    pub notes: Option<String>,
    #[serde(default)] pub expected_total: Option<f64>,
    #[serde(default)] pub expected_duration: Option<i64>,
    #[serde(default)] pub early_start_requested: bool,
    #[serde(default)] pub expected_using_credit: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RescheduleForm {
    pub date: String,
    pub time: String,
}

// Waitlist entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaitlistEntry {
    pub id: i64,
    pub customer_id: i64,
    pub date: String,
    pub notified: bool,
    pub created_at: String,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
}

impl Appointment {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Appointment {
            id: row.get(0)?,
            customer_id: row.get(1)?,
            start_time: row.get(2)?,
            end_time: row.get(3)?,
            status: row.get(4)?,
            appointment_type: row.get(5)?,
            is_home_visit: row.get::<_, i32>(6)? != 0,
            notes: row.get(7)?,
            therapist_notes: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
            reminder_sent: row.get::<_, i32>(9)? != 0,
            review_reminder_sent: row.get::<_, i32>(10)? != 0,
            created_at: row.get(11)?,
            customer_name: None,
            customer_email: None,
            customer_phone: None,
        })
    }

    fn from_row_with_customer(row: &Row) -> Result<Self> {
        Ok(Appointment {
            id: row.get(0)?,
            customer_id: row.get(1)?,
            start_time: row.get(2)?,
            end_time: row.get(3)?,
            status: row.get(4)?,
            appointment_type: row.get(5)?,
            is_home_visit: row.get::<_, i32>(6)? != 0,
            notes: row.get(7)?,
            therapist_notes: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
            reminder_sent: row.get::<_, i32>(9)? != 0,
            review_reminder_sent: row.get::<_, i32>(10)? != 0,
            created_at: row.get(11)?,
            customer_name: Some(format!("{} {}", row.get::<_, String>(12)?, row.get::<_, String>(13)?)),
            customer_email: Some(row.get(14)?),
            customer_phone: row.get::<_, Option<String>>(15)?,
        })
    }

    pub fn create(conn: &Connection, customer_id: i64, start_time: &str, end_time: &str, appointment_type: &str, is_home_visit: bool, notes: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO appointments (customer_id, start_time, end_time, appointment_type, is_home_visit, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![customer_id, start_time, end_time, appointment_type, is_home_visit as i32, notes],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, start_time, end_time, status, appointment_type, is_home_visit, notes, therapist_notes, reminder_sent, review_reminder_sent, created_at FROM appointments WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::from_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn find_by_customer(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, start_time, end_time, status, appointment_type, is_home_visit, notes, therapist_notes, reminder_sent, review_reminder_sent, created_at FROM appointments WHERE customer_id = ?1 ORDER BY start_time DESC"
        )?;
        let rows = stmt.query_map(params![customer_id], Self::from_row)?;
        rows.collect()
    }

    pub fn find_upcoming_by_customer(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let now = chrono::Utc::now().with_timezone(&chrono_tz::Europe::Berlin).format("%Y-%m-%d %H:%M:%S").to_string();
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, start_time, end_time, status, appointment_type, is_home_visit, notes, therapist_notes, reminder_sent, review_reminder_sent, created_at FROM appointments WHERE customer_id = ?1 AND status = 'confirmed' AND start_time > ?2 ORDER BY start_time ASC"
        )?;
        let rows = stmt.query_map(params![customer_id, now], Self::from_row)?;
        rows.collect()
    }

    pub fn find_all_with_customer(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.therapist_notes, a.reminder_sent, a.review_reminder_sent, a.created_at, c.first_name, c.last_name, c.email, c.phone FROM appointments a JOIN customers c ON a.customer_id = c.id ORDER BY a.start_time DESC"
        )?;
        let rows = stmt.query_map([], Self::from_row_with_customer)?;
        rows.collect()
    }

    pub fn find_upcoming_all(conn: &Connection) -> Result<Vec<Self>> {
        let now = chrono::Utc::now().with_timezone(&chrono_tz::Europe::Berlin).format("%Y-%m-%d %H:%M:%S").to_string();
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.therapist_notes, a.reminder_sent, a.review_reminder_sent, a.created_at, c.first_name, c.last_name, c.email, c.phone FROM appointments a JOIN customers c ON a.customer_id = c.id WHERE a.status = 'confirmed' AND a.start_time > ?1 ORDER BY a.start_time ASC"
        )?;
        let rows = stmt.query_map([now], Self::from_row_with_customer)?;
        rows.collect()
    }

    #[allow(dead_code)]
    pub fn find_by_customer_with_details(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.therapist_notes, a.reminder_sent, a.review_reminder_sent, a.created_at, c.first_name, c.last_name, c.email, c.phone FROM appointments a JOIN customers c ON a.customer_id = c.id WHERE a.customer_id = ?1 ORDER BY a.start_time DESC"
        )?;
        let rows = stmt.query_map(params![customer_id], Self::from_row_with_customer)?;
        rows.collect()
    }

    /// Find appointments 23–25 real hours ahead. Stored appointment times are
    /// Berlin wall-clock time, so convert both UTC boundaries independently.
    pub fn find_needing_reminders(conn: &Connection) -> Result<Vec<Self>> {
        let now = chrono::Utc::now();
        let from = (now + chrono::Duration::hours(23)).with_timezone(&chrono_tz::Europe::Berlin).format("%Y-%m-%d %H:%M:%S").to_string();
        let until = (now + chrono::Duration::hours(25)).with_timezone(&chrono_tz::Europe::Berlin).format("%Y-%m-%d %H:%M:%S").to_string();
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.therapist_notes, a.reminder_sent, a.review_reminder_sent, a.created_at, c.first_name, c.last_name, c.email, c.phone FROM appointments a JOIN customers c ON a.customer_id = c.id WHERE a.status = 'confirmed' AND a.reminder_sent = 0 AND a.start_time > ?1 AND a.start_time <= ?2"
        )?;
        let rows = stmt.query_map(params![from, until], Self::from_row_with_customer)?;
        rows.collect()
    }

    pub fn mark_reminder_sent(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("UPDATE appointments SET reminder_sent = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_therapist_notes(conn: &Connection, id: i64, notes: &str) -> Result<()> {
        conn.execute("UPDATE appointments SET therapist_notes = ?1 WHERE id = ?2", params![notes, id])?;
        Ok(())
    }

    pub fn cancel(conn: &Connection, id: i64) -> Result<bool> {
        use rusqlite::OptionalExtension;
        let tx = rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate)?;
        let booking: Option<(i64, String, Option<i64>)> = tx.query_row(
            "SELECT customer_id, appointment_type, credit_package_id FROM appointments WHERE id = ?1 AND status = 'confirmed'",
            [id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).optional()?;
        let Some((customer_id, kind, credit_id)) = booking else { return Ok(false); };
        if kind == "pack" {
            let credit_id = match credit_id {
                Some(id) => id,
                None => {
                    // Historical bookings did not store their card. Only infer a
                    // card when there is exactly one possibility, even if exhausted.
                    let mut stmt = tx.prepare("SELECT id FROM credit_packages WHERE customer_id = ?1 AND used_sessions > 0")?;
                    let candidates: Vec<i64> = stmt.query_map([customer_id], |row| row.get(0))?.collect::<Result<_>>()?;
                    if candidates.len() != 1 { return Err(rusqlite::Error::InvalidQuery); }
                    candidates[0]
                }
            };
            let restored = tx.execute(
                "UPDATE credit_packages SET used_sessions = used_sessions - 1 WHERE id = ?1 AND customer_id = ?2 AND used_sessions > 0",
                params![credit_id, customer_id],
            )?;
            if restored != 1 { return Err(rusqlite::Error::InvalidQuery); }
            tx.execute("UPDATE appointments SET credit_package_id = ?1 WHERE id = ?2", params![credit_id, id])?;
        }
        tx.execute("UPDATE appointments SET status = 'cancelled' WHERE id = ?1", [id])?;
        tx.execute("UPDATE payments SET status = 'cancelled' WHERE appointment_id = ?1 AND status = 'pending'", [id])?;
        tx.commit()?;
        Ok(true)
    }

    #[allow(dead_code)]
    pub fn complete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE appointments SET status = 'completed' WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn is_slot_available(conn: &Connection, start_time: &str, end_time: &str) -> Result<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM appointments WHERE status = 'confirmed' AND ((start_time < ?2 AND end_time > ?1))",
            params![start_time, end_time],
            |row| row.get(0),
        )?;
        Ok(count == 0)
    }

    /// Move only the appointment version that was checked before an async calendar request.
    /// Keeping its identity also preserves invoices, credits and therapist notes.
    pub fn reschedule_if_unchanged(conn: &Connection, old: &Self, start: &str, end: &str) -> Result<bool> {
        let changed = conn.execute(
            "UPDATE appointments SET start_time = ?1, end_time = ?2, reminder_sent = 0
             WHERE id = ?3 AND status = 'confirmed' AND start_time = ?4 AND end_time = ?5
             AND NOT EXISTS (SELECT 1 FROM appointments other WHERE other.id != ?3
                 AND other.status = 'confirmed' AND other.start_time < ?2 AND other.end_time > ?1)",
            params![start, end, old.id, old.start_time, old.end_time],
        )?;
        Ok(changed == 1)
    }

    pub fn count_by_customer(conn: &Connection, customer_id: i64) -> Result<i64> {
        conn.query_row(
            "SELECT COUNT(*) FROM appointments WHERE customer_id = ?1 AND status != 'cancelled'",
            params![customer_id],
            |row| row.get(0),
        )
    }
}

// ============= Waitlist =============

impl WaitlistEntry {
    pub fn add(conn: &Connection, customer_id: i64, date: &str) -> Result<i64> {
        // Remove duplicate first
        let _ = conn.execute(
            "DELETE FROM waitlist WHERE customer_id = ?1 AND date = ?2",
            params![customer_id, date],
        );
        conn.execute(
            "INSERT INTO waitlist (customer_id, date) VALUES (?1, ?2)",
            params![customer_id, date],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn remove(conn: &Connection, customer_id: i64, date: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM waitlist WHERE customer_id = ?1 AND date = ?2",
            params![customer_id, date],
        )?;
        Ok(())
    }

    pub fn find_by_date(conn: &Connection, date: &str) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT w.id, w.customer_id, w.date, w.notified, w.created_at, c.first_name || ' ' || c.last_name, c.email FROM waitlist w JOIN customers c ON w.customer_id = c.id WHERE w.date = ?1 AND w.notified = 0"
        )?;
        let rows = stmt.query_map(params![date], |row| {
            Ok(WaitlistEntry {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                date: row.get(2)?,
                notified: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                customer_name: row.get(5)?,
                customer_email: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn find_by_customer(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, date, notified, created_at FROM waitlist WHERE customer_id = ?1 ORDER BY date ASC"
        )?;
        let rows = stmt.query_map(params![customer_id], |row| {
            Ok(WaitlistEntry {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                date: row.get(2)?,
                notified: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                customer_name: None,
                customer_email: None,
            })
        })?;
        rows.collect()
    }

    pub fn mark_notified(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("UPDATE waitlist SET notified = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_on_waitlist(conn: &Connection, customer_id: i64, date: &str) -> Result<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM waitlist WHERE customer_id = ?1 AND date = ?2",
            params![customer_id, date],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use crate::models::payment::{CreditPackage, Payment};

    fn fixture() -> (Connection, i64, i64) {
        let conn = crate::db::test_connection();
        conn.execute("INSERT INTO customers(email, password_hash) VALUES('booking@example.com', 'test')", []).unwrap();
        let customer = conn.last_insert_rowid();
        let appointment = Appointment::create(&conn, customer, "2030-01-07 16:00:00", "2030-01-07 17:30:00", "single", false, "Customer notes").unwrap();
        (conn, customer, appointment)
    }

    fn appointment_at(conn: &Connection, customer: i64, instant: chrono::DateTime<chrono::Utc>) -> i64 {
        let start = instant.with_timezone(&chrono_tz::Europe::Berlin).format("%Y-%m-%d %H:%M:%S").to_string();
        let end = (instant + chrono::Duration::minutes(90)).with_timezone(&chrono_tz::Europe::Berlin).format("%Y-%m-%d %H:%M:%S").to_string();
        Appointment::create(conn, customer, &start, &end, "single", false, "").unwrap()
    }

    #[test]
    fn upcoming_lists_exclude_recently_started_berlin_appointments() {
        let (conn, customer, original) = fixture();
        conn.execute("DELETE FROM appointments WHERE id=?1", [original]).unwrap();
        let now = chrono::Utc::now();
        appointment_at(&conn, customer, now - chrono::Duration::minutes(30));
        let future = appointment_at(&conn, customer, now + chrono::Duration::minutes(30));
        let personal = Appointment::find_upcoming_by_customer(&conn, customer).unwrap();
        let all = Appointment::find_upcoming_all(&conn).unwrap();
        assert_eq!(personal.iter().map(|a| a.id).collect::<Vec<_>>(), vec![future]);
        assert_eq!(all.iter().map(|a| a.id).collect::<Vec<_>>(), vec![future]);
    }

    #[test]
    fn reminders_use_real_hours_with_the_current_berlin_offset() {
        let (conn, customer, original) = fixture();
        conn.execute("DELETE FROM appointments WHERE id=?1", [original]).unwrap();
        let now = chrono::Utc::now();
        // A UTC string comparison incorrectly includes +22.5h and excludes
        // +24.5h at both Berlin offsets (+01:00 and +02:00).
        appointment_at(&conn, customer, now + chrono::Duration::minutes(22 * 60 + 30));
        let due = appointment_at(&conn, customer, now + chrono::Duration::minutes(24 * 60 + 30));
        appointment_at(&conn, customer, now + chrono::Duration::hours(26));
        let already_sent = appointment_at(&conn, customer, now + chrono::Duration::hours(24));
        Appointment::mark_reminder_sent(&conn, already_sent).unwrap();
        let reminders = Appointment::find_needing_reminders(&conn).unwrap();
        assert_eq!(reminders.iter().map(|a| a.id).collect::<Vec<_>>(), vec![due]);
    }

    #[test]
    fn reschedule_preserves_paid_invoice_notes_and_rejects_stale_requests() {
        let (conn, customer, id) = fixture();
        let payment = Payment::create(&conn, customer, Some(id), 123.0, "single").unwrap();
        Payment::mark_paid(&conn, payment).unwrap();
        conn.execute("UPDATE appointments SET therapist_notes='Private notes', reminder_sent=1 WHERE id=?1", [id]).unwrap();
        let old = Appointment::find_by_id(&conn, id).unwrap().unwrap();
        assert!(Appointment::reschedule_if_unchanged(&conn, &old, "2030-01-08 16:00:00", "2030-01-08 17:30:00").unwrap());
        assert!(!Appointment::reschedule_if_unchanged(&conn, &old, "2030-01-09 16:00:00", "2030-01-09 17:30:00").unwrap());
        let moved = Appointment::find_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(moved.therapist_notes, "Private notes");
        assert_eq!(moved.notes, "Customer notes");
        assert!(!moved.reminder_sent);
        assert_eq!(Appointment::find_by_customer(&conn, customer).unwrap().len(), 1);
        let payments = Payment::find_by_customer(&conn, customer).unwrap();
        assert_eq!(payments.len(), 1);
        assert_eq!(payments[0].status, "paid");
        assert_eq!(payments[0].amount, 123.0);
        assert_eq!(payments[0].appointment_id, Some(id));
    }

    #[test]
    fn reschedule_cannot_overwrite_cancellation_or_an_occupied_slot() {
        let (conn, customer, id) = fixture();
        let old = Appointment::find_by_id(&conn, id).unwrap().unwrap();
        Appointment::create(&conn, customer, "2030-01-08 16:00:00", "2030-01-08 17:30:00", "single", false, "").unwrap();
        assert!(!Appointment::reschedule_if_unchanged(&conn, &old, "2030-01-08 16:00:00", "2030-01-08 17:30:00").unwrap());
        assert!(Appointment::cancel(&conn, id).unwrap());
        assert!(!Appointment::reschedule_if_unchanged(&conn, &old, "2030-01-09 16:00:00", "2030-01-09 17:30:00").unwrap());
    }

    #[test]
    fn cancellation_restores_the_original_exhausted_card_once() {
        let (conn, customer, id) = fixture();
        let original = CreditPackage::create(&conn, customer, 1, 100.0, "2099-01-01").unwrap();
        let other = CreditPackage::create(&conn, customer, 10, 110.0, "2099-01-01").unwrap();
        CreditPackage::use_session(&conn, original).unwrap();
        CreditPackage::use_session(&conn, other).unwrap();
        conn.execute("UPDATE appointments SET appointment_type='pack', credit_package_id=?1 WHERE id=?2", params![original, id]).unwrap();
        let payment = Payment::create(&conn, customer, Some(id), 100.0, "pack").unwrap();
        assert!(Appointment::cancel(&conn, id).unwrap());
        assert!(!Appointment::cancel(&conn, id).unwrap());
        let packages = CreditPackage::find_all_by_customer(&conn, customer).unwrap();
        assert_eq!(packages.iter().find(|p| p.id == original).unwrap().used_sessions, 0);
        assert_eq!(packages.iter().find(|p| p.id == other).unwrap().used_sessions, 1);
        assert_eq!(Payment::find_by_id(&conn, payment).unwrap().unwrap().status, "cancelled");
    }

    #[test]
    fn ambiguous_legacy_card_cancellation_leaves_everything_unchanged() {
        let (conn, customer, id) = fixture();
        for _ in 0..2 {
            let card = CreditPackage::create(&conn, customer, 1, 100.0, "2099-01-01").unwrap();
            CreditPackage::use_session(&conn, card).unwrap();
        }
        conn.execute("UPDATE appointments SET appointment_type='pack' WHERE id=?1", [id]).unwrap();
        let payment = Payment::create(&conn, customer, Some(id), 100.0, "pack").unwrap();
        assert!(Appointment::cancel(&conn, id).is_err());
        assert_eq!(Appointment::find_by_id(&conn, id).unwrap().unwrap().status, "confirmed");
        assert_eq!(Payment::find_by_id(&conn, payment).unwrap().unwrap().status, "pending");
        assert!(CreditPackage::find_all_by_customer(&conn, customer).unwrap().iter().all(|p| p.used_sessions == 1));
    }
}
