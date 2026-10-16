use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;

use crate::auth;
use crate::models::customer::{Customer, ProfileUpdate, PasswordChange, EmailChange};
use crate::models::appointment::{Appointment, WaitlistEntry};
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
        Err(_) => super::api::internal_error("render page"),
    }
}

/// Renders the profile page with a completion prompt when the customer registered
/// with only email + password and hasn't filled in name/address/phone yet.
fn profile_incomplete_response(tmpl: &Tera, customer: &Customer, is_admin: bool) -> HttpResponse {
    let mut ctx = tera::Context::new();
    ctx.insert("customer", customer);
    ctx.insert("is_admin", &is_admin);
    ctx.insert("profile_incomplete", &true);
    match tmpl.render("customer/profile.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => super::api::internal_error("render page"),
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
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(customer)) => customer,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error":"Nicht angemeldet"})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Das Konto konnte nicht geladen werden."})),
    };

    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }
    if customer.first_name.is_empty() {
        return profile_incomplete_response(&tmpl, &customer, claims.is_admin);
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
    let active_credits = CreditPackage::find_active_by_customer(&conn,claims.sub).unwrap_or_default();
    let price = active_credits.first().map(|c| c.price_per_session).unwrap_or_else(|| SiteSetting::get_or_default(&conn,"price_single","195").replace(',',".").parse::<f64>().unwrap_or(195.0));
    ctx.insert("session_price", &price);
    ctx.insert("customer_address", &customer.full_address());
    ctx.insert("remaining_credits", &remaining_credits);
    ctx.insert("settings", &settings);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/dashboard.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => super::api::internal_error("render customer dashboard"),
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
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(customer)) => customer,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error":"Nicht angemeldet"})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Das Konto konnte nicht geladen werden."})),
    };
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }
    if customer.first_name.is_empty() {
        return profile_incomplete_response(&tmpl, &customer, claims.is_admin);
    }

    let appointments = Appointment::find_by_customer(&conn, claims.sub).unwrap_or_default();
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("appointments", &appointments);
    ctx.insert("settings", &settings);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/appointments.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => super::api::internal_error("render page"),
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
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(customer)) => customer,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error":"Nicht angemeldet"})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Das Konto konnte nicht geladen werden."})),
    };
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }
    if customer.first_name.is_empty() {
        return profile_incomplete_response(&tmpl, &customer, claims.is_admin);
    }

    let remaining_credits = CreditPackage::remaining_sessions(&conn, claims.sub).unwrap_or(0);
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();

    let mut ctx = tera::Context::new();
    let active_credits = CreditPackage::find_active_by_customer(&conn,claims.sub).unwrap_or_default();
    let price = active_credits.first().map(|c| c.price_per_session).unwrap_or_else(|| SiteSetting::get_or_default(&conn,"price_single","195").replace(',',".").parse::<f64>().unwrap_or(195.0));
    ctx.insert("session_price", &price);
    ctx.insert("customer_address", &customer.full_address());
    ctx.insert("is_guest", &false);
    ctx.insert("remaining_credits", &remaining_credits);
    ctx.insert("settings", &settings);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/book_appointment.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => super::api::internal_error("render page"),
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
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(customer)) => customer,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error":"Nicht angemeldet"})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Das Konto konnte nicht geladen werden."})),
    };
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }

    let mut ctx = tera::Context::new();
    ctx.insert("customer", &customer);
    ctx.insert("is_admin", &claims.is_admin);
    ctx.insert("profile_incomplete", &customer.first_name.is_empty());

    match tmpl.render("customer/profile.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => super::api::internal_error("render page"),
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
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(customer)) => customer,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error":"Nicht angemeldet"})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Das Konto konnte nicht geladen werden."})),
    };
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }
    if customer.first_name.is_empty() {
        return profile_incomplete_response(&tmpl, &customer, claims.is_admin);
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
        Err(_) => super::api::internal_error("render page"),
    }
}

