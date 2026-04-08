use actix_web::{web, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use chrono::NaiveDateTime;

use crate::models::customer::Customer;
use crate::models::appointment::Appointment;
use crate::models::settings::SiteSetting;

fn format_ical_dt(dt: &NaiveDateTime) -> String {
    dt.format("%Y%m%dT%H%M%S").to_string()
}

fn escape_ical(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace(';', "\\;")
     .replace(',', "\\,")
     .replace('\n', "\\n")
}

fn build_vcalendar(events: &[String]) -> String {
    let mut cal = String::new();
    cal.push_str("BEGIN:VCALENDAR\r\n");
    cal.push_str("VERSION:2.0\r\n");
    cal.push_str("PRODID:-//Faszienbehandlung Thilo Seifried//Terminkalender//DE\r\n");
    cal.push_str("CALSCALE:GREGORIAN\r\n");
    cal.push_str("METHOD:PUBLISH\r\n");
    cal.push_str("X-WR-CALNAME:Faszienbehandlung Thilo Seifried\r\n");
    cal.push_str("X-WR-TIMEZONE:Europe/Berlin\r\n");
    for event in events {
        cal.push_str(event);
    }
    cal.push_str("END:VCALENDAR\r\n");
    cal
}

fn build_vevent(
    uid: &str,
    start: &NaiveDateTime,
    end: &NaiveDateTime,
    summary: &str,
    description: &str,
    location: &str,
) -> String {
    let mut ev = String::new();
    ev.push_str("BEGIN:VEVENT\r\n");
    ev.push_str(&format!("UID:{}\r\n", uid));
    ev.push_str(&format!("DTSTART:{}\r\n", format_ical_dt(start)));
    ev.push_str(&format!("DTEND:{}\r\n", format_ical_dt(end)));
    ev.push_str(&format!("SUMMARY:{}\r\n", escape_ical(summary)));
    if !description.is_empty() {
        ev.push_str(&format!("DESCRIPTION:{}\r\n", escape_ical(description)));
    }
    if !location.is_empty() {
        ev.push_str(&format!("LOCATION:{}\r\n", escape_ical(location)));
    }
    ev.push_str("END:VEVENT\r\n");
    ev
}

/// GET /api/calendar/{calendar_token}/termine.ics
/// Returns the confirmed appointments of the customer identified by the calendar_token.
pub async fn customer_calendar_ics(
    path: web::Path<String>,
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let token = path.into_inner();
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    let customer = match Customer::find_by_calendar_token(&conn, &token) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().body("Kalender nicht gefunden"),
    };

    let settings = SiteSetting::get_all(&conn).unwrap_or_default();
    let practice_address = format!(
        "{}, {} {}",
        settings.get("address_street").map(|s| s.as_str()).unwrap_or(""),
        settings.get("address_zip").map(|s| s.as_str()).unwrap_or(""),
        settings.get("address_city").map(|s| s.as_str()).unwrap_or(""),
    );

    let appointments = Appointment::find_by_customer(&conn, customer.id).unwrap_or_default();

    let events: Vec<String> = appointments
        .iter()
        .filter(|a| a.status == "confirmed")
        .filter_map(|a| {
            let start = NaiveDateTime::parse_from_str(&a.start_time, "%Y-%m-%d %H:%M:%S").ok()?;
            let end = NaiveDateTime::parse_from_str(&a.end_time, "%Y-%m-%d %H:%M:%S").ok()?;
            let uid = format!("apt-{}@faszienbehandlung.jetzt", a.id);
            let summary = format!("Faszienbehandlung – {}", customer.full_name());
            let mut desc_parts = Vec::new();
            if !a.notes.is_empty() { desc_parts.push(format!("Notiz: {}", a.notes)); }
            if !customer.phone.is_empty() { desc_parts.push(format!("Telefon: {}", customer.phone)); }
            let description = desc_parts.join("\\n");
            let location = if a.is_home_visit {
                let addr = customer.full_address();
                format!("Hausbesuch: {}", if addr.is_empty() { "Adresse nicht hinterlegt".to_string() } else { addr })
            } else {
                practice_address.clone()
            };
            Some(build_vevent(&uid, &start, &end, &summary, &description, &location))
        })
        .collect();

    let cal = build_vcalendar(&events);

    HttpResponse::Ok()
        .content_type("text/calendar; charset=utf-8")
        .insert_header(("Content-Disposition", "inline; filename=\"termine.ics\""))
        .body(cal)
}

/// GET /api/admin/calendar/{calendar_token}/alle-termine.ics
/// Returns all confirmed appointments. Only accessible with the admin account's calendar_token.
pub async fn admin_calendar_ics(
    path: web::Path<String>,
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let token = path.into_inner();
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    // Validate the token belongs to an admin
    let admin = match Customer::find_by_calendar_token(&conn, &token) {
        Ok(Some(c)) if c.is_admin => c,
        _ => return HttpResponse::Forbidden().body("Zugriff verweigert"),
    };

    let _ = admin; // admin validated

    let settings = SiteSetting::get_all(&conn).unwrap_or_default();
    let practice_address = format!(
        "{}, {} {}",
        settings.get("address_street").map(|s| s.as_str()).unwrap_or(""),
        settings.get("address_zip").map(|s| s.as_str()).unwrap_or(""),
        settings.get("address_city").map(|s| s.as_str()).unwrap_or(""),
    );

    let appointments = Appointment::find_all_with_customer(&conn).unwrap_or_default();

    let events: Vec<String> = appointments
        .iter()
        .filter(|a| a.status == "confirmed")
        .filter_map(|a| {
            let start = NaiveDateTime::parse_from_str(&a.start_time, "%Y-%m-%d %H:%M:%S").ok()?;
            let end = NaiveDateTime::parse_from_str(&a.end_time, "%Y-%m-%d %H:%M:%S").ok()?;
            let uid = format!("apt-{}@faszienbehandlung.jetzt", a.id);
            let cust_name = a.customer_name.as_deref().unwrap_or("Unbekannt");
            let summary = format!("Faszienbehandlung – {}", cust_name);
            let mut desc_parts = Vec::new();
            if !a.notes.is_empty() { desc_parts.push(format!("Notiz: {}", a.notes)); }
            if let Some(email) = &a.customer_email { desc_parts.push(format!("E-Mail: {}", email)); }
            let description = desc_parts.join("\\n");
            let location = if a.is_home_visit {
                // We need customer address – load on demand from customer_id
                format!("Hausbesuch (Kunde-ID: {})", a.customer_id)
            } else {
                practice_address.clone()
            };
            Some(build_vevent(&uid, &start, &end, &summary, &description, &location))
        })
        .collect();

    let cal = build_vcalendar(&events);

    HttpResponse::Ok()
        .content_type("text/calendar; charset=utf-8")
        .insert_header(("Content-Disposition", "inline; filename=\"alle-termine.ics\""))
        .body(cal)
}
