use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;
use chrono::{Utc, Duration};
use uuid::Uuid;

use crate::auth;
use crate::config::Config;
use crate::email::EmailService;
use crate::models::customer::{Customer, LoginForm, RegisterForm, PasswordResetRequest, PasswordReset};

pub async fn login_page(tmpl: web::Data<Tera>, req: HttpRequest, jwt_secret: web::Data<String>) -> HttpResponse {
    if auth::get_claims(&req, &jwt_secret).is_some() {
        return HttpResponse::SeeOther().insert_header(("Location", "/portal")).finish();
    }
    let ctx = tera::Context::new();
    match tmpl.render("auth/login.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => HttpResponse::InternalServerError().body("Die Seite konnte nicht geladen werden."),
    }
}

pub async fn register_page(tmpl: web::Data<Tera>, req: HttpRequest, jwt_secret: web::Data<String>) -> HttpResponse {
    if auth::get_claims(&req, &jwt_secret).is_some() {
        return HttpResponse::SeeOther().insert_header(("Location", "/portal")).finish();
    }
    let ctx = tera::Context::new();
    match tmpl.render("auth/register.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => HttpResponse::InternalServerError().body("Die Seite konnte nicht geladen werden."),
    }
}

pub async fn reset_password_page(tmpl: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    match tmpl.render("auth/reset_password.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => HttpResponse::InternalServerError().body("Die Seite konnte nicht geladen werden."),
    }
}

pub async fn reset_password_token_page(tmpl: web::Data<Tera>, path: web::Path<String>) -> HttpResponse {
    let token = path.into_inner();
    let mut ctx = tera::Context::new();
    ctx.insert("token", &token);
    match tmpl.render("auth/reset_password_confirm.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => HttpResponse::InternalServerError().body("Die Seite konnte nicht geladen werden."),
    }
}

/// GET /verify-email/{token}
/// Called when the customer clicks the link in the verification email.
pub async fn verify_email_page(
    tmpl: web::Data<Tera>,
    path: web::Path<String>,
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let token = path.into_inner();
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());

    let mut ctx = tera::Context::new();
    match Customer::find_by_verification_token(&conn, &token) {
        Ok(Some(customer)) => {
            if Customer::mark_email_verified(&conn, customer.id).is_err() {
                return HttpResponse::InternalServerError().body("Bestätigung fehlgeschlagen. Bitte erneut versuchen.");
            }
            ctx.insert("success", &true);
            ctx.insert("message", "Deine E-Mail-Adresse wurde erfolgreich bestätigt. Du kannst jetzt alle Funktionen nutzen.");
        }
        _ => {
            ctx.insert("success", &false);
            ctx.insert("message", "Der Bestätigungslink ist ungültig oder abgelaufen. Bitte fordere einen neuen Link in deinem Kundenportal an.");
        }
    }

    match tmpl.render("auth/verify_email_result.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => HttpResponse::InternalServerError().body("Die Seite konnte nicht geladen werden."),
    }
}

/// POST /api/customer/resend-verification
/// Resends the e-mail verification link (5-minute cooldown).
pub async fn api_resend_verification(
    req: HttpRequest,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
    email_service: web::Data<EmailService>,
    config: web::Data<Config>,
) -> HttpResponse {
    let claims = match auth::get_claims(&req, &jwt_secret) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Nicht angemeldet"})),
    };

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let customer = match Customer::find_by_id(&conn, claims.sub) {
        Ok(Some(c)) => c,
        _ => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Konto nicht gefunden"})),
    };

    if customer.email_verified {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "E-Mail bereits bestätigt"}));
    }

    let token = Uuid::new_v4().to_string();
    let expires = (Utc::now() + Duration::hours(24)).format("%Y-%m-%d %H:%M:%S").to_string();
    // Expiry is always issued 24 hours ahead, so this also enforces the
    // documented five-minute cooldown without an additional timestamp column.
    match conn.execute(
        "UPDATE customers SET verification_token = ?1, verification_token_expires = ?2
         WHERE id = ?3 AND (verification_token_expires IS NULL
             OR verification_token_expires <= datetime('now', '+23 hours', '+55 minutes'))",
        rusqlite::params![token, expires, customer.id],
    ) {
        Ok(1) => {},
        Ok(_) => return HttpResponse::TooManyRequests().insert_header(("Retry-After", "300"))
            .json(serde_json::json!({"error": "Bitte warte fünf Minuten, bevor du erneut eine Bestätigung anforderst."})),
        Err(_) => return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Bestätigungslink konnte nicht erstellt werden."})),
    }

    let base_url = config.base_url.clone();
    let verify_url = format!("{}/verify-email/{}", base_url, token);
    let es = email_service.get_ref().clone();
    let email = customer.email.clone();
    let name = customer.first_name.clone();
    tokio::spawn(async move {
        es.send_email_verification(&email, &name, &verify_url);
    });

    HttpResponse::Ok().json(serde_json::json!({"success": true, "message": "Bestätigungs-E-Mail wurde erneut gesendet."}))
}

