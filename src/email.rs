use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
};
use log::{info, error};
use crate::config::Config;

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

    pub fn send_welcome(&self, to_email: &str, first_name: &str) {
        let subject = "Willkommen bei Faszienbehandlung Thilo Seifried";
        let body = format!(r#"
            <div style="font-family: 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; background: #f7fbfd; border-radius: 16px; overflow: hidden;">
                <div style="background: linear-gradient(135deg, #70AECD, #4d93b8); padding: 32px; text-align: center;">
                    <h1 style="color: white; font-size: 24px; margin: 0;">Willkommen, {}!</h1>
                </div>
                <div style="padding: 32px;">
                    <p style="color: #1a2a33; font-size: 16px; line-height: 1.7;">
                        Vielen Dank für deine Registrierung bei Faszienbehandlung Thilo Seifried.
                    </p>
                    <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7;">
                        Du kannst jetzt ganz einfach online Termine buchen, verwalten und deine Behandlungshistorie einsehen.
                    </p>
                    <div style="text-align: center; margin: 32px 0;">
                        <a href="https://faszien-behandlung.jetzt/portal" style="display: inline-block; background: #964279; color: white; padding: 14px 36px; border-radius: 50px; text-decoration: none; font-weight: 600; font-size: 15px;">Zum Kundenportal</a>
                    </div>
                    <p style="color: #6a8fa0; font-size: 13px; text-align: center;">
                        Bei Fragen erreichst du uns unter +49 152 34 00 72 25
                    </p>
                </div>
            </div>
        "#, if first_name.is_empty() { "Neuer Kunde" } else { first_name });

        if let Err(e) = self.send_html(to_email, first_name, subject, &body) {
            error!("Failed to send welcome email: {}", e);
        }
    }

    pub fn send_appointment_confirmation(&self, to_email: &str, name: &str, date: &str, time: &str, is_home_visit: bool) {
        let visit_note = if is_home_visit { "<p style='color: #964279;'>🏠 Hausbesuch – Fahrtpauschale: +15 €</p>" } else { "" };
        let subject = format!("Terminbestätigung – {}", date);
        let body = format!(r#"
            <div style="font-family: 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; background: #f7fbfd; border-radius: 16px; overflow: hidden;">
                <div style="background: linear-gradient(135deg, #70AECD, #4d93b8); padding: 32px; text-align: center;">
                    <h1 style="color: white; font-size: 22px; margin: 0;">Termin bestätigt ✓</h1>
                </div>
                <div style="padding: 32px;">
                    <p style="color: #1a2a33; font-size: 16px;">Hallo {},</p>
                    <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7;">
                        Dein Termin für die Gantke® Faszienbehandlung wurde bestätigt:
                    </p>
                    <div style="background: white; border: 1px solid rgba(112,174,205,0.2); border-radius: 12px; padding: 20px; margin: 20px 0;">
                        <p style="margin: 4px 0; color: #1a2a33;"><strong>📅 Datum:</strong> {}</p>
                        <p style="margin: 4px 0; color: #1a2a33;"><strong>🕐 Uhrzeit:</strong> {}</p>
                        <p style="margin: 4px 0; color: #1a2a33;"><strong>⏱ Dauer:</strong> ca. 90 Minuten</p>
                        {}
                    </div>
                    <p style="color: #6a8fa0; font-size: 13px;">
                        Stornierung bis 24 Stunden vor dem Termin möglich über dein Kundenportal.
                    </p>
                </div>
            </div>
        "#, name, date, time, visit_note);

        if let Err(e) = self.send_html(to_email, name, &subject, &body) {
            error!("Failed to send appointment confirmation: {}", e);
        }
    }

    pub fn send_appointment_cancellation(&self, to_email: &str, name: &str, date: &str, time: &str) {
        let subject = format!("Termin storniert – {}", date);
        let body = format!(r#"
            <div style="font-family: 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; background: #f7fbfd; border-radius: 16px; overflow: hidden;">
                <div style="background: linear-gradient(135deg, #964279, #7a3661); padding: 32px; text-align: center;">
                    <h1 style="color: white; font-size: 22px; margin: 0;">Termin storniert</h1>
                </div>
                <div style="padding: 32px;">
                    <p style="color: #1a2a33; font-size: 16px;">Hallo {},</p>
                    <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7;">
                        Dein Termin am {} um {} wurde erfolgreich storniert.
                    </p>
                    <div style="text-align: center; margin: 32px 0;">
                        <a href="https://faszien-behandlung.jetzt/portal/book" style="display: inline-block; background: #964279; color: white; padding: 14px 36px; border-radius: 50px; text-decoration: none; font-weight: 600;">Neuen Termin buchen</a>
                    </div>
                </div>
            </div>
        "#, name, date, time);

        if let Err(e) = self.send_html(to_email, name, &subject, &body) {
            error!("Failed to send cancellation email: {}", e);
        }
    }

    pub fn send_admin_cancellation_with_suggestions(&self, to_email: &str, name: &str, date: &str, time: &str, slots_html: &str) {
        let subject = format!("Wichtige Information zu deinem Termin am {}", date);
        let body = format!(r#"
            <div style="font-family: 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; background: #f7fbfd; border-radius: 16px; overflow: hidden;">
                <div style="background: linear-gradient(135deg, #964279, #7a3661); padding: 32px; text-align: center;">
                    <h1 style="color: white; font-size: 22px; margin: 0;">Terminänderung nötig</h1>
                </div>
                <div style="padding: 32px;">
                    <p style="color: #1a2a33; font-size: 16px;">Hallo {},</p>
                    <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7;">
                        Leider muss ich unseren Termin am <strong>{} um {}</strong> kurzfristig absagen.
                    </p>
                    <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7;">
                        Hier sind zwei alternative Termine, die für mich besser passen würden. Bitte lass mich wissen, ob einer davon für dich in Frage kommt, oder ob wir einen anderen Termin finden sollen:
                    </p>
                    <div style="margin: 24px 0;">
                        {}
                    </div>
                    <div style="text-align: center; margin: 32px 0;">
                        <a href="https://faszien-behandlung.jetzt/portal/book" style="display: inline-block; background: #964279; color: white; padding: 14px 36px; border-radius: 50px; text-decoration: none; font-weight: 600;">Alternativen Termin buchen</a>
                    </div>
                </div>
            </div>
        "#, name, date, time, slots_html);

        if let Err(e) = self.send_html(to_email, name, &subject, &body) {
            error!("Failed to send cancellation with suggestions email: {}", e);
        }
    }


    pub fn send_password_reset(&self, to_email: &str, name: &str, reset_url: &str) {
        let subject = "Passwort zurücksetzen – Faszienbehandlung";
        let body = format!(r#"
            <div style="font-family: 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; background: #f7fbfd; border-radius: 16px; overflow: hidden;">
                <div style="background: linear-gradient(135deg, #70AECD, #4d93b8); padding: 32px; text-align: center;">
                    <h1 style="color: white; font-size: 22px; margin: 0;">Passwort zurücksetzen</h1>
                </div>
                <div style="padding: 32px;">
                    <p style="color: #1a2a33; font-size: 16px;">Hallo {},</p>
                    <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7;">
                        Klicke auf den folgenden Link, um dein Passwort zurückzusetzen. Der Link ist 1 Stunde gültig.
                    </p>
                    <div style="text-align: center; margin: 32px 0;">
                        <a href="{}" style="display: inline-block; background: #964279; color: white; padding: 14px 36px; border-radius: 50px; text-decoration: none; font-weight: 600;">Passwort zurücksetzen</a>
                    </div>
                    <p style="color: #6a8fa0; font-size: 13px;">
                        Falls du diese Anfrage nicht gestellt hast, ignoriere diese E-Mail.
                    </p>
                </div>
            </div>
        "#, name, reset_url);

        if let Err(e) = self.send_html(to_email, name, subject, &body) {
            error!("Failed to send password reset email: {}", e);
        }
    }

    pub fn send_appointment_suggestion(&self, to_email: &str, name: &str, slots_html: &str) {
        let subject = "Terminvorschläge – Faszienbehandlung Thilo Seifried";
        let body = format!(r#"
            <div style="font-family: 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; background: #f7fbfd; border-radius: 16px; overflow: hidden;">
                <div style="background: linear-gradient(135deg, #70AECD, #4d93b8); padding: 32px; text-align: center;">
                    <h1 style="color: white; font-size: 22px; margin: 0;">Terminvorschläge für dich</h1>
                </div>
                <div style="padding: 32px;">
                    <p style="color: #1a2a33; font-size: 16px;">Hallo {},</p>
                    <p style="color: #3d5a6b; font-size: 15px; line-height: 1.7;">
                        Ich habe folgende Termine für dich reserviert. Bitte buche einen davon über dein Kundenportal:
                    </p>
                    <div style="background: white; border: 1px solid rgba(112,174,205,0.2); border-radius: 12px; padding: 20px; margin: 20px 0;">
                        {}
                    </div>
                    <div style="text-align: center; margin: 32px 0;">
                        <a href="https://faszien-behandlung.jetzt/portal/book" style="display: inline-block; background: #964279; color: white; padding: 14px 36px; border-radius: 50px; text-decoration: none; font-weight: 600;">Termin buchen</a>
                    </div>
                </div>
            </div>
        "#, name, slots_html);

        if let Err(e) = self.send_html(to_email, name, subject, &body) {
            error!("Failed to send appointment suggestion: {}", e);
        }
    }
}