pub async fn api_update_profile(
    req: HttpRequest,
    form: web::Json<ProfileUpdate>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::require_verified_claims(&req,&jwt_secret) {
        Ok(claims) => claims,
        Err(response) => return response,
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(customer)) => customer,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error":"Nicht angemeldet"})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Das Konto konnte nicht geladen werden."})),
    };

    let first_name = form.first_name.as_deref().unwrap_or(&customer.first_name);
    let last_name = form.last_name.as_deref().unwrap_or(&customer.last_name);
    let phone = form.phone.as_deref().unwrap_or(&customer.phone);
    let street = form.street.as_deref().unwrap_or(&customer.street);
    let zip_code = form.zip_code.as_deref().unwrap_or(&customer.zip_code);
    let city = form.city.as_deref().unwrap_or(&customer.city);

    match Customer::update_profile(&conn, claims.sub, first_name, last_name, phone, street, zip_code, city) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(_) => super::api::internal_error("database operation"),
    }
}

pub async fn api_change_password(
    req: HttpRequest,
    form: web::Json<PasswordChange>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    config: web::Data<crate::config::Config>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    if let Some(error) = auth::password_validation_error(&form.new_password) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": error}));
    }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(customer)) => customer,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    if !auth::verify_password(&form.current_password, &customer.password_hash) {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Aktuelles Passwort ist falsch"}));
    }

    let password_hash = auth::hash_password(&form.new_password);
    let token = match auth::create_token(customer.id, &customer.email, customer.is_admin, &password_hash, &jwt_secret, config.jwt_expiry_hours) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Passwort konnte nicht geändert werden."})),
    };
    match Customer::update_password(&conn, claims.sub, &password_hash) {
        Ok(_) => HttpResponse::Ok().cookie(auth::auth_cookie(&token, &config)).json(serde_json::json!({"success": true})),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "Passwort konnte nicht geändert werden."})),
    }
}

pub async fn api_change_email(
    req: HttpRequest,
    form: web::Json<EmailChange>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    config: web::Data<crate::config::Config>,
    email_service: web::Data<crate::email::EmailService>,
) -> HttpResponse {
    let claims = match auth::require_verified_claims(&req,&jwt_secret) {
        Ok(claims) => claims,
        Err(response) => return response,
    };

    let email = form.new_email.trim();
    if email.parse::<lettre::Address>().is_err() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Bitte eine gültige E-Mail-Adresse angeben."}));
    }
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(customer)) => customer,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    if !auth::verify_password(&form.password, &customer.password_hash) {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Passwort ist falsch"}));
    }

    if let Ok(Some(_)) = Customer::find_by_email(&conn, email) {
        return HttpResponse::Conflict().json(serde_json::json!({"error": "Diese E-Mail-Adresse wird bereits verwendet"}));
    }

    let token = match auth::create_token(customer.id, email, customer.is_admin, &customer.password_hash, &jwt_secret, config.jwt_expiry_hours) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "E-Mail konnte nicht geändert werden."})),
    };
    let verification_token = uuid::Uuid::new_v4().to_string();
    let expires = (chrono::Utc::now() + chrono::Duration::hours(24)).format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = match conn.unchecked_transaction() {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "E-Mail konnte nicht geändert werden."})),
    };
    if Customer::update_email(&tx, claims.sub, email).is_err()
        || Customer::set_verification_token(&tx, claims.sub, &verification_token, &expires).is_err() {
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "E-Mail konnte nicht geändert werden."}));
    }
    if tx.commit().is_err() {
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "E-Mail konnte nicht geändert werden."}));
    }
    drop(conn);
    let verify_url = format!("{}/verify-email/{}", config.base_url.trim_end_matches('/'), verification_token);
    let es = email_service.get_ref().clone();
    let email = email.to_owned();
    tokio::spawn(async move {
        es.send_email_verification(&email, &customer.first_name, &verify_url);
    });
    HttpResponse::Ok().cookie(auth::auth_cookie(&token, &config)).json(serde_json::json!({"success": true}))
}

