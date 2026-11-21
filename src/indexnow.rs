//! IndexNow: tells Bing, Yandex, Seznam and other participating search engines
//! when public pages change, so they recrawl without waiting for their own schedule.
//! Google does not take part; it is notified through Search Console and the sitemap.

use rusqlite::{params, Connection, Result};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const ENDPOINT: &str = "https://api.indexnow.org/indexnow";
const KEY_SETTING: &str = "indexnow_key";
/// Settings saves can come in bursts; one submission per window is enough.
const MIN_INTERVAL: Duration = Duration::from_secs(10 * 60);

static LAST_SUBMISSION: Mutex<Option<Instant>> = Mutex::new(None);

/// Public pages listed in sitemap.xml. A test keeps both in sync.
pub fn public_paths() -> Vec<&'static str> {
    ["/", "/ueber-mich", "/unternehmen"].into_iter().chain(crate::marketing::TOPICS.iter().map(|t| t.0)).collect()
}

pub fn valid_key(key: &str) -> bool {
    (8..=128).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// Returns the stored key, creating one on first use. The key is public by design:
/// it is served at `/{key}.txt` so search engines can verify submissions.
pub fn ensure_key(conn: &Connection) -> Result<String> {
    let stored: Option<String> = conn.query_row(
        "SELECT setting_value FROM site_settings WHERE setting_key=?1", params![KEY_SETTING], |r| r.get(0),
    ).ok();
    if let Some(key) = stored.filter(|k| valid_key(k)) { return Ok(key); }
    let key = uuid::Uuid::new_v4().simple().to_string();
    conn.execute(
        "INSERT INTO site_settings(setting_key, setting_value) VALUES(?1, ?2) ON CONFLICT(setting_key) DO UPDATE SET setting_value=excluded.setting_value",
        params![KEY_SETTING, key],
    )?;
    Ok(key)
}

/// The key file content when `requested` is `{key}.txt`.
pub fn key_file(conn: &Connection, requested: &str) -> Option<String> {
    let key = requested.strip_suffix(".txt")?;
    let stored: String = conn.query_row(
        "SELECT setting_value FROM site_settings WHERE setting_key=?1", params![KEY_SETTING], |r| r.get(0),
    ).ok()?;
    (valid_key(&stored) && stored == key).then_some(stored)
}

/// Only a public HTTPS deployment submits; local and preview servers stay silent.
fn public_host(base_url: &str) -> Option<String> {
    let url = reqwest::Url::parse(base_url).ok()?;
    let host = url.host_str()?.to_owned();
    let local = host == "localhost" || host.ends_with(".localhost") || host.ends_with(".test") || host.ends_with(".local")
        || host.parse::<std::net::IpAddr>().is_ok();
    (url.scheme() == "https" && !local).then_some(host)
}

pub fn payload(base_url: &str, key: &str) -> Option<serde_json::Value> {
    let host = public_host(base_url)?;
    let urls: Vec<String> = public_paths().iter().map(|p| format!("{base_url}{p}")).collect();
    Some(serde_json::json!({
        "host": host,
        "key": key,
        "keyLocation": format!("{base_url}/{key}.txt"),
        "urlList": urls,
    }))
}

fn claim_slot(force: bool) -> bool {
    let mut last = LAST_SUBMISSION.lock().unwrap_or_else(|e| e.into_inner());
    if !force && last.is_some_and(|t| t.elapsed() < MIN_INTERVAL) { return false; }
    *last = Some(Instant::now());
    true
}

/// Submits all public pages in the background. `force` skips the rate limit (used at startup,
/// where a deploy may have changed every page).
pub fn submit_in_background(base_url: String, key: String, reason: &'static str, force: bool) {
    let Some(body) = payload(&base_url, &key) else { return; };
    if !claim_slot(force) { return; }
    tokio::spawn(async move {
        // Give a fresh process time to accept the key-file verification request.
        tokio::time::sleep(Duration::from_secs(5)).await;
        let client = match reqwest::Client::builder().timeout(Duration::from_secs(15)).build() {
            Ok(client) => client,
            Err(_) => { log::warn!("IndexNow: HTTP client unavailable"); return; }
        };
        match client.post(ENDPOINT).json(&body).send().await {
            Ok(response) if response.status().is_success() =>
                log::info!("IndexNow: {} pages submitted ({reason}), status {}", public_paths().len(), response.status()),
            Ok(response) => log::warn!("IndexNow: submission ({reason}) answered with status {}", response.status()),
            Err(_) => log::warn!("IndexNow: submission ({reason}) failed to reach the endpoint"),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE site_settings(setting_key TEXT PRIMARY KEY, setting_value TEXT);").unwrap();
        conn
    }

    #[test]
    fn key_is_created_once_and_served_only_for_its_own_file() {
        let conn = conn();
        let key = ensure_key(&conn).unwrap();
        assert!(valid_key(&key) && key.len() == 32);
        assert_eq!(ensure_key(&conn).unwrap(), key);
        assert_eq!(key_file(&conn, &format!("{key}.txt")).as_deref(), Some(key.as_str()));
        assert!(key_file(&conn, &key).is_none());
        assert!(key_file(&conn, "robots.txt").is_none());
        conn.execute("UPDATE site_settings SET setting_value='<bad>' WHERE setting_key='indexnow_key'", []).unwrap();
        assert!(key_file(&conn, "<bad>.txt").is_none());
        assert_ne!(ensure_key(&conn).unwrap(), "<bad>");
    }

    #[test]
    fn only_public_https_hosts_submit_every_sitemap_page() {
        for local in ["http://127.0.0.1:8080", "http://localhost:8080", "https://localhost", "https://192.168.178.150", "http://faszienbehandlung.jetzt"] {
            assert!(payload(local, "abcdef12").is_none(), "{local}");
        }
        let body = payload("https://faszienbehandlung.jetzt", "abcdef12").unwrap();
        assert_eq!(body["host"], "faszienbehandlung.jetzt");
        assert_eq!(body["keyLocation"], "https://faszienbehandlung.jetzt/abcdef12.txt");
        let urls: Vec<&str> = body["urlList"].as_array().unwrap().iter().map(|u| u.as_str().unwrap()).collect();
        assert_eq!(urls[0], "https://faszienbehandlung.jetzt/");

        let sitemap = std::fs::read_to_string("sitemap.xml").unwrap();
        let listed: Vec<&str> = sitemap.split("<loc>").skip(1).map(|s| s.split("</loc>").next().unwrap()).collect();
        assert_eq!(listed, urls, "sitemap.xml and IndexNow must list the same pages");
    }
}
