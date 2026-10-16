use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, Duration, Datelike, Weekday};

use crate::auth;
use crate::config::Config;
use crate::email::EmailService;
use crate::models::appointment::{Appointment, BookingForm, RescheduleForm, WaitlistEntry};
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

    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        match Customer::find_by_id(&conn, claims.sub) {
            Ok(Some(c)) if c.email_verified && !c.first_name.is_empty() && !c.last_name.is_empty() => {},
            _ => return HttpResponse::Forbidden().json(serde_json::json!({"error":"Bitte bestätige deine E-Mail-Adresse und vervollständige dein Profil."})),
        }
    }
    complete_booking(Some(claims.sub), None, form.into_inner(), db, email_service, config).await
}

pub(crate) async fn complete_booking(
    customer_id: Option<i64>,
    guest: Option<super::guest_booking::GuestRequest>,
    form: BookingForm,
    db: web::Data<Mutex<Connection>>,
    email_service: web::Data<EmailService>,
    config: web::Data<Config>,
) -> HttpResponse {
    // Guest records get an inaccessible random password; portal access uses the existing reset flow.
    let guest_password = guest.as_ref().map(|_| auth::hash_password(&uuid::Uuid::new_v4().to_string()));
    // Parse date and time first (no lock needed)
    let date = match NaiveDate::parse_from_str(&form.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültiges Datum"})),
    };
    let time = match NaiveTime::parse_from_str(&form.time, "%H:%M") {
        Ok(t) => t,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültige Uhrzeit"})),
    };

    let now = chrono::Utc::now().with_timezone(&chrono_tz::Europe::Berlin).naive_local();

    // --- Phase 1: validate and read CalDAV settings under the lock ---
    let (mut caldav_auth, calendar_urls, primary_url, duration_min) = {
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
    let end = match Duration::try_minutes(duration_min).and_then(|duration| start.checked_add_signed(duration)) {
        Some(end) if (15..=360).contains(&duration_min) => end,
        _ => return HttpResponse::ServiceUnavailable().json(serde_json::json!({"error":"Die Termindauer ist nicht gültig konfiguriert."})),
    };
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

    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let pause = SiteSetting::get_or_default(&conn, "appointment_break_min", "0").parse().unwrap_or(0);
        if !valid_booking_window(&form.date, &form.time, duration_min, pause) {
            return HttpResponse::BadRequest().json(serde_json::json!({"error":"Bitte wähle einen verfügbaren Termin aus dem Kalender."}));
        }
    }

    // --- Phase 2: check Nextcloud CalDAV (async, outside the lock) ---
    if !calendar_urls.is_empty() {
        // Refresh OAuth token if needed
        let auth = match &caldav_auth {
            crate::caldav::CalDavAuth::Bearer(_) => {
                if let Some(tok) = crate::handlers::oauth_handler::ensure_valid_token(&db).await {
                    crate::caldav::CalDavAuth::Bearer(tok)
                } else {
                    return HttpResponse::ServiceUnavailable().json(serde_json::json!({"error":"Die Kalenderverbindung ist abgelaufen oder wurde getrennt. Bitte versuche es später erneut."}));
                }
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
                "error": "Dieser Zeitraum ist durch einen anderen Termin belegt"
            }));
        }
    }

    // --- Phase 3: create booking under the lock ---
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    // Acquire SQLite's write reservation before the availability re-check, so
    // another server process cannot insert between this check and the booking.
    let tx = match rusqlite::Transaction::new_unchecked(&conn, rusqlite::TransactionBehavior::Immediate) {
        Ok(tx) => tx,
        Err(_) => return super::api::internal_error("begin booking transaction"),
    };
    match Appointment::is_slot_available(&tx, &start_str, &end_str) {
        Ok(false) => return HttpResponse::Conflict().json(serde_json::json!({"error": "Dieser Zeitraum ist leider bereits belegt"})),
        Err(_) => return super::api::internal_error("database operation"),
        _ => {}
    }

    let customer_id = if let Some(ref g) = guest {
        if !matches!(super::guest_booking::load_request(&tx, &g.token), Ok(Some(_))) {
            return HttpResponse::BadRequest().json(serde_json::json!({"error":"Dieser Link ist abgelaufen oder wurde bereits verwendet."}));
        }
        match super::guest_booking::customer_for_request(&tx, g, guest_password.as_deref().unwrap_or("")) {
            Ok(id) => id,
            Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error":"Die Buchung konnte nicht zugeordnet werden. Falls du dich bereits registriert hast, bestätige zuerst deine E-Mail-Adresse und nutze das Kundenportal. Bei Problemen rufe mich bitte an."})),
        }
    } else if let Some(id) = customer_id { id } else {
        return HttpResponse::BadRequest().finish();
    };

    let is_home_visit = form.is_home_visit.unwrap_or(false);
    let notes = form.notes.as_deref().unwrap_or("");
    if notes.chars().count() > 1000 {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"Bitte kürze die organisatorischen Hinweise auf höchstens 1000 Zeichen."}));
    }

    let active_credits = if guest.is_none() { CreditPackage::find_active_by_customer(&tx, customer_id).unwrap_or_default() } else { vec![] };
    let appointment_type = if !active_credits.is_empty() { "pack" } else { "single" };

    let price_single: f64 = SiteSetting::get_or_default(&tx,"price_single","195").replace(',',".").parse().unwrap_or(195.0);
    let home_surcharge: f64 = SiteSetting::get_or_default(&tx,"home_visit_surcharge","15").replace(',',".").parse().unwrap_or(15.0);
    let total = active_credits.first().map(|c| c.price_per_session).unwrap_or(price_single) + if is_home_visit { home_surcharge } else { 0.0 };
    if form.expected_total.is_none_or(|v| !v.is_finite() || (v-total).abs() > 0.005) || form.expected_duration != Some(duration_min) || form.expected_using_credit != Some(!active_credits.is_empty()) {
        return HttpResponse::Conflict().json(serde_json::json!({"error":"Preis, Guthaben oder Dauer haben sich geändert. Bitte lade die Buchungsseite neu und prüfe die aktualisierte Zusammenfassung."}));
    }
    if start - now < Duration::days(14) && !form.early_start_requested {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"Für einen Termin innerhalb von 14 Tagen ist deine ausdrückliche Zustimmung zum vorzeitigen Leistungsbeginn erforderlich. Alternativ wähle einen späteren Termin."}));
    }
    if is_home_visit && guest.is_none() && Customer::find_by_id(&tx,customer_id).ok().flatten().is_none_or(|c| c.street.trim().is_empty() || c.zip_code.trim().is_empty() || c.city.trim().is_empty()) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"Bitte ergänze für den Hausbesuch Straße, Postleitzahl und Ort im Profil."}));
    }

    let appointment_id = match Appointment::create(&tx, customer_id, &start_str, &end_str, appointment_type, is_home_visit, notes) {
        Ok(id) => id,
        Err(_) => return super::api::internal_error("database operation"),
    };

    let price_single: f64 = SiteSetting::get_or_default(&tx, "price_single", "195").replace(',', ".").parse().unwrap_or(195.0);
    let home_surcharge: f64 = SiteSetting::get_or_default(&tx, "home_visit_surcharge", "15").replace(',', ".").parse().unwrap_or(15.0);

    if !active_credits.is_empty() {
        let credit = &active_credits[0];
        if CreditPackage::use_session(&tx, credit.id).is_err() { return HttpResponse::InternalServerError().finish(); }
        if tx.execute("UPDATE appointments SET credit_package_id = ?1 WHERE id = ?2", rusqlite::params![credit.id, appointment_id]).is_err() { return HttpResponse::InternalServerError().finish(); }
        let amount = credit.price_per_session + if is_home_visit { home_surcharge } else { 0.0 };
        if Payment::create(&tx, customer_id, Some(appointment_id), amount, "pack").is_err() { return HttpResponse::InternalServerError().finish(); }
    } else {
        let amount = price_single + if is_home_visit { home_surcharge } else { 0.0 };
        if Payment::create(&tx, customer_id, Some(appointment_id), amount, "single").is_err() { return HttpResponse::InternalServerError().finish(); }
    }

    let customer = match Customer::find_by_id(&tx, customer_id) { Ok(Some(c)) => c, _ => return HttpResponse::InternalServerError().finish() };
    if let Some(ref g) = guest {
        if tx.execute("UPDATE booking_requests SET consumed_at=datetime('now') WHERE token=?1 AND consumed_at IS NULL", [&g.token]).is_err() || tx.execute("INSERT INTO booking_sources(appointment_id,source,city) VALUES(?1,?2,?3)", rusqlite::params![appointment_id,g.source,g.city]).is_err() {
            return HttpResponse::InternalServerError().finish();
        }
    }
    let public_settings = SiteSetting::get_public(&tx).unwrap_or_default();
    let withdrawal = super::legal::withdrawal_text(&public_settings);
    let name = guest.as_ref().map(|g| format!("{} {}",g.first_name,g.last_name)).unwrap_or_else(|| customer.full_name());
    let practice_address = format!("{}, {} {}",SiteSetting::get_or_default(&tx,"address_street","Sulgauer Straße 24"),SiteSetting::get_or_default(&tx,"address_zip","78713"),SiteSetting::get_or_default(&tx,"address_city","Schramberg-Sulgen"));
    let location = if is_home_visit { guest.as_ref().map(|g| format!("{}, {} {}",g.street,g.zip_code,g.city)).unwrap_or_else(|| customer.full_address()) } else { practice_address.clone() };
    let receipt = format!("Verbindliche Buchungsbestätigung\nBuchungsnummer: {}\nName: {}\nTermin: {} um {} Uhr (Europe/Berlin)\nOrt: {}\nLeistung: Gantke® Faszienbehandlung, Wellness- und Körperarbeit mit Vorgespräch und Übungen\nDauer: ca. {} Minuten\nGesamtpreis: {:.2} EUR\n{}\nZahlung bar oder per Überweisung, keine Vorauszahlung. Keine Erstattung durch Krankenkassen oder Versicherungen.\nReguläre Absage bis {} Stunden vor dem Termin per Portal, Telefon oder E-Mail. Das gesetzliche Widerrufsrecht bleibt unberührt.\nAnbieter: Thilo Seifried, {}\nKontakt: {} / {}\nBuchungsbedingungen: https://faszienbehandlung.jetzt/buchungsbedingungen\nVersion: {}\nZustimmung zum vorzeitigen Leistungsbeginn / Kenntnis des Erlöschens bei vollständiger Leistung: {}\n\n{}",
        appointment_id,name,date.format("%d.%m.%Y"),time.format("%H:%M"),location,duration_min,total,
        if active_credits.is_empty() { "Keine Guthabenverrechnung." } else { "Eine Sitzung deiner vorhandenen Karte wird verwendet. Abrechnung zum hinterlegten Kartentarif; der Gesamtpreis umfasst eine ggf. gewählte Fahrtpauschale." },
        SiteSetting::get_or_default(&tx,"cancellation_hours","24"),practice_address,
        SiteSetting::get_or_default(&tx,"phone","+49 152 34 00 72 25"),SiteSetting::get_or_default(&tx,"email_contact","termin@faszienbehandlung.jetzt"),super::legal::TERMS_VERSION,form.early_start_requested,withdrawal);
    if tx.execute("INSERT INTO booking_terms(appointment_id,version,early_start_requested,agreed_total,duration_min,terms_text) VALUES(?1,?2,?3,?4,?5,?6)", rusqlite::params![appointment_id,super::legal::TERMS_VERSION,form.early_start_requested,total,duration_min,receipt]).is_err()
        || crate::email::enqueue_text(&tx,&customer.email,"Verbindliche Buchungsbestätigung",&receipt).is_err() {
        return super::api::internal_error("persist booking confirmation");
    }
    if tx.commit().is_err() { return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Die Buchung konnte nicht gespeichert werden. Bitte versuche es erneut."})); }
    let date_str = date.format("%d.%m.%Y").to_string();
    let time_str = time.format("%H:%M").to_string();
    let customer_address = guest.as_ref().filter(|g| g.is_home_visit).map(|g| format!("{}, {} {}", g.street, g.zip_code, g.city)).unwrap_or_else(|| customer.full_address());
    let admin_email = config.admin_email.clone();
    let es = email_service.get_ref().clone();
    let outbox_db = db.clone();
    let cust_name = guest.as_ref().map(|g| format!("{} {}", g.first_name, g.last_name)).unwrap_or_else(|| customer.full_name());
    let cust_phone = guest.as_ref().map(|g| g.phone.clone()).unwrap_or_else(|| customer.phone.clone());
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
        crate::email::dispatch_outbox(outbox_db,es.clone()).await;
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
    let claims = match auth::require_verified_claims(&req,&jwt_secret) {
        Ok(claims) => claims,
        Err(response) => return response,
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
    let cancellation_hours: i64 = SiteSetting::get_or_default(&conn, "cancellation_hours", "24").parse::<i64>().ok().filter(|hours| (0..=720).contains(hours)).unwrap_or(24);
    if let Ok(start) = NaiveDateTime::parse_from_str(&appointment.start_time, "%Y-%m-%d %H:%M:%S") {
        let now = chrono::Utc::now().with_timezone(&chrono_tz::Europe::Berlin).naive_local();
        let min_cancel_time = start - Duration::hours(cancellation_hours);
        if now > min_cancel_time && !claims.is_admin {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Stornierung nur bis {} Stunden vor dem Termin möglich", cancellation_hours)
            }));
        }
    }

    match Appointment::cancel(&conn, appointment_id) {
        Ok(true) => {},
        Ok(false) => return HttpResponse::Conflict().json(serde_json::json!({"error":"Der Termin wurde bereits geändert."})),
        Err(rusqlite::Error::InvalidQuery) => return HttpResponse::Conflict().json(serde_json::json!({"error":"Die verwendete Mehrfachkarte muss vor der Stornierung zugeordnet werden. Bitte kontaktiere die Praxis."})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Der Termin konnte nicht storniert werden."})),
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

    // Notify waitlist for the cancelled date
    let cancelled_date = appointment.start_time[..10].to_string();
    let waitlist = WaitlistEntry::find_by_date(&conn, &cancelled_date).unwrap_or_default();
    for entry in &waitlist {
        let _ = WaitlistEntry::mark_notified(&conn, entry.id);
    }

    let es = email_service.get_ref().clone();
    let email = customer.email.clone();
    let name = customer.full_name();
    let start_time = appointment.start_time.clone();
    drop(conn);

    tokio::spawn(async move {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%d %H:%M:%S") {
            let date_str = dt.format("%d.%m.%Y").to_string();
            let time_str = dt.format("%H:%M").to_string();
            es.send_appointment_cancellation(&email, &name, &date_str, &time_str);
            // Notify waitlisted customers
            for entry in &waitlist {
                if let (Some(w_email), Some(w_name)) = (&entry.customer_email, &entry.customer_name) {
                    es.send_waitlist_notification(w_email, w_name, &date_str);
                }
            }
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
        if !(15..=360).contains(&duration_min) || !(0..=120).contains(&break_min) {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({"error":"Die Terminzeiten sind nicht gültig konfiguriert."}));
        }
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

            if valid_booking_window(date_str, &start_time.format("%H:%M").to_string(), duration_min, break_min) {
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
                    return HttpResponse::ServiceUnavailable().json(serde_json::json!({"error":"Die Kalenderverbindung ist abgelaufen oder wurde getrennt. Bitte versuche es später erneut."}));
                }
            }
            other => other.clone(),
        };
        match crate::caldav::fetch_busy_periods_multi(&slot_urls, &auth, &date).await {
            Ok(busy) => busy,
            Err(_) => return HttpResponse::ServiceUnavailable().json(serde_json::json!({"error":"Der Kalender kann gerade nicht geprüft werden. Bitte versuche es später erneut."})),
        }
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

/// POST /api/appointments/{id}/reschedule — move the existing booking atomically.
pub async fn api_reschedule(
    req: HttpRequest,
    path: web::Path<i64>,
    form: web::Json<RescheduleForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    email_service: web::Data<EmailService>,
    config: web::Data<Config>,
) -> HttpResponse {
    let claims = match auth::require_verified_claims(&req,&jwt_secret) {
        Ok(claims) => claims,
        Err(response) => return response,
    };

    let appointment_id = path.into_inner();

    let date = match NaiveDate::parse_from_str(&form.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültiges Datum"})),
    };
    let time = match NaiveTime::parse_from_str(&form.time, "%H:%M") {
        Ok(t) => t,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Ungültige Uhrzeit"})),
    };

    let now = chrono::Utc::now().with_timezone(&chrono_tz::Europe::Berlin).naive_local();

    // Phase 1: Load settings + validate old appointment
    let (old_appointment, mut caldav_auth, calendar_urls, primary_url, duration_min) = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());

        let old = match Appointment::find_by_id(&conn, appointment_id) {
            Ok(Some(a)) => a,
            _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Termin nicht gefunden"})),
        };

        if old.customer_id != claims.sub && !claims.is_admin {
            return HttpResponse::Forbidden().json(serde_json::json!({"error": "Zugriff verweigert"}));
        }
        if old.status != "confirmed" {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": "Nur bestätigte Termine können verschoben werden"}));
        }

        // 24h cancellation policy
        let cancellation_hours: i64 = SiteSetting::get_or_default(&conn, "cancellation_hours", "24").parse::<i64>().ok().filter(|hours| (0..=720).contains(hours)).unwrap_or(24);
        if let Ok(old_start) = NaiveDateTime::parse_from_str(&old.start_time, "%Y-%m-%d %H:%M:%S") {
            if now > old_start - Duration::hours(cancellation_hours) && !claims.is_admin {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Verschieben nur bis {} Stunden vor dem Termin möglich", cancellation_hours)
                }));
            }
        }

        let duration_min: i64 = SiteSetting::get_or_default(&conn, "appointment_duration_min", "90").parse().unwrap_or(90);
        let pause: i64 = SiteSetting::get_or_default(&conn, "appointment_break_min", "0").parse().unwrap_or(0);
        if !valid_booking_window(&form.date, &form.time, duration_min, pause) {
            return HttpResponse::BadRequest().json(serde_json::json!({"error":"Bitte wähle einen verfügbaren Termin aus dem Kalender."}));
        }
        let access_token = SiteSetting::get_or_default(&conn, "nextcloud_access_token", "");
        let all_urls_str = SiteSetting::get_or_default(&conn, "nextcloud_all_calendar_urls", "");
        let mut primary_url = SiteSetting::get_or_default(&conn, "nextcloud_primary_calendar_url", "");

        let (auth, urls) = if !access_token.is_empty() && !all_urls_str.is_empty() {
            let urls = all_urls_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            (crate::caldav::CalDavAuth::Bearer(access_token), urls)
        } else {
            let url = SiteSetting::get_or_default(&conn, "nextcloud_caldav_url", "");
            let user = SiteSetting::get_or_default(&conn, "nextcloud_caldav_username", "");
            let pass = SiteSetting::get_or_default(&conn, "nextcloud_caldav_password", "");
            primary_url = url.clone();
            let urls = if url.is_empty() { vec![] } else { vec![url] };
            (crate::caldav::CalDavAuth::Basic { user, pass }, urls)
        };

        (old, auth, urls, primary_url, duration_min)
    };

    let new_start = NaiveDateTime::new(date, time);
    let new_end = new_start + Duration::minutes(duration_min);
    let new_start_str = new_start.format("%Y-%m-%d %H:%M:%S").to_string();
    let new_end_str = new_end.format("%Y-%m-%d %H:%M:%S").to_string();

    if new_start <= now {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Neuer Termin muss in der Zukunft liegen"}));
    }
    if new_start - now < Duration::hours(24) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Termine können frühestens 24 Stunden im Voraus gebucht werden"}));
    }

    // Phase 2: CalDAV conflict check for new slot
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
        let own_uid = format!("faszien-{}@faszienbehandlung.jetzt", appointment_id);
        let busy: Vec<_> = busy.into_iter().filter(|period| period.uid.as_deref() != Some(own_uid.as_str())).collect();
        if crate::caldav::has_conflict(&busy, &new_start, &new_end) {
            return HttpResponse::Conflict().json(serde_json::json!({"error": "Der neue Zeitraum ist durch einen anderen Termin belegt"}));
        }
    }

    // Phase 3: compare-and-swap prevents a concurrent cancellation or reschedule
    // from being overwritten after the CalDAV request released the database lock.
    let customer = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let customer = match Customer::find_by_id(&conn, old_appointment.customer_id) {
            Ok(Some(customer)) => customer,
            _ => return HttpResponse::NotFound().json(serde_json::json!({"error":"Kunde nicht gefunden"})),
        };
        match Appointment::reschedule_if_unchanged(&conn, &old_appointment, &new_start_str, &new_end_str) {
            Ok(true) => customer,
            Ok(false) => return HttpResponse::Conflict().json(serde_json::json!({"error":"Der Termin wurde inzwischen geändert oder der neue Zeitraum ist bereits belegt. Bitte lade die Termine neu."})),
            Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Der Termin konnte nicht verschoben werden."})),
        }
    };
    let new_id = appointment_id;

    let date_str = date.format("%d.%m.%Y").to_string();
    let time_str = time.format("%H:%M").to_string();
    let admin_email = config.admin_email.clone();
    let es = email_service.get_ref().clone();
    let cust_email = customer.email.clone();
    let cust_name = customer.full_name();
    let cust_phone = customer.phone.clone();
    let cust_addr = customer.full_address();
    let notes_owned = old_appointment.notes.clone();
    let is_home_visit = old_appointment.is_home_visit;
    let push_url = primary_url.clone();
    let push_auth = caldav_auth.clone();

    tokio::spawn(async move {
        es.send_appointment_confirmation(&cust_email, &cust_name, &date_str, &time_str, is_home_visit, duration_min);
        es.send_admin_booking_notification(&admin_email, &cust_name, &cust_phone, &cust_addr, &date_str, &time_str, &notes_owned, is_home_visit);
        // PUT replaces the same event because the booking keeps its identity.
        if !push_url.is_empty() {
            match &push_auth {
                crate::caldav::CalDavAuth::Bearer(tok) =>
                    crate::caldav::push_event_bearer(&push_url, tok, new_id, &new_start, &new_end, &cust_name, &cust_phone, &cust_addr, &notes_owned, is_home_visit).await,
                crate::caldav::CalDavAuth::Basic { user, pass } =>
                    crate::caldav::push_event(&push_url, user, pass, new_id, &new_start, &new_end, &cust_name, &cust_phone, &cust_addr, &notes_owned, is_home_visit).await,
            }
        }
    });

    HttpResponse::Ok().json(serde_json::json!({"success": true, "new_appointment_id": new_id}))
}

