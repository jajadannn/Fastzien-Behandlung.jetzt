use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;
use serde::Deserialize;
use reqwest;

use crate::auth;
use crate::email::EmailService;
use crate::models::customer::Customer;
use crate::models::appointment::{Appointment, WaitlistEntry};
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

    let withdrawals = (|| -> rusqlite::Result<Vec<serde_json::Value>> {
        let mut statement = conn.prepare("SELECT id,name,email,contract,created_at FROM withdrawals WHERE processed_at IS NULL ORDER BY created_at DESC")?;
        let rows = statement.query_map([], |r| Ok(serde_json::json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"email":r.get::<_,String>(2)?,"contract":r.get::<_,String>(3)?,"created_at":r.get::<_,String>(4)?})))?;
        rows.collect()
    })();
    let Ok(withdrawals) = withdrawals else { return super::api::internal_error("load withdrawal inbox"); };
    ctx.insert("withdrawals", &withdrawals);
    let outbox_pending: i64 = conn.query_row("SELECT COUNT(*) FROM email_outbox WHERE sent_at IS NULL",[],|r|r.get(0)).unwrap_or(0);
    ctx.insert("outbox_pending", &outbox_pending);

    match tmpl.render("admin/dashboard.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => super::api::internal_error("render page"),
    }
}

pub async fn process_withdrawal(req: HttpRequest, path: web::Path<String>, db: web::Data<Mutex<Connection>>, jwt_secret: web::Data<String>) -> HttpResponse {
    if let Err(response) = require_admin(&req,&jwt_secret) { return response; }
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    match conn.execute("UPDATE withdrawals SET processed_at=datetime('now') WHERE id=?1 AND processed_at IS NULL",[path.into_inner()]) {
        Ok(_) => HttpResponse::SeeOther().insert_header(("Location","/admin")).finish(),
        Err(_) => super::api::internal_error("process withdrawal"),
    }
}

pub async fn retained_invoices(req: HttpRequest, db: web::Data<Mutex<Connection>>, jwt_secret: web::Data<String>) -> HttpResponse {
    if let Err(response) = require_admin(&req,&jwt_secret) { return response; }
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let records = (|| -> rusqlite::Result<Vec<serde_json::Value>> {
        let mut statement = conn.prepare("SELECT record_json,retain_until FROM retained_invoices ORDER BY payment_id")?;
        let rows = statement.query_map([],|r| {
            let raw: String = r.get(0)?;
            Ok(serde_json::json!({"record":serde_json::from_str::<serde_json::Value>(&raw).map_err(|_| rusqlite::Error::InvalidQuery)?,"retain_until":r.get::<_,String>(1)?}))
        })?;
        rows.collect()
    })();
    match records {
        Ok(records) => HttpResponse::Ok().insert_header(("Content-Disposition","attachment; filename=\"buchungsbelege-archiv.json\""))
            .content_type("application/json; charset=utf-8").body(serde_json::to_string_pretty(&records).unwrap_or_default()),
        Err(_) => super::api::internal_error("export retained invoices"),
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
        Err(_) => super::api::internal_error("render page"),
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
        Err(_) => super::api::internal_error("render page"),
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
        Err(_) => super::api::internal_error("render page"),
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
        Err(_) => super::api::internal_error("render page"),
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
        Err(_) => super::api::internal_error("render page"),
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
        Err(_) => super::api::internal_error("render page"),
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
        Err(_) => super::api::internal_error("render page"),
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
        Err(_) => super::api::internal_error("database operation"),
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
            Err(_) => super::api::internal_error("database operation"),
        }
    } else {
        match Faq::create(&conn, &form.question, &form.answer_html, sort_order) {
            Ok(id) => HttpResponse::Ok().json(serde_json::json!({"success": true, "id": id})),
            Err(_) => super::api::internal_error("database operation"),
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
        Err(_) => super::api::internal_error("database operation"),
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
    let is_active = form.is_active.unwrap_or(false);
    let approved = form.publication_approved.unwrap_or(false);
    let text = form.content.to_lowercase();
    if !(1..=5).contains(&stars) || form.author_name.trim().is_empty() || form.content.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"Bitte Name, Originaltext und 1 bis 5 Sterne angeben."}));
    }
    if is_active && (!approved || ["schmerz", "migräne", "geheilt", "arthrose", "burnout"].iter().any(|w| text.contains(w))) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"Veröffentliche nur freigegebene Originalstimmen über Erlebnis und Service mit dokumentierter Einwilligung, ohne Krankheits- oder Heilversprechen."}));
    }
    let tx = match conn.unchecked_transaction() { Ok(tx) => tx, Err(_) => return HttpResponse::InternalServerError().finish() };
    let result = if let Some(id) = form.id {
        Review::update(&tx, id, &form.author_name, &form.author_location, &form.content, stars, sort_order, is_active).map(|_| id)
    } else {
        Review::create(&tx, &form.author_name, &form.author_location, &form.content, stars, sort_order)
    };
    match result {
        Ok(id) => {
            if tx.execute("UPDATE reviews SET publication_approved=?1, is_active=?2 WHERE id=?3", rusqlite::params![approved, is_active, id]).is_err() || tx.commit().is_err() {
                return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Bewertung konnte nicht gespeichert werden."}));
            }
            HttpResponse::Ok().json(serde_json::json!({"success":true,"id":id}))
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error":"Bewertung konnte nicht gespeichert werden."})),
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
        Err(_) => super::api::internal_error("database operation"),
    }
}

