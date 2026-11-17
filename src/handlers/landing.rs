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
        "address":{"@type":"PostalAddress","streetAddress":get("address_street","Im Kapellenösch 9"),"postalCode":get("address_zip","78628"),"addressLocality":get("address_city","Rottweil"),"addressRegion":get("address_region",""),"addressCountry":"DE"},
        "telephone":get("phone_raw","+4915234007225"),"email":get("email_contact","termin@faszienbehandlung.jetzt"),
        "hasOfferCatalog":{"@type":"OfferCatalog","name":"Gantke® Faszienbehandlung","itemListElement":[{"@type":"Offer","price":get("price_single","195").replace(',',"."),"priceCurrency":"EUR","itemOffered":{"@type":"Service","name":format!("Gantke® Einzelbehandlung (ca. {} Min.)",get("appointment_duration_min","90"))}}]}
    });
    let mut schema = schema;
    location_schema(&mut schema, &settings);
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

/// Adds coordinates and the home-visit area from the admin settings; empty values are left out.
fn location_schema(schema: &mut serde_json::Value, settings: &std::collections::HashMap<String, String>) {
    let value = |key: &str| settings.get(key).map(|v| v.trim().replace(',', ".")).filter(|v| !v.is_empty());
    if let (Some(lat), Some(lng)) = (value("geo_lat").and_then(|v| v.parse::<f64>().ok()), value("geo_lng").and_then(|v| v.parse::<f64>().ok())) {
        schema["geo"] = serde_json::json!({"@type":"GeoCoordinates","latitude":lat,"longitude":lng});
    }
    let places: Vec<_> = settings.get("service_area").map(|v| v.split(',').map(str::trim).filter(|p| !p.is_empty())
        .map(|p| serde_json::json!({"@type":"Place","name":p})).collect()).unwrap_or_default();
    if !places.is_empty() { schema["areaServed"] = serde_json::Value::Array(places); }
    if schema["address"]["addressRegion"].as_str().is_some_and(str::is_empty) {
        if let Some(address) = schema["address"].as_object_mut() { address.remove("addressRegion"); }
    }
}

fn script_json(value: &serde_json::Value) -> String {
    value.to_string().replace('<', "\\u003c").replace('>', "\\u003e").replace('&', "\\u0026")
}

pub async fn topic(req: HttpRequest, tmpl: web::Data<Tera>, db: web::Data<Mutex<Connection>>) -> HttpResponse {
    let Some((path, title, description, sections)) = crate::marketing::TOPICS.iter().find(|t| t.0 == req.path()) else { return HttpResponse::NotFound().finish(); };
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let mut ctx = tera::Context::new();
    let settings = SiteSetting::get_public(&conn).unwrap_or_default();
    let localize = |text: &str| crate::marketing::localize(text, &settings);
    let sections: Vec<(String, String)> = sections.iter().map(|(heading, body)| (localize(heading), localize(body))).collect();
    ctx.insert("path", path);
    ctx.insert("title", &localize(title));
    ctx.insert("description", &localize(description));
    ctx.insert("sections", &sections);
    ctx.insert("settings", &settings);
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
    #[test] fn location_schema_follows_settings() {
        let mut settings = std::collections::HashMap::new();
        settings.insert("geo_lat".to_owned(), "48,1".to_owned());
        settings.insert("geo_lng".to_owned(), "8.4".to_owned());
        settings.insert("service_area".to_owned(), "Rottweil, Dunningen, ".to_owned());
        let mut schema = serde_json::json!({"address":{"addressLocality":"Rottweil","addressRegion":""}});
        location_schema(&mut schema, &settings);
        assert_eq!(schema["geo"]["latitude"], 48.1);
        assert_eq!(schema["areaServed"][1]["name"], "Dunningen");
        assert_eq!(schema["areaServed"].as_array().unwrap().len(), 2);
        assert!(schema["address"].get("addressRegion").is_none());
        let mut empty = serde_json::json!({"address":{}});
        location_schema(&mut empty, &std::collections::HashMap::new());
        assert!(empty.get("geo").is_none() && empty.get("areaServed").is_none());
    }
    #[test] fn topic_texts_use_configured_location() {
        let mut settings = std::collections::HashMap::new();
        settings.insert("address_city".to_owned(), "Rottweil".to_owned());
        for (_, title, description, sections) in crate::marketing::TOPICS {
            for text in [*title, *description].into_iter().chain(sections.iter().flat_map(|(h, b)| [*h, *b])) {
                let localized = crate::marketing::localize(text, &settings);
                assert!(!localized.contains("Sulgen") && !localized.contains("{ort}"), "{localized}");
            }
        }
    }
    #[test] fn structured_data_is_valid_and_cannot_close_script() {
        let data = serde_json::json!({"text":"Zitat: \"Beweglichkeit\" </script>","price":"169,90"});
        let serialized = script_json(&data);
        assert!(!serialized.contains("</script>"));
        assert_eq!(serde_json::from_str::<serde_json::Value>(&serialized).unwrap(), data);
    }
}
