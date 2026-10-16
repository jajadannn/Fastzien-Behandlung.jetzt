use actix_web::{web, HttpRequest, HttpResponse, cookie::{Cookie, SameSite}};
use rusqlite::Connection;
use std::sync::Mutex;

use crate::auth;
use crate::config::Config;
use crate::models::settings::SiteSetting;

const STATE_COOKIE: &str = "nextcloud_oauth_state";
const STATE_TTL_SECONDS: i64 = 600;

#[derive(serde::Serialize, serde::Deserialize)]
struct PendingAuthorization {
    state: String,
    admin_id: i64,
    expires_at: i64,
    consumed: bool,
}

fn consume_state(conn: &Connection, state: &str, cookie: &str, admin_id: i64) -> bool {
    if state.is_empty() || cookie != state {
        return false;
    }
    let saved = SiteSetting::get_or_default(conn, STATE_COOKIE, "");
    let mut pending: PendingAuthorization = match serde_json::from_str(&saved) {
        Ok(pending) => pending,
        Err(_) => return false,
    };
    if pending.consumed || pending.state != state || pending.admin_id != admin_id
        || pending.expires_at <= chrono::Utc::now().timestamp() {
        return false;
    }
    // Consume before the network request, including failed exchanges; a callback
    // can only be used once and only by the browser that initiated this flow.
    pending.consumed = true;
    matches!(conn.execute(
        "UPDATE site_settings SET setting_value = ?3 WHERE setting_key = ?1 AND setting_value = ?2",
        rusqlite::params![STATE_COOKIE, saved, serde_json::to_string(&pending).expect("serializable OAuth state")],
    ), Ok(1))
}

fn store_settings(conn: &Connection, settings: &[(&str, &str)]) -> rusqlite::Result<()> {
    for (key, value) in settings {
        conn.execute("INSERT OR REPLACE INTO site_settings (setting_key, setting_value) VALUES (?1, ?2)",
            rusqlite::params![key, value])?;
    }
    Ok(())
}

fn save_authorization_result(
    conn: &Connection, state: &str, admin_id: i64, access_token: &str,
    refresh_token: &str, expiry: &str, calendars: &[String],
) -> rusqlite::Result<bool> {
    let Some(primary) = calendars.first() else { return Ok(false); };
    let tx = conn.unchecked_transaction()?;
    let saved = SiteSetting::get_or_default(&tx, STATE_COOKIE, "");
    let pending: PendingAuthorization = match serde_json::from_str(&saved) {
        Ok(pending) => pending,
        Err(_) => return Ok(false),
    };
    // A disconnect or newer connect invalidates callbacks already in flight.
    if !pending.consumed || pending.state != state || pending.admin_id != admin_id
        || pending.expires_at <= chrono::Utc::now().timestamp() {
        return Ok(false);
    }
    store_settings(&tx, &[
        ("nextcloud_access_token", access_token),
        ("nextcloud_refresh_token", refresh_token),
        ("nextcloud_token_expiry", expiry),
        ("nextcloud_all_calendar_urls", &calendars.join(",")),
        ("nextcloud_primary_calendar_url", primary),
        (STATE_COOKIE, ""),
    ])?;
    tx.commit()?;
    Ok(true)
}

fn save_refresh_result(
    conn: &Connection, old_access: &str, old_refresh: &str,
    new_access: &str, new_refresh: &str, expiry: &str,
) -> rusqlite::Result<Option<String>> {
    let tx = conn.unchecked_transaction()?;
    let current_access = SiteSetting::get_or_default(&tx, "nextcloud_access_token", "");
    let current_refresh = SiteSetting::get_or_default(&tx, "nextcloud_refresh_token", "");
    // Do not resurrect a disconnected integration or overwrite newer tokens.
    if current_access != old_access || current_refresh != old_refresh {
        return Ok((!current_access.is_empty()).then_some(current_access));
    }
    store_settings(&tx, &[
        ("nextcloud_access_token", new_access),
        ("nextcloud_refresh_token", if new_refresh.is_empty() { old_refresh } else { new_refresh }),
        ("nextcloud_token_expiry", expiry),
    ])?;
    tx.commit()?;
    Ok(Some(new_access.to_owned()))
}

fn current_valid_token(db: &Mutex<Connection>) -> Option<String> {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let token = SiteSetting::get_or_default(&conn, "nextcloud_access_token", "");
    let expiry = SiteSetting::get_or_default(&conn, "nextcloud_token_expiry", "");
    let expiry = chrono::NaiveDateTime::parse_from_str(&expiry, "%Y-%m-%d %H:%M:%S").ok()?;
    (!token.is_empty() && expiry > chrono::Utc::now().naive_utc()).then_some(token)
}

