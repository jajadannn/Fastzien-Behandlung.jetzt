use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result, params};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteSetting {
    pub key: String,
    pub value: String,
}

impl SiteSetting {
    pub fn get(conn: &Connection, key: &str) -> Result<String> {
        conn.query_row(
            "SELECT setting_value FROM site_settings WHERE setting_key = ?1",
            params![key],
            |row| row.get(0),
        )
    }

    pub fn get_or_default(conn: &Connection, key: &str, default: &str) -> String {
        Self::get(conn, key).unwrap_or_else(|_| default.to_string())
    }

    pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT INTO site_settings (setting_key, setting_value) VALUES (?1, ?2) ON CONFLICT(setting_key) DO UPDATE SET setting_value = ?2, updated_at = datetime('now')",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all(conn: &Connection) -> Result<HashMap<String, String>> {
        let mut stmt = conn.prepare("SELECT setting_key, setting_value FROM site_settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (key, value) = row?;
            map.insert(key, value);
        }
        Ok(map)
    }
}
