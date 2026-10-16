use actix_web::{web, HttpResponse};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tera::Tera;
use crate::{config::Config, email::EmailService, models::settings::SiteSetting};

pub const TERMS_VERSION: &str = "2026-10-09";

pub fn withdrawal_text(settings: &std::collections::HashMap<String, String>) -> String {
    let get = |k: &str, fallback: &str| settings.get(k).cloned().unwrap_or_else(|| fallback.into());
    format!("Widerrufsbelehrung für Dienstleistungen\n\nDu hast das Recht, binnen vierzehn Tagen ohne Angabe von Gründen diesen Vertrag zu widerrufen. Die Widerrufsfrist beträgt vierzehn Tage ab dem Tag des Vertragsabschlusses.\n\nUm dein Widerrufsrecht auszuüben, musst du uns (Thilo Seifried, {}, {} {}, Telefon {}, E-Mail {}) mittels einer eindeutigen Erklärung (z. B. ein mit der Post versandter Brief oder eine E-Mail) über deinen Entschluss, diesen Vertrag zu widerrufen, informieren. Du kannst dafür das unten stehende Muster-Widerrufsformular verwenden, das jedoch nicht vorgeschrieben ist. Du kannst deine Erklärung auch auf https://faszienbehandlung.jetzt/widerruf elektronisch übermitteln. In diesem Fall bestätigen wir den Eingang unverzüglich per E-Mail. Zur Wahrung der Widerrufsfrist reicht es aus, dass du die Mitteilung über die Ausübung des Widerrufsrechts vor Ablauf der Widerrufsfrist absendest.\n\nFolgen des Widerrufs\nWenn du diesen Vertrag widerrufst, haben wir dir alle Zahlungen, die wir von dir erhalten haben, einschließlich der Lieferkosten (mit Ausnahme zusätzlicher Kosten, die sich daraus ergeben, dass du eine andere Art der Lieferung als die von uns angebotene günstigste Standardlieferung gewählt hast), unverzüglich und spätestens binnen vierzehn Tagen ab dem Tag zurückzuzahlen, an dem die Mitteilung über deinen Widerruf bei uns eingegangen ist. Für diese Rückzahlung verwenden wir dasselbe Zahlungsmittel, das du bei der ursprünglichen Transaktion eingesetzt hast, es sei denn, mit dir wurde ausdrücklich etwas anderes vereinbart; in keinem Fall werden dir wegen dieser Rückzahlung Entgelte berechnet.\n\nHast du verlangt, dass die Dienstleistungen während der Widerrufsfrist beginnen sollen, so hast du uns einen angemessenen Betrag zu zahlen, der dem Anteil der bis zu dem Zeitpunkt, zu dem du uns von der Ausübung des Widerrufsrechts hinsichtlich dieses Vertrags unterrichtest, bereits erbrachten Dienstleistungen im Vergleich zum Gesamtumfang der im Vertrag vorgesehenen Dienstleistungen entspricht.\n\nBei vollständiger Leistungserbringung erlischt das Widerrufsrecht nur unter den gesetzlichen Voraussetzungen: ausdrückliche Zustimmung zum Beginn vor Fristablauf und Bestätigung der Kenntnis, dass das Recht bei vollständiger Vertragserfüllung erlischt. Eine gewöhnliche Terminabsage ist vom gesetzlichen Widerruf unabhängig.\n\nMuster-Widerrufsformular\n(Wenn du den Vertrag widerrufen willst, fülle dieses Formular aus und sende es zurück.)\nAn Thilo Seifried, {}, {} {}, E-Mail {}:\nHiermit widerrufe(n) ich/wir (*) den von mir/uns (*) abgeschlossenen Vertrag über die Erbringung der folgenden Dienstleistung: …\nBestellt am: …\nName des/der Verbraucher(s): …\nAnschrift des/der Verbraucher(s): …\nUnterschrift des/der Verbraucher(s) (nur bei Mitteilung auf Papier): …\nDatum: …\n(*) Unzutreffendes streichen.",
        get("address_street", "Sulgauer Straße 24"), get("address_zip", "78713"), get("address_city", "Schramberg-Sulgen"), get("phone", "+49 152 34 00 72 25"), get("email_contact", "termin@faszienbehandlung.jetzt"),
        get("address_street", "Sulgauer Straße 24"), get("address_zip", "78713"), get("address_city", "Schramberg-Sulgen"), get("email_contact", "termin@faszienbehandlung.jetzt"))
}

pub async fn information(req: actix_web::HttpRequest, tmpl: web::Data<Tera>, db: web::Data<Mutex<Connection>>) -> HttpResponse {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let mut ctx = tera::Context::new();
    let settings = SiteSetting::get_public(&conn).unwrap_or_default();
    ctx.insert("settings", &settings);
    ctx.insert("withdrawal_text", &withdrawal_text(&settings));
    ctx.insert("accessibility", &(req.path() == "/barrierefreiheit"));
    render(&tmpl, "legal.html", &ctx)
}

pub async fn withdrawal_page(tmpl: web::Data<Tera>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    ctx.insert("step", "entry");
    ctx.insert("form", &WithdrawalForm::default());
    render(&tmpl, "withdrawal.html", &ctx)
}

#[derive(Default, Deserialize, Serialize)]
pub struct WithdrawalForm {
    pub name: String,
    pub email: String,
    pub contract: String,
    #[serde(default)] pub request_id: String,
    #[serde(default)] pub website: String,
}

