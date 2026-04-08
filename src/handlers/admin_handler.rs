use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;
use serde::Deserialize;
use reqwest;

use crate::auth;
use crate::email::EmailService;
use crate::models::customer::Customer;
use crate::models::appointment::Appointment;
use crate::models::payment::{Payment, CreditPackage};
use crate::models::faq::{Faq, FaqForm};
use crate::models::review::{Review, ReviewForm};
use crate::models::settings::SiteSetting;

fn require_admin(req: &HttpRequest, jwt_secret: &str) -> Result<auth::Claims, HttpResponse> {
    match auth::get_claims(req, jwt_secret) {
        Some(claims) if claims.is_admin => Ok(claims),
        Some(_) => Err(HttpResponse::Forbidden().body("Zugriff verweigert")),
        None => Err(HttpResponse::SeeOther().insert_header(("Location", "/login")).finish()),
    }
}

pub async fn dashboard(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customers = Customer::find_all_non_admin(&conn).unwrap_or_default();
    let upcoming = Appointment::find_upcoming_all(&conn).unwrap_or_default();
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();

    // Calculate totals
    let total_customers = customers.len();
    let total_upcoming = upcoming.len();

    let mut total_pending: f64 = 0.0;
    let mut total_paid: f64 = 0.0;

    // Build enriched customer data with per-customer amounts
    let customer_data: Vec<serde_json::Value> = customers.iter().map(|c| {
        let pending = Payment::pending_total_by_customer(&conn, c.id).unwrap_or(0.0);
        let paid = Payment::paid_total_by_customer(&conn, c.id).unwrap_or(0.0);
        total_pending += pending;
        total_paid += paid;
        serde_json::json!({
            "id": c.id,
            "email": c.email,
            "first_name": c.first_name,
            "last_name": c.last_name,
            "phone": c.phone,
            "pending_amount": pending,
        })
    }).collect();

    let mut ctx = tera::Context::new();
    ctx.insert("customers", &customer_data);
    ctx.insert("upcoming_appointments", &upcoming);
    ctx.insert("total_customers", &total_customers);
    ctx.insert("total_upcoming", &total_upcoming);
    ctx.insert("total_pending", &total_pending);
    ctx.insert("total_paid", &total_paid);
    ctx.insert("settings", &settings);
    ctx.insert("is_admin", &true);

    match tmpl.render("admin/dashboard.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn customers_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customers = Customer::find_all_non_admin(&conn).unwrap_or_default();

    // Get pending amounts for each customer
    let customer_data: Vec<serde_json::Value> = customers.iter().map(|c| {
        let pending = Payment::pending_total_by_customer(&conn, c.id).unwrap_or(0.0);
        let paid = Payment::paid_total_by_customer(&conn, c.id).unwrap_or(0.0);
        let appt_count = Appointment::count_by_customer(&conn, c.id).unwrap_or(0);
        serde_json::json!({
            "id": c.id,
            "email": c.email,
            "first_name": c.first_name,
            "last_name": c.last_name,
            "phone": c.phone,
            "street": c.street,
            "zip_code": c.zip_code,
            "city": c.city,
            "pending_amount": pending,
            "paid_amount": paid,
            "appointment_count": appt_count,
            "created_at": c.created_at,
        })
    }).collect();

    let mut ctx = tera::Context::new();
    ctx.insert("customers", &customer_data);
    ctx.insert("is_admin", &true);

    match tmpl.render("admin/customers.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn customer_detail(
    req: HttpRequest,
    path: web::Path<i64>,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let customer_id = path.into_inner();
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    let customer = match Customer::find_by_id(&conn, customer_id) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().body("Kunde nicht gefunden"),
    };

    let appointments = Appointment::find_by_customer(&conn, customer_id).unwrap_or_default();
    let payments = Payment::find_by_customer(&conn, customer_id).unwrap_or_default();
    let credit_packages = CreditPackage::find_all_by_customer(&conn, customer_id).unwrap_or_default();
    let pending = Payment::pending_total_by_customer(&conn, customer_id).unwrap_or(0.0);
    let paid = Payment::paid_total_by_customer(&conn, customer_id).unwrap_or(0.0);
    let remaining_credits = CreditPackage::remaining_sessions(&conn, customer_id).unwrap_or(0);

    let mut ctx = tera::Context::new();
    ctx.insert("customer", &customer);
    ctx.insert("appointments", &appointments);
    ctx.insert("payments", &payments);
    ctx.insert("credit_packages", &credit_packages);
    ctx.insert("pending_amount", &pending);
    ctx.insert("paid_amount", &paid);
    ctx.insert("remaining_credits", &remaining_credits);
    ctx.insert("is_admin", &true);

    match tmpl.render("admin/customer_detail.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn appointments_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let appointments = Appointment::find_all_with_customer(&conn).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("appointments", &appointments);
    ctx.insert("is_admin", &true);

    match tmpl.render("admin/appointments.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn payments_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let payments = Payment::find_all(&conn).unwrap_or_default();
    let customers = Customer::find_all_non_admin(&conn).unwrap_or_default();

    // Find any admin accounts too for looking up customer names
    let all_customers_map: std::collections::HashMap<i64, String> = {
        let mut map = std::collections::HashMap::new();
        for c in &customers {
            map.insert(c.id, format!("{} {}", c.first_name, c.last_name));
        }
        // Also check admin
        if let Ok(Some(admin)) = Customer::find_by_id(&conn, 1) {
            map.insert(admin.id, format!("{} {}", admin.first_name, admin.last_name));
        }
        map
    };

    // Enrich payments with customer names
    let payment_data: Vec<serde_json::Value> = payments.iter().map(|p| {
        let customer_name = all_customers_map.get(&p.customer_id)
            .cloned()
            .unwrap_or_else(|| format!("Kunde #{}", p.customer_id));
        serde_json::json!({
            "id": p.id,
            "customer_id": p.customer_id,
            "customer_name": customer_name,
            "amount": p.amount,
            "payment_type": p.payment_type,
            "status": p.status,
            "created_at": p.created_at,
        })
    }).collect();

    let mut ctx = tera::Context::new();
    ctx.insert("payments", &payment_data);
    ctx.insert("customers", &customers);
    ctx.insert("is_admin", &true);

    match tmpl.render("admin/payments.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn faq_editor(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let faqs = Faq::find_all(&conn).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("faqs", &faqs);
    ctx.insert("is_admin", &true);

    match tmpl.render("admin/faq_editor.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn review_editor(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let reviews = Review::find_all(&conn).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("reviews", &reviews);
    ctx.insert("is_admin", &true);

    match tmpl.render("admin/review_editor.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

pub async fn settings_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("settings", &settings);
    ctx.insert("is_admin", &true);

    match tmpl.render("admin/settings.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

// API endpoints

pub async fn api_mark_paid(
    req: HttpRequest,
    path: web::Path<i64>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let payment_id = path.into_inner();
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    match Payment::mark_paid(&conn, payment_id) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
    }
}

pub async fn api_save_faq(
    req: HttpRequest,
    form: web::Json<FaqForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let sort_order = form.sort_order.unwrap_or(0);
    let is_active = form.is_active.unwrap_or(true);

    if let Some(id) = form.id {
        match Faq::update(&conn, id, &form.question, &form.answer_html, sort_order, is_active) {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true, "id": id})),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
        }
    } else {
        match Faq::create(&conn, &form.question, &form.answer_html, sort_order) {
            Ok(id) => HttpResponse::Ok().json(serde_json::json!({"success": true, "id": id})),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
        }
    }
}

pub async fn api_delete_faq(
    req: HttpRequest,
    path: web::Path<i64>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    match Faq::delete(&conn, path.into_inner()) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
    }
}

pub async fn api_save_review(
    req: HttpRequest,
    form: web::Json<ReviewForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let stars = form.stars.unwrap_or(5);
    let sort_order = form.sort_order.unwrap_or(0);
    let is_active = form.is_active.unwrap_or(true);

    if let Some(id) = form.id {
        match Review::update(&conn, id, &form.author_name, &form.author_location, &form.content, stars, sort_order, is_active) {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true, "id": id})),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
        }
    } else {
        match Review::create(&conn, &form.author_name, &form.author_location, &form.content, stars, sort_order) {
            Ok(id) => HttpResponse::Ok().json(serde_json::json!({"success": true, "id": id})),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
        }
    }
}