pub async fn api_save_settings(
    req: HttpRequest,
    form: web::Json<std::collections::HashMap<String, String>>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    if form.iter().any(|(key, value)| !SiteSetting::valid_value(key, value)) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"Bitte prüfe die Angaben für Preise, Dauer, Pausen und Stornierungsfrist."}));
    }
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let tx = match conn.unchecked_transaction() { Ok(tx) => tx, Err(_) => return HttpResponse::InternalServerError().finish() };
    for (key, value) in form.iter() {
        if SiteSetting::set(&tx, key, value).is_err() {
            return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Einstellungen konnten nicht gespeichert werden."}));
        }
    }
    if tx.commit().is_err() { return HttpResponse::InternalServerError().finish(); }
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
    let slots_html = match crate::email::suggestion_rows(&form.slots) {
        Ok(html) => html,
        Err(error) => return HttpResponse::BadRequest().json(serde_json::json!({"error":error})),
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = match Customer::find_by_id(&conn, form.customer_id) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Kunde nicht gefunden"})),
    };



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
    let slots_html = match crate::email::suggestion_rows(&form.slots) {
        Ok(html) => html,
        Err(error) => return HttpResponse::BadRequest().json(serde_json::json!({"error":error})),
    };

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

    match Appointment::cancel(&conn, form.appointment_id) {
        Ok(true) => {},
        Ok(false) => return HttpResponse::Conflict().json(serde_json::json!({"error":"Der Termin wurde bereits geändert."})),
        Err(rusqlite::Error::InvalidQuery) => return HttpResponse::Conflict().json(serde_json::json!({"error":"Die verwendete Mehrfachkarte ist nicht eindeutig zugeordnet. Bitte prüfe die bestehende Buchung."})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Der Termin konnte nicht storniert werden."})),
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
    // Notify waitlist for the cancelled date
    let cancelled_date = appointment.start_time[..10].to_string();
    let waitlist = WaitlistEntry::find_by_date(&conn, &cancelled_date).unwrap_or_default();

    // Mark waitlist entries as notified before drop
    for entry in &waitlist {
        let _ = WaitlistEntry::mark_notified(&conn, entry.id);
    }

    let es = email_service.get_ref().clone();
    let email = customer.email.clone();
    let name = customer.full_name();
    let start_time = appointment.start_time.clone();
    let cancelled_id = form.appointment_id;
    drop(conn);



    tokio::spawn(async move {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%d %H:%M:%S") {
            let date_str = dt.format("%d.%m.%Y").to_string();
            let time_str = dt.format("%H:%M").to_string();

            if slots_html.is_empty() {
                es.send_appointment_cancellation(&email, &name, &date_str, &time_str);
            } else {
                es.send_admin_cancellation_with_suggestions(&email, &name, &date_str, &time_str, &slots_html);
            }
            // Waitlist notifications
            for entry in &waitlist {
                if let (Some(w_email), Some(w_name)) = (&entry.customer_email, &entry.customer_name) {
                    es.send_waitlist_notification(w_email, w_name, &date_str);
                }
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
        Err(_) => return super::api::internal_error("create calendar client"),
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
            "error": "Verbindung zum Kalender fehlgeschlagen"
        })),
        Err(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "Kalenderanfrage fehlgeschlagen"
        })),
    }
}

