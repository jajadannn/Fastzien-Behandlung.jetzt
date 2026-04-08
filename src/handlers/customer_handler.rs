use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;

use crate::auth;
use crate::models::customer::{Customer, ProfileUpdate, PasswordChange, EmailChange};
use crate::models::appointment::Appointment;
use crate::models::payment::{Payment, CreditPackage};
use crate::models::settings::SiteSetting;

fn require_auth(req: &HttpRequest, jwt_secret: &str) -> Result<auth::Claims, HttpResponse> {
    match auth::get_claims(req, jwt_secret) {
        Some(claims) => Ok(claims),
        None => Err(HttpResponse::SeeOther().insert_header(("Location", "/login")).finish()),
    }
}

/// Renders the "please verify your email" page when the customer is not yet verified.
fn email_not_verified_response(tmpl: &Tera, email: &str, is_admin: bool) -> HttpResponse {
    let mut ctx = tera::Context::new();
    ctx.insert("email", email);
    ctx.insert("is_admin", &is_admin);
    match tmpl.render("customer/email_not_verified.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn dashboard(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();

    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }

    let upcoming = Appointment::find_upcoming_by_customer(&conn, claims.sub).unwrap_or_default();
    let pending_amount = Payment::pending_total_by_customer(&conn, claims.sub).unwrap_or(0.0);
    let remaining_credits = CreditPackage::remaining_sessions(&conn, claims.sub).unwrap_or(0);
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();

    let next_appointment = upcoming.first().cloned();

    let mut ctx = tera::Context::new();
    ctx.insert("customer", &customer);
    ctx.insert("upcoming_appointments", &upcoming);
    ctx.insert("next_appointment", &next_appointment);
    ctx.insert("pending_amount", &pending_amount);
    ctx.insert("remaining_credits", &remaining_credits);
    ctx.insert("settings", &settings);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/dashboard.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => {
            let mut err_msg = format!("Template error: {}", e);
            let mut cause = std::error::Error::source(&e);
            while let Some(src) = cause {
                err_msg.push_str(&format!("\nCaused by: {}", src));
                cause = std::error::Error::source(src);
            }
            HttpResponse::InternalServerError().body(err_msg)
        }
    }
}

pub async fn appointments_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }

    let appointments = Appointment::find_by_customer(&conn, claims.sub).unwrap_or_default();
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("appointments", &appointments);
    ctx.insert("settings", &settings);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/appointments.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn book_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }

    let remaining_credits = CreditPackage::remaining_sessions(&conn, claims.sub).unwrap_or(0);
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("remaining_credits", &remaining_credits);
    ctx.insert("settings", &settings);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/book_appointment.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn profile_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }

    let mut ctx = tera::Context::new();
    ctx.insert("customer", &customer);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/profile.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn credits_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }

    let payments = Payment::find_by_customer(&conn, claims.sub).unwrap_or_default();
    let credit_packages = CreditPackage::find_all_by_customer(&conn, claims.sub).unwrap_or_default();
    let pending_amount = Payment::pending_total_by_customer(&conn, claims.sub).unwrap_or(0.0);
    let paid_amount = Payment::paid_total_by_customer(&conn, claims.sub).unwrap_or(0.0);
    let remaining_credits = CreditPackage::remaining_sessions(&conn, claims.sub).unwrap_or(0);

    let mut ctx = tera::Context::new();
    ctx.insert("payments", &payments);
    ctx.insert("credit_packages", &credit_packages);
    ctx.insert("pending_amount", &pending_amount);
    ctx.insert("paid_amount", &paid_amount);
    ctx.insert("remaining_credits", &remaining_credits);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/credits.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn api_update_profile(
    req: HttpRequest,
    form: web::Json<ProfileUpdate>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();

    let first_name = form.first_name.as_deref().unwrap_or(&customer.first_name);
    let last_name = form.last_name.as_deref().unwrap_or(&customer.last_name);
    let phone = form.phone.as_deref().unwrap_or(&customer.phone);
    let street = form.street.as_deref().unwrap_or(&customer.street);
    let zip_code = form.zip_code.as_deref().unwrap_or(&customer.zip_code);
    let city = form.city.as_deref().unwrap_or(&customer.city);

    match Customer::update_profile(&conn, claims.sub, first_name, last_name, phone, street, zip_code, city) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
    }
}

pub async fn api_change_password(
    req: HttpRequest,
    form: web::Json<PasswordChange>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    if form.new_password.len() < 6 {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Neues Passwort muss mindestens 6 Zeichen lang sein"}));
    }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();

    if !auth::verify_password(&form.current_password, &customer.password_hash) {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Aktuelles Passwort ist falsch"}));
    }

    let password_hash = auth::hash_password(&form.new_password);
    match Customer::update_password(&conn, claims.sub, &password_hash) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
    }
}

pub async fn api_change_email(
    req: HttpRequest,
    form: web::Json<EmailChange>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();

    if !auth::verify_password(&form.password, &customer.password_hash) {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Passwort ist falsch"}));
    }

    if let Ok(Some(_)) = Customer::find_by_email(&conn, &form.new_email) {
        return HttpResponse::Conflict().json(serde_json::json!({"error": "Diese E-Mail-Adresse wird bereits verwendet"}));
    }

    match Customer::update_email(&conn, claims.sub, &form.new_email) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
    }
}

/// GET /api/customer/export-data — DSGVO Art. 20 data portability export
pub async fn api_export_data(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Konto nicht gefunden"})),
    };
    let appointments = Appointment::find_by_customer(&conn, claims.sub).unwrap_or_default();
    let payments = Payment::find_by_customer(&conn, claims.sub).unwrap_or_default();

    let export = serde_json::json!({
        "export_date": chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        "account": {
            "id": customer.id,
            "email": customer.email,
            "first_name": customer.first_name,
            "last_name": customer.last_name,
            "phone": customer.phone,
            "street": customer.street,
            "zip_code": customer.zip_code,
            "city": customer.city,
            "created_at": customer.created_at,
            "email_verified": customer.email_verified,
        },
        "appointments": appointments,
        "payments": payments,
    });

    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .insert_header(("Content-Disposition", "attachment; filename=\"meine-daten.json\""))
        .body(serde_json::to_string_pretty(&export).unwrap_or_default())
}

/// DELETE /api/customer/delete-account — DSGVO Art. 17 right to erasure
pub async fn api_delete_account(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    // Prevent deletion of admin account
    if claims.is_admin {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Admin-Konto kann nicht selbst gelöscht werden"}));
    }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    // Cascade delete: appointments and payments are linked via foreign key or we delete them explicitly
    let customer_id = claims.sub;
    let _ = conn.execute("DELETE FROM payments WHERE customer_id = ?1", rusqlite::params![customer_id]);
    let _ = conn.execute("DELETE FROM appointments WHERE customer_id = ?1", rusqlite::params![customer_id]);
    let _ = conn.execute("DELETE FROM credit_packages WHERE customer_id = ?1", rusqlite::params![customer_id]);
    match conn.execute("DELETE FROM customers WHERE id = ?1", rusqlite::params![customer_id]) {
        Ok(_) => {
            // Clear the auth cookie
            HttpResponse::Ok()
                .cookie(
                    actix_web::cookie::Cookie::build("auth_token", "")
                        .path("/")
                        .max_age(actix_web::cookie::time::Duration::seconds(0))
                        .http_only(true)
                        .finish()
                )
                .json(serde_json::json!({"success": true}))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
    }
}
