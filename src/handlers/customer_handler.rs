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

    let settings = SiteSetting::get_all(&conn);
    let settings = settings.unwrap_or_default();
    let address_street = settings.get("address_street").map(|s| s.as_str()).unwrap_or("Sulgauer Straße 24");
    let address_zip = settings.get("address_zip").map(|s| s.as_str()).unwrap_or("78713");
    let address_city = settings.get("address_city").map(|s| s.as_str()).unwrap_or("Sulgen");
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

<p style="font-size:13px;color:#3d5a6b;">* Gemäß § 4 Nr. 14 UStG handelt es sich bei Heilbehandlungen im Bereich der Humanmedizin um steuerbefreite Leistungen. Diese Rechnung enthält keine gesondert ausgewiesene Umsatzsteuer.</p>

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
        address_street, address_zip, address_city, phone,
        payment.id, invoice_date,
        customer.full_name(),
        customer_address,
        customer.email,
        appointment_info,
        payment_type_label,
        payment.amount,
        payment.amount,
        address_street, address_zip, address_city, phone,
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
    let customer = Customer::find_by_id(&conn, claims.sub).unwrap().unwrap();
    if !customer.email_verified {
        return email_not_verified_response(&tmpl, &customer.email, claims.is_admin);
    }

    let waitlist = WaitlistEntry::find_by_customer(&conn, claims.sub).unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("waitlist", &waitlist);
    ctx.insert("is_admin", &claims.is_admin);

    match tmpl.render("customer/waitlist.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}
