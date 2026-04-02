use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;
use serde::Deserialize;

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

    let conn = db.lock().unwrap();
    let customers = Customer::find_all_non_admin(&conn).unwrap_or_default();
    let upcoming = Appointment::find_upcoming_all(&conn).unwrap_or_default();
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();

    // Calculate totals
    let total_customers = customers.len();
    let total_upcoming = upcoming.len();

    let mut total_pending: f64 = 0.0;
    let mut total_paid: f64 = 0.0;
    for c in &customers {
        total_pending += Payment::pending_total_by_customer(&conn, c.id).unwrap_or(0.0);
        total_paid += Payment::paid_total_by_customer(&conn, c.id).unwrap_or(0.0);
    }

    let mut ctx = tera::Context::new();
    ctx.insert("customers", &customers);
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

    let conn = db.lock().unwrap();
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
    let conn = db.lock().unwrap();

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

    let conn = db.lock().unwrap();
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

    let conn = db.lock().unwrap();
    let payments = Payment::find_all(&conn).unwrap_or_default();
    let customers = Customer::find_all_non_admin(&conn).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("payments", &payments);
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

    let conn = db.lock().unwrap();
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

    let conn = db.lock().unwrap();
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

    let conn = db.lock().unwrap();
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
    let conn = db.lock().unwrap();
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

    let conn = db.lock().unwrap();
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

    let conn = db.lock().unwrap();
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

    let conn = db.lock().unwrap();
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

    let conn = db.lock().unwrap();
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

    let conn = db.lock().unwrap();
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

    let conn = db.lock().unwrap();
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