// ============ Admin: create appointment manually ============

#[derive(Deserialize)]
pub struct AdminCreateAppointmentForm {
    pub start_time: String,  // "YYYY-MM-DDTHH:MM"
    pub customer_id: Option<i64>,  // None = blocked slot
    pub is_home_visit: Option<bool>,
    pub notes: Option<String>,
    pub appointment_type: Option<String>, // "single", "pack", "blocked"
}

/// Create the appointment and its financial records in one transaction while the
/// caller holds the database lock. None means the requested slot is occupied.
fn record_appointment(
    conn: &Connection, customer_id: i64, start: &str, end: &str,
    blocked: bool, home_visit: bool, notes: &str,
) -> rusqlite::Result<Option<i64>> {
    let tx = rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate)?;
    if !Appointment::is_slot_available(&tx, start, end)? { return Ok(None); }
    let credits = if blocked { Vec::new() } else { CreditPackage::find_active_by_customer(&tx, customer_id)? };
    let appointment_type = if blocked { "blocked" } else if credits.is_empty() { "single" } else { "pack" };
    let id = Appointment::create(&tx, customer_id, start, end, appointment_type, home_visit, notes)?;
    if !blocked {
        let surcharge: f64 = SiteSetting::get_or_default(&tx, "home_visit_surcharge", "15").replace(',', ".").parse().unwrap_or(15.0);
        let price = if let Some(credit) = credits.first() {
            CreditPackage::use_session(&tx, credit.id)?;
            tx.execute("UPDATE appointments SET credit_package_id = ?1 WHERE id = ?2", rusqlite::params![credit.id, id])?;
            credit.price_per_session
        } else {
            SiteSetting::get_or_default(&tx, "price_single", "195").replace(',', ".").parse().unwrap_or(195.0)
        };
        let amount = price + if home_visit { surcharge } else { 0.0 };
        if !amount.is_finite() || amount < 0.0 { return Err(rusqlite::Error::InvalidQuery); }
        Payment::create(&tx, customer_id, Some(id), amount, appointment_type)?;
    }
    tx.commit()?;
    Ok(Some(id))
}

