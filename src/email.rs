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
            <p style="color: #6a8fa0; font-size: 12px; margin: 0 0 4px; line-height: 1.6;">Faszienbehandlung Thilo Seifried &middot; Sulgauer Stra&szlig;e 24, 78713 Schramberg-Sulgen</p>
            <p style="color: #6a8fa0; font-size: 12px; margin: 0; line-height: 1.6;">Tel. +49 152 34 00 72 25 &middot; termin@faszienbehandlung.jetzt</p>
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
        url = url, label = label
    )
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

pub struct BookingDetails {
    pub name: String,
    pub date: String,
    pub time: String,
    pub location: String,
    pub duration: i64,
    pub cancellation_hours: i64,
    pub amount: f64,
    pub phone: String,
    pub contact_email: String,
}

impl EmailService {
    pub fn send_booking_confirmation(&self, to: &str, details: &BookingDetails) {
        let rows = info_row("Datum", &escape_html(&details.date))
            + &info_row("Uhrzeit", &escape_html(&details.time))
            + &info_row("Ort", &escape_html(&details.location))
            + &info_row("Dauer", &format!("ca. {} Minuten", details.duration))
            + &info_row("Preis", &format!("{} €", format!("{:.2}", details.amount).replace('.', ",")));
        let body = shell(SKY_FROM, SKY_TO, "Dein Termin ist gebucht", "Datum, Ort und Preis deiner bestätigten Buchung.",
            &format!(r#"<p style="color:#1a2a33;font-size:16px;line-height:1.7;">Hallo {},</p>
            <p style="color:#3d5a6b;font-size:15px;line-height:1.7;">dein Termin für die Gantke&reg; Faszienbehandlung ist verbindlich gebucht.</p>{}
            <p style="color:#3d5a6b;font-size:15px;line-height:1.7;">Zahlung bar oder per Überweisung. Keine Erstattung durch Krankenkassen oder Versicherungen; du erhältst eine Rechnung.</p>
            <p style="color:#3d5a6b;font-size:15px;line-height:1.7;">Für Änderungen oder eine Absage bis {} Stunden vor dem Termin erreichst du mich unter {} oder {}. Ein eingerichtetes Kundenkonto kannst du zusätzlich zur Terminverwaltung nutzen.</p>"#,
            escape_html(&details.name), info_card(&rows), details.cancellation_hours, escape_html(&details.phone), escape_html(&details.contact_email)));
        if let Err(error) = self.send_html(to, "", &format!("Terminbestätigung – {}", details.date), &body) { error!("Failed to send booking confirmation: {}", error); }
    }
    pub fn send_guest_booking_link(&self, to_email: &str, name: &str, date: &str, time: &str, url: &str) -> Result<(), String> {
        let escape = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
        let body = shell(SKY_FROM, SKY_TO, "Deine Buchung bestätigen", "Ein Schritt fehlt noch: bestätige deinen Termin.",
            &format!(r#"<p style="color:#1a2a33;font-size:16px;line-height:1.7;">Hallo {},</p>
            <p style="color:#3d5a6b;font-size:15px;line-height:1.7;">du möchtest am <strong>{} um {} Uhr</strong> eine Sitzung buchen. Öffne den folgenden Link und bestätige deine Buchung dort innerhalb von 30 Minuten.</p>
            <p style="color:#3d5a6b;font-size:15px;line-height:1.7;">Bis zur Bestätigung ist der Termin noch nicht reserviert. Es ist keine Registrierung erforderlich.</p>{}
            <p style="color:#3d5a6b;font-size:13px;line-height:1.7;">Falls du keine Buchung angefragt hast, ignoriere diese E-Mail. Es wird dann kein Termin gebucht.</p>"#, escape(name), escape(date), escape(time), cta_button(&escape(url), "Buchung prüfen")));
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
        let from_mailbox: Mailbox = format!("{} <{}>", self.from_name, self.from_address)
            .parse()
            .map_err(|e| format!("Invalid from address: {}", e))?;
        let to_mailbox: Mailbox = if to_name.is_empty() {
            to_email.parse().map_err(|e| format!("Invalid to address: {}", e))?
        } else {
            format!("{} <{}>", to_name, to_email)
                .parse()
                .map_err(|e| format!("Invalid to address: {}", e))?
        };

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
            .build();

        mailer.send(&email).map_err(|e| format!("SMTP send error: {}", e))?;
        info!("Email sent to {} - Subject: {}", to_email, subject);
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

    pub fn send_appointment_confirmation(&self, to_email: &str, name: &str, date: &str, time: &str, is_home_visit: bool) {
        let mut rows = String::new();
        rows.push_str(&info_row("Datum", date));
        rows.push_str(&info_row("Uhrzeit", time));
        rows.push_str(&info_row("Dauer", "ca. 90 Minuten"));
        if is_home_visit {
            rows.push_str(&info_row(
                "Hinweis",
                r#"<span style="color:#964279;font-weight:600;">Hausbesuch &ndash; Fahrtpauschale: +15&nbsp;&euro;</span>"#,
            ));
        }
        let subject = format!("Terminbest\u{e4}tigung \u{2013} {}", date);
        let preheader = format!("Dein Termin am {} um {} Uhr ist best\u{e4}tigt.", date, time);
        let body = shell(
            SKY_FROM, SKY_TO, "Termin best\u{e4}tigt \u{2713}", &preheader,
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
                name, date, time,
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
            "Ich muss unseren Termin leider verschieben \u{2013} hier zwei Alternativen.",
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; margin: 0 0 12px;">Hallo {},</p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0 0 12px;">
                    Leider muss ich unseren Termin am <strong>{} um {}</strong> kurzfristig absagen.
                </p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Hier sind zwei alternative Termine, die f&uuml;r mich besser passen w&uuml;rden. Bitte lass mich wissen, ob einer davon f&uuml;r dich in Frage kommt, oder ob wir einen anderen Termin finden sollen:
                </p>
                <div style="margin: 20px 0;">{}</div>
                {}"#,
                name, date, time, slots_html,
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
                name, cta_button(reset_url, "Passwort zur\u{fc}cksetzen")
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
                    Bei Fragen erreichst du uns unter +49 152 34 00 72 25
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
        rows.push_str(&info_row("Datum", date_str));
        rows.push_str(&info_row("Uhrzeit", time_str));
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
    pub fn send_appointment_reminder(&self, to_email: &str, name: &str, date: &str, time: &str, is_home_visit: bool) {
        let location = if is_home_visit {
            "Hausbesuch \u{2013} Thilo kommt zu dir".to_string()
        } else {
            "Praxis \u{b7} Sulgauer Stra\u{df}e 24, 78713 Schramberg-Sulgen".to_string()
        };
        let mut rows = String::new();
        rows.push_str(&info_row("Datum", date));
        rows.push_str(&info_row("Uhrzeit", time));
        rows.push_str(&info_row("Ort", &location));
        rows.push_str(&info_row("Dauer", "ca. 90 Minuten"));
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
                    Bitte trinke am Behandlungstag ausreichend Wasser (mind. 1,5 Liter) und trage bequeme Kleidung.
                </p>"#,
                name, info_card(&rows)
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
                name, date,
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
            "Ich habe neue Terminvorschl\u{e4}ge f\u{fc}r dich reserviert.",
            &format!(
                r#"<p style="color: #1a2a33; font-size: 16px; margin: 0 0 12px;">Hallo {},</p>
                <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7; margin: 0;">
                    Ich habe folgende Termine f&uuml;r dich reserviert. Bitte buche einen davon &uuml;ber dein Kundenportal:
                </p>
                <div style="margin: 20px 0;">{}</div>
                {}"#,
                name, slots_html,
                cta_button("https://faszienbehandlung.jetzt/portal/book", "Termin buchen")
            ),
        );

        if let Err(e) = self.send_html(to_email, name, subject, &body) {
            error!("Failed to send appointment suggestion: {}", e);
        }
    }
}
