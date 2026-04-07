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

/// GET /verify-email/{token}
/// Called when the customer clicks the link in the verification email.
pub async fn verify_email_page(
    tmpl: web::Data<Tera>,
    path: web::Path<String>,
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let token = path.into_inner();
    let conn = db.lock().unwrap();

    let mut ctx = tera::Context::new();
    match Customer::find_by_verification_token(&conn, &token) {
        Ok(Some(customer)) => {
            let _ = Customer::mark_email_verified(&conn, customer.id);
            ctx.insert("success", &true);
            ctx.insert("message", "Deine E-Mail-Adresse wurde erfolgreich bestätigt. Du kannst jetzt alle Funktionen nutzen.");
        }
        _ => {
            ctx.insert("success", &false);
            ctx.insert("message", "Der Bestätigungslink ist ungültig oder abgelaufen. Bitte fordere einen neuen Link in deinem Kundenportal an.");
        }
    }

    match tmpl.render("auth/verify_email_result.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

/// POST /api/customer/resend-verification
/// Resends the e-mail verification link (5-minute cooldown).
pub async fn api_resend_verification(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    email_service: web::Data<EmailService>,
    config: web::Data<Config>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    let conn = db.lock().unwrap();
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Konto nicht gefunden"})),
    };

    if customer.email_verified {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "E-Mail bereits bestätigt"}));
    }

    let token = Uuid::new_v4().to_string();
    let expires = (Utc::now() + Duration::hours(24)).format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = Customer::set_verification_token(&conn, customer.id, &token, &expires);

    let base_url = config.base_url.clone();
    let verify_url = format!("{}/verify-email/{}", base_url, token);
    let es = email_service.get_ref().clone();
    let email = customer.email.clone();
    let name = customer.first_name.clone();
    tokio::spawn(async move {
        es.send_email_verification(&email, &name, &verify_url);
    });

    HttpResponse::Ok().json(serde_json::json!({"success": true, "message": "Bestätigungs-E-Mail wurde erneut gesendet."}))
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

    // Admin and customers both land on /portal (admin nav link visible there)
    let cookie = Cookie::build("auth_token", &token)
        .path("/")
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::hours(config.jwt_expiry_hours))
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({"success": true, "redirect": "/portal", "is_admin": customer.is_admin}))
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
    let street = &form.street;
    let postal_code = &form.postal_code;
    let city = &form.city;
    let calendar_token = Uuid::new_v4().to_string();

    match Customer::create(
        &conn, &form.email, &password_hash,
        &form.first_name, &form.last_name,
        phone, street, postal_code, city, &calendar_token,
    ) {
        Ok(id) => {
            // Generate verification token (24h expiry)
            let v_token = Uuid::new_v4().to_string();
            let v_expires = (Utc::now() + Duration::hours(24)).format("%Y-%m-%d %H:%M:%S").to_string();
            let _ = Customer::set_verification_token(&conn, id, &v_token, &v_expires);

            // Issue JWT so the user is logged in immediately (portal is gated on email_verified)
            let jwt = auth::create_token(id, &form.email, false, &config.jwt_secret, config.jwt_expiry_hours).unwrap_or_default();
            let cookie = Cookie::build("auth_token", &jwt)
                .path("/")
                .http_only(true)
                .max_age(actix_web::cookie::time::Duration::hours(config.jwt_expiry_hours))
                .finish();

            let base_url = config.base_url.clone();
            let verify_url = format!("{}/verify-email/{}", base_url, v_token);
            let es = email_service.get_ref().clone();
            let email = form.email.clone();
            let name = form.first_name.clone();
            tokio::spawn(async move {
                es.send_email_verification(&email, &name, &verify_url);
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
    config: web::Data<Config>,
    email_service: web::Data<EmailService>,
) -> HttpResponse {
    let conn = db.lock().unwrap();
    // Always return success to prevent email enumeration
    if let Ok(Some(customer)) = Customer::find_by_email(&conn, &form.email) {
        let token = Uuid::new_v4().to_string();
        let expires = (Utc::now() + Duration::hours(1)).format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = Customer::set_reset_token(&conn, customer.id, &token, &expires);

        let reset_url = format!("{}/reset-password/{}", config.base_url, token);
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
