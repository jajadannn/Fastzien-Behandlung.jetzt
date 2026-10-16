use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
};
use log::{info, error};
use crate::config::Config;

const SKY_FROM: &str = "#70AECD";
const SKY_TO: &str = "#4d93b8";
const PLUM_FROM: &str = "#964279";
const PLUM_TO: &str = "#7a3661";
const LOGO_URL: &str = "https://faszienbehandlung.jetzt/static/images/logo-240.png";

/// Wraps a block of body HTML in the shared branded shell: a hidden preheader (the
/// one-line summary inbox apps show before the subject is opened), a gradient header
/// with a flat background-color fallback (Outlook desktop ignores linear-gradient)
/// carrying the logo and a serif heading, a white body area, and a footer with
/// business contact details.
fn escape_html(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('\"', "&quot;")
}

fn appointment_duration_text(minutes: i64) -> String {
    if minutes > 0 { format!("ca. {minutes} Minuten") } else { "Wie in deiner Buchung vereinbart".into() }
}

pub(crate) fn suggestion_rows(slots: &[String]) -> Result<String, &'static str> {
    if slots.len() > 8 { return Err("Bitte höchstens acht alternative Termine angeben."); }
    slots.iter().map(|slot| {
        let parsed = ["%Y-%m-%dT%H:%M", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M", "%Y-%m-%d %H:%M:%S"]
            .iter().find_map(|format| chrono::NaiveDateTime::parse_from_str(slot,format).ok())
            .ok_or("Bitte gültige Datums- und Uhrzeitangaben für die Vorschläge verwenden.")?;
        Ok(format!("<p style='margin:8px 0;padding:10px 14px;background:#dff0f7;color:#1a2a33;font-size:15px;'>{} Uhr</p>", parsed.format("%d.%m.%Y %H:%M")))
    }).collect::<Result<Vec<_>, _>>().map(|rows|rows.join(""))
}

fn shell(top: &str, bottom: &str, heading: &str, preheader: &str, body: &str) -> String {
    format!(
        r#"<div style="display:none;max-height:0;overflow:hidden;opacity:0;mso-hide:all;font-size:1px;line-height:1px;color:#f7fbfd;">{preheader}</div>
<div style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; max-width: 600px; margin: 0 auto; background: #f7fbfd;">
    <div style="background-color: {bottom}; background-image: linear-gradient(135deg, {top}, {bottom}); border-radius: 20px 20px 0 0; padding: 30px 32px 32px; text-align: center;">
        <img src="{logo}" width="100" height="41" alt="Faszienbehandlung Thilo Seifried" style="display:block;width:100px;height:41px;margin:0 auto 20px;border:0;">
        <h1 style="font-family: Georgia, 'Times New Roman', serif; color: #ffffff; font-size: 27px; margin: 0; font-weight: 600; letter-spacing: -0.01em; line-height: 1.25;">{heading}</h1>
    </div>
    <div style="background-color:#ffffff; padding: 36px 32px 8px;">
        {body}
    </div>
    <div style="background-color:#ffffff; border-radius: 0 0 20px 20px; padding: 0 32px 30px;">
        <div style="border-top: 1px solid rgba(112,174,205,0.18); padding-top: 18px; text-align: center;">
            <p style="color: #6a8fa0; font-size: 12px; margin: 0 0 4px; line-height: 1.6;">Faszienbehandlung Thilo Seifried</p>
            <p style="color: #6a8fa0; font-size: 12px; margin: 0; line-height: 1.6;"><a href="https://faszienbehandlung.jetzt/impressum" style="color:#3d5a6b;text-decoration:underline;">Kontakt und aktuelle Anbieterangaben</a></p>
        </div>
    </div>
</div>"#,
        top = top, bottom = bottom, heading = escape_html(heading), preheader = escape_html(preheader), body = body, logo = LOGO_URL
    )
}

/// One label/value row for an info_card table. Value may contain inline HTML (e.g. a styled span).
fn info_row(label: &str, value: &str) -> String {
    format!(
        r#"<tr>
            <td style="padding:8px 0;color:#6a8fa0;font-size:13px;font-weight:600;letter-spacing:0.01em;white-space:nowrap;vertical-align:top;">{label}</td>
            <td style="padding:8px 0 8px 18px;color:#1a2a33;font-size:15px;">{value}</td>
        </tr>"#,
        label = label, value = value
    )
}

