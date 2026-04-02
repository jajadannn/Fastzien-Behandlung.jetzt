use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result, params, Row};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payment {
    pub id: i64,
    pub customer_id: i64,
    pub appointment_id: Option<i64>,
    pub amount: f64,
    pub payment_type: String,  // single, pack
    pub status: String,        // pending, paid
    pub notes: String,
    pub paid_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreditPackage {
    pub id: i64,
    pub customer_id: i64,
    pub total_sessions: i32,
    pub used_sessions: i32,
    pub price_per_session: f64,
    pub valid_until: String,
    pub created_at: String,
}

impl Payment {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Payment {
            id: row.get(0)?,
            customer_id: row.get(1)?,
            appointment_id: row.get(2)?,
            amount: row.get(3)?,
            payment_type: row.get(4)?,
            status: row.get(5)?,
            notes: row.get(6)?,
            paid_at: row.get(7)?,
            created_at: row.get(8)?,
        })
    }

    pub fn create(conn: &Connection, customer_id: i64, appointment_id: Option<i64>, amount: f64, payment_type: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO payments (customer_id, appointment_id, amount, payment_type) VALUES (?1, ?2, ?3, ?4)",
            params![customer_id, appointment_id, amount, payment_type],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn find_by_customer(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, appointment_id, amount, payment_type, status, notes, paid_at, created_at FROM payments WHERE customer_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![customer_id], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, appointment_id, amount, payment_type, status, notes, paid_at, created_at FROM payments ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn mark_paid(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE payments SET status = 'paid', paid_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn pending_total_by_customer(conn: &Connection, customer_id: i64) -> Result<f64> {
        conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM payments WHERE customer_id = ?1 AND status = 'pending'",
            params![customer_id],
            |row| row.get(0),
        )
    }

    pub fn paid_total_by_customer(conn: &Connection, customer_id: i64) -> Result<f64> {
        conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM payments WHERE customer_id = ?1 AND status = 'paid'",
            params![customer_id],
            |row| row.get(0),
        )
    }
}

impl CreditPackage {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(CreditPackage {
            id: row.get(0)?,
            customer_id: row.get(1)?,
            total_sessions: row.get(2)?,
            used_sessions: row.get(3)?,
            price_per_session: row.get(4)?,
            valid_until: row.get(5)?,
            created_at: row.get(6)?,
        })
    }

    pub fn find_active_by_customer(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, total_sessions, used_sessions, price_per_session, valid_until, created_at FROM credit_packages WHERE customer_id = ?1 AND used_sessions < total_sessions AND valid_until > datetime('now') ORDER BY created_at ASC"
        )?;
        let rows = stmt.query_map(params![customer_id], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn find_all_by_customer(conn: &Connection, customer_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, total_sessions, used_sessions, price_per_session, valid_until, created_at FROM credit_packages WHERE customer_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![customer_id], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn create(conn: &Connection, customer_id: i64, total_sessions: i32, price_per_session: f64, valid_until: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO credit_packages (customer_id, total_sessions, price_per_session, valid_until) VALUES (?1, ?2, ?3, ?4)",
            params![customer_id, total_sessions, price_per_session, valid_until],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn use_session(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE credit_packages SET used_sessions = used_sessions + 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn remaining_sessions(conn: &Connection, customer_id: i64) -> Result<i32> {
        conn.query_row(
            "SELECT COALESCE(SUM(total_sessions - used_sessions), 0) FROM credit_packages WHERE customer_id = ?1 AND used_sessions < total_sessions AND valid_until > datetime('now')",
            params![customer_id],
            |row| row.get(0),
        )
    }
}
