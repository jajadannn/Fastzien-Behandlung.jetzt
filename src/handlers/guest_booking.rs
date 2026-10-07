use actix_web::{web, HttpResponse};
use chrono::{NaiveDate, NaiveTime};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tera::Tera;
use uuid::Uuid;

use crate::{config::Config, email::EmailService};
use crate::models::{appointment::BookingForm, customer::Customer, settings::SiteSetting};

#[derive(Deserialize)]
pub struct GuestForm {
    pub date: String,
    pub time: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    #[serde(default)] pub is_home_visit: bool,
    #[serde(default)] pub street: String,
    #[serde(default)] pub zip_code: String,
    #[serde(default)] pub city: String,
    #[serde(default)] pub source: String,
    #[serde(default)] pub privacy_acknowledged: bool,
    #[serde(default)] pub website: String,
}

#[derive(Clone, Serialize)]
pub struct GuestRequest {
    pub token: String,
    pub date: String,
    pub time: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub is_home_visit: bool,
    pub street: String,
    pub zip_code: String,
    pub city: String,
    pub source: String,
}

#[derive(Deserialize)]
pub struct ConfirmForm { pub token: String }

pub fn load_request(conn: &Connection, token: &str) -> rusqlite::Result<Option<GuestRequest>> {
    conn.query_row("SELECT token, date, time, first_name, last_name, email, phone, is_home_visit, street, zip_code, city, source FROM booking_requests WHERE token=?1 AND expires_at > datetime('now') AND consumed_at IS NULL", [token], |r| Ok(GuestRequest {
        token: r.get(0)?, date: r.get(1)?, time: r.get(2)?, first_name: r.get(3)?, last_name: r.get(4)?, email: r.get(5)?, phone: r.get(6)?, is_home_visit: r.get(7)?, street: r.get(8)?, zip_code: r.get(9)?, city: r.get(10)?, source: r.get(11)?,
    })).optional()
}

pub async fn page(tmpl: web::Data<Tera>, db: web::Data<Mutex<Connection>>) -> HttpResponse {
    render_page(&tmpl, &db, None)
}

pub async fn confirmation_page(tmpl: web::Data<Tera>, db: web::Data<Mutex<Connection>>, token: web::Path<String>) -> HttpResponse {
    render_page(&tmpl, &db, Some(token.into_inner()))
}

fn render_page(tmpl: &Tera, db: &Mutex<Connection>, token: Option<String>) -> HttpResponse {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let mut ctx = tera::Context::new();
    ctx.insert("is_guest", &true);
    ctx.insert("is_admin", &false);
    ctx.insert("remaining_credits", &0);
    ctx.insert("settings", &SiteSetting::get_all(&conn).unwrap_or_default());
    ctx.insert("confirmation_mode", &token.is_some());
    if let Some(token) = token {
        if let Ok(Some(request)) = load_request(&conn, &token) {
            ctx.insert("guest_request", &request);
        }
    }
    match tmpl.render("customer/book_appointment.html", &ctx) {
        Ok(body) => HttpResponse::Ok().insert_header(("Cache-Control", "no-store")).insert_header(("Referrer-Policy", "no-referrer")).content_type("text/html; charset=utf-8").body(body),
        Err(e) => { log::error!("Booking template: {:#}", e); HttpResponse::InternalServerError().body("Die Buchungsseite ist gerade nicht erreichbar. Bitte rufe mich an.") }
    }
}

pub fn validate_form(form: &GuestForm) -> Result<(), &'static str> {
    if !form.website.is_empty() { return Err("Die Anfrage konnte nicht verarbeitet werden."); }
    if !form.privacy_acknowledged { return Err("Bitte bestätige, dass du die Datenschutzhinweise gelesen hast."); }
    for value in [&form.first_name, &form.last_name, &form.phone] {
        if value.trim().is_empty() || value.chars().count() > 100 || value.chars().any(char::is_control) { return Err("Bitte gib deinen Namen und eine gültige Telefonnummer an."); }
    }
    let email = form.email.trim();
    if email.len() > 254 || email.parse::<lettre::Address>().is_err() { return Err("Bitte gib eine gültige E-Mail-Adresse an."); }
    if NaiveDate::parse_from_str(&form.date, "%Y-%m-%d").is_err() || NaiveTime::parse_from_str(&form.time, "%H:%M").is_err() { return Err("Bitte wähle einen gültigen Termin."); }
    if form.is_home_visit && (form.street.trim().is_empty() || form.street.len() > 200 || form.city.trim().is_empty() || form.city.len() > 100 || form.zip_code.len() != 5 || !form.zip_code.bytes().all(|b| b.is_ascii_digit())) { return Err("Für einen Hausbesuch brauche ich Straße, fünfstellige Postleitzahl und Ort."); }
    if !["", "Google", "Empfehlung", "Social Media", "Verein / Netzwerk", "Sonstiges"].contains(&form.source.as_str()) { return Err("Bitte wähle eine der Angaben zur Herkunft aus."); }
    Ok(())
}

