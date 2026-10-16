# Website-Richtlinien: Umsetzung und Abnahme

Stand: **9. Oktober 2026**. Grundlage: `Webseiten-Best-Practices-und-Recht.md`, insbesondere die Checklisten in Kapitel 26. Die Richtliniendatei bleibt unverändert. Empfehlungen, kontextabhängige Pflichten und Betriebsaufgaben werden getrennt bewertet.

Die anwendbaren lokalen Änderungen sind umgesetzt. **Die vollständige Abnahme aller Punkte bleibt offen:** Die USt-ID trägt der Nutzer selbst ein. Datenschutzvereinbarungen und Protokollfristen sind laut Nutzer noch nicht vollständig festgelegt. Produktivbetrieb, Originalbelege und echte Geräte benötigen eigene Nachweise. Es wurde nichts produktiv veröffentlicht und keine echte Kunden-E-Mail versandt.

## Bestätigte Betreiberangaben

- Betreiber: Thilo Seifried; Website und Nextcloud auf eigener Hardware.
- E-Mail-Anbieter: mail.de; Unternehmensname laut Anbieterimpressum: mail.de GmbH.
- Logo, Fotos und Bewertungen: Freigabe durch den Nutzer bestätigt. Originale bleiben erhalten. Herkunfts- und Einwilligungsnachweise betrieblich ablegen.
- Weniger als zehn Beschäftigte und weniger als 2 Mio. Euro Jahresumsatz: Kleinstunternehmensausnahme für Dienstleistungen nach § 3 Abs. 3 BFSG dokumentiert. Technisches Prüfziel trotzdem WCAG 2.2 AA.
- USt-ID vorhanden; Nutzer ergänzt sie unter **Admin → Einstellungen → Rechtliche Angaben → Umsatzsteuer-Identifikationsnummer**. Das Dashboard zeigt bei fehlender Nummer einen Hinweis. Keine Nummer wurde erfunden.
- Datenschutzvereinbarungen und Protokollfristen: noch nicht bzw. teilweise festgelegt; offen.

## Änderungen

- Gemeinsame Service-Navigation mit Impressum, Datenschutz, Buchungsbedingungen, Widerruf und Hilfe auf öffentlichen, Konto- und Verwaltungsseiten.
- Pflichtbestätigung der Datenschutzhinweise entfernt. Adresse nur für Hausbesuche erforderlich. Verständliche Hinweise vor Datenerhebung, ohne eine reine Information zur Einwilligung umzudeuten.
- Buchung zeigt Gesamtpreis, Dauer, Ort und Kartenverwendung. Der Server gleicht die angezeigte Preis-/Dauer-/Kartenangabe vor dem Abschluss ab. Karten werden nicht ohne Zahlungsnachweis als vorausbezahlt bezeichnet.
- Widerrufsbelehrung und Musterformular vor Abschluss. Für Termine innerhalb von 14 Tagen ist ein ausdrückliches Verlangen nach vorzeitigem Beginn mit Kenntnis des Erlöschens bei vollständiger Erfüllung erforderlich. Die Bedingungen und Zustimmung werden versioniert gespeichert.
- `/widerruf`: ohne Konto, zwei Schritte, Korrekturmöglichkeit, eindeutige Bestätigung, Referenz und UTC-Eingangszeit. Funktioniert ohne JavaScript. Ungeprüfte Angaben verändern keine fremde Buchung; die Praxis ordnet Erklärung, Termin und gegebenenfalls Rückzahlung zu.
- Buchungs- und Widerrufsbestätigungen werden vor SMTP persistiert. Outbox mit Lease und Wiederholungsversand; erfolgreiche Versandinhalte werden entfernt. Bei Crash nach SMTP-Annahme kann ein Beleg nochmals verschickt werden, ohne nochmals zu buchen.
- Datenschutztexte an tatsächliche Cookies, Logging, Gastbuchung, Kalender und E-Mail angepasst. Unbelegte Telekom-Zugehörigkeit und vermeintliche Einwilligung allein durch Gesundheitsnachrichten entfernt; Belegfrist grundsätzlich acht Jahre.
- Kontolöschung entfernt nicht mehr benötigte Daten transaktional und trennt notwendige Abrechnungsdaten ab. Zukünftige Termine müssen vorher geklärt werden. Archivzugriff nur für Admin. Diese zusätzliche Datenablage ersetzt keine ordnungsgemäße Buchhaltung oder aufzubewahrenden Originalbelege.
- Labels, Autofill, sichtbarer Fokus, Skip-Link vor Navigation, Kalenderzustände, Dialog-Fokus, Kontraste, Reflow und reduzierte Bewegung verbessert.
- E-Mail-Link im Impressum verwendet auch für Hilfstechnologien die tatsächlich konfigurierte Adresse; eine veraltete feste Beschriftung wurde entfernt.
- Externer Quill-Editor durch native Textarea für das weiterhin bereinigte HTML ersetzt. Keine externen Skripte, Schriften, Karten oder Pixel; Karten/Messenger bleiben freiwillige Links.
- Datenexport enthält auch Buchungsherkunft, gespeicherte Anfragen, ursprüngliche Widerrufsangaben und ausstehende Vertragsbestätigungen. Abfragefehler brechen den Export ab. E-Mail-Verifikation schützt Export, Rechnungen, Kalender und Änderungen an bestehenden Kontakt-/Termindaten; eine Gastbestätigung wird keinem unbestätigten Konto zugeordnet. Vor Verifikation ausgegebene Kalenderlinks werden ersetzt.
- Individuelle JWT-Sitzungs-ID und serverseitige Sperre bei expliziter POST-Abmeldung. Neue/geänderte Passwörter mindestens 12 Zeichen, maximal 72 UTF-8-Bytes für bcrypt.
- CSP auf eigene Ressourcen begrenzt, vorhandene Inline-Skripte/-Styles bewusst erlaubt. Permissions-Policy, private Cache-/Indexierungsregeln, statischer Cache und Kompression ergänzt.
- Verständliche HTML-404 mit Status 404. Alte `.html`-URLs führen zum jeweiligen aktuellen Nachfolger. Sitemap nur mit indexierbaren Inhalten.
- Termin-E-Mails verwenden die tatsächliche Termindauer statt fester 90 Minuten. Unbelegte Fahrtpauschalen und Reservierungszusagen entfernt; HTML-Einfügungen escaped und Terminvorschläge vor einer Stornierung als Datumswerte geprüft.
- Webfont- und Abhängigkeitslizenzen dokumentiert. Konsistentes Backup mit isolierter Restore-Verifikation vorbereitet.

