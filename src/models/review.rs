use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result, params, Row};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Review {
    pub id: i64,
    pub author_name: String,
    pub author_location: String,
    pub content: String,
    pub stars: i32,
    pub sort_order: i32,
    pub is_active: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ReviewForm {
    pub id: Option<i64>,
    pub author_name: String,
    pub author_location: String,
    pub content: String,
    pub stars: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

impl Review {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Review {
            id: row.get(0)?,
            author_name: row.get(1)?,
            author_location: row.get(2)?,
            content: row.get(3)?,
            stars: row.get(4)?,
            sort_order: row.get(5)?,
            is_active: row.get::<_, i32>(6)? != 0,
            updated_at: row.get(7)?,
        })
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, author_name, author_location, content, stars, sort_order, is_active, updated_at FROM reviews ORDER BY sort_order ASC"
        )?;
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn find_active(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, author_name, author_location, content, stars, sort_order, is_active, updated_at FROM reviews WHERE is_active = 1 ORDER BY sort_order ASC"
        )?;
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        rows.collect()
    }

    pub fn create(conn: &Connection, name: &str, location: &str, content: &str, stars: i32, sort_order: i32) -> Result<i64> {
        conn.execute(
            "INSERT INTO reviews (author_name, author_location, content, stars, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, location, content, stars, sort_order],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update(conn: &Connection, id: i64, name: &str, location: &str, content: &str, stars: i32, sort_order: i32, is_active: bool) -> Result<()> {
        conn.execute(
            "UPDATE reviews SET author_name = ?1, author_location = ?2, content = ?3, stars = ?4, sort_order = ?5, is_active = ?6, updated_at = datetime('now') WHERE id = ?7",
            params![name, location, content, stars, sort_order, is_active as i32, id],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM reviews WHERE id = ?1", params![id])?;
        Ok(())
    }
}
