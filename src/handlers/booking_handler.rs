use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, Duration, Datelike, Weekday};

use crate::auth;
use crate::email::EmailService;
use crate::models::appointment::{Appointment, BookingForm};
use crate::models::payment::{Payment, CreditPackage};
use crate::models::customer::Customer;
use crate::models::settings::SiteSetting;

pub async fn api_book(
    req: HttpRequest,
    form: web::Json<BookingForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    email_service: web::Data<EmailService>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    let conn = db.lock().unwrap();

    // Parse date and time
    let date = match NaiveDate::parse_from_str(&form.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültiges Datum"})),
    };
    let time = match NaiveTime::parse_from_str(&form.time, "%H:%M") {
        Ok(t) => t,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültige Uhrzeit"})),
    };

    let duration_min: i64 = SiteSetting::get_or_default(&conn, "appointment_duration_min", "90").parse().unwrap_or(90);
    let start = NaiveDateTime::new(date, time);
    let end = start + Duration::minutes(duration_min);

    let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_str = end.format("%Y-%m-%d %H:%M:%S").to_string();

    // Check if in the future
    let now = chrono::Utc::now().naive_utc();
    if start <= now {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Termin muss in der Zukunft liegen"}));
    }

    // Check slot availability
    match Appointment::is_slot_available(&conn, &start_str, &end_str) {
        Ok(false) => return HttpResponse::Conflict().json(serde_json::json!({"error": "Dieser Zeitraum ist leider bereits belegt"})),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
        _ => {}
    }

    let is_home_visit = form.is_home_visit.unwrap_or(false);
    let notes = form.notes.as_deref().unwrap_or("");

    // Check for credit package
    let active_credits = CreditPackage::find_active_by_customer(&conn, claims.sub).unwrap_or_default();
    let appointment_type = if !active_credits.is_empty() { "pack" } else { "single" };

    // Create appointment
    let appointment_id = match Appointment::create(&conn, claims.sub, &start_str, &end_str, appointment_type, is_home_visit, notes) {
        Ok(id) => id,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
    };

    // Handle payment/credits
    let price_single: f64 = SiteSetting::get_or_default(&conn, "price_single", "195").replace(',', ".").parse().unwrap_or(195.0);
    let home_surcharge: f64 = SiteSetting::get_or_default(&conn, "home_visit_surcharge", "15").replace(',', ".").parse().unwrap_or(15.0);

    if !active_credits.is_empty() {
        // Use credit from pack
        let credit = &active_credits[0];
        let _ = CreditPackage::use_session(&conn, credit.id);
        let amount = credit.price_per_session + if is_home_visit { home_surcharge } else { 0.0 };
        let _ = Payment::create(&conn, claims.sub, Some(appointment_id), amount, "pack");
    } else {
        let amount = price_single + if is_home_visit { home_surcharge } else { 0.0 };
        let _ = Payment::create(&conn, claims.sub, Some(appointment_id), amount, "single");
    }

    // Send confirmation email
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();
    let es = email_service.get_ref().clone();
    let email = customer.email.clone();
    let name = customer.full_name();
    let date_str = date.format("%d.%m.%Y").to_string();
    let time_str = time.format("%H:%M").to_string();
    tokio::spawn(async move {
        es.send_appointment_confirmation(&email, &name, &date_str, &time_str, is_home_visit);
    });

    HttpResponse::Ok().json(serde_json::json!({"success": true, "appointment_id": appointment_id}))
}

pub async fn api_cancel(
    req: HttpRequest,
    path: web::Path<i64>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    email_service: web::Data<EmailService>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    let appointment_id = path.into_inner();
    let conn = db.lock().unwrap();

    let appointment = match Appointment::find_by_id(&conn, appointment_id) {
        Ok(Some(a)) => a,
        _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Termin nicht gefunden"})),
    };

    // Check ownership (unless admin)
    if appointment.customer_id != claims.sub && !claims.is_admin {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Zugriff verweigert"}));
    }

    if appointment.status != "confirmed" {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Termin kann nicht storniert werden"}));
    }

    // Check 24h cancellation policy
    let cancellation_hours: i64 = SiteSetting::get_or_default(&conn, "cancellation_hours", "24").parse().unwrap_or(24);
    if let Ok(start) = NaiveDateTime::parse_from_str(&appointment.start_time, "%Y-%m-%d %H:%M:%S") {
        let now = chrono::Utc::now().naive_utc();
        let min_cancel_time = start - Duration::hours(cancellation_hours);
        if now > min_cancel_time && !claims.is_admin {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Stornierung nur bis {} Stunden vor dem Termin möglich", cancellation_hours)
            }));
        }
    }

    let _ = Appointment::cancel(&conn, appointment_id);

    // If it was a pack appointment, return the credit
    if appointment.appointment_type == "pack" {
        let active_credits = CreditPackage::find_active_by_customer(&conn, appointment.customer_id).unwrap_or_default();
        if let Some(credit) = active_credits.first() {
            // Decrement used_sessions
            let _ = conn.execute(
                "UPDATE credit_packages SET used_sessions = MAX(used_sessions - 1, 0) WHERE id = ?1",
                rusqlite::params![credit.id],
            );
        }
    }

    // Send cancellation email
    let customer = Customer::find_by_id(&conn, appointment.customer_id).unwrap().unwrap();
    let es = email_service.get_ref().clone();
    let email = customer.email.clone();
    let name = customer.full_name();
    let start_time = appointment.start_time.clone();
    tokio::spawn(async move {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%d %H:%M:%S") {
            es.send_appointment_cancellation(
                &email, &name,
                &dt.format("%d.%m.%Y").to_string(),
                &dt.format("%H:%M").to_string(),
            );
        }
    });

    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

pub async fn api_available_slots(
    db: web::Data<Mutex<Connection>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let date_str = match query.get("date") {
        Some(d) => d,
        None => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Datum erforderlich"})),
    };

    let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültiges Datum"})),
    };

    let conn = db.lock().unwrap();
    let duration_min: i64 = SiteSetting::get_or_default(&conn, "appointment_duration_min", "90").parse().unwrap_or(90);

    // Determine opening hours based on day of week
    let weekday = date.weekday();
    let (start_hour, end_hour) = match weekday {
        Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri => (16, 22),
        Weekday::Sat => (9, 19),
        Weekday::Sun => return HttpResponse::Ok().json(serde_json::json!({"slots": []})),
    };

    let mut slots = Vec::new();
    let mut current_hour = start_hour;
    let mut current_min = 0;

    while current_hour < end_hour {
        let start_time = NaiveDateTime::new(date, NaiveTime::from_hms_opt(current_hour, current_min, 0).unwrap());
        let end_time = start_time + Duration::minutes(duration_min);

        // Don't go past closing time
        let closing = NaiveDateTime::new(date, NaiveTime::from_hms_opt(end_hour, 0, 0).unwrap());
        if end_time > closing { break; }

        // Don't show past times
        let now = chrono::Utc::now().naive_utc();
        if start_time > now {
            let start_str = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
            let end_str = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

            if Appointment::is_slot_available(&conn, &start_str, &end_str).unwrap_or(false) {
                slots.push(serde_json::json!({
                    "time": start_time.format("%H:%M").to_string(),
                    "display": format!("{} – {}", start_time.format("%H:%M"), end_time.format("%H:%M")),
                }));
            }
        }

        // Move to next 90-min slot
        current_min += duration_min as u32;
        while current_min >= 60 {
            current_hour += 1;
            current_min -= 60;
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"slots": slots}))
}