pub async fn api_delete_review(
    req: HttpRequest,
    path: web::Path<i64>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    match Review::delete(&conn, path.into_inner()) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
    }
}

pub async fn api_save_settings(
    req: HttpRequest,
    form: web::Json<std::collections::HashMap<String, String>>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    for (key, value) in form.iter() {
        if let Err(e) = SiteSetting::set(&conn, key, value) {
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)}));
        }
    }
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

#[derive(Deserialize)]
pub struct SuggestForm {
    pub customer_id: i64,
    pub slots: Vec<String>,  // list of datetime strings
    #[allow(dead_code)]
    pub message: Option<String>,
}

pub async fn api_suggest_appointment(
    req: HttpRequest,
    form: web::Json<SuggestForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    email_service: web::Data<EmailService>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = match Customer::find_by_id(&conn, form.customer_id) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Kunde nicht gefunden"})),
    };

    let slots_html = form.slots.iter()
        .map(|s| format!("<p style='margin: 8px 0; padding: 8px 12px; background: #dff0f7; border-radius: 8px;'>📅 {}</p>", s))
        .collect::<Vec<_>>()
        .join("");

    let es = email_service.get_ref().clone();
    let email = customer.email.clone();
    let name = customer.full_name();
    tokio::spawn(async move {
        es.send_appointment_suggestion(&email, &name, &slots_html);
    });

    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

