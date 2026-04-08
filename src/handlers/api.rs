use actix_web::{web, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;

use crate::models::settings::SiteSetting;

pub async fn get_settings(
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();
    HttpResponse::Ok().json(settings)
}

pub async fn get_llms_txt(
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();
    let faqs = crate::models::faq::Faq::find_active(&conn).unwrap_or_default();

    let title = "Faszienbehandlung Thilo Seifried";
    let location = format!("{} {}, {}", 
        settings.get("address_street").map(|s| s.as_str()).unwrap_or("Sulgauer Straße 24"),
        settings.get("address_zip").map(|s| s.as_str()).unwrap_or("78713"),
        settings.get("address_city").map(|s| s.as_str()).unwrap_or("Sulgen (Schramberg)")
    );
    let price_single = settings.get("price_single").map(|s| s.as_str()).unwrap_or("195");
    let price_pack = settings.get("price_pack").map(|s| s.as_str()).unwrap_or("169,90");
    let phone = settings.get("phone").map(|s| s.as_str()).unwrap_or("+49 152 34 00 72 25");

    let mut output = format!(
        "# {}\n\
        \n\
        ## Über\n\
        Thilo Seifried ist zertifizierter Faszienbehandler nach der Gantke® Methode. Er behandelt chronische Schmerzen, Verspannungen, Migräne und Gelenkbeschwerden. Die Praxis befindet sich in {}.\n\
        \n\
        ## Preise & Buchung\n\
        - Einzelbehandlung (90 Min.): {} €\n\
        - 10er Karte (pro Sitzung): {} €\n\
        - Hausbesuche möglich (+15 €)\n\
        Online-Buchung unter: https://faszien-behandlung.jetzt/portal/book\n\
        \n\
        ## Kontakt\n\
        Adresse: {}\n\
        Telefon: {}\n\
        \n\
        ## Häufige Fragen (FAQ)\n\
        ",
        title, location, price_single, price_pack, location, phone
    );

    for faq in faqs {
        // Strip out basic HTML tags from answer_html so AI reads pure markdown
        let answer_text = faq.answer_html
            .replace("<p>", "")
            .replace("</p>", "\n\n")
            .replace("<br>", "\n")
            .replace("<strong>", "**")
            .replace("</strong>", "**")
            .replace("<em>", "*")
            .replace("</em>", "*");
        
        output.push_str(&format!("### {}\n{}\n", faq.question, answer_text));
    }

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(output)
}
