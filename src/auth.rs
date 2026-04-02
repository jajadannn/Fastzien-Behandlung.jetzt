use actix_web::HttpRequest;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,       // customer id
    pub email: String,
    pub is_admin: bool,
    pub exp: usize,     // expiry timestamp
    pub iat: usize,     // issued at
}

pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Failed to hash password")
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

pub fn create_token(customer_id: i64, email: &str, is_admin: bool, secret: &str, expiry_hours: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: customer_id,
        email: email.to_string(),
        is_admin,
        exp: (now + Duration::hours(expiry_hours)).timestamp() as usize,
        iat: now.timestamp() as usize,
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
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }
    None
}

pub fn get_claims(req: &HttpRequest, secret: &str) -> Option<Claims> {
    let token = extract_token(req)?;
    validate_token(&token, secret).ok()
}