## Prüfprotokoll

| Prüfung | Ergebnis / Nachweis |
|---|---|
| Rust | `cargo test --locked`: 69 Tests bestanden, darunter Rückabwicklung, Widerruf-Idempotenz, Session-Sperre, XSS, Kalenderzeit und Belegdaten |
| Build | `cargo build --locked --target-dir target/compliance` erfolgreich; eigener Pfad wegen des bereits laufenden bisherigen lokalen Programms |
| Clippy | Erfolgreich; vier bestehende Strukturhinweise in CalDAV/Marketing |
| HTTP-Sicherheit/Buchung | `scripts/verify_security.py`, einschließlich `verify_marketing.py`: lokale Anwendungsprüfung mit temporärer DB und SMTP ausschließlich auf Loopback; zusätzlich `verify_security.py --compliance` für ein getrenntes Vertrags-/Datenschutz-Testsystem |
| Vertrags-/Datenschutzabläufe | `scripts/verify_compliance.py`: Widerruf, Teilangaben, SMTP-Ausfall, Replay, geänderte Preise, Export, Löschung, Archivzugang und Logout |
| Browser/axe-core | Edge headless, axe-core 4.14.0; 26 Routen bei 1440/390 px plus Layoutkontrollen bei 320/768/1024 px; 82 Kombinationen, 0 automatische Befunde, 0 Seitenüberlauf, 0 externe Origins, 0 JS-Fehler. `docs/richtlinien/browser-audit.json` |
| Ergänzende Browserprüfung | Nach Erweiterung von Export und Verifikationsschutz: Profil und Datenschutz bei 320/390/1440 px sowie Profil mit 200%-Text; sieben Prüfungen ohne automatische Befunde oder Seitenüberlauf. Hinweis zum erneuerten Kalenderlink auch visuell geprüft. `docs/richtlinien/export-privacy-audit.json` |
| Öffentlicher Betrieb, nur lesend | Am 9.10.2026 um 05:16 UTC gültiger TLS-Handshake (TLS 1.3), Zertifikat bis 29.11.2026. Direkte HTTP-Abfragen aller neun Prüfrouten erhielten 403 / Fehler 1010; keine Abnahme von Anwendungsheadern, Inhalt, Widerruf oder Indexierung möglich. `docs/richtlinien/production-readonly-audit.json` |
| Öffentliche Mail-DNS | Domain-SPF verweist auf IONOS; MX bei IONOS. `_dmarc.faszienbehandlung.jetzt` beantwortete die DNS-Abfrage mit NAME_ERROR. Tatsächliche Absenderdomain, DKIM-Signatur, Anbieterzuordnung und Zustellung bleiben zu prüfen; keine DNS-Änderung vorgenommen. `docs/richtlinien/production-dns-audit.json` |
| Ohne JavaScript | Browser-POST von Widerrufseingabe über Zusammenfassung bis Eingangsbestätigung erfolgreich |
| Tastatur | Erfolgreich: zusätzliche Prüfung in `scripts/verify_keyboard.cjs`: Skip-Link, Menü, Kalender/Slots, Hausbesuchsfelder, Dialogfokus und 200%-Text; erfolgreich per Tastatur geprüft |
| Backup | 89 Dateien einschließlich konsistentem SQLite-Snapshot gesichert; SHA-256, wiederhergestellte DB und Fremdschlüssel erfolgreich geprüft, nur synthetische Testdaten |
| Assets | Hero-AVIF ca. 186 KiB, Portrait 640/960 ca. 30/59 KiB, Haupt-CSS ca. 36 KiB; vorhandene lokale WOFF2 und Bildabmessungen |
| Design-Detektor | Einmal ausgeführt; zusätzliche Überschriften-/Textbefunde bearbeitet. Stilwarnungen zur etablierten Identität sind keine WCAG-Befunde und begründen keinen unangeforderten Redesign |