#[derive(Deserialize)]
pub struct CancelSuggestForm {
    pub appointment_id: i64,
    pub slots: Vec<String>,
}

pub async fn api_cancel_with_suggestions(
    req: HttpRequest,
    form: web::Json<CancelSuggestForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    email_service: web::Data<EmailService>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    let appointment = match Appointment::find_by_id(&conn, form.appointment_id) {
        Ok(Some(a)) => a,
        _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Termin nicht gefunden"})),
    };

    if appointment.status != "confirmed" {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Termin kann nicht storniert werden"}));
    }

    let customer = match Customer::find_by_id(&conn, appointment.customer_id) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Kunde nicht gefunden"})),
    };

    // Cancel the appointment in database
    let _ = Appointment::cancel(&conn, form.appointment_id);

    // Reset pending payment for this appointment
    let _ = conn.execute(
        "UPDATE payments SET status = 'cancelled' WHERE appointment_id = ?1 AND status = 'pending'",
        rusqlite::params![form.appointment_id],
    );

    // If it was a pack appointment, return the credit
    if appointment.appointment_type == "pack" {
        let active_credits = CreditPackage::find_active_by_customer(&conn, appointment.customer_id).unwrap_or_default();
        if let Some(credit) = active_credits.first() {
            let _ = conn.execute(
                "UPDATE credit_packages SET used_sessions = MAX(used_sessions - 1, 0) WHERE id = ?1",
                rusqlite::params![credit.id],
            );
        }
    }

    let (del_auth, del_url) = {
        let access_token = SiteSetting::get_or_default(&conn, "nextcloud_access_token", "");
        let primary_url  = SiteSetting::get_or_default(&conn, "nextcloud_primary_calendar_url", "");
        if !access_token.is_empty() && !primary_url.is_empty() {
            (crate::caldav::CalDavAuth::Bearer(access_token), primary_url)
        } else {
            let url  = SiteSetting::get_or_default(&conn, "nextcloud_caldav_url", "");
            let user = SiteSetting::get_or_default(&conn, "nextcloud_caldav_username", "");
            let pass = SiteSetting::get_or_default(&conn, "nextcloud_caldav_password", "");
            (crate::caldav::CalDavAuth::Basic { user, pass }, url)
        }
    };
    let es = email_service.get_ref().clone();
    let email = customer.email.clone();
    let name = customer.full_name();
    let start_time = appointment.start_time.clone();
    let cancelled_id = form.appointment_id;
    drop(conn);

    let slots_html = form.slots.iter()
        .map(|s| format!("<p style='margin: 8px 0; padding: 8px 12px; background: #dff0f7; border-radius: 8px;'>📅 {}</p>", s))
        .collect::<Vec<_>>()
        .join("");

    tokio::spawn(async move {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%d %H:%M:%S") {
            let date_str = dt.format("%d.%m.%Y").to_string();
            let time_str = dt.format("%H:%M").to_string();

            if slots_html.is_empty() {
                es.send_appointment_cancellation(&email, &name, &date_str, &time_str);
            } else {
                es.send_admin_cancellation_with_suggestions(&email, &name, &date_str, &time_str, &slots_html);
            }
        }
        if !del_url.is_empty() {
            match &del_auth {
                crate::caldav::CalDavAuth::Bearer(tok) =>
                    crate::caldav::delete_event_bearer(&del_url, tok, cancelled_id).await,
                crate::caldav::CalDavAuth::Basic { user, pass } =>
                    crate::caldav::delete_event(&del_url, user, pass, cancelled_id).await,
            }
        }
    });

    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}