pub async fn api_create_appointment(
    req: HttpRequest,
    form: web::Json<AdminCreateAppointmentForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    email_service: web::Data<EmailService>,
    config: web::Data<crate::config::Config>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let start = match chrono::NaiveDateTime::parse_from_str(&form.start_time, "%Y-%m-%dT%H:%M") {
        Ok(dt) => dt,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültiges Datum/Uhrzeit"})),
    };

    // --- Phase 1: read settings under lock ---
    let (mut caldav_auth, calendar_urls, primary_url, duration_min) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let duration_min: i64 = SiteSetting::get_or_default(&conn, "appointment_duration_min", "90").parse().unwrap_or(90);

        let access_token = SiteSetting::get_or_default(&conn, "nextcloud_access_token", "");
        let all_urls_str = SiteSetting::get_or_default(&conn, "nextcloud_all_calendar_urls", "");
        let primary_url = SiteSetting::get_or_default(&conn, "nextcloud_primary_calendar_url", "");

        if !access_token.is_empty() && !all_urls_str.is_empty() {
            let urls = all_urls_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
            (crate::caldav::CalDavAuth::Bearer(access_token), urls, primary_url, duration_min)
        } else {
            let url  = SiteSetting::get_or_default(&conn, "nextcloud_caldav_url", "");
            let user = SiteSetting::get_or_default(&conn, "nextcloud_caldav_username", "");
            let pass = SiteSetting::get_or_default(&conn, "nextcloud_caldav_password", "");
            let primary = url.clone();
            let urls = if url.is_empty() { vec![] } else { vec![url] };
            (crate::caldav::CalDavAuth::Basic { user, pass }, urls, primary, duration_min)
        }
    }; // lock released

    let end = match chrono::Duration::try_minutes(duration_min).and_then(|duration| start.checked_add_signed(duration)) {
        Some(end) if (15..=360).contains(&duration_min) => end,
        _ => return HttpResponse::ServiceUnavailable().json(serde_json::json!({"error":"Die Termindauer ist nicht gültig konfiguriert."})),
    };
    let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_str   = end.format("%Y-%m-%d %H:%M:%S").to_string();
    let date = start.date();

    let is_blocked = form.appointment_type.as_deref() == Some("blocked") || form.customer_id.is_none();
    if !["single", "pack", "blocked"].contains(&form.appointment_type.as_deref().unwrap_or("single")) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"Ungültige Terminart"}));
    }

    // --- Phase 2: check ALL CalDAV calendars for conflicts (async, outside lock) ---
    if !calendar_urls.is_empty() {
        let auth = match &caldav_auth {
            crate::caldav::CalDavAuth::Bearer(_) => {
                if let Some(tok) = crate::handlers::oauth_handler::ensure_valid_token(&db).await {
                    crate::caldav::CalDavAuth::Bearer(tok)
                } else { return HttpResponse::ServiceUnavailable().json(serde_json::json!({"error":"Die Kalenderverbindung ist abgelaufen oder wurde getrennt. Bitte versuche es später erneut."})); }
            }
            other => other.clone(),
        };
        caldav_auth = auth.clone();
        let busy = match crate::caldav::fetch_busy_periods_multi(&calendar_urls, &auth, &date).await {
            Ok(busy) => busy,
            Err(_) => return HttpResponse::ServiceUnavailable().json(serde_json::json!({"error":"Der Kalender kann gerade nicht geprüft werden. Bitte versuche es später erneut."})),
        };
        if crate::caldav::has_conflict(&busy, &start, &end) {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "Dieser Zeitraum ist in einem deiner Nextcloud-Kalender belegt"
            }));
        }
    }

    // --- Phase 3: create appointment under lock ---
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    let customer_id = if is_blocked {
        conn.query_row("SELECT id FROM customers WHERE is_admin = 1 LIMIT 1", [], |r| r.get(0)).unwrap_or(1)
    } else {
        form.customer_id.unwrap()
    };

    let is_home_visit = form.is_home_visit.unwrap_or(false);
    let notes = form.notes.as_deref().unwrap_or("");

    if !matches!(Customer::find_by_id(&conn, customer_id), Ok(Some(_))) {
        return HttpResponse::NotFound().json(serde_json::json!({"error":"Kunde nicht gefunden"}));
    }
    let appointment_id = match record_appointment(&conn, customer_id, &start_str, &end_str, is_blocked, is_home_visit, notes) {
        Ok(Some(id)) => id,
        Ok(None) => return HttpResponse::Conflict().json(serde_json::json!({"error":"Dieser Zeitraum ist bereits belegt"})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Der Termin konnte nicht gespeichert werden."})),
    };

    // For real appointments (not blocked): payment + push to CalDAV + send email
    if !is_blocked {
        if let Ok(Some(customer)) = Customer::find_by_id(&conn, customer_id) {
            let date_str   = start.format("%d.%m.%Y").to_string();
            let time_str   = start.format("%H:%M").to_string();
            let es         = email_service.get_ref().clone();
            let cust_email = customer.email.clone();
            let cust_name  = customer.full_name();
            let cust_phone = customer.phone.clone();
            let cust_addr  = customer.full_address();
            let admin_email = config.admin_email.clone();
            let notes_owned = notes.to_string();
            let push_url   = primary_url.clone();
            let push_auth  = caldav_auth.clone();
            drop(conn);
            tokio::spawn(async move {
                es.send_appointment_confirmation(&cust_email, &cust_name, &date_str, &time_str, is_home_visit, duration_min);
                es.send_admin_booking_notification(&admin_email, &cust_name, &cust_phone, &cust_addr, &date_str, &time_str, &notes_owned, is_home_visit);
                if !push_url.is_empty() {
                    match &push_auth {
                        crate::caldav::CalDavAuth::Bearer(tok) =>
                            crate::caldav::push_event_bearer(&push_url, tok, appointment_id, &start, &end, &cust_name, &cust_phone, &cust_addr, &notes_owned, is_home_visit).await,
                        crate::caldav::CalDavAuth::Basic { user, pass } =>
                            crate::caldav::push_event(&push_url, user, pass, appointment_id, &start, &end, &cust_name, &cust_phone, &cust_addr, &notes_owned, is_home_visit).await,
                    }
                }
            });
            return HttpResponse::Ok().json(serde_json::json!({"success": true, "appointment_id": appointment_id}));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"success": true, "appointment_id": appointment_id}))
}