fn callback_uri(config: &Config) -> String {
    format!("{}/admin/nextcloud/callback", config.base_url.trim_end_matches('/'))
}

fn token_expiry(expires_in: i64) -> Option<String> {
    if expires_in <= 0 {
        return None;
    }
    let expiry = chrono::Utc::now().checked_add_signed(chrono::Duration::try_seconds(expires_in)?)?;
    Some(expiry.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn require_admin(req: &HttpRequest, jwt_secret: &str) -> Result<auth::Claims, HttpResponse> {
    match auth::get_claims(req, jwt_secret) {
        Some(c) if c.is_admin => Ok(c),
        Some(_) => Err(HttpResponse::Forbidden().finish()),
        None => Err(HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()),
    }
}

/// GET /admin/nextcloud/connect
/// Generates a random CSRF state, stores it in DB, then redirects the browser
/// to the Nextcloud OAuth2 authorize endpoint.
pub async fn nextcloud_connect(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    config: web::Data<Config>,
) -> HttpResponse {
    let claims = match require_admin(&req, &jwt_secret) {
        Ok(claims) => claims,
        Err(response) => return response,
    };

    let (base_url, client_id) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let base = SiteSetting::get_or_default(&conn, "nextcloud_base_url", "");
        let client_id = SiteSetting::get_or_default(&conn, "nextcloud_oauth_client_id", "");
        (base, client_id)
    };

    if base_url.is_empty() || client_id.is_empty() {
        return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=nextcloud_not_configured"))
            .finish();
    }

    let state = uuid::Uuid::new_v4().to_string();
    let pending = PendingAuthorization {
        state: state.clone(),
        admin_id: claims.sub,
        expires_at: chrono::Utc::now().timestamp() + STATE_TTL_SECONDS,
        consumed: false,
    };

    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        if conn.execute(
            "INSERT OR REPLACE INTO site_settings (setting_key, setting_value) VALUES ('nextcloud_oauth_state', ?1)",
            rusqlite::params![serde_json::to_string(&pending).expect("serializable OAuth state")],
        ).is_err() {
            return HttpResponse::InternalServerError().body("OAuth-Verbindung konnte nicht vorbereitet werden.");
        }
    }

    let redirect_uri = callback_uri(&config);
    let encoded_redirect = urlencoding::encode(&redirect_uri);
    let encoded_client_id = urlencoding::encode(&client_id);
    let encoded_state = urlencoding::encode(&state);

    let authorize_url = format!(
        "{}/apps/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&state={}",
        base_url.trim_end_matches('/'),
        encoded_client_id,
        encoded_redirect,
        encoded_state,
    );

    HttpResponse::Found()
        .cookie(Cookie::build(STATE_COOKIE, state)
            .path("/admin/nextcloud")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(reqwest::Url::parse(&config.base_url).is_ok_and(|url| url.scheme() == "https"))
            .max_age(actix_web::cookie::time::Duration::seconds(STATE_TTL_SECONDS))
            .finish())
        .append_header(("Location", authorize_url))
        .finish()
}

