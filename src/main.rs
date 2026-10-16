mod config;
mod db;
mod auth;
mod email;
mod caldav;
mod models;
mod handlers;
mod marketing;
mod security;

use actix_web::{web, App, HttpServer, middleware::from_fn};
use actix_files as fs;
use std::sync::Mutex;
use tera::Tera;

use config::Config;
use email::EmailService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = Config::from_env();
    let bind_addr = format!("{}:{}", config.host, config.port);

    // Initialize database
    let conn = db::initialize(&config.database_url, &config.admin_email, &config.admin_password)
        .expect("Failed to initialize database");
    let db_data = web::Data::new(Mutex::new(conn));

    // Initialize templates
    let mut tera = Tera::new("templates/**/*.html").expect("Failed to load templates");

    // Custom filter: format_date - converts "YYYY-MM-DD HH:MM:SS" to German format
    tera.register_filter("format_date", |value: &tera::Value, args: &std::collections::HashMap<String, tera::Value>| {
        let s = value.as_str().unwrap_or("");
        let format = args.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("%d.%m.%Y %H:%M");

        Ok(tera::Value::String(format_date(s, format)))
    });

    let tera_data = web::Data::new(tera);

    // Initialize email service
    let email_service = EmailService::new(&config);
    let email_data = web::Data::new(email_service);

    // JWT secret
    let jwt_secret = web::Data::new(config.jwt_secret.clone());
    let config_data = web::Data::new(config.clone());
    let auth_limiter = web::Data::new(security::AuthLimiter::default());

    // Persisted confirmations survive SMTP outages and process restarts.
    {
        let outbox_db = db_data.clone();
        let outbox_email = email_data.get_ref().clone();
        tokio::spawn(async move {
            loop {
                email::dispatch_outbox(outbox_db.clone(), outbox_email.clone()).await;
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }
    // Background task: send reminders + review prompts every hour
    {
        let db_bg = db_data.clone();
        let es_bg = email_data.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                let appointments = {
                    let conn = db_bg.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = conn.execute("DELETE FROM booking_requests WHERE created_at < datetime('now','-1 day')", []);
                    let _ = conn.execute("DELETE FROM revoked_sessions WHERE expires_at < strftime('%s','now')", []);
                    let _ = conn.execute("DELETE FROM email_outbox WHERE sent_at < datetime('now','-30 days')", []);
                    crate::models::appointment::Appointment::find_needing_reminders(&conn).unwrap_or_default()
                };
                for apt in appointments {
                    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&apt.start_time, "%Y-%m-%d %H:%M:%S") {
                        let date_s = dt.format("%d.%m.%Y").to_string();
                        let time_s = dt.format("%H:%M").to_string();
                        if let (Some(email), Some(name)) = (&apt.customer_email, &apt.customer_name) {
                            let duration = chrono::NaiveDateTime::parse_from_str(&apt.end_time, "%Y-%m-%d %H:%M:%S").map(|end| (end-dt).num_minutes()).unwrap_or(0);
                            es_bg.send_appointment_reminder(email, name, &date_s, &time_s, apt.is_home_visit, duration);
                        }
                        let conn = db_bg.lock().unwrap_or_else(|e| e.into_inner());
                        let _ = crate::models::appointment::Appointment::mark_reminder_sent(&conn, apt.id);
                    }
                }
            }
        });
    }

    log::info!("Starting server at http://{}", bind_addr);

    HttpServer::new(move || {
        App::new()
            .wrap(from_fn(security::protect))
            .wrap(security::access_logger())
            .wrap(actix_web::middleware::Compress::default())
            .app_data(web::JsonConfig::default().limit(64 * 1024))
            .app_data(web::FormConfig::default().limit(16 * 1024))
            .app_data(db_data.clone())
            .app_data(tera_data.clone())
            .app_data(email_data.clone())
            .app_data(jwt_secret.clone())
            .app_data(config_data.clone())
            .app_data(auth_limiter.clone())
            // Landing page
            .route("/", web::get().to(handlers::landing::index))
            .route("/index.html", web::get().to(|| async { actix_web::HttpResponse::PermanentRedirect().insert_header(("Location","/")).finish() }))
            .route("/impressum.html", web::get().to(|| async { actix_web::HttpResponse::PermanentRedirect().insert_header(("Location","/impressum")).finish() }))
            .route("/datenschutz.html", web::get().to(|| async { actix_web::HttpResponse::PermanentRedirect().insert_header(("Location","/datenschutz")).finish() }))
            .route("/faszienbehandlung-ruecken", web::get().to(handlers::landing::topic))
            .route("/faszienbehandlung-nacken-schulter", web::get().to(handlers::landing::topic))
            .route("/faszienbehandlung-haltung", web::get().to(handlers::landing::topic))
            .route("/faszienbehandlung-sport", web::get().to(handlers::landing::topic))
            .route("/preise-und-ablauf", web::get().to(handlers::landing::topic))
            .route("/datenschutz", web::get().to(handlers::landing::datenschutz))
            .route("/impressum", web::get().to(handlers::landing::impressum))
            // Consumer information and guest-accessible withdrawal, including without JavaScript.
            .route("/buchungsbedingungen", web::get().to(handlers::legal::information))
            .route("/barrierefreiheit", web::get().to(handlers::legal::information))
            .route("/widerruf", web::get().to(handlers::legal::withdrawal_page))
            .route("/widerruf/pruefen", web::post().to(handlers::legal::withdrawal_review))
            .route("/widerruf/bestaetigen", web::post().to(handlers::legal::withdrawal_confirm))
            // Public booking: no registration, confirmation proves email ownership.
            .route("/buchen", web::get().to(handlers::guest_booking::page))
            .route("/buchen/bestaetigen/{token}", web::get().to(handlers::guest_booking::confirmation_page))
            .route("/api/guest-booking/request", web::post().to(handlers::guest_booking::request))
            .route("/api/guest-booking/confirm", web::post().to(handlers::guest_booking::confirm))
            // Auth pages
            .route("/login", web::get().to(handlers::auth_handler::login_page))
            .route("/register", web::get().to(handlers::auth_handler::register_page))
            .route("/reset-password", web::get().to(handlers::auth_handler::reset_password_page))
            .route("/reset-password/{token}", web::get().to(handlers::auth_handler::reset_password_token_page))
            // E-Mail verification
            .route("/verify-email/{token}", web::get().to(handlers::auth_handler::verify_email_page))
            // Auth API
            .route("/logout", web::get().to(handlers::auth_handler::logout_page))
            .route("/logout", web::post().to(handlers::auth_handler::logout_page))
            .route("/api/auth/login", web::post().to(handlers::auth_handler::api_login))
            .route("/api/auth/register", web::post().to(handlers::auth_handler::api_register))
            .route("/api/auth/reset-password", web::post().to(handlers::auth_handler::api_reset_password_request))
            .route("/api/auth/reset-password/{token}", web::post().to(handlers::auth_handler::api_reset_password_confirm))
            // Customer portal pages
            .route("/portal", web::get().to(handlers::customer_handler::dashboard))
            .route("/portal/appointments", web::get().to(handlers::customer_handler::appointments_page))
            .route("/portal/book", web::get().to(handlers::customer_handler::book_page))
            .route("/portal/profile", web::get().to(handlers::customer_handler::profile_page))
            .route("/portal/credits", web::get().to(handlers::customer_handler::credits_page))
            // Customer API
            .route("/api/customer/profile", web::post().to(handlers::customer_handler::api_update_profile))
            .route("/api/customer/change-password", web::post().to(handlers::customer_handler::api_change_password))
            .route("/api/customer/change-email", web::post().to(handlers::customer_handler::api_change_email))
            .route("/api/customer/resend-verification", web::post().to(handlers::auth_handler::api_resend_verification))
            .route("/api/customer/export-data", web::get().to(handlers::customer_handler::api_export_data))
            .route("/api/customer/delete-account", web::delete().to(handlers::customer_handler::api_delete_account))
            .route("/api/customer/invoices/{payment_id}", web::get().to(handlers::customer_handler::api_invoice))
            .route("/portal/waitlist", web::get().to(handlers::customer_handler::waitlist_page))
            // Booking API
            .route("/api/appointments/book", web::post().to(handlers::booking_handler::api_book))
            .route("/api/appointments/{id}/cancel", web::post().to(handlers::booking_handler::api_cancel))
            .route("/api/appointments/{id}/reschedule", web::post().to(handlers::booking_handler::api_reschedule))
            .route("/api/appointments/available-slots", web::get().to(handlers::booking_handler::api_available_slots))
            .route("/api/appointments/waitlist/add", web::post().to(handlers::booking_handler::api_waitlist_add))
            .route("/api/appointments/waitlist/remove", web::post().to(handlers::booking_handler::api_waitlist_remove))
            // Calendar exports (token-based, Nextcloud/iOS/Google compatible)
            .route("/api/calendar/{token}/termine.ics", web::get().to(handlers::calendar_handler::customer_calendar_ics))
            .route("/api/admin/calendar/{token}/alle-termine.ics", web::get().to(handlers::calendar_handler::admin_calendar_ics))
            // Admin pages
            .route("/admin", web::get().to(handlers::admin_handler::dashboard))
            .route("/admin/widerruf/{id}/bearbeitet", web::post().to(handlers::admin_handler::process_withdrawal))
            .route("/api/admin/retained-invoices", web::get().to(handlers::admin_handler::retained_invoices))
            .route("/admin/customers", web::get().to(handlers::admin_handler::customers_page))
            .route("/admin/customers/{id}", web::get().to(handlers::admin_handler::customer_detail))
            .route("/admin/appointments", web::get().to(handlers::admin_handler::appointments_page))
            .route("/admin/payments", web::get().to(handlers::admin_handler::payments_page))
            .route("/admin/faq", web::get().to(handlers::admin_handler::faq_editor))
            .route("/admin/reviews", web::get().to(handlers::admin_handler::review_editor))
            .route("/admin/settings", web::get().to(handlers::admin_handler::settings_page))
            // Admin API
            .route("/api/admin/payments/{id}/mark-paid", web::post().to(handlers::admin_handler::api_mark_paid))
            .route("/api/admin/faq", web::post().to(handlers::admin_handler::api_save_faq))
            .route("/api/admin/faq/{id}", web::delete().to(handlers::admin_handler::api_delete_faq))
            .route("/api/admin/reviews", web::post().to(handlers::admin_handler::api_save_review))
            .route("/api/admin/reviews/{id}", web::delete().to(handlers::admin_handler::api_delete_review))
            .route("/api/admin/settings", web::post().to(handlers::admin_handler::api_save_settings))
            .route("/api/admin/appointments/suggest", web::post().to(handlers::admin_handler::api_suggest_appointment))
            .route("/api/admin/appointments/cancel-suggest", web::post().to(handlers::admin_handler::api_cancel_with_suggestions))
            .route("/api/admin/appointments/create", web::post().to(handlers::admin_handler::api_create_appointment))
            .route("/api/admin/appointments/{id}/therapist-notes", web::post().to(handlers::admin_handler::api_update_therapist_notes))
            .route("/api/admin/test-caldav", web::get().to(handlers::admin_handler::api_test_caldav))
            .route("/admin/stats", web::get().to(handlers::admin_handler::stats_page))
            // Nextcloud OAuth2
            .route("/admin/nextcloud/connect", web::get().to(handlers::oauth_handler::nextcloud_connect))
            .route("/admin/nextcloud/callback", web::get().to(handlers::oauth_handler::nextcloud_callback))
            .route("/admin/nextcloud/disconnect", web::post().to(handlers::oauth_handler::nextcloud_disconnect))
            // Public API
            .route("/api/settings", web::get().to(handlers::api::get_settings))
            .route("/llms.txt", web::get().to(handlers::api::get_llms_txt))
            // Static files
            .service(fs::Files::new("/static", "static").prefer_utf8(true))
            // Root-level static files (images, fonts, favicon, etc.)
            .service(fs::Files::new("/fonts", "fonts").prefer_utf8(true))
            .route("/{asset}", web::get().to(handlers::landing::public_asset))
            .default_service(web::to(handlers::landing::not_found))
    })
    .bind(&bind_addr)?
    .run()
    .await
}

fn format_date(value: &str, format: &str) -> String {
    ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"]
        .iter().find_map(|input| chrono::NaiveDateTime::parse_from_str(value, input).ok())
        .map(|date| date.format(format).to_string()).unwrap_or_else(|| value.to_owned())
}

#[cfg(test)]
mod date_filter_tests {
    #[test]
    fn invalid_unicode_dates_are_not_sliced_at_byte_boundaries() {
        let value = "äöü日本語ungültiges Datum";
        assert_eq!(super::format_date(value, "%d.%m.%Y"), value);
        assert_eq!(super::format_date("2026-10-08 16:30", "%d.%m.%Y %H:%M"), "08.10.2026 16:30");
    }
}