#[derive(serde::Deserialize)]
pub struct WaitlistForm {
    pub date: String,
}

/// POST /api/appointments/waitlist/add
pub async fn api_waitlist_add(
    req: HttpRequest,
    form: web::Json<WaitlistForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    match WaitlistEntry::add(&conn, claims.sub, &form.date) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(_) => super::api::internal_error("database operation"),
    }
}

/// POST /api/appointments/waitlist/remove
pub async fn api_waitlist_remove(
    req: HttpRequest,
    form: web::Json<WaitlistForm>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let claims = match auth::require_verified_claims(&req,&jwt_secret) {
        Ok(claims) => claims,
        Err(response) => return response,
    };
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    match WaitlistEntry::remove(&conn, claims.sub, &form.date) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(_) => super::api::internal_error("database operation"),
    }
}

/// Validate the same slot grid offered by the public calendar, in Europe/Berlin.
pub(crate) fn valid_booking_window(date: &str, time: &str, duration: i64, pause: i64) -> bool {
    use chrono::{TimeZone, Timelike};
    let (Ok(date), Ok(time)) = (NaiveDate::parse_from_str(date, "%Y-%m-%d"), NaiveTime::parse_from_str(time, "%H:%M")) else { return false; };
    if !(15..=360).contains(&duration) || !(0..=120).contains(&pause) { return false; }
    let (open, close) = match date.weekday() { Weekday::Sun => return false, Weekday::Sat => (9*60,19*60), _ => (16*60,22*60) };
    let minute = time.hour() as i64 * 60 + time.minute() as i64;
    if minute < open || minute + duration > close || (minute - open) % (duration + pause) != 0 { return false; }
    let Some(start) = chrono_tz::Europe::Berlin.from_local_datetime(&NaiveDateTime::new(date,time)).single() else { return false; };
    let now = chrono::Utc::now();
    start >= now + Duration::hours(24) && start <= now + Duration::days(180)
}

