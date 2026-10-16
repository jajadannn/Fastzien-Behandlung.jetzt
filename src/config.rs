use std::env;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub admin_email: String,
    pub admin_password: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from: String,
    pub smtp_from_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        let mut config = Config {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
            base_url: env::var("BASE_URL")
                .unwrap_or_else(|_| "https://faszienbehandlung.jetzt".to_string()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "data/faszien.db".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRY_HOURS must be a number"),
            admin_email: env::var("ADMIN_EMAIL")
                .expect("ADMIN_EMAIL must be set"),
            admin_password: env::var("ADMIN_PASSWORD")
                .expect("ADMIN_PASSWORD must be set"),
            smtp_host: env::var("SMTP_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "465".to_string())
                .parse()
                .expect("SMTP_PORT must be a number"),
            smtp_user: env::var("SMTP_USER").unwrap_or_default(),
            smtp_pass: env::var("SMTP_PASS").unwrap_or_default(),
            smtp_from: env::var("SMTP_FROM")
                .unwrap_or_else(|_| "noreply@example.com".to_string()),
            smtp_from_name: env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "Faszienbehandlung".to_string()),
        };
        config.validate().expect("Invalid server configuration");
        config.base_url = reqwest::Url::parse(&config.base_url).expect("validated BASE_URL")
            .as_str().trim_end_matches('/').to_owned();
        config
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.jwt_secret.len() < 32 || self.jwt_secret.starts_with("change_this") {
            return Err("JWT_SECRET must contain at least 32 random bytes; replace example values");
        }
        if !(1..=168).contains(&self.jwt_expiry_hours) {
            return Err("JWT_EXPIRY_HOURS must be between 1 and 168");
        }
        if self.admin_password.len() < 12 || self.admin_password.len() > 72
            || self.admin_password.starts_with("change_this") {
            return Err("ADMIN_PASSWORD must contain 12 to 72 bytes; replace example values");
        }
        if self.admin_email.parse::<lettre::Address>().is_err() {
            return Err("ADMIN_EMAIL must be a valid email address");
        }
        validate_base_url(&self.base_url)
    }
}

fn validate_base_url(value: &str) -> Result<(), &'static str> {
    let url = reqwest::Url::parse(value).map_err(|_| "BASE_URL must be an absolute URL")?;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
        || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some()
        || url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err("BASE_URL must be an HTTPS origin (HTTP is allowed only for loopback development)");
    }
    Ok(())
}

#[cfg(test)]
pub fn test_config() -> Config {
    Config {
        host: "127.0.0.1".into(), port: 8080, base_url: "https://example.com".into(),
        database_url: ":memory:".into(), jwt_secret: "test-only-signing-key-with-at-least-32-bytes".into(),
        jwt_expiry_hours: 24, admin_email: "admin@example.com".into(), admin_password: "test-admin-password".into(),
        smtp_host: "localhost".into(), smtp_port: 465, smtp_user: String::new(), smtp_pass: String::new(),
        smtp_from: "test@example.com".into(), smtp_from_name: "Test".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_requires_a_trusted_origin() {
        for value in ["https://example.com", "https://example.com/", "http://localhost:8080", "http://127.0.0.1:8080"] {
            assert!(validate_base_url(value).is_ok(), "{value}");
        }
        for value in ["http://example.com", "https://user:pass@example.com", "https://example.com/path", "https://example.com/?q=secret", "https://example.com/#fragment", "javascript:alert(1)"] {
            assert!(validate_base_url(value).is_err(), "{value}");
        }
    }

    #[test]
    fn unsafe_credentials_and_expiry_fail_at_startup() {
        let mut config = test_config();
        assert!(config.validate().is_ok());
        config.jwt_secret = "change_this_to_a_random_64_char_hex_string".into();
        assert!(config.validate().is_err());
        config = test_config();
        config.admin_password = "admin".into();
        assert!(config.validate().is_err());
        config = test_config();
        config.jwt_expiry_hours = i64::MAX;
        assert!(config.validate().is_err());
    }
}