/// Card wrapper for a set of info_row()s: a flat top accent (never a colored side
/// border) marks it as the surface's one structured data block. Table-based for
/// cross-client layout safety.
fn info_card(rows: &str) -> String {
    format!(
        r#"<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:#ffffff;border:1px solid rgba(112,174,205,0.22);border-top:3px solid {sky_to};border-radius:12px;margin:22px 0;">
            <tr><td style="padding:6px 20px;">
                <table role="presentation" width="100%" cellpadding="0" cellspacing="0">{rows}</table>
            </td></tr>
        </table>"#,
        sky_to = SKY_TO, rows = rows
    )
}

fn cta_button(url: &str, label: &str) -> String {
    format!(
        r#"<div style="text-align: center; margin: 34px 0 8px;">
            <a href="{url}" style="display: inline-block; background-color: #964279; color: #ffffff; padding: 15px 40px; border-radius: 50px; text-decoration: none; font-weight: 600; font-size: 15px; letter-spacing: 0.01em;">{label}</a>
        </div>"#,
        url = escape_html(url), label = escape_html(label)
    )
}


fn appointment_confirmation_html(name: &str, date: &str, time: &str, is_home_visit: bool, duration_min: i64) -> String {
        let mut rows = String::new();
        rows.push_str(&info_row("Datum", &escape_html(date)));
        rows.push_str(&info_row("Uhrzeit", &escape_html(time)));
        rows.push_str(&info_row("Dauer", &appointment_duration_text(duration_min)));
        if is_home_visit {
            rows.push_str(&info_row(
                "Hinweis",
                r#"<span style="color:#964279;font-weight:600;">Hausbesuch – es gelten die vereinbarten Konditionen deiner Buchung</span>"#,
            ));
        }
        let preheader = format!("Dein Termin am {} um {} Uhr ist best\u{e4}tigt.", date, time);
        let body = shell(
            SKY_FROM, SKY_TO, "Termin bestätigt", &preheader,
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; margin: 0 0 12px;">Hallo {},</p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Dein Termin f&uuml;r die Gantke&reg; Faszienbehandlung wurde best&auml;tigt:
                </p>
                {}
                <p style="color: #6a8fa0; font-size: 13px; margin: 0;">
                    F&uuml;r Absagen und &Auml;nderungen erreichst du mich telefonisch oder per E-Mail. Mit einem eingerichteten Kundenkonto kannst du Termine auch im Portal verwalten.
                </p>"#,
                escape_html(name), info_card(&rows)
            ),
        );

        body
}

#[derive(Clone)]
pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
    from_address: String,
    from_name: String,
}

