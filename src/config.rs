use std::env;

#[derive(Clone, Debug)]
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
        Config {
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
                .unwrap_or_else(|_| "admin".to_string()),
            admin_password: env::var("ADMIN_PASSWORD")
                .unwrap_or_else(|_| "admin".to_string()),
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
        }
    }
}