pub async fn api_login(
    form: web::Json<LoginForm>,
    db: web::Data<Mutex<Connection>>,
    config: web::Data<Config>,
) -> HttpResponse {
    let customer = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        match Customer::find_by_email(&conn, &form.email) {
            Ok(Some(c)) => c,
            _ => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Ungültige Anmeldedaten"})),
        }
    };

    if !auth::verify_password(&form.password, &customer.password_hash) {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Ungültige Anmeldedaten"}));
    }

    let token = match auth::create_token(customer.id, &customer.email, customer.is_admin, &customer.password_hash, &config.jwt_secret, config.jwt_expiry_hours) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Token-Erstellung fehlgeschlagen"})),
    };

    // Admin and customers both land on /portal (admin nav link visible there)
    let cookie = auth::auth_cookie(&token, &config);

    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({"success": true, "redirect": "/portal", "is_admin": customer.is_admin}))
}

pub async fn api_register(
    form: web::Json<RegisterForm>,
    db: web::Data<Mutex<Connection>>,
    config: web::Data<Config>,
    email_service: web::Data<EmailService>,
) -> HttpResponse {
    if form.email.is_empty() || form.password.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Bitte E-Mail-Adresse und Passwort angeben"}));
    }
    if let Some(error) = auth::password_validation_error(&form.password) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": error}));
    }
    if form.email.parse::<lettre::Address>().is_err() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Bitte eine gültige E-Mail-Adresse angeben."}));
    }

    let password_hash = auth::hash_password(&form.password);
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    if let Ok(Some(_)) = Customer::find_by_email(&conn, &form.email) {
        return HttpResponse::Conflict().json(serde_json::json!({"error": "E-Mail-Adresse ist bereits registriert"}));
    }

    // Only email + password are collected at signup; name, phone and address are
    // filled in later via the profile-completion gate (see customer_handler's
    // profile_incomplete_response), which every portal page redirects into until
    // first_name is no longer empty.
    let first_name = form.first_name.as_deref().unwrap_or("");
    let last_name = form.last_name.as_deref().unwrap_or("");
    let phone = form.phone.as_deref().unwrap_or("");
    let street = form.street.as_deref().unwrap_or("");
    let postal_code = form.postal_code.as_deref().unwrap_or("");
    let city = form.city.as_deref().unwrap_or("");
    let calendar_token = Uuid::new_v4().to_string();
    let tx = match conn.unchecked_transaction() {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Registrierung fehlgeschlagen."})),
    };

    match Customer::create(
        &tx, &form.email, &password_hash,
        first_name, last_name,
        phone, street, postal_code, city, &calendar_token,
    ) {
        Ok(id) => {
            // Generate verification token (24h expiry)
            let v_token = Uuid::new_v4().to_string();
            let v_expires = (Utc::now() + Duration::hours(24)).format("%Y-%m-%d %H:%M:%S").to_string();
            if Customer::set_verification_token(&tx, id, &v_token, &v_expires).is_err() {
                return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Bestätigungslink konnte nicht erstellt werden."}));
            }

            // Issue JWT so the user is logged in immediately (portal is gated on
            // email_verified, then on profile completeness)
            let jwt = match auth::create_token(id, &form.email, false, &password_hash, &config.jwt_secret, config.jwt_expiry_hours) {
                Ok(jwt) => jwt,
                Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Anmeldung fehlgeschlagen."})),
            };
            let cookie = auth::auth_cookie(&jwt, &config);
            if tx.commit().is_err() {
                return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Registrierung fehlgeschlagen."}));
            }

            let base_url = config.base_url.clone();
            let verify_url = format!("{}/verify-email/{}", base_url, v_token);
            let es = email_service.get_ref().clone();
            let email = form.email.clone();
            let name = first_name.to_string();
            tokio::spawn(async move {
                es.send_email_verification(&email, &name, &verify_url);
            });

            HttpResponse::Ok()
                .cookie(cookie)
                .json(serde_json::json!({"success": true, "redirect": "/portal"}))
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "Registrierung fehlgeschlagen."})),
    }
}