impl EmailService {
    pub fn send_text(&self, recipient: &str, subject: &str, text: &str) -> Result<(), String> {
        let message = Message::builder()
            .from(Mailbox::new(Some(self.from_name.clone()), self.from_address.parse().map_err(|_| "Ungültiger Absender")?))
            .to(recipient.parse().map_err(|_| "Ungültiger Empfänger")?)
            .subject(subject).header(ContentType::TEXT_PLAIN).body(text.to_owned()).map_err(|_| "E-Mail konnte nicht erstellt werden")?;
        let transport = SmtpTransport::relay(&self.smtp_host).map_err(|_| "SMTP-Verbindung fehlgeschlagen")?
            .port(self.smtp_port).credentials(Credentials::new(self.smtp_user.clone(), self.smtp_pass.clone()))
            .timeout(Some(std::time::Duration::from_secs(15))).build();
        transport.send(&message).map_err(|_| "SMTP-Versand fehlgeschlagen")?;
        Ok(())
    }
    pub fn send_guest_booking_link(&self, to_email: &str, name: &str, date: &str, time: &str, url: &str) -> Result<(), String> {
        let escape = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
        let body = shell(SKY_FROM, SKY_TO, "Deine Buchung bestätigen", "Ein Schritt fehlt noch: bestätige deinen Termin.",
            &format!(r#"<p style="color:#1a2a33;font-size:16px;line-height:1.7;">Hallo {},</p>
            <p style="color:#3d5a6b;font-size:15px;line-height:1.7;">du möchtest am <strong>{} um {} Uhr</strong> eine Sitzung buchen. Öffne den folgenden Link und bestätige deine Buchung dort innerhalb von 30 Minuten.</p>
            <p style="color:#3d5a6b;font-size:15px;line-height:1.7;">Bis zur Bestätigung ist der Termin noch nicht reserviert. Es ist keine Registrierung erforderlich.</p>{}
            <p style="color:#3d5a6b;font-size:13px;line-height:1.7;">Falls du keine Buchung angefragt hast, ignoriere diese E-Mail. Es wird dann kein Termin gebucht.</p>"#, escape(name), escape(date), escape(time), cta_button(url, "Buchung prüfen")));
        self.send_html(to_email, "", "Deinen Termin bei Thilo Seifried bestätigen", &body)
    }
    pub fn new(config: &Config) -> Self {
        EmailService {
            smtp_host: config.smtp_host.clone(),
            smtp_port: config.smtp_port,
            smtp_user: config.smtp_user.clone(),
            smtp_pass: config.smtp_pass.clone(),
            from_address: config.smtp_from.clone(),
            from_name: config.smtp_from_name.clone(),
        }
    }

    fn send_html(&self, to_email: &str, to_name: &str, subject: &str, html_body: &str) -> Result<(), String> {
        let from_mailbox = Mailbox::new(Some(self.from_name.clone()),self.from_address.parse().map_err(|_| "Invalid from address")?);
        let to_mailbox = Mailbox::new(if to_name.is_empty() { None } else { Some(to_name.to_owned()) },to_email.parse().map_err(|_| "Invalid to address")?);

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
            .map_err(|e| format!("Email build error: {}", e))?;

        let creds = Credentials::new(self.smtp_user.clone(), self.smtp_pass.clone());
        let mailer = SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| format!("SMTP relay error: {}", e))?
            .port(self.smtp_port)
            .credentials(creds)
            .timeout(Some(std::time::Duration::from_secs(15)))
            .build();

        mailer.send(&email).map_err(|e| format!("SMTP send error: {}", e))?;
        info!("Transactional email sent");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn send_welcome(&self, to_email: &str, first_name: &str) {
        let display = if first_name.is_empty() { "Neuer Kunde" } else { first_name };
        let subject = "Willkommen bei Faszienbehandlung Thilo Seifried";
        let intro = format!(
            r#"<p style="color: #1a2a33; font-size: 16px; line-height: 1.7; margin: 0 0 16px;">
                Vielen Dank f&uuml;r deine Registrierung bei Faszienbehandlung Thilo Seifried.
            </p>
            <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                Du kannst jetzt ganz einfach online Termine buchen, verwalten und deine Behandlungshistorie einsehen.
            </p>
            {}"#,
            cta_button("https://faszienbehandlung.jetzt/portal", "Zum Kundenportal")
        );
        let body = shell(
            SKY_FROM, SKY_TO, &format!("Willkommen, {}!", display),
            "Dein Kundenportal ist bereit &ndash; buche jetzt deinen ersten Termin.",
            &intro,
        );

        if let Err(e) = self.send_html(to_email, first_name, subject, &body) {
            error!("Failed to send welcome email: {}", e);
        }
    }

    pub fn send_appointment_confirmation(&self, to_email: &str, name: &str, date: &str, time: &str, is_home_visit: bool, duration_min: i64) {
        let subject = format!("Terminbestätigung – {}", date);
        let body = appointment_confirmation_html(name,date,time,is_home_visit,duration_min);

        if let Err(e) = self.send_html(to_email, name, &subject, &body) {
            error!("Failed to send appointment confirmation: {}", e);
        }
    }

    pub fn send_appointment_cancellation(&self, to_email: &str, name: &str, date: &str, time: &str) {
        let subject = format!("Termin storniert \u{2013} {}", date);
        let preheader = format!("Dein Termin am {} wurde storniert.", date);
        let body = shell(
            PLUM_FROM, PLUM_TO, "Termin storniert", &preheader,
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; margin: 0 0 12px;">Hallo {},</p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Dein Termin am {} um {} wurde erfolgreich storniert.
                </p>
                {}"#,
                escape_html(name), escape_html(date), escape_html(time),
                cta_button("https://faszienbehandlung.jetzt/portal/book", "Neuen Termin buchen")
            ),
        );

        if let Err(e) = self.send_html(to_email, name, &subject, &body) {
            error!("Failed to send cancellation email: {}", e);
        }
    }

    pub fn send_admin_cancellation_with_suggestions(&self, to_email: &str, name: &str, date: &str, time: &str, slots_html: &str) {
        let subject = format!("Wichtige Information zu deinem Termin am {}", date);
        let body = shell(
            PLUM_FROM, PLUM_TO, "Termin\u{e4}nderung n\u{f6}tig",
            "Ich muss unseren Termin leider verschieben \u{2013} hier alternative Vorschläge.",
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; margin: 0 0 12px;">Hallo {},</p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0 0 12px;">
                    Leider muss ich unseren Termin am <strong>{} um {}</strong> kurzfristig absagen.
                </p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Hier sind alternative Terminvorschläge, die noch nicht reserviert sind. Bitte lass mich wissen, ob einer davon f&uuml;r dich in Frage kommt, oder ob wir einen anderen Termin finden sollen:
                </p>
                <div style="margin: 20px 0;">{}</div>
                {}"#,
                escape_html(name), escape_html(date), escape_html(time), slots_html,
                cta_button("https://faszienbehandlung.jetzt/portal/book", "Alternativen Termin buchen")
            ),
        );

        if let Err(e) = self.send_html(to_email, name, &subject, &body) {
            error!("Failed to send cancellation with suggestions email: {}", e);
        }
    }

