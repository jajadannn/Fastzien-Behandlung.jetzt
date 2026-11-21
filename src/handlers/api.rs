use actix_web::{web, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;

use crate::models::settings::SiteSetting;

pub(crate) fn internal_error(operation: &'static str) -> HttpResponse {
    // Keep SQL, template context, tokens and personal data out of responses/logs.
    log::error!("Request failed during {}", operation);
    HttpResponse::InternalServerError().json(serde_json::json!({
        "error": "Die Anfrage konnte gerade nicht verarbeitet werden. Bitte versuche es später erneut."
    }))
}

pub async fn get_settings(
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let settings = SiteSetting::get_public(&conn).unwrap_or_default();
    HttpResponse::Ok().json(settings)
}

pub async fn get_llms_txt(
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let settings = SiteSetting::get_public(&conn).unwrap_or_default();
    let faqs: Vec<(String, String)> = crate::models::faq::Faq::find_active(&conn).unwrap_or_default()
        .into_iter().map(|f| (f.question, f.answer_html)).collect();
    drop(conn);

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(render_llms_txt(&settings, &faqs))
}

const SITE: &str = "https://faszienbehandlung.jetzt";

/// Plain-text summary for AI systems. Wording must stay within the published
/// positioning: wellness and bodywork, no medical diagnosis, cure or treatment claims.
fn render_llms_txt(settings: &std::collections::HashMap<String, String>, faqs: &[(String, String)]) -> String {
    let get = |key: &str, default: &str| settings.get(key).map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or(default).to_owned();
    let address = format!("{}, {} {}", get("address_street", "Im Kapellenösch 9"), get("address_zip", "78628"), get("address_city", "Rottweil"));
    let duration = get("appointment_duration_min", "90");
    let city = get("address_city", "Rottweil");
    let region = settings.get("address_region").map(|v| v.trim()).filter(|v| !v.is_empty()).map(|v| format!(" ({v})")).unwrap_or_default();
    let area = settings.get("service_area").map(|v| v.trim()).filter(|v| !v.is_empty()).map(|v| format!("- Hausbesuche: {v}\n")).unwrap_or_default();

    let total = |price: &str, count: &str| -> String {
        let value = price.replace(',', ".").parse::<f64>().unwrap_or(0.0) * count.parse::<f64>().unwrap_or(0.0);
        if value.fract() == 0.0 { format!("{value:.0}") } else { format!("{value:.2}").replace('.', ",") }
    };
    let mut out = format!(
"# Faszienbehandlung Thilo Seifried

> Persönliche Gantke® Faszienbehandlung bei Thilo Seifried in {city}{region}. Wellness- und Körperarbeit für Erwachsene, die sich Zeit für Beweglichkeit, Entspannung, Haltung und Wohlbefinden nehmen möchten. Keine Heilkunde: Die Sitzung ersetzt keine ärztliche Diagnose oder Behandlung.

## Über das Angebot
- Anbieter: Thilo Seifried, Faszien-Behandler (Zertifikat der Gantke Akademie für ganzheitliche Körperarbeit vom 17. Juli 2024, fünf Module mit insgesamt 144 Stunden)
- Leitidee: Körper. Bewusstsein. Entwicklung. Die körperliche Erfahrung ist der Ausgangspunkt; wie weit das Gespräch geht, entscheidet die Person selbst.
- Methode: Gantke® Faszienbehandlung, manuelle Faszien- und Körperarbeit mit dem gesamten Fasziennetz, ohne Geräte
- Ablauf eines Termins (ca. {duration} Minuten): Gespräch über Wünsche, Körperempfinden und Ziele, Betrachtung von Haltung und Bewegungsgewohnheiten, manuelle Gantke® Körperarbeit nach der Rückmeldung des Gegenübers, Abschluss mit einer vertrauten Bewegung und bei Bedarf einer Anregung für den Alltag
- Für wen: Erwachsene, etwa nach langen Tagen am Schreibtisch, bei einseitiger Belastung im Beruf oder mit viel Sport; auch Unternehmer, Selbstständige und Führungskräfte
- Nicht geeignet bei akuten Entzündungen, Fieber, frischen Operationswunden oder Thrombosen. Beschwerden mit Krankheitswert vorher ärztlich abklären.
- Es werden keine bestimmten Ergebnisse und keine feste Anzahl an Sitzungen versprochen.

## Preise
- Einzelbehandlung (ca. {duration} Min.): {single} €, Vorgespräch inklusive
- {small_count} Termine: {small_total} € ({small} € je Termin); als Einstieg empfohlen, möglichst innerhalb von acht Wochen
- {pack_count} Termine: {pack_total} € ({pack} € je Termin)
- Für Unternehmen: Faszien-Tag im Betrieb auf Anfrage, Umfang und Preis werden individuell geklärt
- Hausbesuch: zusätzlich {surcharge} € Fahrtpauschale
- Zahlung bar oder per Überweisung, keine Vorauszahlung. Rechnung für die eigenen Unterlagen.
- Keine Erstattung durch Krankenkassen oder Versicherungen (Selbstzahlerleistung).

## Termine und Buchung
- Online-Buchung ohne Konto: {SITE}/buchen (Termin wählen, Kontaktdaten angeben, per E-Mail bestätigen)
- Öffnungszeiten: {weekday}; {saturday}; Sonntag geschlossen
- Absage bis {cancel} Stunden vor dem Termin

## Ort
- Praxis: {address}
- Kostenlose Parkplätze direkt vor dem Haus
{area}
## Kontakt
- Telefon: {phone}
- E-Mail: {email}

## Seiten
- [Startseite]({SITE}/): Methode, Ablauf, Preise, Anfahrt und häufige Fragen
- [Über mich]({SITE}/ueber-mich): Thilo Seifried, Haltung, Arbeitsweise und Qualifikation
- [Für Unternehmen]({SITE}/unternehmen): Faszien-Tag im Unternehmen auf Anfrage
",
        single = get("price_single", "195"), pack = get("price_pack", "169"), pack_count = get("price_pack_count", "10"),
        small = get("price_pack_small", "185"), small_count = get("price_pack_small_count", "3"),
        pack_total = total(&get("price_pack", "169"), &get("price_pack_count", "10")),
        small_total = total(&get("price_pack_small", "185"), &get("price_pack_small_count", "3")),
        surcharge = get("home_visit_surcharge", "15"), weekday = get("hours_weekday", "Mo–Fr 16:00–22:00 Uhr"),
        saturday = get("hours_saturday", "Sa 9:00–19:00 Uhr"), cancel = get("cancellation_hours", "24"),
        phone = get("phone", "+49 152 34 00 72 25"), email = get("email_contact", "termin@faszienbehandlung.jetzt"),
    );
    for (path, title, description, _) in crate::marketing::TOPICS {
        let (title, description) = (crate::marketing::localize(title, settings), crate::marketing::localize(description, settings));
        out.push_str(&format!("- [{title}]({SITE}{path}): {description}\n"));
    }
    out.push_str(&format!("- [Termin buchen]({SITE}/buchen): Online-Terminwahl ohne Registrierung\n- [Impressum]({SITE}/impressum)\n- [Datenschutz]({SITE}/datenschutz)\n"));

    if !faqs.is_empty() {
        out.push_str("\n## Häufige Fragen\n");
        for (question, answer_html) in faqs {
            out.push_str(&format!("\n### {}\n{}\n", question.trim(), html_to_text(answer_html)));
        }
    }
    out
}

fn html_to_text(html: &str) -> String {
    let with_breaks = html.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n").replace("</p>", "\n\n");
    let mut text = String::with_capacity(with_breaks.len());
    let mut in_tag = false;
    for c in with_breaks.chars() {
        match c { '<' => in_tag = true, '>' if in_tag => in_tag = false, _ if !in_tag => text.push(c), _ => {} }
    }
    let text = text.replace("&nbsp;", " ").replace("&amp;", "&").replace("&quot;", "\"").replace("&lt;", "<").replace("&gt;", ">");
    let mut lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    lines.dedup_by(|a, b| a.is_empty() && b.is_empty());
    lines.join("\n").trim().to_owned()
}

#[cfg(test)]
mod llms_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn llms_txt_stays_within_positioning_and_points_to_public_booking() {
        let mut settings = HashMap::new();
        settings.insert("address_city".to_owned(), "Rottweil".to_owned());
        let faqs = vec![("Frage?".to_owned(), "<p>Erste Zeile<br>Zweite <strong>Zeile</strong></p>".to_owned())];
        let text = render_llms_txt(&settings, &faqs);
        let lower = text.to_lowercase();
        for forbidden in ["behandelt chronische", "migräne", "gelenkbeschwerden", "/portal/book", "<p>", "<strong>"] {
            assert!(!lower.contains(forbidden), "unexpected {forbidden:?}");
        }
        assert!(text.contains("https://faszienbehandlung.jetzt/buchen"));
        assert!(text.contains("Im Kapellenösch 9, 78628 Rottweil"));
        assert!(text.contains("Keine Heilkunde"));
        assert!(text.contains("https://faszienbehandlung.jetzt/preise-und-ablauf"));
        assert!(text.contains("3 Termine: 555 € (185 € je Termin)") && text.contains("10 Termine: 1690 € (169 € je Termin)"));
        assert!(text.contains("/ueber-mich") && text.contains("/unternehmen"));
        assert!(text.contains("### Frage?\nErste Zeile\nZweite Zeile"));
        assert!(!text.contains("{ort}") && !text.contains("Hausbesuche:"));
        settings.insert("address_city".to_owned(), "Rottweil".to_owned());
        settings.insert("service_area".to_owned(), "Rottweil, Zimmern".to_owned());
        let moved = render_llms_txt(&settings, &faqs);
        assert!(!moved.contains("Sulgen") && moved.contains("in Rottweil") && moved.contains("Hausbesuche: Rottweil, Zimmern"));
    }
}
