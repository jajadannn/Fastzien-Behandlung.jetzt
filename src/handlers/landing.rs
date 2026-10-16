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
    let claims = auth::get_claims(&req, &jwt_secret);
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let settings = SiteSetting::get_public(&conn).unwrap_or_default();
    let faqs = Faq::find_active(&conn).unwrap_or_default();
    let reviews = Review::find_active(&conn).unwrap_or_default();

    let is_logged_in = claims.is_some();
    let is_admin = claims.as_ref().map(|c| c.is_admin).unwrap_or(false);

    let review_count = reviews.len();
    let mut avg_rating: f64 = 0.0;
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
    let pack_price: f64 = settings.get("price_pack").map(|s| s.replace(',',".")).unwrap_or_default().parse().unwrap_or(169.90);
    let pack_count: i32 = settings.get("price_pack_count").and_then(|s| s.parse().ok()).unwrap_or(10);
    ctx.insert("pack_total", &format!("{:.2}",pack_price * pack_count as f64).replace('.',","));
    let get = |key: &str, default: &str| settings.get(key).cloned().unwrap_or_else(|| default.to_owned());
    let schema = serde_json::json!({
        "@context":"https://schema.org", "@type":"HealthAndBeautyBusiness", "name":"Faszienbehandlung Thilo Seifried – Gantke® Methode", "url":"https://faszienbehandlung.jetzt/",
        "image":"https://faszienbehandlung.jetzt/static/images/portrait-960.webp", "description":"Persönliche Gantke® Faszienbehandlung als Wellness- und Körperarbeitsleistung für Beweglichkeit, Entspannung und Wohlbefinden.",
        "address":{"@type":"PostalAddress","streetAddress":get("address_street","Sulgauer Straße 24"),"postalCode":get("address_zip","78713"),"addressLocality":get("address_city","Schramberg-Sulgen"),"addressCountry":"DE"},
        "telephone":get("phone_raw","+4915234007225"),"email":get("email_contact","termin@faszienbehandlung.jetzt"),
        "hasOfferCatalog":{"@type":"OfferCatalog","name":"Gantke® Faszienbehandlung","itemListElement":[{"@type":"Offer","price":get("price_single","195").replace(',',"."),"priceCurrency":"EUR","itemOffered":{"@type":"Service","name":format!("Gantke® Einzelbehandlung (ca. {} Min.)",get("appointment_duration_min","90"))}}]}
    });
    ctx.insert("business_schema", &script_json(&schema));
    let questions: Vec<_> = faqs.iter().map(|f| serde_json::json!({"@type":"Question","name":f.question,"acceptedAnswer":{"@type":"Answer","text":f.answer_html}})).collect();
    ctx.insert("faq_schema", &if questions.is_empty() { String::new() } else { script_json(&serde_json::json!({"@context":"https://schema.org","@type":"FAQPage","mainEntity":questions})) });

    // Insert individual settings for template convenience
    for (key, value) in &settings {
        ctx.insert(key, value);
    }

    match tmpl.render("landing.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => {
            log::error!("Template error: {:#}", e);
            HttpResponse::InternalServerError().body("Die Seite ist vorübergehend nicht verfügbar.")
        }
    }
}

pub async fn datenschutz(tmpl: web::Data<Tera>, db: web::Data<Mutex<Connection>>, config: web::Data<crate::config::Config>) -> HttpResponse {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let mut ctx = tera::Context::new();
    ctx.insert("settings", &SiteSetting::get_public(&conn).unwrap_or_default());
    ctx.insert("session_hours", &config.jwt_expiry_hours);
    render_information(&tmpl, "datenschutz.html", &ctx)
}

pub async fn impressum(tmpl: web::Data<Tera>, db: web::Data<Mutex<Connection>>) -> HttpResponse {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let mut ctx = tera::Context::new();
    ctx.insert("settings", &SiteSetting::get_public(&conn).unwrap_or_default());
    render_information(&tmpl, "impressum.html", &ctx)
}

fn render_information(tmpl: &Tera, name: &str, ctx: &tera::Context) -> HttpResponse {
    match tmpl.render(name, ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => HttpResponse::InternalServerError().body("Die Seite ist vorübergehend nicht erreichbar."),
    }
}

pub async fn not_found(tmpl: web::Data<Tera>) -> HttpResponse {
    HttpResponse::NotFound().content_type("text/html; charset=utf-8")
        .body(tmpl.render("404.html", &tera::Context::new()).unwrap_or_else(|_| "Seite nicht gefunden. Zur Startseite: /".into()))
}

fn script_json(value: &serde_json::Value) -> String {
    value.to_string().replace('<', "\\u003c").replace('>', "\\u003e").replace('&', "\\u0026")
}

pub async fn topic(req: HttpRequest, tmpl: web::Data<Tera>, db: web::Data<Mutex<Connection>>) -> HttpResponse {
    let Some((path, title, description, sections)) = crate::marketing::TOPICS.iter().find(|t| t.0 == req.path()) else { return HttpResponse::NotFound().finish(); };
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let mut ctx = tera::Context::new();
    ctx.insert("path", path);
    ctx.insert("title", title);
    ctx.insert("description", description);
    ctx.insert("sections", sections);
    ctx.insert("settings", &SiteSetting::get_public(&conn).unwrap_or_default());
    match tmpl.render("marketing_topic.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => { log::error!("Topic template: {:#}", e); HttpResponse::InternalServerError().finish() }
    }
}

pub async fn public_asset(path: web::Path<String>, tmpl: web::Data<Tera>) -> actix_web::Result<actix_web::Either<actix_files::NamedFile, HttpResponse>> {
    let name = path.into_inner();
    if !["Logo.png","Logo.avif","Portrait.jpg","Start.avif","og-hero.jpg","favicon.v1.ico","favicon.ico","robots.txt","sitemap.xml"].contains(&name.as_str()) {
        return Ok(actix_web::Either::Right(not_found(tmpl).await));
    }
    Ok(actix_web::Either::Left(actix_files::NamedFile::open_async(name).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn structured_data_is_valid_and_cannot_close_script() {
        let data = serde_json::json!({"text":"Zitat: \"Beweglichkeit\" </script>","price":"169,90"});
        let serialized = script_json(&data);
        assert!(!serialized.contains("</script>"));
        assert_eq!(serde_json::from_str::<serde_json::Value>(&serialized).unwrap(), data);
    }
}