    pub fn send_password_reset(&self, to_email: &str, name: &str, reset_url: &str) {
        let subject = "Passwort zur\u{fc}cksetzen \u{2013} Faszienbehandlung";
        let body = shell(
            SKY_FROM, SKY_TO, "Passwort zur\u{fc}cksetzen",
            "Setze dein Passwort mit einem Klick zur\u{fc}ck.",
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; margin: 0 0 12px;">Hallo {},</p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Klicke auf den folgenden Link, um dein Passwort zur&uuml;ckzusetzen. Der Link ist 1 Stunde g&uuml;ltig.
                </p>
                {}
                <p style="color: #6a8fa0; font-size: 13px; margin: 0;">
                    Falls du diese Anfrage nicht gestellt hast, ignoriere diese E-Mail.
                </p>"#,
                escape_html(name), cta_button(reset_url, "Passwort zur\u{fc}cksetzen")
            ),
        );

        if let Err(e) = self.send_html(to_email, name, subject, &body) {
            error!("Failed to send password reset email: {}", e);
        }
    }

    /// Sent to customer after registration so they can confirm their e-mail address.
    pub fn send_email_verification(&self, to_email: &str, name: &str, verify_url: &str) {
        let display = if name.is_empty() { "Neuer Kunde" } else { name };
        let subject = "E-Mail best\u{e4}tigen \u{2013} Faszienbehandlung Thilo Seifried";
        let body = shell(
            SKY_FROM, SKY_TO, &format!("Willkommen, {}!", display),
            "Bitte best\u{e4}tige deine E-Mail-Adresse, um dein Konto zu aktivieren.",
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; line-height: 1.7; margin: 0 0 16px;">
                    Vielen Dank f&uuml;r deine Registrierung bei Faszienbehandlung Thilo Seifried.
                </p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Bitte best&auml;tige deine E-Mail-Adresse mit einem Klick auf den Button. Der Link ist <strong>24 Stunden</strong> g&uuml;ltig.
                </p>
                {}
                <p style="color: #6a8fa0; font-size: 13px; text-align: center; line-height: 1.6; margin: 0;">
                    Falls du diese Registrierung nicht vorgenommen hast, ignoriere diese E-Mail.<br>
                    Bei Fragen findest du die aktuellen Kontaktwege in unseren Anbieterangaben.
                </p>"#,
                cta_button(verify_url, "E-Mail-Adresse best\u{e4}tigen")
            ),
        );

        if let Err(e) = self.send_html(to_email, name, subject, &body) {
            error!("Failed to send email verification: {}", e);
        }
    }

    /// Sent to the admin email when a customer books a new appointment.
    #[allow(clippy::too_many_arguments)]
    pub fn send_admin_booking_notification(
        &self,
        admin_email: &str,
        customer_name: &str,
        customer_phone: &str,
        customer_address: &str,
        date_str: &str,
        time_str: &str,
        notes: &str,
        is_home_visit: bool,
    ) {
        let visit_label = if is_home_visit { "Hausbesuch" } else { "Praxis" };
        let mut rows = String::new();
        rows.push_str(&info_row("Kunde", &escape_html(customer_name)));
        rows.push_str(&info_row("Telefon", &escape_html(customer_phone)));
        rows.push_str(&info_row("Adresse", &escape_html(customer_address)));
        rows.push_str(&info_row("Datum", &escape_html(date_str)));
        rows.push_str(&info_row("Uhrzeit", &escape_html(time_str)));
        rows.push_str(&info_row("Art", visit_label));
        if !notes.is_empty() {
            rows.push_str(&info_row("Notiz", &escape_html(notes)));
        }
        let subject = format!("Neue Buchung: {} am {}", customer_name, date_str);
        let preheader = format!("{} hat einen Termin am {} gebucht.", customer_name, date_str);
        let body = shell(
            SKY_FROM, SKY_TO, "Neue Terminbuchung", &preheader,
            &format!(
                r#"<p style="color: #3d5a6b; font-size: 15px; margin: 0;">Ein Kunde hat soeben einen Termin gebucht:</p>
                {}
                {}"#,
                info_card(&rows),
                cta_button("https://faszienbehandlung.jetzt/admin", "Admin-Bereich \u{f6}ffnen")
            ),
        );

        if let Err(e) = self.send_html(admin_email, "", &subject, &body) {
            error!("Failed to send admin booking notification: {}", e);
        }
    }

    /// 24h reminder sent to customer before appointment.
    pub fn send_appointment_reminder(&self, to_email: &str, name: &str, date: &str, time: &str, is_home_visit: bool, duration_min: i64) {
        let location = if is_home_visit {
            "Hausbesuch \u{2013} Thilo kommt zu dir".to_string()
        } else {
            "Praxis – die aktuelle Anschrift findest du in den Anbieterangaben".to_string()
        };
        let mut rows = String::new();
        rows.push_str(&info_row("Datum", &escape_html(date)));
        rows.push_str(&info_row("Uhrzeit", &escape_html(time)));
        rows.push_str(&info_row("Ort", &location));
        rows.push_str(&info_row("Dauer", &appointment_duration_text(duration_min)));
        let subject = format!("Erinnerung: Dein Termin morgen um {}", time);
        let preheader = format!("Dein Termin ist morgen um {} Uhr.", time);
        let body = shell(
            SKY_FROM, SKY_TO, "Erinnerung: Termin morgen", &preheader,
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; margin: 0 0 12px;">Hallo {},</p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Dein Termin bei Faszienbehandlung Thilo Seifried ist morgen:
                </p>
                {}
                <p style="color: #6a8fa0; font-size: 13px; margin: 0;">
                    Trage am Behandlungstag bitte bequeme Kleidung.
                </p>"#,
                escape_html(name), info_card(&rows)
            ),
        );

        if let Err(e) = self.send_html(to_email, name, &subject, &body) {
            error!("Failed to send appointment reminder: {}", e);
        }
    }

    /// Notify waitlisted customer that a slot opened up.
    pub fn send_waitlist_notification(&self, to_email: &str, name: &str, date: &str) {
        let subject = format!("Ein Termin am {} ist frei geworden!", date);
        let preheader = format!("Ein Termin am {} ist gerade frei geworden.", date);
        let body = shell(
            SKY_FROM, SKY_TO, "Ein Termin ist frei!", &preheader,
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; margin: 0 0 12px;">Hallo {},</p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0 0 12px;">
                    Du stehst auf der Warteliste f&uuml;r den <strong>{}</strong> &ndash; und es ist gerade ein Termin freigeworden!
                </p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Bitte schnell sein &ndash; Termine werden nach dem Erstgebucht-Prinzip vergeben:
                </p>
                {}"#,
                escape_html(name), escape_html(date),
                cta_button("https://faszienbehandlung.jetzt/portal/book", "Jetzt Termin buchen")
            ),
        );

        if let Err(e) = self.send_html(to_email, name, &subject, &body) {
            error!("Failed to send waitlist notification: {}", e);
        }
    }

    pub fn send_appointment_suggestion(&self, to_email: &str, name: &str, slots_html: &str) {
        let subject = "Terminvorschl\u{e4}ge \u{2013} Faszienbehandlung Thilo Seifried";
        let body = shell(
            SKY_FROM, SKY_TO, "Terminvorschl\u{e4}ge f\u{fc}r dich",
            "Unverbindliche Terminvorschläge für dich – die Verfügbarkeit prüfst du im Kalender.",
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; margin: 0 0 12px;">Hallo {},</p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Die folgenden Termine schlage ich dir vor. Sie sind noch nicht reserviert. Prüfe die Verfügbarkeit im Kalender und buche einen passenden Termin über dein Kundenportal:
                </p>
                <div style="margin: 20px 0;">{}</div>
                {}"#,
                escape_html(name), slots_html,
                cta_button("https://faszienbehandlung.jetzt/portal/book", "Termin buchen")
            ),
        );

        if let Err(e) = self.send_html(to_email, name, subject, &body) {
            error!("Failed to send appointment suggestion: {}", e);
        }
    }
}

