use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, Duration, Datelike, Weekday};

use crate::auth;
use crate::config::Config;
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
    config: web::Data<Config>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    // Parse date and time first (no lock needed)
    let date = match NaiveDate::parse_from_str(&form.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültiges Datum"})),
    };
    let time = match NaiveTime::parse_from_str(&form.time, "%H:%M") {
        Ok(t) => t,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültige Uhrzeit"})),
    };

    let now = chrono::Utc::now().naive_utc();

    // --- Phase 1: validate and read CalDAV settings under the lock ---
    let (caldav_auth, calendar_urls, primary_url, duration_min) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let duration_min: i64 = SiteSetting::get_or_default(&conn, "appointment_duration_min", "90").parse().unwrap_or(90);

        let access_token = SiteSetting::get_or_default(&conn, "nextcloud_access_token", "");
        let all_urls_str = SiteSetting::get_or_default(&conn, "nextcloud_all_calendar_urls", "");
        let primary_url = SiteSetting::get_or_default(&conn, "nextcloud_primary_calendar_url", "");

        if !access_token.is_empty() && !all_urls_str.is_empty() {
            let urls: Vec<String> = all_urls_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            (crate::caldav::CalDavAuth::Bearer(access_token), urls, primary_url, duration_min)
        } else {
            let caldav_url = SiteSetting::get_or_default(&conn, "nextcloud_caldav_url", "");
            let caldav_user = SiteSetting::get_or_default(&conn, "nextcloud_caldav_username", "");
            let caldav_pass = SiteSetting::get_or_default(&conn, "nextcloud_caldav_password", "");
            let primary = caldav_url.clone();
            let urls = if caldav_url.is_empty() { vec![] } else { vec![caldav_url] };
            (crate::caldav::CalDavAuth::Basic { user: caldav_user, pass: caldav_pass }, urls, primary, duration_min)
        }
    }; // lock released

    let start = NaiveDateTime::new(date, time);
    let end = start + Duration::minutes(duration_min);
    let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_str = end.format("%Y-%m-%d %H:%M:%S").to_string();

    if start <= now {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Termin muss in der Zukunft liegen"}));
    }
    if start - now < Duration::hours(24) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Termine können frühestens 24 Stunden im Voraus gebucht werden"
        }));
    }

    // --- Phase 2: check Nextcloud CalDAV (async, outside the lock) ---
    if !calendar_urls.is_empty() {
        // Refresh OAuth token if needed
        let auth = match &caldav_auth {
            crate::caldav::CalDavAuth::Bearer(_) => {
                if let Some(tok) = crate::handlers::oauth_handler::ensure_valid_token(&db).await {
                    crate::caldav::CalDavAuth::Bearer(tok)
                } else {
                    caldav_auth.clone()
                }
            }
            other => other.clone(),
        };
        let busy = crate::caldav::fetch_busy_periods_multi(&calendar_urls, &auth, &date).await;
        if crate::caldav::has_conflict(&busy, &start, &end) {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "Dieser Zeitraum ist durch einen anderen Termin belegt"
            }));
        }
    }

    // --- Phase 3: create booking under the lock ---
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    // Re-check DB availability (race condition guard)
    match Appointment::is_slot_available(&conn, &start_str, &end_str) {
        Ok(false) => return HttpResponse::Conflict().json(serde_json::json!({"error": "Dieser Zeitraum ist leider bereits belegt"})),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
        _ => {}
    }

    let is_home_visit = form.is_home_visit.unwrap_or(false);
    let notes = form.notes.as_deref().unwrap_or("");

    let active_credits = CreditPackage::find_active_by_customer(&conn, claims.sub).unwrap_or_default();
    let appointment_type = if !active_credits.is_empty() { "pack" } else { "single" };

    let appointment_id = match Appointment::create(&conn, claims.sub, &start_str, &end_str, appointment_type, is_home_visit, notes) {
        Ok(id) => id,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)})),
    };

    let price_single: f64 = SiteSetting::get_or_default(&conn, "price_single", "195").replace(',', ".").parse().unwrap_or(195.0);
    let home_surcharge: f64 = SiteSetting::get_or_default(&conn, "home_visit_surcharge", "15").replace(',', ".").parse().unwrap_or(15.0);

    if !active_credits.is_empty() {
        let credit = &active_credits[0];
        let _ = CreditPackage::use_session(&conn, credit.id);
        let amount = credit.price_per_session + if is_home_visit { home_surcharge } else { 0.0 };
        let _ = Payment::create(&conn, claims.sub, Some(appointment_id), amount, "pack");
    } else {
        let amount = price_single + if is_home_visit { home_surcharge } else { 0.0 };
        let _ = Payment::create(&conn, claims.sub, Some(appointment_id), amount, "single");
    }

    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();
    let date_str = date.format("%d.%m.%Y").to_string();
    let time_str = time.format("%H:%M").to_string();
    let customer_address = customer.full_address();
    let admin_email = config.admin_email.clone();
    let es = email_service.get_ref().clone();
    let cust_email = customer.email.clone();
    let cust_name = customer.full_name();
    let cust_phone = customer.phone.clone();
    let notes_owned = notes.to_string();
    let date_str2 = date_str.clone();
    let time_str2 = time_str.clone();
    drop(conn); // release lock before spawning

    let push_url    = primary_url.clone();
    let push_auth   = caldav_auth.clone();
    let cust_name2  = cust_name.clone();
    let cust_phone2 = cust_phone.clone();
    let cust_addr2  = customer_address.clone();
    let notes2      = notes_owned.clone();

    tokio::spawn(async move {
        es.send_appointment_confirmation(&cust_email, &cust_name, &date_str, &time_str, is_home_visit);
        es.send_admin_booking_notification(
            &admin_email, &cust_name, &cust_phone, &customer_address,
            &date_str2, &time_str2, &notes_owned, is_home_visit,
        );
        if !push_url.is_empty() {
            let (url, user, pass, token) = match &push_auth {
                crate::caldav::CalDavAuth::Basic { user, pass } => (push_url.clone(), user.clone(), pass.clone(), String::new()),
                crate::caldav::CalDavAuth::Bearer(tok) => (push_url.clone(), String::new(), String::new(), tok.clone()),
            };
            if !token.is_empty() {
                crate::caldav::push_event_bearer(
                    &url, &token,
                    appointment_id, &start, &end,
                    &cust_name2, &cust_phone2, &cust_addr2, &notes2, is_home_visit,
                ).await;
            } else {
                crate::caldav::push_event(
                    &url, &user, &pass,
                    appointment_id, &start, &end,
                    &cust_name2, &cust_phone2, &cust_addr2, &notes2, is_home_visit,
                ).await;
            }
        }
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
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    let appointment = match Appointment::find_by_id(&conn, appointment_id) {
        Ok(Some(a)) => a,
        _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Termin nicht gefunden"})),
    };

    if appointment.customer_id != claims.sub && !claims.is_admin {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Zugriff verweigert"}));
    }

    if appointment.status != "confirmed" {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Termin kann nicht storniert werden"}));
    }

    // 24h cancellation policy (admin bypasses)
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

    // Reset pending payment
    let _ = conn.execute(
        "UPDATE payments SET status = 'cancelled' WHERE appointment_id = ?1 AND status = 'pending'",
        rusqlite::params![appointment_id],
    );

    // Return credit if pack appointment
    if appointment.appointment_type == "pack" {
        let active_credits = CreditPackage::find_active_by_customer(&conn, appointment.customer_id).unwrap_or_default();
        if let Some(credit) = active_credits.first() {
            let _ = conn.execute(
                "UPDATE credit_packages SET used_sessions = MAX(used_sessions - 1, 0) WHERE id = ?1",
                rusqlite::params![credit.id],
            );
        }
    }

    let customer = Customer::find_by_id(&conn, appointment.customer_id).unwrap().unwrap();
    let (del_auth, del_url) = {
        let access_token = SiteSetting::get_or_default(&conn, "nextcloud_access_token", "");
        let primary_url = SiteSetting::get_or_default(&conn, "nextcloud_primary_calendar_url", "");
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
    drop(conn);

    tokio::spawn(async move {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%d %H:%M:%S") {
            es.send_appointment_cancellation(
                &email, &name,
                &dt.format("%d.%m.%Y").to_string(),
                &dt.format("%H:%M").to_string(),
            );
        }
        if !del_url.is_empty() {
            match &del_auth {
                crate::caldav::CalDavAuth::Bearer(tok) =>
                    crate::caldav::delete_event_bearer(&del_url, tok, appointment_id).await,
                crate::caldav::CalDavAuth::Basic { user, pass } =>
                    crate::caldav::delete_event(&del_url, user, pass, appointment_id).await,
            }
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

    // --- Phase 1: compute DB-available candidates under the lock ---
    struct Candidate {
        start: NaiveDateTime,
        end: NaiveDateTime,
        time_str: String,
        display: String,
    }

    let (candidates, slot_auth, slot_urls) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let duration_min: i64 = SiteSetting::get_or_default(&conn, "appointment_duration_min", "90").parse().unwrap_or(90);
        let break_min: i64 = SiteSetting::get_or_default(&conn, "appointment_break_min", "0").parse().unwrap_or(0);
        let slot_step = duration_min + break_min;

        // OAuth2 takes priority over legacy Basic Auth
        let access_token = SiteSetting::get_or_default(&conn, "nextcloud_access_token", "");
        let all_urls_str = SiteSetting::get_or_default(&conn, "nextcloud_all_calendar_urls", "");
        let (slot_auth, slot_urls): (crate::caldav::CalDavAuth, Vec<String>) =
            if !access_token.is_empty() && !all_urls_str.is_empty() {
                let urls = all_urls_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                (crate::caldav::CalDavAuth::Bearer(access_token), urls)
            } else {
                let caldav_url  = SiteSetting::get_or_default(&conn, "nextcloud_caldav_url", "");
                let caldav_user = SiteSetting::get_or_default(&conn, "nextcloud_caldav_username", "");
                let caldav_pass = SiteSetting::get_or_default(&conn, "nextcloud_caldav_password", "");
                let urls = if caldav_url.is_empty() { vec![] } else { vec![caldav_url] };
                (crate::caldav::CalDavAuth::Basic { user: caldav_user, pass: caldav_pass }, urls)
            };

        let weekday = date.weekday();
        let (start_hour, end_hour): (i64, i64) = match weekday {
            Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri => (16, 22),
            Weekday::Sat => (9, 19),
            Weekday::Sun => return HttpResponse::Ok().json(serde_json::json!({"slots": []})),
        };

        let now = chrono::Utc::now().naive_utc();
        let mut candidates = Vec::new();
        let mut current_minutes: i64 = start_hour * 60;

        loop {
            let h = (current_minutes / 60) as u32;
            let m = (current_minutes % 60) as u32;
            if h as i64 >= end_hour { break; }

            let start_time = NaiveDateTime::new(date, NaiveTime::from_hms_opt(h, m, 0).unwrap());
            let end_time = start_time + Duration::minutes(duration_min);

            let closing = NaiveDateTime::new(date, NaiveTime::from_hms_opt(end_hour as u32, 0, 0).unwrap());
            if end_time > closing { break; }

            if start_time > now + Duration::hours(24) {
                let start_str = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
                let end_str = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

                if Appointment::is_slot_available(&conn, &start_str, &end_str).unwrap_or(false) {
                    candidates.push(Candidate {
                        start: start_time,
                        end: end_time,
                        time_str: start_time.format("%H:%M").to_string(),
                        display: format!("{} – {}", start_time.format("%H:%M"), end_time.format("%H:%M")),
                    });
                }
            }

            current_minutes += slot_step;
        }

        (candidates, slot_auth, slot_urls)
    }; // DB lock released here

    // --- Phase 2: fetch Nextcloud busy periods (async, outside the lock) ---
    let busy = if !slot_urls.is_empty() {
        // Refresh token if needed (OAuth mode)
        let auth = match &slot_auth {
            crate::caldav::CalDavAuth::Bearer(_) => {
                if let Some(tok) = crate::handlers::oauth_handler::ensure_valid_token(&db).await {
                    crate::caldav::CalDavAuth::Bearer(tok)
                } else {
                    slot_auth.clone()
                }
            }
            other => other.clone(),
        };
        crate::caldav::fetch_busy_periods_multi(&slot_urls, &auth, &date).await
    } else {
        vec![]
    };

    // --- Phase 3: filter out Nextcloud conflicts ---
    let slots: Vec<_> = candidates
        .into_iter()
        .filter(|c| !crate::caldav::has_conflict(&busy, &c.start, &c.end))
        .map(|c| serde_json::json!({"time": c.time_str, "display": c.display}))
        .collect();

    HttpResponse::Ok()
        .append_header(("Cache-Control", "no-store, no-cache, must-revalidate"))
        .append_header(("Pragma", "no-cache"))
        .json(serde_json::json!({"slots": slots}))
}