// ============ Admin: therapist notes ============

#[derive(Deserialize)]
pub struct TherapistNotesForm {
    pub notes: String,
}

pub async fn api_update_therapist_notes(
    req: HttpRequest,
    path: web::Path<i64>,
    form: web::Json<TherapistNotesForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let appointment_id = path.into_inner();
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    match Appointment::update_therapist_notes(&conn, appointment_id, &form.notes) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(_) => super::api::internal_error("database operation"),
    }
}

// ============ Admin: statistics page ============

pub async fn stats_page(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    if let Err(r) = require_admin(&req, &jwt_secret) { return r; }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    // Monthly revenue + booking count for last 6 months
    let months_data: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare(
            "SELECT strftime('%Y-%m', a.start_time) as month, COUNT(*) as count, COALESCE(SUM(p.amount), 0) as revenue FROM appointments a LEFT JOIN payments p ON p.appointment_id = a.id AND p.status != 'cancelled' WHERE a.status != 'cancelled' AND a.appointment_type != 'blocked' AND a.start_time >= datetime('now', '-6 months') GROUP BY month ORDER BY month ASC"
        ).unwrap();
        stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "month": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?,
                "revenue": row.get::<_, f64>(2)?,
            }))
        }).unwrap().filter_map(|r| r.ok()).collect()
    };

    // Total stats
    let total_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM payments WHERE status = 'paid'", [], |r| r.get(0)
    ).unwrap_or(0.0);

    let total_appointments: i64 = conn.query_row(
        "SELECT COUNT(*) FROM appointments WHERE status != 'cancelled' AND appointment_type != 'blocked'", [], |r| r.get(0)
    ).unwrap_or(0);

    let pending_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM payments WHERE status = 'pending'", [], |r| r.get(0)
    ).unwrap_or(0.0);

    // This month
    let this_month_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(p.amount), 0) FROM payments p JOIN appointments a ON p.appointment_id = a.id WHERE p.status = 'paid' AND strftime('%Y-%m', a.start_time) = strftime('%Y-%m', 'now')", [], |r| r.get(0)
    ).unwrap_or(0.0);

    let this_month_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM appointments WHERE status != 'cancelled' AND appointment_type != 'blocked' AND strftime('%Y-%m', start_time) = strftime('%Y-%m', 'now')", [], |r| r.get(0)
    ).unwrap_or(0);

    // Popular booking hours
    let popular_hours: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare(
            "SELECT strftime('%H', start_time) as hour, COUNT(*) as count FROM appointments WHERE status != 'cancelled' AND appointment_type != 'blocked' GROUP BY hour ORDER BY count DESC LIMIT 5"
        ).unwrap();
        stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "hour": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?,
            }))
        }).unwrap().filter_map(|r| r.ok()).collect()
    };

    // Home visit vs practice
    let home_visit_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM appointments WHERE is_home_visit = 1 AND status != 'cancelled' AND appointment_type != 'blocked'", [], |r| r.get(0)
    ).unwrap_or(0);

    let practice_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM appointments WHERE is_home_visit = 0 AND status != 'cancelled' AND appointment_type != 'blocked'", [], |r| r.get(0)
    ).unwrap_or(0);

    // Cancellation rate
    let cancelled_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM appointments WHERE status = 'cancelled' AND appointment_type != 'blocked'", [], |r| r.get(0)
    ).unwrap_or(0);

    let total_all: i64 = conn.query_row(
        "SELECT COUNT(*) FROM appointments WHERE appointment_type != 'blocked'", [], |r| r.get(0)
    ).unwrap_or(1);

    let cancel_rate = if total_all > 0 { (cancelled_count as f64 / total_all as f64 * 100.0) as i64 } else { 0 };

    let sources: Vec<serde_json::Value> = conn.prepare("SELECT CASE WHEN b.source='' THEN 'Keine Angabe' ELSE b.source END,COUNT(*) FROM booking_sources b JOIN appointments a ON a.id=b.appointment_id WHERE a.status!='cancelled' AND a.created_at>=datetime('now','-90 days') GROUP BY b.source ORDER BY COUNT(*) DESC")
        .and_then(|mut stmt| stmt.query_map([], |r| Ok(serde_json::json!({"channel":r.get::<_,String>(0)?,"count":r.get::<_,i64>(1)?})))?.collect()).unwrap_or_default();
    let new_customers: i64 = conn.query_row("SELECT COUNT(*) FROM (SELECT customer_id,MIN(start_time) AS first_visit FROM appointments WHERE status IN ('confirmed','completed') AND appointment_type!='blocked' GROUP BY customer_id) WHERE substr(first_visit,1,7)=strftime('%Y-%m','now')", [], |r| r.get(0)).unwrap_or(0);
    let mut ctx = tera::Context::new();
    ctx.insert("booking_sources", &sources);
    ctx.insert("new_customers", &new_customers);
    ctx.insert("months_data", &months_data);
    ctx.insert("total_revenue", &total_revenue);
    ctx.insert("total_appointments", &total_appointments);
    ctx.insert("pending_revenue", &pending_revenue);
    ctx.insert("this_month_revenue", &this_month_revenue);
    ctx.insert("this_month_count", &this_month_count);
    ctx.insert("popular_hours", &popular_hours);
    ctx.insert("home_visit_count", &home_visit_count);
    ctx.insert("practice_count", &practice_count);
    ctx.insert("cancel_rate", &cancel_rate);
    ctx.insert("is_admin", &true);

    match tmpl.render("admin/stats.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => super::api::internal_error("render page"),
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[actix_web::test]
    async fn malformed_suggestion_cannot_cancel_the_existing_appointment() {
        let (conn,customer,card) = fixture();
        let id = record_appointment(&conn,customer,"2030-01-07 16:00:00","2030-01-07 17:30:00",false,false,"").unwrap().unwrap();
        conn.execute("UPDATE customers SET is_admin=1 WHERE id=?1",[customer]).unwrap();
        let secret = crate::auth::test_support::SECRET;
        let token = crate::auth::create_token(customer,"admin-booking@example.com",true,"test",secret,24).unwrap();
        let db = web::Data::new(Mutex::new(conn));
        let req = actix_web::test::TestRequest::post().app_data(db.clone()).cookie(actix_web::cookie::Cookie::new("auth_token",token)).to_http_request();
        let response = api_cancel_with_suggestions(req,web::Json(CancelSuggestForm{appointment_id:id,slots:vec!["<img src=x onerror=alert(1)>".into()]}),db.clone(),web::Data::new(secret.to_owned()),web::Data::new(EmailService::new(&crate::auth::test_support::config()))).await;
        assert_eq!(response.status(),actix_web::http::StatusCode::BAD_REQUEST);
        let conn = db.lock().unwrap();
        assert_eq!(Appointment::find_by_id(&conn,id).unwrap().unwrap().status,"confirmed");
        assert_eq!(CreditPackage::find_all_by_customer(&conn,customer).unwrap().iter().find(|c|c.id==card).unwrap().used_sessions,1);
        assert_eq!(Payment::find_by_customer(&conn,customer).unwrap().len(),1);
    }

    fn fixture() -> (Connection, i64, i64) {
        let conn = crate::db::test_connection();
        conn.execute("INSERT INTO customers(email,password_hash) VALUES('admin-booking@example.com','test')", []).unwrap();
        let customer = conn.last_insert_rowid();
        let card = CreditPackage::create(&conn, customer, 1, 123.0, "2099-01-01").unwrap();
        (conn, customer, card)
    }

    #[test]
    fn manual_bookings_reject_overlaps_and_record_the_actual_credit_card() {
        let (conn, customer, card) = fixture();
        let id = record_appointment(&conn, customer, "2030-01-07 16:00:00", "2030-01-07 17:30:00", false, false, "").unwrap().unwrap();
        assert!(record_appointment(&conn, customer, "2030-01-07 16:30:00", "2030-01-07 18:00:00", false, false, "").unwrap().is_none());
        let appointment = Appointment::find_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(appointment.appointment_type, "pack");
        let recorded_card: i64 = conn.query_row("SELECT credit_package_id FROM appointments WHERE id=?1", [id], |row| row.get(0)).unwrap();
        assert_eq!(recorded_card, card);
        assert_eq!(Payment::find_by_customer(&conn, customer).unwrap().len(), 1);
        assert_eq!(CreditPackage::find_all_by_customer(&conn, customer).unwrap()[0].used_sessions, 1);
        assert!(CreditPackage::use_session(&conn, card).is_err());
    }

    #[test]
    fn payment_failure_rolls_back_the_booking_and_credit_debit() {
        let (conn, customer, _) = fixture();
        conn.execute_batch("CREATE TRIGGER reject_payment BEFORE INSERT ON payments BEGIN SELECT RAISE(ABORT, 'test payment failure'); END;").unwrap();
        assert!(record_appointment(&conn, customer, "2030-01-07 16:00:00", "2030-01-07 17:30:00", false, false, "").is_err());
        assert!(Appointment::find_by_customer(&conn, customer).unwrap().is_empty());
        assert_eq!(CreditPackage::find_all_by_customer(&conn, customer).unwrap()[0].used_sessions, 0);
    }

    #[test]
    fn unsafe_numeric_settings_are_rejected_before_saving() {
        for (key, value) in [("appointment_duration_min", "0"), ("appointment_duration_min", "9223372036854775807"), ("appointment_break_min", "-90"), ("price_single", "NaN"), ("price_pack", "inf"), ("cancellation_hours", "-1")] {
            assert!(!SiteSetting::valid_value(key, value), "accepted {key}={value}");
        }
        assert!(SiteSetting::valid_value("price_pack", "169,90"));
        assert!(SiteSetting::valid_value("appointment_duration_min", "90"));
    }
}
