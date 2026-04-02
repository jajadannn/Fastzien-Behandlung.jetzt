mod config;
mod db;
mod auth;
mod email;
mod models;
mod handlers;

use actix_web::{web, App, HttpServer, middleware::Logger};
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
    let tera = Tera::new("templates/**/*.html").expect("Failed to load templates");
    let tera_data = web::Data::new(tera);

    // Initialize email service
    let email_service = EmailService::new(&config);
    let email_data = web::Data::new(email_service);

    // JWT secret
    let jwt_secret = web::Data::new(config.jwt_secret.clone());
    let config_data = web::Data::new(config.clone());

    log::info!("Starting server at http://{}", bind_addr);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(db_data.clone())
            .app_data(tera_data.clone())
            .app_data(email_data.clone())
            .app_data(jwt_secret.clone())
            .app_data(config_data.clone())
            // Landing page
            .route("/", web::get().to(handlers::landing::index))
            .route("/datenschutz", web::get().to(handlers::landing::datenschutz))
            .route("/impressum", web::get().to(handlers::landing::impressum))
            // Auth pages
            .route("/login", web::get().to(handlers::auth_handler::login_page))
            .route("/register", web::get().to(handlers::auth_handler::register_page))
            .route("/reset-password", web::get().to(handlers::auth_handler::reset_password_page))
            .route("/reset-password/{token}", web::get().to(handlers::auth_handler::reset_password_token_page))
            // Auth API
            .route("/api/auth/login", web::post().to(handlers::auth_handler::api_login))
            .route("/api/auth/register", web::post().to(handlers::auth_handler::api_register))
            .route("/api/auth/logout", web::post().to(handlers::auth_handler::api_logout))
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
            // Booking API
            .route("/api/appointments/book", web::post().to(handlers::booking_handler::api_book))
            .route("/api/appointments/{id}/cancel", web::post().to(handlers::booking_handler::api_cancel))
            .route("/api/appointments/available-slots", web::get().to(handlers::booking_handler::api_available_slots))
            // Admin pages
            .route("/admin", web::get().to(handlers::admin_handler::dashboard))
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
            // Public API
            .route("/api/settings", web::get().to(handlers::api::get_settings))
            // Static files
            .service(fs::Files::new("/static", "static").show_files_listing().prefer_utf8(true))
            // Legacy image files (for landing page backward compat)
            .service(fs::Files::new("/", ".").show_files_listing().prefer_utf8(true).index_file("_none_"))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
