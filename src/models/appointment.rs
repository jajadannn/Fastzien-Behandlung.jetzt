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
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, start_time, end_time, status, appointment_type, is_home_visit, notes, therapist_notes, reminder_sent, review_reminder_sent, created_at FROM appointments WHERE customer_id = ?1 AND status = 'confirmed' AND start_time > datetime('now') ORDER BY start_time ASC"
        )?;
        let rows = stmt.query_map(params![customer_id], Self::from_row)?;
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
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.therapist_notes, a.reminder_sent, a.review_reminder_sent, a.created_at, c.first_name, c.last_name, c.email, c.phone FROM appointments a JOIN customers c ON a.customer_id = c.id WHERE a.status = 'confirmed' AND a.start_time > datetime('now') ORDER BY a.start_time ASC"
        )?;
        let rows = stmt.query_map([], Self::from_row_with_customer)?;
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

    /// Find appointments starting in the next 24–25h that haven't had a reminder sent yet.
    pub fn find_needing_reminders(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.therapist_notes, a.reminder_sent, a.review_reminder_sent, a.created_at, c.first_name, c.last_name, c.email, c.phone FROM appointments a JOIN customers c ON a.customer_id = c.id WHERE a.status = 'confirmed' AND a.reminder_sent = 0 AND a.start_time > datetime('now', '+23 hours') AND a.start_time <= datetime('now', '+25 hours')"
        )?;
        let rows = stmt.query_map([], Self::from_row_with_customer)?;
        rows.collect()
    }

    /// Find completed appointments that need a review reminder (finished > 2h ago, review not sent yet).
    pub fn find_needing_review_reminders(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.therapist_notes, a.reminder_sent, a.review_reminder_sent, a.created_at, c.first_name, c.last_name, c.email, c.phone FROM appointments a JOIN customers c ON a.customer_id = c.id WHERE a.status = 'confirmed' AND a.review_reminder_sent = 0 AND a.appointment_type != 'blocked' AND a.end_time <= datetime('now', '-2 hours') AND a.end_time >= datetime('now', '-48 hours')"
        )?;
        let rows = stmt.query_map([], Self::from_row_with_customer)?;
        rows.collect()
    }

    pub fn mark_reminder_sent(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("UPDATE appointments SET reminder_sent = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn mark_review_reminder_sent(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("UPDATE appointments SET review_reminder_sent = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_therapist_notes(conn: &Connection, id: i64, notes: &str) -> Result<()> {
        conn.execute("UPDATE appointments SET therapist_notes = ?1 WHERE id = ?2", params![notes, id])?;
        Ok(())
    }

    pub fn cancel(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE appointments SET status = 'cancelled' WHERE id = ?1",
            params![id],
        )?;
        Ok(())
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
