use rusqlite::{params, Connection, Result};

pub const FAQS: &[(i32, &str, &str)] = &[
    (1, "Was ist die Gantke® Faszienbehandlung genau?", "Die Gantke® Methode ist eine manuelle Form der Faszien- und Körperarbeit, entwickelt von Robert Gantke. Die Sitzung betrachtet den Körper als Ganzes und wird individuell auf dich abgestimmt. Sie ist eine Wellnessleistung und ersetzt keine medizinische Diagnose oder Behandlung."),
    (2, "Für wen ist die Faszienbehandlung geeignet?", "Für Erwachsene, die sich verspannt fühlen und sich Zeit für Beweglichkeit, Entspannung und Wohlbefinden nehmen möchten – etwa nach Bildschirmarbeit, körperlicher Arbeit oder Sport.<br><br>Bei Beschwerden mit Krankheitswert bitte vorher ärztlich abklären. Bei akuten Entzündungen, Fieber, frischen Operationswunden oder Thrombosen findet keine Sitzung statt."),
    (3, "Wie fühlt sich die Behandlung an?", "Der Druck wird auf dein Empfinden abgestimmt. Bitte sage mir jederzeit, wenn sich etwas unangenehm anfühlt; ich passe die Grifftechnik dann an. Ein bestimmtes Empfinden oder Ergebnis kann nicht garantiert werden."),
    (4, "Was sollte ich vor der Behandlung beachten?", "Komme in bequemer Kleidung, zum Beispiel einer lockeren Hose und einem T-Shirt. Wenn du gesundheitliche Fragen hast, kläre sie vorab mit deinem Arzt und sprich mich vor der Sitzung darauf an."),
    (5, "Wie viele Termine sind sinnvoll?", "Das besprechen wir individuell. Du kannst zunächst eine einzelne Sitzung buchen und danach in Ruhe entscheiden, ob du weitere Termine möchtest. Eine feste Anzahl oder ein bestimmtes Ergebnis wird nicht versprochen."),
    (6, "Übernehmen Krankenkassen oder Versicherungen die Kosten?", "Die Behandlung ist eine Wellness- und Gesundheitsförderungsleistung und wird von Krankenkassen und Versicherungen nicht erstattet. Du erhältst selbstverständlich eine Rechnung für deine Unterlagen."),
    (7, "Wo findet die Behandlung statt?", "In der Praxis in Schramberg-Sulgen. Die aktuelle Adresse, Öffnungszeiten und die Route findest du im Abschnitt Anfahrt. Kostenlose Parkplätze befinden sich direkt vor dem Haus. Hausbesuche im bisherigen Einzugsgebiet sind gegen die unter Preise genannte Fahrtpauschale möglich."),
    (8, "Was unterscheidet die Methode von klassischer Massage?", "Anders als eine auf einzelne Muskelbereiche ausgerichtete Massage arbeitet die Gantke® Methode mit dem gesamten Fasziennetz. Daraus folgt kein Versprechen einer besseren oder länger anhaltenden Wirkung."),
    (9, "Brauche ich ein Konto oder ein Rezept?", "Für die Online-Buchung brauchst du kein Konto und kein Rezept. Wähle einen Termin, gib deine Kontaktdaten an und bestätige die Buchung über die E-Mail. Erst danach ist der Termin verbindlich gebucht. Ein bestehendes Guthaben kannst du im Kundenportal nutzen."),
    (10, "Ist das Vorgespräch ein eigener kostenloser Termin?", "Nein. Das persönliche Vorgespräch ist Teil der gebuchten Sitzung und im angegebenen Behandlungspreis enthalten. Bei Fragen vor der Buchung erreichst du mich telefonisch oder per E-Mail."),
];

