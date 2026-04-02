use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result, params, Row};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Appointment {
    pub id: i64,
    pub customer_id: i64,
    pub start_time: String,
    pub end_time: String,
    pub status: String,        // confirmed, cancelled, completed
    pub appointment_type: String, // single, pack
    pub is_home_visit: bool,
    pub notes: String,
    pub created_at: String,
    // joined fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookingForm {
    pub date: String,
    pub time: String,
    pub is_home_visit: Option<bool>,
    pub notes: Option<String>,
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
            created_at: row.get(8)?,
            customer_name: None,
            customer_email: None,
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
            created_at: row.get(8)?,
            customer_name: Some(format!("{} {}", row.get::<_, String>(9)?, row.get::<_, String>(10)?)),
            customer_email: Some(row.get(11)?),
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
            "SELECT id, customer_id, start_time, end_time, status, appointment_type, is_home_visit, notes, created_at FROM appointments WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::from_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn find_by_customer(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, start_time, end_time, status, appointment_type, is_home_visit, notes, created_at FROM appointments WHERE customer_id = ?1 ORDER BY start_time DESC"
        )?;
        let rows = stmt.query_map(params![customer_id], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn find_upcoming_by_customer(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, start_time, end_time, status, appointment_type, is_home_visit, notes, created_at FROM appointments WHERE customer_id = ?1 AND status = 'confirmed' AND start_time > datetime('now') ORDER BY start_time ASC"
        )?;
        let rows = stmt.query_map(params![customer_id], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn find_all_with_customer(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.created_at, c.first_name, c.last_name, c.email FROM appointments a JOIN customers c ON a.customer_id = c.id ORDER BY a.start_time DESC"
        )?;
        let rows = stmt.query_map([], |row| Self::from_row_with_customer(row))?;
        rows.collect()
    }

    pub fn find_upcoming_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.created_at, c.first_name, c.last_name, c.email FROM appointments a JOIN customers c ON a.customer_id = c.id WHERE a.status = 'confirmed' AND a.start_time > datetime('now') ORDER BY a.start_time ASC"
        )?;
        let rows = stmt.query_map([], |row| Self::from_row_with_customer(row))?;
        rows.collect()
    }

    pub fn find_by_customer_with_details(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.customer_id, a.start_time, a.end_time, a.status, a.appointment_type, a.is_home_visit, a.notes, a.created_at, c.first_name, c.last_name, c.email FROM appointments a JOIN customers c ON a.customer_id = c.id WHERE a.customer_id = ?1 ORDER BY a.start_time DESC"
        )?;
        let rows = stmt.query_map(params![customer_id], |row| Self::from_row_with_customer(row))?;
        rows.collect()
    }

    pub fn cancel(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE appointments SET status = 'cancelled' WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

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
