//! Cookieless, self-hosted reach measurement.
//!
//! Nothing is stored on or read from the visitor's device. The server counts
//! public page views per day, page, referring website and device class. A
//! "daily visitor" is recognised by a hash of a random salt, the client IP and
//! the user agent. The salt lives only in memory and is replaced every day, so
//! neither IP addresses nor hashes are written anywhere and visitors cannot be
//! recognised across days. Bots, admins and browsers sending `DNT: 1` or
//! `Sec-GPC: 1` are not counted.

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header,
    middleware::Next,
    web, Error,
};
use rusqlite::{params, Connection, Result};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::Mutex;

/// Public pages worth counting (route patterns as registered in `main.rs`).
const PAGES: &[&str] = &[
    "/", "/ueber-mich", "/unternehmen", "/buchen", "/preise-und-ablauf",
    "/faszienbehandlung-ruecken", "/faszienbehandlung-nacken-schulter",
    "/faszienbehandlung-haltung", "/faszienbehandlung-sport",
    "/impressum", "/datenschutz", "/buchungsbedingungen", "/barrierefreiheit", "/widerruf",
];

const BOT_MARKERS: &[&str] = &[
    "bot", "crawl", "spider", "slurp", "curl", "wget", "python", "go-http", "httpclient",
    "headless", "monitor", "uptime", "scan", "preview", "facebookexternalhit", "java/", "lighthouse",
];

pub struct Analytics {
    trusted_proxy: Option<IpAddr>,
    day: Mutex<DayState>,
}

struct DayState {
    day: String,
    salt: [u8; 16],
    seen: HashSet<u64>,
}

impl Analytics {
    /// `TRUSTED_PROXY` names the reverse proxy whose `X-Forwarded-For` header may be
    /// used to tell visitors apart. Without it, only page views are meaningful.
    pub fn from_env() -> Self {
        let trusted_proxy = std::env::var("TRUSTED_PROXY").ok().and_then(|v| v.trim().parse().ok());
        Analytics { trusted_proxy, day: Mutex::new(DayState { day: String::new(), salt: [0; 16], seen: HashSet::new() }) }
    }

    pub fn distinguishes_visitors(&self) -> bool {
        self.trusted_proxy.is_some()
    }

    /// Returns true when this (salted) client was not yet seen today.
    fn first_visit_today(&self, day: &str, client: IpAddr, user_agent: &str) -> bool {
        let mut state = self.day.lock().unwrap_or_else(|e| e.into_inner());
        if state.day != day {
            state.day = day.to_owned();
            state.salt = *uuid::Uuid::new_v4().as_bytes();
            state.seen.clear();
        }
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        state.salt.hash(&mut hasher);
        client.hash(&mut hasher);
        user_agent.hash(&mut hasher);
        state.seen.insert(hasher.finish())
    }
}

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS visit_daily (
            day TEXT NOT NULL, path TEXT NOT NULL, referrer TEXT NOT NULL, device TEXT NOT NULL,
            views INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (day, path, referrer, device)
        );
        CREATE TABLE IF NOT EXISTS visit_uniques (
            day TEXT PRIMARY KEY, visitors INTEGER NOT NULL DEFAULT 0
        );",
    )
}

fn client_ip(req: &actix_web::HttpRequest, trusted_proxy: Option<IpAddr>) -> Option<IpAddr> {
    let peer = req.peer_addr()?.ip();
    if Some(peer) != trusted_proxy {
        return Some(peer);
    }
    // The proxy appends the address it received the request from as the last entry.
    req.headers().get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.rsplit(',').next())
        .and_then(|v| v.trim().parse().ok())
        .or(Some(peer))
}

pub fn is_bot(user_agent: &str) -> bool {
    let ua = user_agent.to_ascii_lowercase();
    ua.is_empty() || BOT_MARKERS.iter().any(|m| ua.contains(m))
}

pub fn device_class(user_agent: &str) -> &'static str {
    let ua = user_agent.to_ascii_lowercase();
    if ua.contains("ipad") || ua.contains("tablet") || (ua.contains("android") && !ua.contains("mobile")) {
        "Tablet"
    } else if ua.contains("mobi") || ua.contains("iphone") || ua.contains("android") {
        "Smartphone"
    } else {
        "Computer"
    }
}

/// Only the host of an external referring website is kept, never the full URL.
pub fn referrer_host(referer: Option<&str>, own_host: &str) -> String {
    let host = referer
        .and_then(|r| reqwest::Url::parse(r).ok())
        .filter(|u| matches!(u.scheme(), "http" | "https"))
        .and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_ascii_lowercase()));
    match host {
        Some(h) if !h.is_empty() && h != own_host.trim_start_matches("www.") => h,
        _ => "direkt".into(),
    }
}

