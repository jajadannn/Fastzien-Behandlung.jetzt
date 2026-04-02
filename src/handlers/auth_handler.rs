use actix_web::{web, HttpRequest, HttpResponse, cookie::Cookie};
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;
use chrono::{Utc, Duration};
use uuid::Uuid;

use crate::auth;
use crate::config::Config;
use crate::email::EmailService;
use crate::models::customer::{Customer, LoginForm, RegisterForm, PasswordResetRequest, PasswordReset};

pub async fn login_page(tmpl: web::Data<Tera>, req: HttpRequest, jwt_secret: web::Data<String>) -> HttpResponse {
    if auth::get_claims(&req, &jwt_secret).is_some() {
        return HttpResponse::SeeOther().insert_header(("Location", "/portal")).finish();
    }
    let ctx = tera::Context::new();
    match tmpl.render("auth/login.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn register_page(tmpl: web::Data<Tera>, req: HttpRequest, jwt_secret: web::Data<String>) -> HttpResponse {
    if auth::get_claims(&req, &jwt_secret).is_some() {
        return HttpResponse::SeeOther().insert_header(("Location", "/portal")).finish();
    }
    let ctx = tera::Context::new();
    match tmpl.render("auth/register.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn reset_password_page(tmpl: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    match tmpl.render("auth/reset_password.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn reset_password_token_page(tmpl: web::Data<Tera>, path: web::Path<String>) -> HttpResponse {
    let token = path.into_inner();
    let mut ctx = tera::Context::new();
    ctx.insert("token", &token);
    match tmpl.render("auth/reset_password_confirm.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn api_login(
    form: web::Json<LoginForm>,
    db: web::Data<Mutex<Connection>>,
    config: web::Data<Config>,
) -> HttpResponse {
    let conn = db.lock().unwrap();
    let customer = match Customer::find_by_email(&conn, &form.email) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Ungültige Anmeldedaten"})),
    };

    if !auth::verify_password(&form.password, &customer.password_hash) {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Ungültige Anmeldedaten"}));
    }

    let token = match auth::create_token(customer.id, &customer.email, customer.is_admin, &config.jwt_secret, config.jwt_expiry_hours) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Token-Erstellung fehlgeschlagen"})),
    };

    let redirect = if customer.is_admin { "/admin" } else { "/portal" };

    let cookie = Cookie::build("auth_token", &token)
        .path("/")
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::hours(config.jwt_expiry_hours))
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({"success": true, "redirect": redirect, "is_admin": customer.is_admin}))
}

pub async fn api_register(
    form: web::Json<RegisterForm>,
    db: web::Data<Mutex<Connection>>,
    config: web::Data<Config>,
    email_service: web::Data<EmailService>,
) -> HttpResponse {
    if form.email.is_empty() || form.password.is_empty() || form.first_name.is_empty() || form.last_name.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Bitte alle Pflichtfelder ausfüllen"}));
    }
    if form.password.len() < 6 {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Passwort muss mindestens 6 Zeichen lang sein"}));
    }

    let conn = db.lock().unwrap();
    if let Ok(Some(_)) = Customer::find_by_email(&conn, &form.email) {
        return HttpResponse::Conflict().json(serde_json::json!({"error": "E-Mail-Adresse ist bereits registriert"}));
    }

    let password_hash = auth::hash_password(&form.password);
    let phone = form.phone.as_deref().unwrap_or("");
    match Customer::create(&conn, &form.email, &password_hash, &form.first_name, &form.last_name, phone) {
        Ok(id) => {
            let token = auth::create_token(id, &form.email, false, &config.jwt_secret, config.jwt_expiry_hours).unwrap_or_default();
            let cookie = Cookie::build("auth_token", &token)
                .path("/")
                .http_only(true)
                .max_age(actix_web::cookie::time::Duration::hours(config.jwt_expiry_hours))
                .finish();

            // Send welcome email in background
            let es = email_service.get_ref().clone();
            let email = form.email.clone();
            let name = form.first_name.clone();
            tokio::spawn(async move {
                es.send_welcome(&email, &name);
            });

            HttpResponse::Ok()
                .cookie(cookie)
                .json(serde_json::json!({"success": true, "redirect": "/portal"}))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Registrierung fehlgeschlagen: {}", e)})),
    }
}

pub async fn logout_page(tmpl: web::Data<Tera>) -> HttpResponse {
    let cookie = Cookie::build("auth_token", "")
        .path("/")
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();

    let ctx = tera::Context::new();
    match tmpl.render("auth/logout.html", &ctx) {
        Ok(body) => HttpResponse::Ok()
            .cookie(cookie)
            .content_type("text/html; charset=utf-8")
            .body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn api_reset_password_request(
    form: web::Json<PasswordResetRequest>,
    db: web::Data<Mutex<Connection>>,
    email_service: web::Data<EmailService>,
) -> HttpResponse {
    let conn = db.lock().unwrap();
    // Always return success to prevent email enumeration
    if let Ok(Some(customer)) = Customer::find_by_email(&conn, &form.email) {
        let token = Uuid::new_v4().to_string();
        let expires = (Utc::now() + Duration::hours(1)).format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = Customer::set_reset_token(&conn, customer.id, &token, &expires);

        let reset_url = format!("https://faszien-behandlung.jetzt/reset-password/{}", token);
        let es = email_service.get_ref().clone();
        let name = customer.full_name();
        let email = customer.email.clone();
        tokio::spawn(async move {
            es.send_password_reset(&email, &name, &reset_url);
        });
    }

    HttpResponse::Ok().json(serde_json::json!({"success": true, "message": "Falls ein Konto mit dieser E-Mail existiert, wurde ein Link zum Zurücksetzen gesendet."}))
}

pub async fn api_reset_password_confirm(
    path: web::Path<String>,
    form: web::Json<PasswordReset>,
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let token = path.into_inner();
    if form.new_password.len() < 6 {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Passwort muss mindestens 6 Zeichen lang sein"}));
    }

    let conn = db.lock().unwrap();
    match Customer::find_by_reset_token(&conn, &token) {
        Ok(Some(customer)) => {
            let password_hash = auth::hash_password(&form.new_password);
            let _ = Customer::update_password(&conn, customer.id, &password_hash);
            let _ = Customer::clear_reset_token(&conn, customer.id);
            HttpResponse::Ok().json(serde_json::json!({"success": true, "message": "Passwort erfolgreich geändert. Du kannst dich jetzt anmelden."}))
        }
        _ => HttpResponse::BadRequest().json(serde_json::json!({"error": "Link ungültig oder abgelaufen"})),
    }
}