/// GET /admin/nextcloud/callback
/// Receives the authorization code from Nextcloud, exchanges it for tokens,
/// then discovers all calendars and stores them.
pub async fn nextcloud_callback(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    config: web::Data<Config>,
) -> HttpResponse {
    let claims = match require_admin(&req, &jwt_secret) {
        Ok(claims) => claims,
        Err(response) => return response,
    };
    let state = query.get("state").cloned().unwrap_or_default();

    // Verify CSRF state
    let (valid_state, base_url, client_id, client_secret) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let state_cookie = req.cookie(STATE_COOKIE);
        (
            consume_state(&conn, &state, state_cookie.as_ref().map(|c| c.value()).unwrap_or(""), claims.sub),
            SiteSetting::get_or_default(&conn, "nextcloud_base_url", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_oauth_client_id", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_oauth_client_secret", ""),
        )
    };

    if !valid_state {
        return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=oauth_state_mismatch"))
            .finish();
    }

    let code = match query.get("code").filter(|code| !code.is_empty()) {
        Some(code) => code,
        None => return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=oauth_denied"))
            .finish(),
    };
    let redirect_uri = callback_uri(&config);

    // Exchange code for tokens
    let token_url = format!("{}/apps/oauth2/api/v1/token", base_url.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(_) => return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=oauth_client_error"))
            .finish(),
    };

    let params = [
        ("grant_type", "authorization_code"),
        ("code", code.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
    ];

    let token_resp = match client
        .post(&token_url)
        .form(&params)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => {
            log::warn!("OAuth callback: token exchange failed");
            return HttpResponse::Found()
                .append_header(("Location", "/admin/settings?error=oauth_token_exchange_failed"))
                .finish();
        }
    };

    if !token_resp.status().is_success() {
        log::warn!("OAuth callback: token endpoint returned {}", token_resp.status());
        return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=oauth_token_rejected"))
            .finish();
    }

    let token_json: serde_json::Value = match token_resp.json().await {
        Ok(j) => j,
        Err(_) => {
            log::warn!("OAuth callback: failed to parse token JSON");
            return HttpResponse::Found()
                .append_header(("Location", "/admin/settings?error=oauth_token_parse_error"))
                .finish();
        }
    };

    let access_token = token_json["access_token"].as_str().unwrap_or("").to_string();
    let refresh_token = token_json["refresh_token"].as_str().unwrap_or("").to_string();
    let expires_in = token_json["expires_in"].as_i64().unwrap_or(3600);

    if access_token.is_empty() {
        return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=oauth_no_access_token"))
            .finish();
    }

    let expiry_str = match token_expiry(expires_in) {
        Some(expiry) => expiry,
        None => return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=oauth_invalid_expiry"))
            .finish(),
    };

    // Discover calendars using the new access token
    let calendar_urls = crate::caldav::discover_calendars(&base_url, &access_token).await;
    if calendar_urls.is_empty() {
        return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=oauth_no_calendars"))
            .finish();
    }

    // Store everything in DB
    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        match save_authorization_result(&conn, &state, claims.sub, &access_token, &refresh_token, &expiry_str, &calendar_urls) {
            Ok(true) => {},
            Ok(false) => return HttpResponse::Found()
                .append_header(("Location", "/admin/settings?error=oauth_connection_cancelled"))
                .finish(),
            Err(_) => return HttpResponse::InternalServerError().body("OAuth-Verbindung konnte nicht gespeichert werden."),
        }
        log::info!("OAuth: connected to Nextcloud, discovered {} calendars", calendar_urls.len());
    }

    HttpResponse::Found()
        .append_header(("Location", "/admin/settings?caldav_connected=1"))
        .finish()
}

/// POST /admin/nextcloud/disconnect
/// Clears all OAuth tokens from the database.
pub async fn nextcloud_disconnect(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if req.method() != actix_web::http::Method::POST {
        return HttpResponse::MethodNotAllowed().insert_header(("Allow", "POST")).finish();
    }
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let keys = [
            "nextcloud_access_token",
            "nextcloud_refresh_token",
            "nextcloud_token_expiry",
            "nextcloud_all_calendar_urls",
            "nextcloud_primary_calendar_url",
            "nextcloud_oauth_state",
        ];
        let tx = match conn.unchecked_transaction() {
            Ok(tx) => tx,
            Err(_) => return HttpResponse::InternalServerError().body("OAuth-Verbindung konnte nicht getrennt werden."),
        };
        let settings: Vec<(&str, &str)> = keys.iter().map(|key| (*key, "")).collect();
        if store_settings(&tx, &settings).is_err() || tx.commit().is_err() {
            return HttpResponse::InternalServerError().body("OAuth-Verbindung konnte nicht getrennt werden.");
        }
        log::info!("OAuth: disconnected from Nextcloud");
    }

    HttpResponse::Found()
        .append_header(("Location", "/admin/settings?caldav_disconnected=1"))
        .finish()
}