fn customer_data_export(conn: &Connection, customer_id: i64) -> rusqlite::Result<serde_json::Value> {
    let customer = Customer::find_by_id(conn,customer_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let appointments = Appointment::find_by_customer(conn,customer_id)?;
    let payments = Payment::find_by_customer(conn,customer_id)?;
    let credits = CreditPackage::find_all_by_customer(conn,customer_id)?;
    let waitlist = WaitlistEntry::find_by_customer(conn,customer_id)?;
    let terms = {
        let mut stmt = conn.prepare("SELECT b.appointment_id,b.version,b.early_start_requested,b.agreed_total,b.duration_min,b.terms_text,b.created_at FROM booking_terms b JOIN appointments a ON a.id=b.appointment_id WHERE a.customer_id=?1 ORDER BY b.appointment_id")?;
        let rows = stmt.query_map([customer_id],|r| Ok(serde_json::json!({
            "appointment_id":r.get::<_,i64>(0)?,"version":r.get::<_,String>(1)?,"early_start_requested":r.get::<_,bool>(2)?,
            "agreed_total":r.get::<_,f64>(3)?,"duration_min":r.get::<_,i64>(4)?,"confirmation":r.get::<_,String>(5)?,"created_at_utc":r.get::<_,String>(6)?
        })))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let sources = {
        let mut stmt = conn.prepare("SELECT s.appointment_id,s.source,s.city FROM booking_sources s JOIN appointments a ON a.id=s.appointment_id WHERE a.customer_id=?1 ORDER BY s.appointment_id")?;
        let rows = stmt.query_map([customer_id],|r| Ok(serde_json::json!({"appointment_id":r.get::<_,i64>(0)?,"source":r.get::<_,String>(1)?,"city":r.get::<_,String>(2)?})))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let requests = {
        // Export recorded contact/request data, never the bearer confirmation link.
        let mut stmt = conn.prepare("SELECT date,time,first_name,last_name,email,phone,is_home_visit,street,zip_code,city,source,created_at,expires_at,consumed_at FROM booking_requests WHERE email=?1 COLLATE NOCASE ORDER BY created_at")?;
        let rows = stmt.query_map([&customer.email],|r| Ok(serde_json::json!({
            "date":r.get::<_,String>(0)?,"time":r.get::<_,String>(1)?,"first_name":r.get::<_,String>(2)?,"last_name":r.get::<_,String>(3)?,
            "email":r.get::<_,String>(4)?,"phone":r.get::<_,String>(5)?,"is_home_visit":r.get::<_,bool>(6)?,"street":r.get::<_,String>(7)?,
            "zip_code":r.get::<_,String>(8)?,"city":r.get::<_,String>(9)?,"source":r.get::<_,String>(10)?,
            "created_at_utc":r.get::<_,String>(11)?,"expires_at_utc":r.get::<_,String>(12)?,"confirmed_at_utc":r.get::<_,Option<String>>(13)?
        })))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let withdrawals = {
        let mut stmt = conn.prepare("SELECT id,name,email,contract,created_at,processed_at FROM withdrawals WHERE email=?1 COLLATE NOCASE ORDER BY created_at")?;
        let rows = stmt.query_map([&customer.email],|r| Ok(serde_json::json!({
            "id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"email":r.get::<_,String>(2)?,"contract":r.get::<_,String>(3)?,
            "received_at_utc":r.get::<_,String>(4)?,"processed_at_utc":r.get::<_,Option<String>>(5)?
        })))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let pending_confirmations = {
        // Only the plain-text contractual receipts currently produced by this
        // application. Future reset/verification messages must not expose links.
        let mut stmt = conn.prepare("SELECT subject,body,created_at,attempts FROM email_outbox WHERE recipient=?1 COLLATE NOCASE AND sent_at IS NULL AND subject IN ('Verbindliche Buchungsbestätigung','Eingangsbestätigung Widerruf') ORDER BY id")?;
        let rows = stmt.query_map([&customer.email],|r| Ok(serde_json::json!({"subject":r.get::<_,String>(0)?,"content":r.get::<_,String>(1)?,"created_at_utc":r.get::<_,String>(2)?,"delivery_attempts":r.get::<_,i64>(3)?})))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    Ok(serde_json::json!({
        "export_date":chrono::Utc::now().to_rfc3339(),
        "account":{"id":customer.id,"email":customer.email,"first_name":customer.first_name,"last_name":customer.last_name,
            "phone":customer.phone,"street":customer.street,"zip_code":customer.zip_code,"city":customer.city,
            "created_at":customer.created_at,"email_verified":customer.email_verified,"notes":customer.notes},
        "appointments":appointments,"payments":payments,"credit_packages":credits,"waitlist":waitlist,
        "booking_confirmations":terms,"booking_sources":sources,"booking_requests":requests,
        "withdrawals":withdrawals,"pending_confirmation_messages":pending_confirmations
    }))
}

/// GET /api/customer/export-data — authenticated data copy, scoped to the current account.
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
    let export = match customer_data_export(&conn,claims.sub) {
        Ok(export) => export,
        Err(rusqlite::Error::QueryReturnedNoRows) => return HttpResponse::NotFound().json(serde_json::json!({"error":"Konto nicht gefunden"})),
        Err(_) => return super::api::internal_error("export customer data"),
    };
    // A password on an unverified signup does not establish ownership of the
    // e-mail that may also identify later guest bookings or withdrawal records.
    if export["account"]["email_verified"] != serde_json::Value::Bool(true) {
        return HttpResponse::Forbidden().json(serde_json::json!({"error":"Bitte bestätige zuerst deine E-Mail-Adresse, bevor du die gespeicherten Daten exportierst. Alternativ kannst du Auskunft direkt bei der Praxis anfragen."}));
    }
    match serde_json::to_string_pretty(&export) {
        Ok(body) => HttpResponse::Ok().content_type("application/json; charset=utf-8")
            .insert_header(("Content-Disposition","attachment; filename=\"meine-daten.json\""))
            .body(body),
        Err(_) => super::api::internal_error("serialize customer data export"),
    }

}

/// DELETE /api/customer/delete-account — DSGVO Art. 17 right to erasure
pub async fn api_delete_account(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::require_verified_claims(&req,&jwt_secret) {
        Ok(claims) => claims,
        Err(response) => return response,
    };

    // Prevent deletion of admin account
    if claims.is_admin {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Admin-Konto kann nicht selbst gelöscht werden"}));
    }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer_id = claims.sub;
    if Appointment::find_upcoming_by_customer(&conn,customer_id).is_ok_and(|a| !a.is_empty()) {
        return HttpResponse::Conflict().json(serde_json::json!({"error":"Es bestehen noch zukünftige Termine. Bitte sage sie zuerst ab oder kontaktiere uns zur Klärung und Löschung. Gesetzliche Rechte bleiben unberührt."}));
    }
    match crate::db::delete_customer(&conn, customer_id) {
        Ok(true) => {
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
        Ok(false) | Err(rusqlite::Error::QueryReturnedNoRows) => HttpResponse::NotFound().json(serde_json::json!({"error":"Konto nicht gefunden"})),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error":"Das Konto konnte nicht gelöscht werden. Es wurden keine Daten verändert."})),
    }
}

/// GET /api/customer/invoices/{payment_id} — printable HTML invoice
pub async fn api_invoice(
    req: HttpRequest,
    path: web::Path<i64>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Nicht angemeldet"),
    };

    let payment_id = path.into_inner();
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    let payment = match Payment::find_by_id(&conn, payment_id) {
        Ok(Some(p)) => p,
        _ => return HttpResponse::NotFound().body("Zahlung nicht gefunden"),
    };

    if payment.customer_id != claims.sub && !claims.is_admin {
        return HttpResponse::Forbidden().body("Zugriff verweigert");
    }

    let customer = match Customer::find_by_id(&conn, payment.customer_id) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().body("Kunde nicht gefunden"),
    };
    if !claims.is_admin && !customer.email_verified {
        return HttpResponse::Forbidden().body("Bitte bestätige zuerst deine E-Mail-Adresse, bevor du Rechnungen öffnest.");
    }

    let settings = SiteSetting::get_all(&conn);
    let settings = settings.unwrap_or_default();
    let address_street = settings.get("address_street").map(|s| s.as_str()).unwrap_or("Sulgauer Straße 24");
    let address_zip = settings.get("address_zip").map(|s| s.as_str()).unwrap_or("78713");
    let address_city = settings.get("address_city").map(|s| s.as_str()).unwrap_or("Schramberg-Sulgen");
    let phone = settings.get("phone").map(|s| s.as_str()).unwrap_or("+49 152 34 00 72 25");

    let appointment_info = payment.appointment_id
        .and_then(|id| Appointment::find_by_id(&conn, id).ok().flatten())
        .map(|a| {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&a.start_time, "%Y-%m-%d %H:%M:%S") {
                format!("Faszienbehandlung am {} um {} Uhr", dt.format("%d.%m.%Y"), dt.format("%H:%M"))
            } else {
                "Faszienbehandlung".to_string()
            }
        })
        .unwrap_or_else(|| "Faszienbehandlung".to_string());

    let payment_type_label = match payment.payment_type.as_str() {
        "pack" => "10er-Karte (Einzelsitzung)",
        _ => "Einzelsitzung",
    };

    let invoice_date = payment.paid_at.as_deref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .or_else(|| chrono::NaiveDateTime::parse_from_str(&payment.created_at, "%Y-%m-%d %H:%M:%S").ok())
        .map(|dt| dt.format("%d.%m.%Y").to_string())
        .unwrap_or_else(|| "–".to_string());

    let customer_address = if customer.street.is_empty() {
        customer.city.clone()
    } else {
        format!("{}, {} {}", customer.street, customer.zip_code, customer.city)
    };

    let escape = |value: &str| value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('\"', "&quot;");
    let html = format!(r#"<!DOCTYPE html>
<html lang="de">
<head>
<meta charset="UTF-8">
<title>Rechnung #{}</title>
<style>
  body {{ font-family: 'Segoe UI', sans-serif; max-width: 700px; margin: 40px auto; color: #1a2a33; }}
  .header {{ display: flex; justify-content: space-between; border-bottom: 2px solid #70AECD; padding-bottom: 20px; margin-bottom: 30px; }}
  .company h2 {{ color: #70AECD; margin: 0 0 4px; }}
  .invoice-meta {{ text-align: right; }}
  .invoice-meta h1 {{ color: #964279; margin: 0; font-size: 28px; }}
  .addresses {{ display: flex; justify-content: space-between; margin-bottom: 30px; }}
  .address-block h4 {{ color: #6a8fa0; font-size: 12px; text-transform: uppercase; margin: 0 0 6px; }}
  table {{ width: 100%; border-collapse: collapse; margin-bottom: 20px; }}
  th {{ background: #f0f7fb; padding: 10px 12px; text-align: left; font-size: 13px; color: #3d5a6b; }}
  td {{ padding: 12px; border-bottom: 1px solid #e8f0f5; }}
  .total-row td {{ font-weight: bold; font-size: 16px; background: #f0f7fb; }}
  .footer {{ margin-top: 40px; font-size: 12px; color: #6a8fa0; border-top: 1px solid #e8f0f5; padding-top: 16px; }}
  @media print {{ .no-print {{ display: none; }} }}
</style>
</head>
<body>
<div class="header">
  <div class="company">
    <h2>Faszienbehandlung Thilo Seifried</h2>
    <p style="margin:0;color:#3d5a6b;">{}, {} {}<br>Tel: {}</p>
  </div>
  <div class="invoice-meta">
    <h1>RECHNUNG</h1>
    <p style="margin:4px 0;color:#3d5a6b;">Nr.: R-{:05}<br>Datum: {}</p>
  </div>
</div>

<div class="addresses">
  <div class="address-block">
    <h4>Rechnungsempfänger</h4>
    <strong>{}</strong><br>
    {}<br>
    {}
  </div>
</div>

<table>
  <thead><tr><th>Leistung</th><th>Art</th><th>Betrag</th></tr></thead>
  <tbody>
    <tr>
      <td>{}</td>
      <td>{}</td>
      <td>{:.2} €</td>
    </tr>
  </tbody>
  <tfoot>
    <tr class="total-row">
      <td colspan="2">Gesamtbetrag*</td>
      <td>{:.2} €</td>
    </tr>
  </tfoot>
</table>

<p style="font-size:13px;color:#3d5a6b;">* Gemäß § 19 UStG wird keine Umsatzsteuer berechnet (Kleinunternehmerregelung). Wellness- und Körperarbeitsleistung; keine Erstattung durch Krankenkassen oder Versicherungen.</p>

<div class="footer">
  <p>Faszienbehandlung Thilo Seifried · {} · {} {} · {} | Kein Mitglied einer Berufsorganisation</p>
</div>

<p class="no-print" style="margin-top:30px;">
  <button onclick="window.print()" style="background:#964279;color:white;border:none;padding:12px 28px;border-radius:8px;cursor:pointer;font-size:15px;">Drucken / Als PDF speichern</button>
  <button onclick="window.close()" style="background:#e5e7eb;color:#374151;border:none;padding:12px 20px;border-radius:8px;cursor:pointer;font-size:15px;margin-left:10px;">Schließen</button>
</p>
</body>
</html>"#,
        payment.id,
        escape(address_street), escape(address_zip), escape(address_city), escape(phone),
        payment.id, invoice_date,
        escape(&customer.full_name()),
        escape(&customer_address),
        escape(&customer.email),
        escape(&appointment_info),
        payment_type_label,
        payment.amount,
        payment.amount,
        escape(address_street), escape(address_zip), escape(address_city), escape(phone),
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

/// GET /portal/waitlist — customer waitlist management page
pub async fn waitlist_page(
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
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(customer)) => customer,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error":"Nicht angemeldet"})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Das Konto konnte nicht geladen werden."})),
    };
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }
    if customer.first_name.is_empty() {
        return profile_incomplete_response(&tmpl, &customer, claims.is_admin);
    }

    let waitlist = WaitlistEntry::find_by_customer(&conn, claims.sub).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("waitlist", &waitlist);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/waitlist.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => super::api::internal_error("render page"),
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    fn export_fixture() -> (Connection,i64,i64) {
        let conn = crate::db::test_connection();
        let mut ids = Vec::new();
        for who in ["owner","other"] {
            let email = format!("{who}@example.test");
            conn.execute("INSERT INTO customers(email,password_hash,first_name,last_name,email_verified,notes) VALUES(?1,'test',?2,'Customer',1,?2)",rusqlite::params![email,who]).unwrap();
            let customer = conn.last_insert_rowid();
            let appointment = Appointment::create(&conn,customer,"2030-01-07 16:00:00","2030-01-07 17:30:00","single",false,"").unwrap();
            conn.execute("INSERT INTO booking_sources(appointment_id,source,city) VALUES(?1,?2,?2)",rusqlite::params![appointment,format!("{who}-source")]).unwrap();
            conn.execute("INSERT INTO booking_requests(token,date,time,first_name,last_name,email,phone,expires_at) VALUES(?1,'2030-01-07','16:00',?2,'Requested',?3,'0123',datetime('now','+1 hour'))",rusqlite::params![format!("never-export-{who}-booking-token"),who,email.to_uppercase()]).unwrap();
            conn.execute("INSERT INTO withdrawals(id,name,email,contract,processed_at) VALUES(?1,?2,?3,?4,'2026-10-09 12:00:00')",rusqlite::params![format!("{who}-withdrawal"),format!("{who} original statement name"),email,format!("{who} contract")]).unwrap();
            crate::email::enqueue_text(&conn,&email,"Eingangsbestätigung Widerruf",&format!("{who} confirmation content")).unwrap();
            crate::email::enqueue_text(&conn,&email,"Passwort zurücksetzen",&format!("never-export-{who}-reset-token")).unwrap();
            ids.push(customer);
        }
        (conn,ids[0],ids[1])
    }

    #[actix_web::test]
    async fn data_export_contains_recorded_requests_and_sources_only_for_its_owner() {
        use actix_web::{test,App};
        let (conn,owner,other) = export_fixture();
        let secret = crate::auth::test_support::SECRET;
        let app = test::init_service(App::new().app_data(web::Data::new(Mutex::new(conn)))
            .app_data(web::Data::new(secret.to_owned())).route("/api/customer/export-data",web::get().to(api_export_data))).await;
        for (id,who,excluded) in [(owner,"owner","other"),(other,"other","owner")] {
            let token = crate::auth::create_token(id,&format!("{who}@example.test"),false,"test",secret,24).unwrap();
            let req = test::TestRequest::get().uri("/api/customer/export-data").cookie(actix_web::cookie::Cookie::new("auth_token",token)).to_request();
            let response = test::call_service(&app,req).await;
            assert_eq!(response.status(),actix_web::http::StatusCode::OK);
            let body = String::from_utf8(test::read_body(response).await.to_vec()).unwrap();
            let data: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(data["booking_sources"][0]["source"],format!("{who}-source"));
            assert_eq!(data["booking_requests"][0]["first_name"],who);
            assert_eq!(data["withdrawals"][0]["name"],format!("{who} original statement name"));
            assert_eq!(data["withdrawals"][0]["processed_at_utc"],"2026-10-09 12:00:00");
            assert_eq!(data["pending_confirmation_messages"].as_array().unwrap().len(),1);
            assert!(!body.contains(&format!("{excluded}@example.test")));
            assert!(!body.contains(&format!("{excluded}-source")));
            assert!(!body.contains("never-export-") && !body.contains("password_hash"));
        }
    }

    #[actix_web::test]
    async fn export_query_failure_returns_an_error_instead_of_a_partial_data_copy() {
        let (conn,owner,_) = export_fixture();
        conn.execute("DROP TABLE booking_sources",[]).unwrap();
        let db = web::Data::new(Mutex::new(conn));
        let secret = crate::auth::test_support::SECRET;
        let token = crate::auth::create_token(owner,"owner@example.test",false,"test",secret,24).unwrap();
        let req = actix_web::test::TestRequest::get().app_data(db.clone()).cookie(actix_web::cookie::Cookie::new("auth_token",token)).to_http_request();
        let response = api_export_data(req,db,web::Data::new(secret.to_owned())).await;
        assert_eq!(response.status(),actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
        let body = actix_web::body::to_bytes(response.into_body()).await.unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert!(!text.contains("booking_sources") && !text.contains("appointments"));
        assert!(!text.contains("SELECT") && !text.contains("no such table"));
    }

    #[actix_web::test]
    async fn unverified_signup_cannot_export_records_matched_to_its_email() {
        let (conn,owner,_) = export_fixture();
        conn.execute("UPDATE customers SET email_verified=0 WHERE id=?1",[owner]).unwrap();
        let db = web::Data::new(Mutex::new(conn));
        let secret = crate::auth::test_support::SECRET;
        let token = crate::auth::create_token(owner,"owner@example.test",false,"test",secret,24).unwrap();
        let req = actix_web::test::TestRequest::get().app_data(db.clone()).cookie(actix_web::cookie::Cookie::new("auth_token",token)).to_http_request();
        let response = api_export_data(req,db,web::Data::new(secret.to_owned())).await;
        assert_eq!(response.status(),actix_web::http::StatusCode::FORBIDDEN);
        let body = actix_web::body::to_bytes(response.into_body()).await.unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert!(text.contains("E-Mail-Adresse"));
        assert!(!text.contains("owner-source") && !text.contains("owner contract") && !text.contains("confirmation content"));
    }

    #[actix_web::test]
    async fn unverified_signup_cannot_read_a_linked_invoice() {
        let (conn,owner,_) = export_fixture();
        conn.execute("UPDATE customers SET email_verified=0 WHERE id=?1",[owner]).unwrap();
        let payment = Payment::create(&conn,owner,None,195.0,"single").unwrap();
        let db = web::Data::new(Mutex::new(conn));
        let secret = crate::auth::test_support::SECRET;
        let token = crate::auth::create_token(owner,"owner@example.test",false,"test",secret,24).unwrap();
        let req = actix_web::test::TestRequest::get().app_data(db.clone()).cookie(actix_web::cookie::Cookie::new("auth_token",token)).to_http_request();
        assert_eq!(api_invoice(req,web::Path::from(payment),db,web::Data::new(secret.to_owned())).await.status(),actix_web::http::StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn unverified_credentials_cannot_change_contacts_email_or_existing_bookings() {
        use actix_web::{test,App};
        let (conn,owner,_) = export_fixture();
        conn.execute("UPDATE customers SET email_verified=0 WHERE id=?1",[owner]).unwrap();
        let appointment: i64 = conn.query_row("SELECT id FROM appointments WHERE customer_id=?1",[owner],|r|r.get(0)).unwrap();
        let db = web::Data::new(Mutex::new(conn));
        let config = crate::auth::test_support::config();
        let token = crate::auth::create_token(owner,"owner@example.test",false,"test",&config.jwt_secret,24).unwrap();
        let app = test::init_service(App::new().app_data(db.clone())
            .app_data(web::Data::new(config.jwt_secret.clone())).app_data(web::Data::new(crate::email::EmailService::new(&config))).app_data(web::Data::new(config))
            .route("/api/customer/profile",web::post().to(api_update_profile))
            .route("/api/customer/change-email",web::post().to(api_change_email))
            .route("/api/customer/delete-account",web::delete().to(api_delete_account))
            .route("/api/appointments/{id}/cancel",web::post().to(crate::handlers::booking_handler::api_cancel))
            .route("/api/appointments/{id}/reschedule",web::post().to(crate::handlers::booking_handler::api_reschedule))
            .route("/api/appointments/waitlist/remove",web::post().to(crate::handlers::booking_handler::api_waitlist_remove))).await;
        for (path,payload) in [
            ("/api/customer/profile".to_owned(),serde_json::json!({"first_name":"Changed"})),
            ("/api/customer/change-email".to_owned(),serde_json::json!({"new_email":"changed@example.test","password":"test"})),
            (format!("/api/appointments/{appointment}/cancel"),serde_json::json!({})),
            (format!("/api/appointments/{appointment}/reschedule"),serde_json::json!({"date":"2030-01-08","time":"16:00"})),
            ("/api/appointments/waitlist/remove".to_owned(),serde_json::json!({"date":"2030-01-07"}))
        ] {
            let req = test::TestRequest::post().uri(&path).cookie(actix_web::cookie::Cookie::new("auth_token",token.clone())).set_json(payload).to_request();
            assert_eq!(test::call_service(&app,req).await.status(),actix_web::http::StatusCode::FORBIDDEN,"{path}");
        }
        let req = test::TestRequest::delete().uri("/api/customer/delete-account").cookie(actix_web::cookie::Cookie::new("auth_token",token)).to_request();
        assert_eq!(test::call_service(&app,req).await.status(),actix_web::http::StatusCode::FORBIDDEN);
        let conn = db.lock().unwrap();
        let customer = Customer::find_by_id(&conn,owner).unwrap().unwrap();
        assert_eq!(customer.email,"owner@example.test");
        assert_eq!(customer.first_name,"owner");
        assert_eq!(Appointment::find_by_id(&conn,appointment).unwrap().unwrap().status,"confirmed");
    }

    #[actix_web::test]
    async fn template_failures_do_not_expose_template_names_or_internal_errors() {
        use actix_web::{test, App};
        let conn = crate::db::test_connection();
        conn.execute("INSERT INTO customers(email,password_hash,email_verified,first_name,last_name) VALUES('sample@example.test','old-hash',1,'Test','Customer')", []).unwrap();
        let db = web::Data::new(Mutex::new(conn));
        let app = test::init_service(App::new()
            .app_data(db)
            .app_data(web::Data::new(crate::auth::test_support::SECRET.to_string()))
            .app_data(web::Data::new(Tera::default()))
            .route("/portal", web::get().to(dashboard))).await;
        let request = test::TestRequest::get().uri("/portal")
            .cookie(actix_web::cookie::Cookie::new("auth_token", crate::auth::test_support::token("old-hash", false)))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
        let body = String::from_utf8(test::read_body(response).await.to_vec()).unwrap();
        assert!(!body.contains("customer/dashboard"));
        assert!(!body.contains("Template"));
        assert!(body.contains("später erneut"));
    }
}