Automatische Prüfungen belegen keine vollständige WCAG-/BFSG-Konformität, Posteingangszustellung oder Core Web Vitals realer Besucher. Reale Smartphones/Bildschirmtastaturen, vollständige Screenreader-Prüfung und Produktivwiederherstellung wurden nicht geprüft. Öffentliche TLS-/DNS-Stichproben sind vorhanden; Zertifikatsüberwachung, Domainkontrolle und Proxy-/Anwendungsverhalten bleiben unbestätigt. Die 403-Antworten erlauben keine Aussage über das Vorhandensein der aktuellen lokalen Änderungen im Produktivsystem.

## Dienste und Aufbewahrung

Betreiber/Fachverantwortlicher: **Thilo Seifried**. Zuständig für Inhalte, Preise, Rechte, Anfragen, Widerrufe, Datenschutz, Rechnungen und Anbieterangaben. Technischer Betrieb liegt nach Nutzerangabe bei ihm; Vertretung und Eskalationskontakt sind noch festzulegen.

| Verarbeitung | Zweck und Empfänger | Technische Umsetzung / Betriebsaufgabe |
|---|---|---|
| Eigene Website/SQLite | Konto, Kontakt, Termine; Thilo Seifried; Vertrag/Anbahnung und Sicherheitsinteresse | Kein Tracker. Infrastrukturprotokolle und deren Fristen noch festlegen |
| SMTP | Transaktionsmails; mail.de GmbH; Vertrag/gesetzliche Bestätigung | Outbox vorbereitet. Tatsächliche Anbieterrolle, Vertrags-/AV-Bedarf, Empfänger/Unterauftragnehmer und Speicherorte prüfen |
| Nextcloud auf eigener Hardware | Kalenderorganisation; Thilo Seifried | Zugänge, Sicherung, Löschabgleich und etwaige weitere Empfänger dokumentieren |
| Persönlicher WebCal | Freiwillig gewählter Kalenderdienst des Kunden | URL wie Zugangsschlüssel behandeln; Rotation bei Verlust; keine pauschale Standortzusage |
| Google/WhatsApp | Freiwilliger externer Linkaufruf | Keine eingebetteten Abrufe; Telefon/E-Mail als Alternative |
| Gast-Linkanfragen | 30 Minuten gültige Bestätigung | Nach einem Tag bei stündlicher Bereinigung entfernt |
| Sitzungs-/Limitdaten | Angeforderte Sitzung und Missbrauchsschutz | Limiter im Speicher für 60 Sekunden, Bereinigung beim nächsten Zugriff; abgelaufene Sperren stündlich entfernt |
| Versandqueue | Versand und Wiederholung | Inhalt nach SMTP-Annahme entfernt; erledigte Metadaten nach 30 Tagen. Ausstehende Bestätigungen im Dashboard bearbeiten |
| Konto/Termine | Nutzung und Vertragsabwicklung | Selbstlöschung nach Klärung zukünftiger Termine; externe Mail-/Kalenderkopien zusätzlich einbeziehen |
| Belegdaten | Erforderliche Abrechnung | Separates Adminarchiv, grundsätzlich acht Jahre ab Jahresende. Originalbelege, Verlängerungen/Legal Holds und spätere Löschung betrieblich prüfen |
| Widerrufe/Vertragsnachweise | Gesetzliche Bearbeitung und Ansprüche | Eigene begründete Frist festlegen; nicht pauschal Rechnungsfrist verwenden |
| Sicherungen | Wiederherstellbarkeit | Werkzeug und Test vorhanden; unabhängige verschlüsselte Ablage, Zugriff, Vorhaltezeit und Löschabgleich nach Restore noch festlegen |
| Gesundheitsangaben außerhalb Formularen | Gegebenenfalls Körperarbeit | Website fordert keine Diagnosen. Bestehende Notizen, erforderliche ausdrückliche Einwilligung, Zugangsschutz und rechtliche Einordnung gesondert prüfen |

