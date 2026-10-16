use actix_web::{web, HttpRequest, cookie::{Cookie, SameSite}};
use jsonwebtoken::{encode, decode, Algorithm, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::Connection;
use std::sync::Mutex;

use crate::{config::Config, models::customer::Customer};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,       // customer id
    pub email: String,
    pub is_admin: bool,
    pub exp: usize,     // expiry timestamp
    pub iat: usize,     // issued at
    // A keyed tag, never the password hash itself. Rotating a password revokes
    // every previously issued token without a session table or timestamp race.
    pub credential_tag: String,
    pub jti: String,
}

pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Failed to hash password")
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    if password.len() > 72 {
        return false;
    }
    verify(password, hash).unwrap_or(false)
}

pub fn password_validation_error(password: &str) -> Option<&'static str> {
    if password.chars().count() < 12 {
        Some("Passwort muss mindestens 12 Zeichen lang sein")
    } else if password.len() > 72 {
        Some("Passwort darf höchstens 72 UTF-8-Bytes lang sein")
    } else {
        None
    }
}

pub fn auth_cookie(token: &str, config: &Config) -> Cookie<'static> {
    Cookie::build("auth_token", token.to_owned())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(reqwest::Url::parse(&config.base_url).is_ok_and(|url| url.scheme() == "https"))
        .max_age(actix_web::cookie::time::Duration::hours(config.jwt_expiry_hours))
        .finish()
}

pub fn create_token(customer_id: i64, email: &str, is_admin: bool, password_hash: &str, secret: &str, expiry_hours: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: customer_id,
        email: email.to_string(),
        is_admin,
        exp: (now + Duration::hours(expiry_hours)).timestamp() as usize,
        iat: now.timestamp() as usize,
        jti: uuid::Uuid::new_v4().to_string(),
        credential_tag: jsonwebtoken::crypto::sign(
            format!("auth-credential-v1:{password_hash}").as_bytes(),
            &EncodingKey::from_secret(secret.as_bytes()),
            Algorithm::HS256,
        )?,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub fn extract_token(req: &HttpRequest) -> Option<String> {
    // Try cookie first
    if let Some(cookie) = req.cookie("auth_token") {
        return Some(cookie.value().to_string());
    }
    // Try Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                return Some(stripped.to_string());
            }
        }
    }
    None
}

pub fn get_claims(req: &HttpRequest, secret: &str) -> Option<Claims> {
    let token = extract_token(req)?;
    let claims = validate_token(&token, secret).ok()?;
    // All callers authenticate before taking the database mutex. Missing DB
    // context and deleted accounts fail closed instead of trusting a stale JWT.
    let db = req.app_data::<web::Data<Mutex<Connection>>>()?;
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    if conn.query_row("SELECT COUNT(*) FROM revoked_sessions WHERE jti=?1 AND expires_at>?2",rusqlite::params![claims.jti,Utc::now().timestamp()],|r|r.get::<_,i64>(0)).ok()? > 0 { return None; }
    let customer = Customer::find_by_id(&conn, claims.sub).ok()??;
    if customer.email != claims.email || customer.is_admin != claims.is_admin {
        return None;
    }
    let valid_credentials = jsonwebtoken::crypto::verify(
        &claims.credential_tag,
        format!("auth-credential-v1:{}", customer.password_hash).as_bytes(),
        &DecodingKey::from_secret(secret.as_bytes()),
        Algorithm::HS256,
    ).ok()?;
    valid_credentials.then_some(claims)
}