pub const TOPICS: &[(&str, &str, &str, &[(&str, &str)])] = &[
    ("/faszienbehandlung-ruecken", "Zeit für deinen Rücken", "Gantke® Faszienbehandlung bei Rückenverspannungen in Schramberg-Sulgen. Persönliche Körperarbeit, transparente Preise und Abendtermine.", &[
        ("Nach einem langen Tag", "Langes Sitzen, einseitige Bewegungen oder körperliche Arbeit können mit einem Gefühl von Verspannung einhergehen. Die Sitzung bietet dir Zeit, deinen Körper bewusst wahrzunehmen und dich mit deinen Bewegungsgewohnheiten zu beschäftigen."),
        ("Deine persönliche Sitzung", "Wir beginnen mit einem Gespräch über deine Wünsche und betrachten Haltung und Bewegung. Anschließend folgt die manuelle Gantke® Körperarbeit, deren Druck auf dein Empfinden abgestimmt wird. Zum Abschluss besprechen wir Übungen für deinen Alltag."),
        ("Die Grenzen des Angebots", "Die Faszienbehandlung ist eine Wellnessleistung. Sie dient nicht der Diagnose oder Behandlung von Rückenerkrankungen. Bei Schmerzen, Verletzungen oder anderen Beschwerden mit Krankheitswert lasse die Ursache vorab ärztlich abklären."),
    ]),
    ("/faszienbehandlung-nacken-schulter", "Eine Auszeit für Nacken und Schultern", "Persönliche Gantke® Faszienbehandlung für Menschen mit Verspannungsgefühl in Nacken und Schultern. Praxis in Schramberg-Sulgen.", &[
        ("Zwischen Bildschirm und Alltag", "Wenn sich Nacken und Schultern nach konzentrierter Bildschirmarbeit oder einem fordernden Alltag verspannt anfühlen, kann es gut tun, sich bewusst Zeit für den Körper zu nehmen."),
        ("Der ganze Körper im Blick", "Die Gantke® Methode arbeitet mit dem gesamten Fasziennetz. In einem persönlichen Vorgespräch besprechen wir deine Wünsche; während der manuellen Körperarbeit gibst du Rückmeldung zu deinem Empfinden."),
        ("Bewusst entscheiden", "Eine bestimmte Wirkung oder Veränderung wird nicht garantiert. Bei wiederkehrenden Kopfschmerzen, Migräne oder anderen Beschwerden mit Krankheitswert ist eine ärztliche Abklärung erforderlich. Diese Sitzung ersetzt keine medizinische Versorgung."),
    ]),
    ("/faszienbehandlung-haltung", "Haltung bewusst wahrnehmen", "Gantke® Körperarbeit für Haltung, Beweglichkeit und Körperwahrnehmung in Schramberg-Sulgen. Erfahre mehr über Ablauf und Preis.", &[
        ("Deine Bewegungsgewohnheiten", "Ob am Schreibtisch, im Handwerk oder in der Freizeit: Deine Haltung begleitet dich durch den Tag. In der Sitzung nehmen wir uns Zeit, deine persönlichen Bewegungsgewohnheiten zu betrachten."),
        ("Auf dich abgestimmt", "Die manuelle Faszien- und Körperarbeit wird auf deine Situation und dein Empfinden abgestimmt. Du erhältst individuelle Hinweise und Übungen, die du in deinen Alltag einbauen kannst."),
        ("Ohne Korrekturversprechen", "Das Angebot verspricht keine dauerhafte Haltungskorrektur und behandelt keine Erkrankungen. Bei gesundheitlichen Beschwerden bitte vorab ärztlich abklären, ob diese Wellnessleistung für dich infrage kommt."),
    ]),
    ("/faszienbehandlung-sport", "Zeit für Regeneration", "Gantke® Faszienbehandlung für sportlich aktive Erwachsene in Schramberg-Sulgen. Körperwahrnehmung und eine bewusste Pause vom Training.", &[
        ("Auch die Pause gehört dazu", "Laufen, Radfahren und andere Sportarten fordern den Körper. Diese Sitzung ist eine bewusste Auszeit für sportlich aktive Erwachsene, die sich mit ihrem Verspannungsgefühl und ihrer Beweglichkeit beschäftigen möchten."),
        ("Persönliche Körperarbeit", "Wir sprechen über deine Aktivitäten und Wünsche. Die Gantke® Körperarbeit betrachtet den Körper als Ganzes; Druck und Grifftechniken werden auf dein Empfinden abgestimmt."),
        ("Keine Sportmedizin", "Die Sitzung verspricht weder eine Leistungssteigerung noch eine schnellere Heilung von Verletzungen. Bei einer Verletzung oder Beschwerden mit Krankheitswert wende dich zuerst an einen Arzt."),
    ]),
    ("/preise-und-ablauf", "Zeit, die deinem Körper gehört", "Preis und Ablauf der Gantke® Faszienbehandlung bei Thilo Seifried: persönliche Sitzung, transparente Kosten und keine Kassenerstattung.", &[
        ("Das ist enthalten", "Du buchst eine persönliche Sitzung mit Vorgespräch, Betrachtung von Haltung und Bewegungsgewohnheiten, manueller Gantke® Faszienbehandlung und individuellen Übungen für zu Hause. Es gibt keine parallelen Termine."),
        ("Du entscheidest in Ruhe", "Du kannst zunächst eine einzelne Sitzung buchen. Ob weitere Termine sinnvoll sind, besprechen wir individuell. Eine feste Behandlungszahl und bestimmte Ergebnisse werden nicht versprochen."),
        ("Selbstzahlerleistung", "Die Behandlung ist eine Wellness- und Gesundheitsförderungsleistung und wird von Krankenkassen und Versicherungen nicht erstattet. Du erhältst eine Rechnung. Die Zahlung erfolgt bar oder per Überweisung; es ist keine Vorauszahlung nötig."),
    ]),
];