/// Queue with bounded batches and a lease, safe across server processes. SMTP is
/// at-least-once: a crash after delivery can repeat a receipt, never a booking.
pub fn enqueue_text(conn: &rusqlite::Connection, recipient: &str, subject: &str, body: &str) -> rusqlite::Result<()> {
    conn.execute("INSERT INTO email_outbox(recipient,subject,body) VALUES(?1,?2,?3)", rusqlite::params![recipient,subject,body])?;
    Ok(())
}

pub async fn dispatch_outbox(db: actix_web::web::Data<std::sync::Mutex<rusqlite::Connection>>, service: EmailService) {
    let _ = tokio::task::spawn_blocking(move || {
        for _ in 0..10 {
            let queued = {
                let conn = db.lock().unwrap_or_else(|e| e.into_inner());
                let Ok(tx) = rusqlite::Transaction::new_unchecked(&conn,rusqlite::TransactionBehavior::Immediate) else { return; };
                use rusqlite::OptionalExtension;
                let row = tx.query_row("SELECT id,recipient,subject,body FROM email_outbox WHERE sent_at IS NULL AND (claimed_until IS NULL OR claimed_until < datetime('now')) ORDER BY id LIMIT 1",[],|r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))).optional();
                let Ok(Some(row)) = row else { return; };
                if tx.execute("UPDATE email_outbox SET claimed_until=datetime('now','+5 minutes'),attempts=attempts+1 WHERE id=?1",[row.0]).is_err() || tx.commit().is_err() { return; }
                row
            };
            let sent = service.send_text(&queued.1,&queued.2,&queued.3).is_ok();
            let conn = db.lock().unwrap_or_else(|e| e.into_inner());
            if sent {
                let _ = conn.execute("UPDATE email_outbox SET sent_at=datetime('now'),recipient='',subject='',body='',claimed_until=NULL WHERE id=?1",[queued.0]);
            } else { error!("Outbox delivery failed for message {} (retry queued)",queued.0); }
        }
    }).await;
}

