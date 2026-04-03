use actix_web::{web, HttpRequest, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;

use crate::auth;
use crate::models::faq::Faq;
use crate::models::review::Review;
use crate::models::settings::SiteSetting;

pub async fn index(
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    db: web::Data<Mutex<Connection>>,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let conn = db.lock().unwrap();
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();
    let faqs = Faq::find_active(&conn).unwrap_or_default();
    let reviews = Review::find_active(&conn).unwrap_or_default();

    let claims = auth::get_claims(&req, &jwt_secret);
    let is_logged_in = claims.is_some();
    let is_admin = claims.as_ref().map(|c| c.is_admin).unwrap_or(false);

    let review_count = reviews.len();
    let mut avg_rating: f64 = 5.0; // fallback default
    if review_count > 0 {
        let sum: i32 = reviews.iter().map(|r| r.stars).sum();
        avg_rating = sum as f64 / review_count as f64;
    }

    let mut ctx = tera::Context::new();
    ctx.insert("settings", &settings);
    ctx.insert("faqs", &faqs);
    ctx.insert("reviews", &reviews);
    ctx.insert("review_count", &review_count);
    ctx.insert("avg_rating", &format!("{:.1}", avg_rating).replace(",", "."));
    ctx.insert("is_logged_in", &is_logged_in);
    ctx.insert("is_admin", &is_admin);

    // Insert individual settings for template convenience
    for (key, value) in &settings {
        ctx.insert(key, value);
    }

    match tmpl.render("landing.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => {
            log::error!("Template error: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

pub async fn datenschutz(tmpl: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    match tmpl.render("datenschutz.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => {
            // Fallback: serve the original file
            match std::fs::read_to_string("datenschutz.html") {
                Ok(content) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(content),
                Err(_) => HttpResponse::NotFound().body("Seite nicht gefunden"),
            }
        }
    }
}

pub async fn impressum(tmpl: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    match tmpl.render("impressum.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => {
            match std::fs::read_to_string("impressum.html") {
                Ok(content) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(content),
                Err(_) => HttpResponse::NotFound().body("Seite nicht gefunden"),
            }
        }
    }
}