pub fn require_verified_claims(req: &HttpRequest, secret: &str) -> Result<Claims,actix_web::HttpResponse> {
    let claims = get_claims(req,secret).ok_or_else(|| actix_web::HttpResponse::Unauthorized().json(serde_json::json!({"error":"Nicht angemeldet"})))?;
    let db = req.app_data::<web::Data<Mutex<Connection>>>().ok_or_else(|| actix_web::HttpResponse::InternalServerError().json(serde_json::json!({"error":"Die Kontodaten konnten nicht geprüft werden."})))?;
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    match Customer::find_by_id(&conn,claims.sub) {
        Ok(Some(customer)) if customer.email_verified => Ok(claims),
        Ok(Some(_)) => Err(actix_web::HttpResponse::Forbidden().json(serde_json::json!({"error":"Bitte bestätige zuerst deine E-Mail-Adresse, bevor du gespeicherte Kontakt- oder Termindaten änderst."}))),
        Ok(None) => Err(actix_web::HttpResponse::Unauthorized().json(serde_json::json!({"error":"Nicht angemeldet"}))),
        Err(_) => Err(actix_web::HttpResponse::InternalServerError().json(serde_json::json!({"error":"Die Kontodaten konnten nicht geprüft werden."}))),
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    pub const SECRET: &str = "test-only-auth-secret-with-at-least-32-bytes";

    pub fn config() -> Config {
        Config {
            host: "127.0.0.1".into(), port: 8080,
            base_url: "https://example.test".into(), database_url: ":memory:".into(),
            jwt_secret: SECRET.into(), jwt_expiry_hours: 24,
            admin_email: "admin@example.test".into(), admin_password: "test-password".into(),
            smtp_host: "localhost".into(), smtp_port: 465, smtp_user: String::new(),
            smtp_pass: String::new(), smtp_from: "test@example.test".into(), smtp_from_name: "Test".into(),
        }
    }

    pub fn database() -> web::Data<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            CREATE TABLE customers (
                id INTEGER PRIMARY KEY, email TEXT NOT NULL, password_hash TEXT NOT NULL,
                first_name TEXT DEFAULT '', last_name TEXT DEFAULT '', phone TEXT DEFAULT '',
                street TEXT DEFAULT '', zip_code TEXT DEFAULT '', city TEXT DEFAULT '', notes TEXT DEFAULT '',
                is_admin INTEGER DEFAULT 0, email_verified INTEGER DEFAULT 1, calendar_token TEXT,
                created_at TEXT DEFAULT '', updated_at TEXT DEFAULT '', reset_token TEXT,
                reset_token_expires TEXT, verification_token TEXT, verification_token_expires TEXT
            );
            INSERT INTO customers (id, email, password_hash, reset_token, reset_token_expires,
                verification_token, verification_token_expires)
            VALUES (1, 'sample@example.test', 'old-hash', 'reset-link', datetime('now', '+1 hour'),
                'verify-link', datetime('now', '+24 hours'));
            CREATE TABLE site_settings (setting_key TEXT PRIMARY KEY, setting_value TEXT NOT NULL);
            CREATE TABLE revoked_sessions (jti TEXT PRIMARY KEY, expires_at INTEGER NOT NULL);
        ").unwrap();
        web::Data::new(Mutex::new(conn))
    }

    pub fn token(password_hash: &str, is_admin: bool) -> String {
        create_token(1, "sample@example.test", is_admin, password_hash, SECRET, 24).unwrap()
    }

    pub fn request(db: &web::Data<Mutex<Connection>>, token: &str) -> HttpRequest {
        actix_web::test::TestRequest::default()
            .app_data(db.clone())
            .cookie(Cookie::new("auth_token", token.to_owned()))
            .to_http_request()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_support::*;

    #[test]
    fn logout_revokes_only_the_specific_session() {
        let db = database();
        let first = token("old-hash", false);
        let second = token("old-hash", false);
        assert_ne!(first, second);
        let claims = validate_token(&first, SECRET).unwrap();
        db.lock().unwrap().execute("INSERT INTO revoked_sessions(jti,expires_at) VALUES(?1,?2)",rusqlite::params![claims.jti,claims.exp as i64]).unwrap();
        assert!(get_claims(&request(&db,&first),SECRET).is_none());
        assert!(get_claims(&request(&db,&second),SECRET).is_some());
    }

    #[test]
    fn password_rotation_revokes_sessions_and_reset_links() {
        let db = database();
        let req = request(&db, &token("old-hash", false));
        assert!(get_claims(&req, SECRET).is_some());
        {
            let conn = db.lock().unwrap();
            Customer::update_password(&conn, 1, "new-hash").unwrap();
            assert!(Customer::find_by_reset_token(&conn, "reset-link").unwrap().is_none());
        }
        assert!(get_claims(&req, SECRET).is_none());
        assert!(get_claims(&request(&db, &token("new-hash", false)), SECRET).is_some());
    }

    #[test]
    fn email_change_revokes_old_sessions_and_mailbox_tokens() {
        let db = database();
        let req = request(&db, &token("old-hash", false));
        {
            let conn = db.lock().unwrap();
            Customer::update_email(&conn, 1, "new@example.test").unwrap();
            assert!(!Customer::find_by_id(&conn, 1).unwrap().unwrap().email_verified);
            assert!(Customer::find_by_reset_token(&conn, "reset-link").unwrap().is_none());
            assert!(Customer::find_by_verification_token(&conn, "verify-link").unwrap().is_none());
        }
        assert!(get_claims(&req, SECRET).is_none());
    }

    #[test]
    fn role_changes_and_account_deletion_revoke_tokens() {
        let db = database();
        let req = request(&db, &token("old-hash", false));
        db.lock().unwrap().execute("UPDATE customers SET is_admin = 1", []).unwrap();
        assert!(get_claims(&req, SECRET).is_none());
        let admin_req = request(&db, &token("old-hash", true));
        assert!(get_claims(&admin_req, SECRET).unwrap().is_admin);
        db.lock().unwrap().execute("DELETE FROM customers", []).unwrap();
        assert!(get_claims(&admin_req, SECRET).is_none());
    }

    #[test]
    fn rejects_passwords_that_bcrypt_would_silently_truncate() {
        let password = "a".repeat(72);
        let stored = bcrypt::hash(&password, 4).unwrap();
        assert!(verify_password(&password, &stored));
        assert!(!verify_password(&format!("{password}suffix"), &stored));
        assert!(password_validation_error(&"ä".repeat(37)).is_some());
        assert!(password_validation_error(&"ä".repeat(36)).is_none());
    }

    #[test]
    fn session_cookies_are_explicitly_restricted() {
        let mut config = config();
        let cookie = auth_cookie("test-token", &config);
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert_eq!(cookie.path(), Some("/"));
        config.base_url = "HTTPS://example.test".into();
        assert_eq!(auth_cookie("test-token", &config).secure(), Some(true));
        config.base_url = "http://localhost:8080".into();
        assert_eq!(auth_cookie("test-token", &config).secure(), Some(false));
    }
}