#[cfg(test)]
mod notice_tests {
    use super::*;

    #[test]
    fn rendered_confirmation_uses_actual_duration_and_escapes_contact_data() {
        let name = "Änne <img src=x onerror=alert(1)> & Familie";
        let date = "09.10.2026 <script>alert(2)</script>";
        let html = appointment_confirmation_html(name,date,"16:30",true,45);
        assert!(html.contains("ca. 45 Minuten"));
        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
        assert!(!html.contains("<img src=x") && !html.contains("<script>"));
        assert!(!html.contains("90 Minuten") && !html.contains("+15"));
        assert!(!appointment_confirmation_html("Test","09.10.2026","16:30",false,0).contains("ca. 0"));
        let button = cta_button("https://example.test/?a=1&next=\" onmouseover=\"alert(3)","<b>Prüfen</b>");
        assert!(button.contains("&quot;"));
        assert!(!button.contains("href=\"https://example.test/?a=1&next=\""));
        assert!(button.contains("&lt;b&gt;Prüfen&lt;/b&gt;"));
    }

    #[test]
    fn suggestion_input_is_a_bounded_datetime_list_not_html() {
        let rows = suggestion_rows(&["2026-11-02T16:30".into(),"2026-11-03 17:00:00".into()]).unwrap();
        assert!(rows.contains("02.11.2026 16:30 Uhr"));
        assert!(suggestion_rows(&["<img src=x onerror=alert(1)>".into()]).is_err());
        assert!(suggestion_rows(&["2026-02-30T16:00".into()]).is_err());
        assert!(suggestion_rows(&vec!["2026-11-02T16:30".into();9]).is_err());
        assert_eq!(suggestion_rows(&[]).unwrap(),"");
    }
}