pub async fn logout_page(req: HttpRequest, tmpl: web::Data<Tera>, config: web::Data<Config>, db: web::Data<Mutex<Connection>>) -> HttpResponse {
    let claims = auth::get_claims(&req,&config.jwt_secret);
    let mut ctx = tera::Context::new();
    if req.method() == actix_web::http::Method::GET && claims.is_some() {
        ctx.insert("logout_confirmation", &true);
        return match tmpl.render("auth/logout.html",&ctx) {
            Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
            Err(_) => super::api::internal_error("render logout confirmation"),
        };
    }
    if let Some(claims) = claims {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        if conn.execute("INSERT OR IGNORE INTO revoked_sessions(jti,expires_at) VALUES(?1,?2)",rusqlite::params![claims.jti,claims.exp as i64]).is_err() {
            return super::api::internal_error("revoke logout session");
        }
    }
    ctx.insert("logout_confirmation", &false);
    let mut cookie = auth::auth_cookie("", &config);
    cookie.make_removal();

    match tmpl.render("auth/logout.html", &ctx) {
        Ok(body) => HttpResponse::Ok()
            .cookie(cookie)
            .content_type("text/html; charset=utf-8")
            .body(body),
        Err(_) => HttpResponse::InternalServerError().body("Die Seite konnte nicht geladen werden."),
    }
}

pub async fn api_reset_password_request(
    form: web::Json<PasswordResetRequest>,
    db: web::Data<Mutex<Connection>>,
    config: web::Data<Config>,
    email_service: web::Data<EmailService>,
) -> HttpResponse {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    // Always return success to prevent email enumeration
    if let Ok(Some(customer)) = Customer::find_by_email(&conn, &form.email) {
        let token = Uuid::new_v4().to_string();
        let expires = (Utc::now() + Duration::hours(1)).format("%Y-%m-%d %H:%M:%S").to_string();
        if Customer::set_reset_token(&conn, customer.id, &token, &expires).is_ok() {
            let reset_url = format!("{}/reset-password/{}", config.base_url, token);
            let es = email_service.get_ref().clone();
            let name = customer.full_name();
            let email = customer.email.clone();
            tokio::spawn(async move {
                es.send_password_reset(&email, &name, &reset_url);
            });
        } else {
            // A failed write must neither send an unusable link nor reveal
            // whether the requested account exists.
            log::error!("Could not create password reset token");
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"success": true, "message": "Falls ein Konto mit dieser E-Mail existiert, wurde ein Link zum Zurücksetzen gesendet."}))
}