/// GET /api/admin/test-caldav — test Nextcloud CalDAV connection with current settings
pub async fn api_test_caldav(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let (url, user, pass) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        (
            SiteSetting::get_or_default(&conn, "nextcloud_caldav_url", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_caldav_username", ""),
            SiteSetting::get_or_default(&conn, "nextcloud_caldav_password", ""),
        )
    };

    if url.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "Keine CalDAV-URL konfiguriert"
        }));
    }

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, "error": format!("HTTP-Client-Fehler: {}", e)
        })),
    };

    let full_url = format!("{}/", url.trim_end_matches('/'));

    let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<D:propfind xmlns:D="DAV:">
  <D:prop><D:resourcetype/><D:displayname/></D:prop>
</D:propfind>"#;

    let resp = client
        .request(
            reqwest::Method::from_bytes(b"PROPFIND").expect("PROPFIND is valid"),
            &full_url,
        )
        .basic_auth(&user, Some(&pass))
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("Depth", "0")
        .body(body)
        .send()
        .await;

    match resp {
        Ok(r) => {
            let status = r.status().as_u16();
            match status {
                207 | 200 => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "Verbindung erfolgreich \u{2013} Kalender erreichbar"
                })),
                401 => HttpResponse::Ok().json(serde_json::json!({
                    "success": false,
                    "error": "Authentifizierung fehlgeschlagen \u{2013} Benutzername oder App-Passwort falsch"
                })),
                403 => HttpResponse::Ok().json(serde_json::json!({
                    "success": false,
                    "error": "Zugriff verweigert \u{2013} App-Passwort hat keine Kalender-Berechtigung"
                })),
                404 => HttpResponse::Ok().json(serde_json::json!({
                    "success": false,
                    "error": "Kalender nicht gefunden \u{2013} URL pr\u{fc}fen"
                })),
                _ => HttpResponse::Ok().json(serde_json::json!({
                    "success": false,
                    "error": format!("Unerwarteter HTTP-Status: {}", status)
                })),
            }
        }
        Err(e) if e.is_timeout() => HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "Zeit\u{fc}berschreitung \u{2013} Server nicht erreichbar"
        })),
        Err(e) if e.is_connect() => HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": format!("Verbindung fehlgeschlagen \u{2013} {}", e)
        })),
        Err(e) => HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": format!("Fehler: {}", e)
        })),
    }
}
