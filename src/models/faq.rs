use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result, params, Row};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Faq {
    pub id: i64,
    pub question: String,
    pub answer_html: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct FaqForm {
    pub id: Option<i64>,
    pub question: String,
    pub answer_html: String,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

impl Faq {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Faq {
            id: row.get(0)?,
            question: row.get(1)?,
            answer_html: row.get(2)?,
            sort_order: row.get(3)?,
            is_active: row.get::<_, i32>(4)? != 0,
            updated_at: row.get(5)?,
        })
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, question, answer_html, sort_order, is_active, updated_at FROM faqs ORDER BY sort_order ASC"
        )?;
        let rows = stmt.query_map([], Self::from_row)?;
        rows.collect()
    }

    pub fn find_active(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, question, answer_html, sort_order, is_active, updated_at FROM faqs WHERE is_active = 1 ORDER BY sort_order ASC"
        )?;
        let rows = stmt.query_map([], Self::from_row)?;
        rows.collect()
    }

    #[allow(dead_code)]
    pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, question, answer_html, sort_order, is_active, updated_at FROM faqs WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::from_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn create(conn: &Connection, question: &str, answer_html: &str, sort_order: i32) -> Result<i64> {
        conn.execute(
            "INSERT INTO faqs (question, answer_html, sort_order) VALUES (?1, ?2, ?3)",
            params![question, answer_html, sort_order],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update(conn: &Connection, id: i64, question: &str, answer_html: &str, sort_order: i32, is_active: bool) -> Result<()> {
        conn.execute(
            "UPDATE faqs SET question = ?1, answer_html = ?2, sort_order = ?3, is_active = ?4, updated_at = datetime('now') WHERE id = ?5",
            params![question, answer_html, sort_order, is_active as i32, id],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM faqs WHERE id = ?1", params![id])?;
        Ok(())
    }
}