pub async fn api_reset_password_confirm(
    path: web::Path<String>,
    form: web::Json<PasswordReset>,
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let token = path.into_inner();
    if let Some(error) = auth::password_validation_error(&form.new_password) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": error}));
    }

    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    match Customer::find_by_reset_token(&conn, &token) {
        Ok(Some(customer)) => {
            let password_hash = auth::hash_password(&form.new_password);
            // Updating the password also consumes the reset token in the same
            // SQL statement. Never report success after a failed database write.
            if Customer::update_password(&conn, customer.id, &password_hash).is_err() {
                return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Passwort konnte nicht geändert werden."}));
            }
            HttpResponse::Ok().json(serde_json::json!({"success": true, "message": "Passwort erfolgreich geändert. Du kannst dich jetzt anmelden."}))
        }
        _ => HttpResponse::BadRequest().json(serde_json::json!({"error": "Link ungültig oder abgelaufen"})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::test_support::*;
    use actix_web::http::StatusCode;

    #[actix_web::test]
    async fn reset_link_is_single_use_and_revokes_existing_sessions() {
        let db = database();
        let old_request = request(&db, &token("old-hash", false));
        assert!(auth::get_claims(&old_request, SECRET).is_some());
        let response = api_reset_password_confirm(web::Path::from("reset-link".to_owned()),
            web::Json(PasswordReset { new_password: "new-password".into() }), db.clone()).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(auth::get_claims(&old_request, SECRET).is_none());
        let response = api_reset_password_confirm(web::Path::from("reset-link".to_owned()),
            web::Json(PasswordReset { new_password: "another-password".into() }), db.clone()).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let customer = Customer::find_by_id(&db.lock().unwrap(), 1).unwrap().unwrap();
        assert!(auth::verify_password("new-password", &customer.password_hash));
    }

    #[actix_web::test]
    async fn failed_reset_does_not_claim_success_or_consume_link() {
        let db = database();
        db.lock().unwrap().execute_batch("CREATE TRIGGER reject_password_update BEFORE UPDATE OF password_hash ON customers
            BEGIN SELECT RAISE(ABORT, 'test write failure'); END;").unwrap();
        let response = api_reset_password_confirm(web::Path::from("reset-link".to_owned()),
            web::Json(PasswordReset { new_password: "new-password".into() }), db.clone()).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let customer = Customer::find_by_reset_token(&db.lock().unwrap(), "reset-link").unwrap().unwrap();
        assert_eq!(customer.password_hash, "old-hash");
    }

    #[actix_web::test]
    async fn expired_reset_link_and_oversized_password_are_rejected() {
        let db = database();
        let response = api_reset_password_confirm(web::Path::from("reset-link".to_owned()),
            web::Json(PasswordReset { new_password: "a".repeat(73) }), db.clone()).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        db.lock().unwrap().execute("UPDATE customers SET reset_token_expires = datetime('now', '-1 minute')", []).unwrap();
        let response = api_reset_password_confirm(web::Path::from("reset-link".to_owned()),
            web::Json(PasswordReset { new_password: "new-password".into() }), db.clone()).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(Customer::find_by_id(&db.lock().unwrap(), 1).unwrap().unwrap().password_hash, "old-hash");
    }

    #[actix_web::test]
    async fn verification_resend_enforces_cooldown_without_rotating_link() {
        let db = database();
        db.lock().unwrap().execute("UPDATE customers SET email_verified = 0", []).unwrap();
        let config = config();
        let response = api_resend_verification(request(&db, &token("old-hash", false)), db.clone(),
            web::Data::new(SECRET.to_owned()), web::Data::new(EmailService::new(&config)), web::Data::new(config)).await;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(Customer::find_by_verification_token(&db.lock().unwrap(), "verify-link").unwrap().is_some());
    }

    #[actix_web::test]
    async fn reset_request_database_failures_do_not_enumerate_accounts() {
        let db = database();
        db.lock().unwrap().execute_batch("CREATE TRIGGER reject_reset BEFORE UPDATE OF reset_token ON customers
            BEGIN SELECT RAISE(ABORT, 'test write failure'); END;").unwrap();
        let config = config();
        let service = web::Data::new(EmailService::new(&config));
        let config = web::Data::new(config);
        let existing = api_reset_password_request(web::Json(PasswordResetRequest { email: "sample@example.test".into() }),
            db.clone(), config.clone(), service.clone()).await;
        let missing = api_reset_password_request(web::Json(PasswordResetRequest { email: "missing@example.test".into() }),
            db.clone(), config, service).await;
        assert_eq!(existing.status(), StatusCode::OK);
        assert_eq!(existing.status(), missing.status());
        assert_eq!(actix_web::body::to_bytes(existing.into_body()).await.unwrap(),
            actix_web::body::to_bytes(missing.into_body()).await.unwrap());
    }
}