## Erforderliche Abschlussaufgaben

1. **USt-ID selbst ergänzen** und tatsächliches Impressum prüfen; Wirtschafts-ID gegebenenfalls ebenfalls.
2. **Datenschutz formal abschließen:** mail.de-Rolle/Vertrag, Empfänger, Speicherung und Drittlandbezug. Verarbeitungsverzeichnis, DSFA-/Datenschutzbeauftragtenbedarf und Gesundheitsdatenprozesse konkret einordnen.
3. **Fristen festlegen:** Protokolle, Widerrufs-/Vertragsnachweise, E-Mails/Kalender und Sicherungen; betriebliche Umsetzung und Verantwortliche benennen.
4. **Bekannten Geheimnisbefund lösen:** offener SMTP-Zugangsdatenbefund aus `SECURITY_REVIEW.md`. Lokaler Code widerruft keine historischen Zugangsdaten beim Anbieter.
5. **Produktivbetrieb testen:** HTTPS/Proxy/CSP, Zertifikate, DNS, SPF/DKIM/DMARC, echte Mailzustellung und Kalenderabgleich. Dashboard ist kein externer Ausfallalarm.
6. **Produktionssicherung üben:** geschützter unabhängiger Zielort und Vorhaltezeit; Restore einschließlich Konfiguration/externer Daten. `.env` wird bewusst nicht archiviert; Geheimnisse separat sichern.
7. **Echte Nutzung prüfen:** Smartphone, Bildschirmtastatur, Screenreader und produktive Ladezeiten.
8. **Fachliche Abnahme:** Branchen-/Vertrags-/Steuerangaben, Originalrechtebelege und tatsächlich bearbeitete Widerrufe/Erstattungen bestätigen.

## Übergabe und Wartung

- Vor Übernahme DB sichern. Neue Tabellen werden beim Start angelegt. Alte JWTs ohne individuelle Session-ID werden ungültig; erneute Anmeldung erforderlich.
- Rollback mit passendem Programm-/Quellstand und DB-Snapshot; später eingegangene Bestätigungen/Widerrufe exportieren und abgleichen, damit keine Erklärung verloren geht.
- Vorschlag: monatlich Inhalte/Preise, offene Widerrufe, Versandqueue, Links und Updates; quartalsweise Rechte, Dienste, Berechtigungen und Restore. Kritische Vorfälle sofort. Keine Automation wurde ohne ausdrücklichen Auftrag eingerichtet.
- Betroffenenanfragen: Eingang, angemessene Identitätsprüfung, Systeme und Antwort dokumentieren; grundsätzlich Monatsfrist. Datenschutzvorfälle und einschlägige Meldeschwellen/Fristen prüfen.
- Backup: `python scripts/backup_site.py backup --database <DB> --output <geschützte ZIP>`. Verifikation: `python scripts/backup_site.py verify --archive <ZIP>`. Nicht über eine laufende Datenbank zurückspielen.
- Quellwahrheit: Rust-Routen und `templates/`. Die alten Root-HTML-Dateien sind keine alternative freigegebene statische Deploymentfassung.

## Quellen

- [§ 5 DDG – Anbieterangaben und vorhandene USt-/Wirtschafts-ID](https://www.gesetze-im-internet.de/ddg/__5.html)
- [§ 356a BGB – elektronische Widerrufsfunktion](https://www.gesetze-im-internet.de/bgb/__356a.html)
- [§ 25 TDDDG – angeforderter notwendiger Dienst, Abs. 2 Nr. 2](https://www.gesetze-im-internet.de/ttdsg/__25.html)
- [§ 147 AO – grundsätzlich acht Jahre für Buchungsbelege](https://www.gesetze-im-internet.de/ao_1977/__147.html)
- [Bundesfachstelle – BFSG und Kleinstunternehmen](https://www.bundesfachstelle-barrierefreiheit.de/DE/Barrierefreiheitsstaerkungsgesetz/FAQ/faq)
- [mail.de – Anbieterimpressum](https://mail.de/de/impressum/)

Weitere Grundlagen in der vorgegebenen Richtliniendatei. Technische Umsetzung, Betreiberangaben und fehlende Betriebsnachweise bleiben unterscheidbar; kein pauschales Rechtskonformitätssiegel.

## Vollständige Checklisten-Zuordnung

Siehe [WEBSITE-CHECKLISTE.md](WEBSITE-CHECKLISTE.md). „Lokal“ bezeichnet Quellcode-/Testnachweise, „Betreiber“ bestätigte Angaben, „offen“ fehlende echte bzw. betriebliche Nachweise und „entfällt“ begründet nicht vorhandene Funktionen.