pub async fn count_visit(req: ServiceRequest, next: Next<impl MessageBody>) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let res = next.call(req).await?;
    let request = res.request();
    let pattern = request.match_pattern().unwrap_or_default();
    let html = res.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).is_some_and(|v| v.starts_with("text/html"));
    if request.method() != actix_web::http::Method::GET || res.status() != actix_web::http::StatusCode::OK || !html || !PAGES.contains(&pattern.as_str()) {
        return Ok(res);
    }
    let headers = request.headers();
    let opted_out = headers.get("dnt").is_some_and(|v| v == "1") || headers.get("sec-gpc").is_some_and(|v| v == "1");
    let user_agent = headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()).unwrap_or("").to_owned();
    if opted_out || is_bot(&user_agent) {
        return Ok(res);
    }
    let (Some(analytics), Some(db), Some(secret), Some(config)) = (
        request.app_data::<web::Data<Analytics>>(),
        request.app_data::<web::Data<Mutex<Connection>>>(),
        request.app_data::<web::Data<String>>(),
        request.app_data::<web::Data<crate::config::Config>>(),
    ) else { return Ok(res); };
    if crate::auth::get_claims(request, secret).is_some_and(|c| c.is_admin) {
        return Ok(res);
    }
    let own_host = reqwest::Url::parse(&config.base_url).ok().and_then(|u| u.host_str().map(str::to_owned)).unwrap_or_default();
    let referrer = referrer_host(headers.get(header::REFERER).and_then(|v| v.to_str().ok()), &own_host);
    let device = device_class(&user_agent);
    let day = chrono::Utc::now().with_timezone(&chrono_tz::Europe::Berlin).format("%Y-%m-%d").to_string();
    let new_visitor = analytics.distinguishes_visitors()
        && client_ip(request, analytics.trusted_proxy).is_some_and(|ip| analytics.first_visit_today(&day, ip, &user_agent));
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    if let Err(e) = record(&conn, &day, &pattern, &referrer, device, new_visitor) {
        log::warn!("Reichweitenmessung: Eintrag fehlgeschlagen ({e})");
    }
    drop(conn);
    Ok(res)
}

pub fn record(conn: &Connection, day: &str, path: &str, referrer: &str, device: &str, new_visitor: bool) -> Result<()> {
    conn.execute(
        "INSERT INTO visit_daily(day, path, referrer, device, views) VALUES (?1, ?2, ?3, ?4, 1)
         ON CONFLICT(day, path, referrer, device) DO UPDATE SET views = views + 1",
        params![day, path, referrer, device],
    )?;
    if new_visitor {
        conn.execute(
            "INSERT INTO visit_uniques(day, visitors) VALUES (?1, 1) ON CONFLICT(day) DO UPDATE SET visitors = visitors + 1",
            params![day],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_devices_bots_and_referrers() {
        assert_eq!(device_class("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) Mobile/15E148"), "Smartphone");
        assert_eq!(device_class("Mozilla/5.0 (Linux; Android 14; SM-X710) Safari/537.36"), "Tablet");
        assert_eq!(device_class("Mozilla/5.0 (X11; Linux x86_64) Firefox/131.0"), "Computer");
        assert!(is_bot("Mozilla/5.0 (compatible; Googlebot/2.1)") && is_bot("") && !is_bot("Mozilla/5.0 (X11; Linux) Firefox/131.0"));
        assert_eq!(referrer_host(Some("https://www.google.com/search?q=faszien+rottweil"), "faszienbehandlung.jetzt"), "google.com");
        assert_eq!(referrer_host(Some("https://faszienbehandlung.jetzt/preise"), "faszienbehandlung.jetzt"), "direkt");
        assert_eq!(referrer_host(Some("android-app://com.google"), "faszienbehandlung.jetzt"), "direkt");
        assert_eq!(referrer_host(None, "faszienbehandlung.jetzt"), "direkt");
    }

    #[test]
    fn daily_visitors_reset_with_the_day_and_never_keep_ips() {
        let analytics = Analytics { trusted_proxy: Some("192.168.178.198".parse().unwrap()), day: Mutex::new(DayState { day: String::new(), salt: [0; 16], seen: HashSet::new() }) };
        let ip: IpAddr = "203.0.113.7".parse().unwrap();
        assert!(analytics.first_visit_today("2026-11-23", ip, "UA"));
        assert!(!analytics.first_visit_today("2026-11-23", ip, "UA"));
        assert!(analytics.first_visit_today("2026-11-23", ip, "anderer Browser"));
        assert!(analytics.first_visit_today("2026-11-24", ip, "UA"), "a new day starts a new salt");
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        record(&conn, "2026-11-23", "/", "google.com", "Smartphone", true).unwrap();
        record(&conn, "2026-11-23", "/", "google.com", "Smartphone", false).unwrap();
        let (views, visitors): (i64, i64) = conn.query_row("SELECT (SELECT SUM(views) FROM visit_daily), (SELECT visitors FROM visit_uniques)", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!((views, visitors), (2, 1));
        let columns: String = conn.query_row("SELECT group_concat(name) FROM pragma_table_info('visit_daily')", [], |r| r.get(0)).unwrap();
        assert!(!columns.contains("ip") && !columns.contains("agent"));
    }
}