fn valid(form: &WithdrawalForm) -> bool {
    !form.name.trim().is_empty() && form.name.chars().count() <= 200
        && !form.name.chars().any(char::is_control) && form.email.len() <= 254
        && form.email.parse::<lettre::Address>().is_ok() && !form.contract.trim().is_empty()
        && form.contract.chars().count() <= 1000 && form.website.is_empty()
}

pub async fn withdrawal_review(form: web::Form<WithdrawalForm>, tmpl: web::Data<Tera>) -> HttpResponse {
    let mut form = form.into_inner();
    let mut ctx = tera::Context::new();
    ctx.insert("step", if valid(&form) { "review" } else { "entry" });
    if !valid(&form) { ctx.insert("error", "Bitte gib deinen Namen, eine gültige E-Mail-Adresse und den betroffenen Vertrag an."); }
    form.request_id = uuid::Uuid::new_v4().to_string();
    ctx.insert("form", &form);
    render(&tmpl, "withdrawal.html", &ctx)
}

pub async fn withdrawal_confirm(form: web::Form<WithdrawalForm>, tmpl: web::Data<Tera>, db: web::Data<Mutex<Connection>>, es: web::Data<EmailService>, config: web::Data<Config>) -> HttpResponse {
    let form = form.into_inner();
    let mut ctx = tera::Context::new();
    ctx.insert("form", &form);
    if !valid(&form) || uuid::Uuid::parse_str(&form.request_id).is_err() {
        ctx.insert("step", "entry"); ctx.insert("error", "Bitte prüfe deine Angaben und öffne anschließend die Zusammenfassung.");
        return render(&tmpl, "withdrawal.html", &ctx);
    }
    let receipt = {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        match record_withdrawal(&conn, &form, &config.admin_email) {
            Ok(receipt) => receipt,
            Err(_) => {
                ctx.insert("step", "review"); ctx.insert("error", "Dein Widerruf konnte nicht gespeichert werden. Bitte versuche es erneut oder sende ihn per E-Mail.");
                return HttpResponse::ServiceUnavailable().content_type("text/html; charset=utf-8").body(tmpl.render("withdrawal.html", &ctx).unwrap_or_default());
            }
        }
    };
    // Persist first, then attempt delivery. Failed SMTP is retried by the outbox.
    tokio::spawn(crate::email::dispatch_outbox(db.clone(), es.get_ref().clone()));
    ctx.insert("step", "received");
    ctx.insert("receipt", &receipt);
    render(&tmpl, "withdrawal.html", &ctx)
}

pub fn record_withdrawal(conn: &Connection, form: &WithdrawalForm, admin_email: &str) -> rusqlite::Result<String> {
    let tx = rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate)?;
    // An unverified statement never discloses or changes someone else's booking.
    // Same request ID is idempotent; an ID cannot be reused with different data.
    let inserted = tx.execute("INSERT OR IGNORE INTO withdrawals(id,name,email,contract) VALUES(?1,?2,?3,?4)", params![form.request_id,form.name,form.email,form.contract])?;
    let (name,email,contract,created): (String,String,String,String) = tx.query_row("SELECT name,email,contract,created_at FROM withdrawals WHERE id=?1", [&form.request_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
    if (name.as_str(), email.as_str(), contract.as_str()) != (form.name.as_str(),form.email.as_str(),form.contract.as_str()) { return Err(rusqlite::Error::InvalidQuery); }
    let receipt = format!("Eingangsbestätigung Widerruf\nReferenz: {}\nEingang: {} UTC\nName: {}\nE-Mail: {}\nVertrag / Teil des Vertrags: {}\n\nErklärung: Hiermit widerrufe ich den bezeichneten Vertrag bzw. den bezeichneten Teil des Vertrags.\n\nDer Eingang ist gespeichert. Die Zuordnung, rechtlichen Voraussetzungen und gegebenenfalls eine Rückzahlung werden geprüft. Diese Bestätigung bestätigt den Eingang der Erklärung.", form.request_id,created,name,email,contract);
    if inserted == 1 {
        for recipient in [&email, &admin_email.to_owned()] {
            crate::email::enqueue_text(&tx, recipient, "Eingangsbestätigung Widerruf", &receipt)?;
        }
    }
    tx.commit()?;
    Ok(receipt)
}

fn render(tmpl: &Tera, name: &str, ctx: &tera::Context) -> HttpResponse {
    match tmpl.render(name, ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(_) => HttpResponse::InternalServerError().body("Die Seite ist vorübergehend nicht erreichbar."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receipt_is_persistent_idempotent_and_does_not_disclose_bookings() {
        let conn = crate::db::test_connection();
        let mut f = WithdrawalForm { name:"Test <script>".into(),email:"test@example.test".into(),contract:"Teilwiderruf Termin 123".into(),request_id:uuid::Uuid::new_v4().to_string(),website:String::new() };
        let first = record_withdrawal(&conn,&f,"admin@example.test").unwrap();
        assert_eq!(first,record_withdrawal(&conn,&f,"admin@example.test").unwrap());
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM email_outbox",[],|r|r.get::<_,i64>(0)).unwrap(),2);
        f.email = "other@example.test".into();
        assert!(record_withdrawal(&conn,&f,"admin@example.test").is_err());
    }
}
