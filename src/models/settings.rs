use serde::{Serialize, Deserialize};
use rusqlite::{Connection, Result, params};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteSetting {
    pub key: String,
    pub value: String,
}

impl SiteSetting {
    pub fn valid_value(key: &str, value: &str) -> bool {
        match key {
            "appointment_duration_min" => value.parse::<i64>().is_ok_and(|n| (15..=360).contains(&n)),
            "appointment_break_min" => value.parse::<i64>().is_ok_and(|n| (0..=120).contains(&n)),
            "cancellation_hours" => value.parse::<i64>().is_ok_and(|n| (0..=720).contains(&n)),
            "price_pack_count" => value.parse::<i64>().is_ok_and(|n| (1..=1000).contains(&n)),
            "price_single" | "price_pack" | "home_visit_surcharge" => value.replace(',', ".").parse::<f64>().is_ok_and(|n| n.is_finite() && (0.0..=1_000_000.0).contains(&n)),
            "vat_id" => value.is_empty() || (value.len() == 11 && value.starts_with("DE") && value[2..].bytes().all(|b| b.is_ascii_digit())),
            "business_id" => value.chars().count() <= 30 && !value.chars().any(char::is_control),
            _ => true,
        }
    }

    pub fn get_public(conn: &Connection) -> Result<HashMap<String, String>> {
        let mut settings = Self::get_all(conn)?;
        settings.retain(|key, _| ["price_single", "price_pack", "price_pack_count", "home_visit_surcharge", "appointment_duration_min", "cancellation_hours", "hours_weekday", "hours_saturday", "hours_stripe", "hours_stripe_short", "phone", "phone_raw", "email_contact", "address_street", "address_zip", "address_city", "google_reviews_url", "vat_id", "business_id", "hosting_operator", "smtp_provider", "calendar_operator"].contains(&key.as_str()));
        Ok(settings)
    }
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