pub async fn request(form: web::Json<GuestForm>, db: web::Data<Mutex<Connection>>, es: web::Data<EmailService>, config: web::Data<Config>) -> HttpResponse {
    if let Err(message) = validate_form(&form) { return HttpResponse::BadRequest().json(serde_json::json!({"error":message})); }
    let email = form.email.trim().to_lowercase();
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let duration = SiteSetting::get_or_default(&conn, "appointment_duration_min", "90").parse().unwrap_or(90);
        let pause = SiteSetting::get_or_default(&conn, "appointment_break_min", "0").parse().unwrap_or(0);
        if !super::booking_handler::valid_booking_window(&form.date, &form.time, duration, pause) {
            return HttpResponse::BadRequest().json(serde_json::json!({"error":"Bitte wähle einen freien Termin mindestens 24 Stunden und höchstens 180 Tage im Voraus."}));
        }
        let rate: (i64, i64) = conn.query_row("SELECT COUNT(*), COALESCE(SUM(CASE WHEN email=?1 THEN 1 ELSE 0 END),0) FROM booking_requests WHERE created_at > datetime('now','-1 hour')", [&email], |r| Ok((r.get(0)?,r.get(1)?))).unwrap_or((100, 3));
        if rate.0 >= 100 || rate.1 >= 3 { return HttpResponse::TooManyRequests().json(serde_json::json!({"error":"Es wurden bereits mehrere Bestätigungslinks angefordert. Bitte nutze die letzte E-Mail oder versuche es in einer Stunde erneut."})); }
        if conn.execute("INSERT INTO booking_requests(token,date,time,first_name,last_name,email,phone,is_home_visit,street,zip_code,city,source,expires_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,datetime('now','+30 minutes'))", params![token,form.date,form.time,form.first_name.trim(),form.last_name.trim(),email,form.phone.trim(),form.is_home_visit,form.street.trim(),form.zip_code.trim(),form.city.trim(),form.source]).is_err() { return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Deine Anfrage konnte nicht gespeichert werden. Bitte versuche es erneut."})); }
    }
    let url = format!("{}/buchen/bestaetigen/{}", config.base_url.trim_end_matches('/'), token);
    let service = es.get_ref().clone();
    let name = form.first_name.trim().to_string();
    let date = NaiveDate::parse_from_str(&form.date, "%Y-%m-%d").unwrap().format("%d.%m.%Y").to_string();
    let time = form.time.clone();
    match tokio::task::spawn_blocking(move || service.send_guest_booking_link(&email, &name, &date, &time, &url)).await {
        Ok(Ok(())) => HttpResponse::Ok().json(serde_json::json!({"success":true,"message":"Prüfe dein E-Mail-Postfach und bestätige deine Buchung innerhalb von 30 Minuten. Bis zur Bestätigung ist der Termin noch nicht reserviert."})),
        _ => {
            let conn = db.lock().unwrap_or_else(|e| e.into_inner());
            let _ = conn.execute("UPDATE booking_requests SET expires_at=datetime('now') WHERE token=?1", [&token]);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({"error":"Die Bestätigungs-E-Mail konnte nicht versendet werden. Bitte versuche es später erneut oder rufe mich an."}))
        }
    }
}

pub async fn confirm(form: web::Form<ConfirmForm>, db: web::Data<Mutex<Connection>>, es: web::Data<EmailService>, config: web::Data<Config>) -> HttpResponse {
    let request = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        match load_request(&conn, &form.token) {
            Ok(Some(r)) => r,
            _ => return HttpResponse::BadRequest().json(serde_json::json!({"error":"Dieser Link ist abgelaufen oder wurde bereits verwendet. Bitte wähle einen neuen Termin."})),
        }
    };
    let booking = BookingForm { date: request.date.clone(), time: request.time.clone(), is_home_visit: Some(request.is_home_visit), notes: None };
    super::booking_handler::complete_booking(None, Some(request), booking, db, es, config).await
}

/// Called only after email ownership was proven and inside the booking transaction.
pub fn customer_for_request(conn: &Connection, request: &GuestRequest, password_hash: &str) -> rusqlite::Result<i64> {
    if let Some(c) = Customer::find_by_email(conn, &request.email)? {
        if c.is_admin { return Err(rusqlite::Error::InvalidQuery); }
        return Ok(c.id); // Keep account credentials, credits and profile unchanged.
    }
    let id = Customer::create(conn, &request.email, password_hash, &request.first_name, &request.last_name, &request.phone, &request.street, &request.zip_code, &request.city, &Uuid::new_v4().to_string())?;
    Customer::mark_email_verified(conn, id)?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn form() -> GuestForm { serde_json::from_value(serde_json::json!({"date":"2026-11-02","time":"16:00","first_name":"Test","last_name":"Kunde","email":"test@example.com","phone":"0123456789","privacy_acknowledged":true})).unwrap() }
    #[test] fn invalid_contact_and_missing_house_address_are_rejected() {
        let mut f = form(); assert!(validate_form(&f).is_ok());
        f.email = "invalid".into(); assert!(validate_form(&f).is_err());
        f.email = "test@example.com".into(); f.is_home_visit = true; assert!(validate_form(&f).is_err());
        f.street = "Testweg 1".into(); f.zip_code = "78713".into(); f.city = "Schramberg".into(); assert!(validate_form(&f).is_ok());
        f.privacy_acknowledged = false; assert!(validate_form(&f).is_err());
    }
    #[test] fn confirmation_tokens_are_single_use_and_expire() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE booking_requests(token TEXT, date TEXT, time TEXT, first_name TEXT, last_name TEXT, email TEXT, phone TEXT, is_home_visit INTEGER, street TEXT, zip_code TEXT, city TEXT, source TEXT, expires_at TEXT, consumed_at TEXT); INSERT INTO booking_requests VALUES('token','2026-11-02','16:00','Test','Kunde','test@example.com','0123',0,'','','','',datetime('now','+30 minutes'),NULL);").unwrap();
        assert!(load_request(&conn,"token").unwrap().is_some());
        conn.execute("UPDATE booking_requests SET consumed_at=datetime('now')",[]).unwrap();
        assert!(load_request(&conn,"token").unwrap().is_none());
        conn.execute("UPDATE booking_requests SET consumed_at=NULL, expires_at=datetime('now','-1 minute')",[]).unwrap();
        assert!(load_request(&conn,"token").unwrap().is_none());
    }
}
