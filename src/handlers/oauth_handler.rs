use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;

use crate::auth;
use crate::models::settings::SiteSetting;

fn require_admin(req: &HttpRequest, jwt_secret: &str) -> Result<(), HttpResponse> {
    match auth::get_claims(req, jwt_secret) {
        Some(c) if c.is_admin => Ok(()),
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
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let (base_url, client_id, redirect_uri) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let base = SiteSetting::get_or_default(&conn, "nextcloud_base_url", "");
        let client_id = SiteSetting::get_or_default(&conn, "nextcloud_oauth_client_id", "");
        // Derive redirect URI from the Nextcloud base URL context (use the app base URL stored in config)
        // We hardcode to /admin/nextcloud/callback on the same host.
        // The admin sets the BASE_URL in their env; here we reconstruct from the request.
        let host = req.connection_info().host().to_string();
        let scheme = if host.contains("localhost") || host.starts_with("127.") { "http" } else { "https" };
        let redirect = format!("{}://{}/admin/nextcloud/callback", scheme, host);
        (base, client_id, redirect)
    };

    if base_url.is_empty() || client_id.is_empty() {
        return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=nextcloud_not_configured"))
            .finish();
    }

    // Generate random state
    let state: String = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;
        let mut h = DefaultHasher::new();
        SystemTime::now().hash(&mut h);
        std::thread::current().id().hash(&mut h);
        format!("{:016x}{:016x}", h.finish(), h.finish() ^ 0xdeadbeefcafe)
    };

    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let _ = conn.execute(
            "INSERT OR REPLACE INTO site_settings (setting_key, setting_value) VALUES ('nextcloud_oauth_state', ?1)",
            rusqlite::params![state],
        );
    }

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
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let code = match query.get("code") {
        Some(c) => c.clone(),
        None => {
            let error = query.get("error").map(|s| s.as_str()).unwrap_or("unknown_error");
            return HttpResponse::Found()
                .append_header(("Location", format!("/admin/settings?error=oauth_{}", error)))
                .finish();
        }
    };
    let state = query.get("state").cloned().unwrap_or_default();

    // Verify CSRF state
    let (saved_state, base_url, client_id, client_secret) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        (
            SiteSetting::get_or_default(&conn, "nextcloud_oauth_state", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_base_url", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_oauth_client_id", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_oauth_client_secret", ""),
        )
    };

    if state.is_empty() || state != saved_state {
        return HttpResponse::Found()
            .append_header(("Location", "/admin/settings?error=oauth_state_mismatch"))
            .finish();
    }

    // Build redirect URI (same logic as connect)
    let host = req.connection_info().host().to_string();
    let scheme = if host.contains("localhost") || host.starts_with("127.") { "http" } else { "https" };
    let redirect_uri = format!("{}://{}/admin/nextcloud/callback", scheme, host);

    // Exchange code for tokens
    let token_url = format!("{}/apps/oauth2/api/v1/token", base_url.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
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
        Err(e) => {
            log::warn!("OAuth callback: token exchange failed: {}", e);
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
        Err(e) => {
            log::warn!("OAuth callback: failed to parse token JSON: {}", e);
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

    let expiry = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
    let expiry_str = expiry.format("%Y-%m-%d %H:%M:%S").to_string();

    // Discover calendars using the new access token
    let calendar_urls = crate::caldav::discover_calendars(&base_url, &access_token).await;
    let all_urls_str = calendar_urls.join(",");
    let primary_url = calendar_urls.first().cloned().unwrap_or_default();

    // Store everything in DB
    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let settings = [
            ("nextcloud_access_token", access_token.as_str()),
            ("nextcloud_refresh_token", refresh_token.as_str()),
            ("nextcloud_token_expiry", expiry_str.as_str()),
            ("nextcloud_all_calendar_urls", all_urls_str.as_str()),
            ("nextcloud_primary_calendar_url", primary_url.as_str()),
            ("nextcloud_oauth_state", ""), // clear CSRF state
        ];
        for (key, value) in &settings {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO site_settings (setting_key, setting_value) VALUES (?1, ?2)",
                rusqlite::params![key, value],
            );
        }
        log::info!("OAuth: connected to Nextcloud, discovered {} calendars", calendar_urls.len());
    }

    HttpResponse::Found()
        .append_header(("Location", "/admin/settings?caldav_connected=1"))
        .finish()
}

/// GET /admin/nextcloud/disconnect
/// Clears all OAuth tokens from the database.
pub async fn nextcloud_disconnect(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
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
        for key in &keys {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO site_settings (setting_key, setting_value) VALUES (?1, '')",
                rusqlite::params![key],
            );
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
        false
    };

    if !needs_refresh {
        return Some(access_token);
    }

    if refresh_token.is_empty() || base_url.is_empty() {
        return Some(access_token); // return current even if potentially stale
    }

    // Refresh the token
    let token_url = format!("{}/apps/oauth2/api/v1/token", base_url.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Some(access_token),
    };

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.as_str()),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
    ];

    let resp = match client.post(&token_url).form(&params).send().await {
        Ok(r) => r,
        Err(e) => { log::warn!("OAuth refresh: request failed: {}", e); return Some(access_token); }
    };

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => { log::warn!("OAuth refresh: parse failed: {}", e); return Some(access_token); }
    };

    let new_token = json["access_token"].as_str().unwrap_or("").to_string();
    let new_refresh = json["refresh_token"].as_str().unwrap_or("").to_string();
    let expires_in = json["expires_in"].as_i64().unwrap_or(3600);

    if new_token.is_empty() {
        return Some(access_token);
    }

    let expiry = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
    let expiry_str_new = expiry.format("%Y-%m-%d %H:%M:%S").to_string();

    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let _ = conn.execute(
            "INSERT OR REPLACE INTO site_settings (setting_key, setting_value) VALUES ('nextcloud_access_token', ?1)",
            rusqlite::params![new_token],
        );
        if !new_refresh.is_empty() {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO site_settings (setting_key, setting_value) VALUES ('nextcloud_refresh_token', ?1)",
                rusqlite::params![new_refresh],
            );
        }
        let _ = conn.execute(
            "INSERT OR REPLACE INTO site_settings (setting_key, setting_value) VALUES ('nextcloud_token_expiry', ?1)",
            rusqlite::params![expiry_str_new],
        );
    }

    log::info!("OAuth: access token refreshed successfully");
    Some(new_token)
}