/// Refresh the access token if it is expired or about to expire.
/// Returns the current valid access token, or None if not connected.
pub async fn ensure_valid_token(db: &Mutex<Connection>) -> Option<String> {
    let (access_token, refresh_token, expiry_str, base_url, client_id, client_secret) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        (
            SiteSetting::get_or_default(&conn, "nextcloud_access_token", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_refresh_token", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_token_expiry", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_base_url", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_oauth_client_id", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_oauth_client_secret", ""),
        )
    };

    if access_token.is_empty() {
        return None;
    }

    // Check if token expires within 5 minutes
    let needs_refresh = if let Ok(expiry) = chrono::NaiveDateTime::parse_from_str(&expiry_str, "%Y-%m-%d %H:%M:%S") {
        let now = chrono::Utc::now().naive_utc();
        expiry < now + chrono::Duration::minutes(5)
    } else {
        true
    };

    if !needs_refresh {
        return Some(access_token);
    }

    if refresh_token.is_empty() || base_url.is_empty() {
        return current_valid_token(db);
    }

    // Refresh the token
    let token_url = format!("{}/apps/oauth2/api/v1/token", base_url.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return current_valid_token(db),
    };

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.as_str()),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
    ];

    let resp = match client.post(&token_url).form(&params).send().await {
        Ok(r) => r,
        Err(_) => { log::warn!("OAuth refresh: request failed"); return current_valid_token(db); }
    };

    if !resp.status().is_success() {
        log::warn!("OAuth refresh: token endpoint returned {}", resp.status());
        return current_valid_token(db);
    }
    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => { log::warn!("OAuth refresh: parse failed"); return current_valid_token(db); }
    };

    let new_token = json["access_token"].as_str().unwrap_or("").to_string();
    let new_refresh = json["refresh_token"].as_str().unwrap_or("").to_string();
    let expires_in = json["expires_in"].as_i64().unwrap_or(3600);

    if new_token.is_empty() {
        return current_valid_token(db);
    }

    let expiry_str_new = match token_expiry(expires_in) {
        Some(expiry) => expiry,
        None => return current_valid_token(db),
    };

    let saved = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        save_refresh_result(&conn, &access_token, &refresh_token, &new_token, &new_refresh, &expiry_str_new)
    };
    match saved {
        Ok(token) => token,
        Err(_) => { log::warn!("OAuth refresh: storing tokens failed"); current_valid_token(db) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::test_support::*;

    fn save_state(conn: &Connection, expires_at: i64) {
        let pending = PendingAuthorization { state: "random-state".into(), admin_id: 1, expires_at, consumed: false };
        conn.execute("INSERT OR REPLACE INTO site_settings VALUES (?1, ?2)",
            rusqlite::params![STATE_COOKIE, serde_json::to_string(&pending).unwrap()]).unwrap();
    }

    #[test]
    fn oauth_state_requires_initiating_browser_and_admin_and_is_single_use() {
        let db = database();
        let conn = db.lock().unwrap();
        save_state(&conn, chrono::Utc::now().timestamp() + 600);
        assert!(!consume_state(&conn, "random-state", "", 1));
        assert!(!consume_state(&conn, "random-state", "different-browser", 1));
        assert!(!consume_state(&conn, "random-state", "random-state", 2));
        assert!(consume_state(&conn, "random-state", "random-state", 1));
        assert!(!consume_state(&conn, "random-state", "random-state", 1));
    }

    #[test]
    fn expired_oauth_state_and_invalid_token_lifetimes_are_rejected() {
        let db = database();
        let conn = db.lock().unwrap();
        save_state(&conn, chrono::Utc::now().timestamp() - 1);
        assert!(!consume_state(&conn, "random-state", "random-state", 1));
        assert!(token_expiry(3600).is_some());
        assert!(token_expiry(-1).is_none());
        assert!(token_expiry(i64::MAX).is_none());
    }

    #[actix_web::test]
    async fn connect_uses_configured_origin_and_sets_random_browser_state() {
        let db = database();
        {
            let conn = db.lock().unwrap();
            conn.execute("UPDATE customers SET is_admin = 1", []).unwrap();
            conn.execute("INSERT INTO site_settings VALUES ('nextcloud_base_url', 'https://cloud.example.test')", []).unwrap();
            conn.execute("INSERT INTO site_settings VALUES ('nextcloud_oauth_client_id', 'test-client')", []).unwrap();
        }
        let req = actix_web::test::TestRequest::default()
            .app_data(db.clone())
            .cookie(Cookie::new("auth_token", token("old-hash", true)))
            .insert_header(("Host", "attacker.example"))
            .insert_header(("X-Forwarded-Host", "attacker.example"))
            .to_http_request();
        let response = nextcloud_connect(req, db.clone(), web::Data::new(SECRET.to_owned()), web::Data::new(config())).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::FOUND);
        let location = response.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.contains("redirect_uri=https%3A%2F%2Fexample.test%2Fadmin%2Fnextcloud%2Fcallback"));
        assert!(!location.contains("attacker.example"));
        let cookie = response.cookies().find(|cookie| cookie.name() == STATE_COOKIE).unwrap();
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert_eq!(uuid::Uuid::parse_str(cookie.value()).unwrap().get_version_num(), 4);
        assert!(consume_state(&db.lock().unwrap(), cookie.value(), cookie.value(), 1));
    }

    #[actix_web::test]
    async fn disconnect_cannot_be_triggered_by_get() {
        let response = nextcloud_disconnect(actix_web::test::TestRequest::get().to_http_request(),
            database(), web::Data::new(SECRET.to_owned())).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn empty_discovery_preserves_existing_calendar_connection() {
        let db = database();
        let conn = db.lock().unwrap();
        store_settings(&conn, &[("nextcloud_access_token", "existing-access"),
            ("nextcloud_primary_calendar_url", "https://cloud.example.test/calendar/")]).unwrap();
        save_state(&conn, chrono::Utc::now().timestamp() + 600);
        assert!(consume_state(&conn, "random-state", "random-state", 1));
        assert!(!save_authorization_result(&conn, "random-state", 1, "new-access", "new-refresh", "expiry", &[]).unwrap());
        assert_eq!(SiteSetting::get_or_default(&conn, "nextcloud_access_token", ""), "existing-access");
        assert_eq!(SiteSetting::get_or_default(&conn, "nextcloud_primary_calendar_url", ""), "https://cloud.example.test/calendar/");
    }

    #[test]
    fn disconnected_callback_cannot_recreate_connection() {
        let db = database();
        let conn = db.lock().unwrap();
        save_state(&conn, chrono::Utc::now().timestamp() + 600);
        assert!(consume_state(&conn, "random-state", "random-state", 1));
        store_settings(&conn, &[(STATE_COOKIE, ""), ("nextcloud_access_token", "")]).unwrap();
        assert!(!save_authorization_result(&conn, "random-state", 1, "new-access", "new-refresh", "expiry",
            &["https://cloud.example.test/calendar/".into()]).unwrap());
        assert!(SiteSetting::get_or_default(&conn, "nextcloud_access_token", "").is_empty());
    }

    #[test]
    fn refresh_cannot_resurrect_disconnected_or_overwrite_newer_tokens() {
        let db = database();
        let conn = db.lock().unwrap();
        store_settings(&conn, &[("nextcloud_access_token", ""), ("nextcloud_refresh_token", "")]).unwrap();
        assert!(save_refresh_result(&conn, "old-access", "old-refresh", "new-access", "new-refresh", "expiry").unwrap().is_none());
        assert!(SiteSetting::get_or_default(&conn, "nextcloud_access_token", "").is_empty());

        store_settings(&conn, &[("nextcloud_access_token", "old-access"), ("nextcloud_refresh_token", "old-refresh")]).unwrap();
        assert_eq!(save_refresh_result(&conn, "old-access", "old-refresh", "new-access", "new-refresh", "expiry").unwrap().as_deref(), Some("new-access"));
        assert_eq!(save_refresh_result(&conn, "old-access", "old-refresh", "late-access", "late-refresh", "expiry").unwrap().as_deref(), Some("new-access"));
        assert_eq!(SiteSetting::get_or_default(&conn, "nextcloud_refresh_token", ""), "new-refresh");
    }

    #[test]
    fn failed_oauth_save_rolls_back_all_tokens_and_keeps_previous_connection() {
        let db = database();
        let conn = db.lock().unwrap();
        store_settings(&conn, &[("nextcloud_access_token", "existing-access")]).unwrap();
        save_state(&conn, chrono::Utc::now().timestamp() + 600);
        assert!(consume_state(&conn, "random-state", "random-state", 1));
        conn.execute_batch("CREATE TRIGGER reject_token BEFORE INSERT ON site_settings
            WHEN NEW.setting_key = 'nextcloud_refresh_token' BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
        assert!(save_authorization_result(&conn, "random-state", 1, "new-access", "new-refresh", "expiry",
            &["https://cloud.example.test/calendar/".into()]).is_err());
        assert_eq!(SiteSetting::get_or_default(&conn, "nextcloud_access_token", ""), "existing-access");
    }

    #[test]
    fn successful_oauth_save_is_complete_and_cannot_be_replayed() {
        let db = database();
        let conn = db.lock().unwrap();
        save_state(&conn, chrono::Utc::now().timestamp() + 600);
        assert!(consume_state(&conn, "random-state", "random-state", 1));
        let calendars = ["https://cloud.example.test/calendar/".into()];
        assert!(save_authorization_result(&conn, "random-state", 1, "new-access", "new-refresh", "expiry", &calendars).unwrap());
        assert_eq!(SiteSetting::get_or_default(&conn, "nextcloud_access_token", ""), "new-access");
        assert_eq!(SiteSetting::get_or_default(&conn, "nextcloud_primary_calendar_url", ""), calendars[0]);
        assert!(SiteSetting::get_or_default(&conn, STATE_COOKIE, "").is_empty());
        assert!(!save_authorization_result(&conn, "random-state", 1, "late-access", "late-refresh", "expiry", &calendars).unwrap());
    }
}