#[cfg(test)]
mod booking_window_tests {
    use super::*;
    #[test]
    fn rejects_off_grid_closed_days_and_invalid_settings() {
        let mut date = chrono::Utc::now().with_timezone(&chrono_tz::Europe::Berlin).date_naive() + Duration::days(7);
        while date.weekday() != Weekday::Mon { date += Duration::days(1); }
        let day = date.format("%Y-%m-%d").to_string();
        assert!(valid_booking_window(&day,"16:00",90,0));
        assert!(!valid_booking_window(&day,"16:15",90,0));
        assert!(!valid_booking_window(&day,"22:00",90,0));
        assert!(!valid_booking_window(&day,"16:00",0,0));
        let sunday = (date + Duration::days(6)).format("%Y-%m-%d").to_string();
        assert!(!valid_booking_window(&sunday,"16:00",90,0));
    }

    #[actix_web::test]
    async fn reschedule_api_rejects_closed_days_and_off_grid_times() {
        use actix_web::{test, App};
        let config = crate::auth::test_support::config();
        let conn = crate::db::test_connection();
        conn.execute("INSERT INTO customers(email,password_hash,email_verified) VALUES('sample@example.test','old-hash',1)", []).unwrap();
        let mut monday = chrono::Utc::now().with_timezone(&chrono_tz::Europe::Berlin).date_naive() + Duration::days(14);
        while monday.weekday() != Weekday::Mon { monday += Duration::days(1); }
        let original_start = format!("{} 16:00:00", monday);
        let original_end = format!("{} 17:30:00", monday);
        let id = Appointment::create(&conn, 1, &original_start, &original_end, "single", false, "").unwrap();
        let db = web::Data::new(Mutex::new(conn));
        let token = crate::auth::test_support::token("old-hash", false);
        let app = test::init_service(App::new()
            .app_data(db.clone())
            .app_data(web::Data::new(config.jwt_secret.clone()))
            .app_data(web::Data::new(EmailService::new(&config)))
            .app_data(web::Data::new(config))
            .route("/api/appointments/{id}/reschedule", web::post().to(api_reschedule))).await;
        for (date, time) in [((monday + Duration::days(6)).to_string(), "16:00"), (monday.to_string(), "03:00"), (monday.to_string(), "16:15"), ((monday + Duration::days(210)).to_string(), "16:00")] {
            let req = test::TestRequest::post().uri(&format!("/api/appointments/{id}/reschedule"))
                .cookie(actix_web::cookie::Cookie::new("auth_token", token.clone()))
                .set_json(serde_json::json!({"date":date, "time":time})).to_request();
            let response = test::call_service(&app, req).await;
            assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
        }
        let conn = db.lock().unwrap();
        assert_eq!(Appointment::find_by_id(&conn, id).unwrap().unwrap().start_time, original_start);
    }

    #[actix_web::test]
    async fn public_slots_fail_safely_for_corrupt_duration_settings() {
        let conn = crate::db::test_connection();
        SiteSetting::set(&conn, "appointment_duration_min", "9223372036854775807").unwrap();
        let db = web::Data::new(Mutex::new(conn));
        let query = web::Query::from_query("date=2030-01-07").unwrap();
        let response = api_available_slots(db, query).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::SERVICE_UNAVAILABLE);
    }
}
