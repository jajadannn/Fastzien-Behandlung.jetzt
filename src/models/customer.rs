use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result, params, Row};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Customer {
    pub id: i64,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub street: String,
    pub zip_code: String,
    pub city: String,
    pub notes: String,
    pub is_admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ProfileUpdate {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub zip_code: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordChange {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailChange {
    pub new_email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordReset {
    pub new_password: String,
}

impl Customer {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Customer {
            id: row.get(0)?,
            email: row.get(1)?,
            password_hash: row.get(2)?,
            first_name: row.get(3)?,
            last_name: row.get(4)?,
            phone: row.get(5)?,
            street: row.get(6)?,
            zip_code: row.get(7)?,
            city: row.get(8)?,
            notes: row.get(9)?,
            is_admin: row.get::<_, i32>(10)? != 0,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }

    pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, password_hash, first_name, last_name, phone, street, zip_code, city, notes, is_admin, created_at, updated_at FROM customers WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::from_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn find_by_email(conn: &Connection, email: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, password_hash, first_name, last_name, phone, street, zip_code, city, notes, is_admin, created_at, updated_at FROM customers WHERE email = ?1"
        )?;
        let mut rows = stmt.query(params![email])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::from_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, password_hash, first_name, last_name, phone, street, zip_code, city, notes, is_admin, created_at, updated_at FROM customers ORDER BY last_name, first_name"
        )?;
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn find_all_non_admin(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, password_hash, first_name, last_name, phone, street, zip_code, city, notes, is_admin, created_at, updated_at FROM customers WHERE is_admin = 0 ORDER BY last_name, first_name"
        )?;
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn create(conn: &Connection, email: &str, password_hash: &str, first_name: &str, last_name: &str, phone: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO customers (email, password_hash, first_name, last_name, phone) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![email, password_hash, first_name, last_name, phone],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_profile(conn: &Connection, id: i64, first_name: &str, last_name: &str, phone: &str, street: &str, zip_code: &str, city: &str) -> Result<()> {
        conn.execute(
            "UPDATE customers SET first_name = ?1, last_name = ?2, phone = ?3, street = ?4, zip_code = ?5, city = ?6, updated_at = datetime('now') WHERE id = ?7",
            params![first_name, last_name, phone, street, zip_code, city, id],
        )?;
        Ok(())
    }

    pub fn update_password(conn: &Connection, id: i64, password_hash: &str) -> Result<()> {
        conn.execute(
            "UPDATE customers SET password_hash = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![password_hash, id],
        )?;
        Ok(())
    }

    pub fn update_email(conn: &Connection, id: i64, email: &str) -> Result<()> {
        conn.execute(
            "UPDATE customers SET email = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![email, id],
        )?;
        Ok(())
    }

    pub fn set_reset_token(conn: &Connection, id: i64, token: &str, expires: &str) -> Result<()> {
        conn.execute(
            "UPDATE customers SET reset_token = ?1, reset_token_expires = ?2 WHERE id = ?3",
            params![token, expires, id],
        )?;
        Ok(())
    }

    pub fn find_by_reset_token(conn: &Connection, token: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, password_hash, first_name, last_name, phone, street, zip_code, city, notes, is_admin, created_at, updated_at FROM customers WHERE reset_token = ?1 AND reset_token_expires > datetime('now')"
        )?;
        let mut rows = stmt.query(params![token])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::from_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn clear_reset_token(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE customers SET reset_token = NULL, reset_token_expires = NULL WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name).trim().to_string()
    }
}