/// Preserve the original wording for the practice owner; never rewrite quotations.
pub fn migrate_content(conn: &Connection) -> Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS content_migrations (name TEXT PRIMARY KEY); CREATE TABLE IF NOT EXISTS marketing_content_backup (kind TEXT NOT NULL, record_id INTEGER NOT NULL, content TEXT NOT NULL, PRIMARY KEY(kind, record_id));")?;
    let done: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM content_migrations WHERE name='marketing-2026-10')", [], |r| r.get(0))?;
    if done { return Ok(()); }
    let tx = conn.unchecked_transaction()?;
    tx.execute("INSERT OR IGNORE INTO marketing_content_backup SELECT 'faq', id, answer_html FROM faqs", [])?;
    // Only replace the known shipped questions. Owner-authored unrelated FAQs stay intact.
    let legacy = ["Was ist die Gantke® Faszienbehandlung genau?", "Für wen ist die Faszienbehandlung geeignet?", "Ist die Behandlung schmerzhaft?", "Was sollte ich vor der Behandlung beachten?", "Wie viele Termine sind nötig?", "Übernimmt die Krankenkasse die Kosten?", "Wo findet die Behandlung statt – und wie komme ich hin?", "Was ist der Unterschied zur klassischen Massage?"];
    for ((_, question, answer), old_question) in FAQS.iter().take(8).zip(legacy) {
        tx.execute("UPDATE faqs SET question=?1, answer_html=?2, updated_at=datetime('now') WHERE question=?3", params![question, answer, old_question])?;
    }
    for (order, question, answer) in FAQS.iter().skip(8) {
        tx.execute("INSERT INTO faqs(question, answer_html, sort_order) SELECT ?1, ?2, ?3 WHERE NOT EXISTS(SELECT 1 FROM faqs WHERE question=?1)", params![question, answer, order])?;
    }
    tx.execute("UPDATE site_settings SET setting_value='Schramberg-Sulgen' WHERE setting_key='address_city' AND setting_value IN ('Sulgen','Sulgen (Schramberg)')", [])?;
    tx.execute("UPDATE site_settings SET setting_value='78713' WHERE setting_key='address_zip' AND setting_value='78655'", [])?;
    // Existing reviews stay available for review in admin, but require explicit publication approval.
    tx.execute("UPDATE reviews SET is_active=0 WHERE lower(content) LIKE '%schmerz%' OR lower(content) LIKE '%migräne%' OR lower(content) LIKE '%geheilt%'", [])?;
    tx.execute("INSERT INTO content_migrations(name) VALUES('marketing-2026-10')", [])?;
    tx.commit()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_preserves_quotes_and_custom_faqs_and_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE faqs(id INTEGER PRIMARY KEY, question TEXT, answer_html TEXT, sort_order INTEGER, updated_at TEXT); CREATE TABLE site_settings(setting_key TEXT, setting_value TEXT); CREATE TABLE reviews(id INTEGER PRIMARY KEY, content TEXT, is_active INTEGER); INSERT INTO faqs VALUES(1,'Übernimmt die Krankenkasse die Kosten?','Private Zusatzversicherungen erstatten häufig einen Teil.',6,''); INSERT INTO faqs VALUES(2,'Eigene Frage','Eigene Antwort',20,''); INSERT INTO reviews VALUES(1,'Meine Schmerzen sind verschwunden',1); INSERT INTO site_settings VALUES('address_zip','78655');").unwrap();
        migrate_content(&conn).unwrap();
        migrate_content(&conn).unwrap();
        let original: String = conn.query_row("SELECT content FROM marketing_content_backup WHERE kind='faq' AND record_id=1", [], |r| r.get(0)).unwrap();
        assert!(original.contains("erstatten"));
        let custom: String = conn.query_row("SELECT answer_html FROM faqs WHERE id=2", [], |r| r.get(0)).unwrap();
        assert_eq!(custom, "Eigene Antwort");
        let quote: (String, bool) = conn.query_row("SELECT content,is_active FROM reviews WHERE id=1", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(quote, ("Meine Schmerzen sind verschwunden".into(), false));
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM faqs", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 4);
    }
}
