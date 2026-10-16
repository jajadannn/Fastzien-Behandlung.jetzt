**Infrastruktur- und Sicherheitsprüfung — 8. Oktober 2026**

Geprüft wurde der lokale Rust-Branch: Abhängigkeiten, Konfiguration, Authentifizierung/OAuth, Buchung und Abrechnung, Datenbankmigrationen, Kalenderintegration, HTML-Ausgabe und HTTP-Schutz. Die Änderungen liegen im Arbeitsverzeichnis. Produktionsserver, Mailanbieter und Remote-CI wurden nicht verändert.

**Dringender externer Befund: SMTP-Zugangsdaten rotieren.**

Die Git-Historie von `.env.production` enthält `JWT_SECRET`, `ADMIN_PASSWORD` und `SMTP_PASS` (unter anderem in `f9d2b75` und `6c78293`). Ein Vergleich ohne Ausgabe der Werte bestätigt: Das aktuelle lokale `SMTP_PASS` entspricht noch einem historischen Wert. Die lokalen JWT-/Admin-Werte stimmen nicht mit den untersuchten historischen Werten überein; daraus folgt keine Aussage über die Produktionskonfiguration.

Das SMTP-Passwort beim Mailanbieter ersetzen, den alten Zugang widerrufen und die neue Konfiguration in den tatsächlich verwendeten Umgebungen hinterlegen. Historische JWT-/Admin-Zugangsdaten ebenfalls auf dem Produktionssystem abgleichen und gegebenenfalls rotieren. Das Entfernen einer Datei aus dem aktuellen Git-Baum widerruft keine Zugangsdaten. Keine Geheimwerte wurden in diesen Bericht übernommen. `server.log` wurde aus dem Git-Index entfernt und bleibt lokal erhalten; `.env.*` und Logs werden künftig ignoriert, `.env.example` bleibt versioniert.

**Belegte und behobene Codeprobleme**

| Bereich | Befund und Korrektur | Nachweis |
|---|---|---|
| Sessions | JWTs blieben nach Passwort-/E-Mail-/Rollenänderungen oder Kontolöschung gültig. Aktueller Kontostand und ein HMAC-Tag der Passwortversion werden jetzt geprüft. Cookies tragen HttpOnly, SameSite=Lax und bei HTTPS Secure. | `auth::tests` |
| Passwort-/Mail-Links | Passwortreset ist atomar und einmalig; E-Mail-Wechsel verwirft alte Links, verlangt neue Verifikation und versendet diese. Verifikationsversand hat einen echten Cooldown. bcrypt-Eingaben über 72 Bytes werden abgelehnt. | Auth-/Customer-Tests |
| Browseranfragen | Schreibende Anfragen prüfen Origin/Referer gegen das konfigurierte BASE_URL sowie Fetch-Metadata. Fremde/same-site Unterdomains werden nicht automatisch vertraut. Nextcloud-Disconnect verwendet POST. | `security::tests`, OAuth-Test, HTTP-Smoke |
| Ressourcen/Start | Auth-Endpunkte begrenzen Anfragen pro Peer-IP und insgesamt; Payloadgrößen sind begrenzt. Fehlende/unsichere Admin- und JWT-Vorgaben sowie ungültige BASE_URL/Tokenlaufzeiten verhindern den Start. | Config-/Limiter-Tests, HTTP-Smoke |
| OAuth | Vorher vorhersehbarer, globaler State ohne Browserbindung; Callback-URL aus Host-Header. Jetzt zufälliger, zeitbegrenzter, browser-/admingebundener Einmal-State und konfigurierte Callback-URL. Persistenz ist atomar; verspätete Callbacks/Refreshes können Disconnect oder neuere Tokens nicht überschreiben. | `oauth_handler::tests` |
| Buchungen | Verschiebung konnte Geschäftszeiten umgehen und doppelt buchen/abrechnen. Jetzt dieselben Zeitregeln, bedingtes Update des bestehenden Termins und Erhalt von Rechnung, Guthaben und Notizen. Adminbuchungen prüfen Überschneidungen; relevante Schreibvorgänge verwenden IMMEDIATE-Transaktionen. | Booking-/Admin-/Appointment-Tests |
| Guthaben/Löschen | Storno erstattete die falsche bzw. keine vollständig verbrauchte Karte; Löschung ignorierte Fremdschlüsselfehler. Neue Buchungen speichern die konkrete Karte; Storno erstattet einmalig. Kontolöschung inklusive abhängiger Daten erfolgt vollständig oder gar nicht. | Migrations-/Rollback-/Stornotests |
| Kalenderverfügbarkeit | Netzwerk-/HTTP-/Parserfehler galten als freie Zeit. Jetzt 503 bei unbekannter Verfügbarkeit. XML wird mit Namespaces und Größenlimit geparst; Kalendererkennung akzeptiert nur denselben Origin, Requests folgen keinen Redirects. | Parser-/lokale HTTP-Tests |
| Kalenderzeit | Falsche Sommerzeitgrenzen und UTC/Berlin-Vergleiche korrigiert. Lokale Tagesgrenzen werden nach UTC umgerechnet; Wiederholungen werden vom CalDAV-Server expandiert angefordert. UID erlaubt das eigene Ereignis bei Verschiebungen auszunehmen. | IANA/DST-, Upcoming- und Reminder-Tests |
| XSS/ICS | FAQ-Text konnte JavaScript-Template-Literale im Editor ausführen; FAQ-HTML war unbereinigt. Jetzt Datenattribute und Event-Listener sowie serverseitige HTML-Allowlist, auch beim Lesen alter Inhalte. ICS-Zeilenumbrüche sind escaped. | Sanitizer-, echte Tera-Rendering- und ICS-Tests |
| Vertrauliche Ausgabe | Standard-Accesslogs enthielten Token-URLs/Referer. Jetzt nur Route-Muster, Status, Größe und Dauer. Dynamische Antworten erhalten no-store/no-referrer; Framing/MIME-Sniffing werden blockiert. SQL-/Template-Fehlerdetails werden nicht an Clients gegeben. Unicode-Datumswerte verursachen keinen Slice-Panic mehr. | Header-, Rendering-, Unicode- und Log-Smoke-Tests |

**Abhängigkeiten und bekannte Advisories**

Der Ausgangs-Audit meldete drei Vulnerabilities und vier informative Warnungen. Der aktuelle Audit mit `cargo-audit 0.22.2`, aktueller RustSec-Datenbank und `--deny warnings` prüft 347 Lockfile-Pakete und liefert **0 Vulnerabilities / 0 Warnungen**.

| Paket/Befund | Maßnahme und Einordnung |
|---|---|
| crossbeam-epoch 0.9.18, [RUSTSEC-2026-0204](https://rustsec.org/advisories/RUSTSEC-2026-0204.html) | 0.9.20. Pointer-Formatierung konnte ungültige Zeiger dereferenzieren; konkrete Erreichbarkeit durch App-Anfragen nicht nachgewiesen. |
| h2 0.3.27, [RUSTSEC-2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258.html) | Optionales Actix-HTTP/2-Feature entfernt; keine reparierte 0.3-Version verfügbar. Die Anwendung bindet HTTP/1; HTTP/2 kann am vorgeschalteten Proxy terminieren. |
| lettre 0.11.20, [RUSTSEC-2026-0141](https://rustsec.org/advisories/RUSTSEC-2026-0141.html) | 0.11.23. Der beschriebene BoringTLS-Fehler betrifft den hier verwendeten native-tls-Backend nicht. |
| anyhow, [RUSTSEC-2026-0190](https://rustsec.org/advisories/RUSTSEC-2026-0190.html); rand, [RUSTSEC-2026-0097](https://rustsec.org/advisories/RUSTSEC-2026-0097.html) | Aktualisiert auf anyhow 1.0.103 sowie rand 0.8.6/0.9.3. |
| dotenv, [RUSTSEC-2021-0141](https://rustsec.org/advisories/RUSTSEC-2021-0141.html) | Ungepflegtes Paket durch dotenvy ersetzt; ungenutztes actix-session entfernt. |
| SQLite 3.45.0 | rusqlite 0.40.2 bündelt SQLite 3.53.2 mit [Upstream-Korrekturen](https://sqlite.org/cves.html). Die dort beschriebenen Fehler benötigen typischerweise beliebiges SQL oder manipulierte DB-Dateien; entsprechende Ausnutzbarkeit hier nicht nachgewiesen. |

HTML-/XML-Parser: ammonia 4.2.1 und roxmltree 0.21.1. Beide sind vom aktuellen Lockfile-Audit erfasst. Quill 1.3.7 bleibt als CDN-Editor eingebunden. Sein [umstrittenes XSS-Advisory](https://github.com/advisories/GHSA-4943-9vgg-gr5r) betrifft unsanitisiertes HTML; die gespeicherte HTML-Ausgabe wird jetzt serverseitig bereinigt. Ein separates Editor-Upgrade und lokale Bereitstellung würden die externe CDN-Abhängigkeit zusätzlich reduzieren.

**Verifikation und Wiederholung**

- `cargo test --locked`: 57 Tests erfolgreich, einschließlich Fehler-, Replay-, Race-/Rollback-, Migrations-, XSS- und Zeitregressionen.
- `cargo clippy --locked --all-targets`: erfolgreich; vier bestehende Strukturhinweise (drei umfangreiche CalDAV-Funktionssignaturen und der Marketing-Topic-Typ), keine Compilerfehler.
- `cargo build --locked` und `python scripts/verify_security.py`: echter lokaler Server mit zufälligem Testschlüssel, separater temporärer Datenbank und SMTP ausschließlich auf Loopback. Prüft Seiten/Anmeldung, Autorisierung, Cookieflags, CSRF, Payloadlimits, Buchungs-/Rechnungsabläufe, SMTP-Fehler, Rate-Limits und Log-Redaktion. Temporäre Daten werden entfernt.
- `cargo audit --deny warnings`: erfolgreich. Verifizierte offizielle Audit-Binary; Datenbank-Stand dieses Durchlaufs: `550efd3d587a29b2e2c2b21b17a440da4fede999`.
- Workflow-Syntax mit actionlint sowie bearbeitete Editor-Skripte mit `node --check` geprüft; `git diff --check` ohne Fehler.

GitHub Actions führt künftig Tests/Clippy/HTTP-Smoke auf Windows und Linux sowie täglich RustSec-Audits aus. Dependabot prüft Cargo und Actions wöchentlich. Die Workflow-Dateien sind lokal vorbereitet; Remote-Ergebnisse entstehen nach Veröffentlichung im Repository. CSRF-Entwurf orientiert sich an der [OWASP-Empfehlung zu Origin und Fetch-Metadata](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html).

**Betrieb und verbleibende Grenzen**

- Vor Übernahme Datenbank sichern. Die neue nullable Spalte `appointments.credit_package_id` wird beim Start angelegt; Migrationswiederholung ist getestet. Bei historischen Kartenterminen mit mehreren möglichen Karten wird Storno ohne Änderung abgelehnt; die tatsächlich belastete Karte muss zugeordnet werden.
- Bestehende JWTs sind nach Übernahme ungültig; eine erneute Anmeldung ist erforderlich. BASE_URL muss exakt zum Browser-Origin passen. Produktionszugriff benötigt HTTPS am Proxy; lokales HTTP ist nur für Loopback vorgesehen.
- Auth-Limit: 20 Anfragen/Minute je TCP-Peer und 120 insgesamt. Hinter einem Reverse Proxy teilen dessen Clients das Peer-Limit. Fremde Forwarded-Header werden bewusst nicht als Identität akzeptiert. Für größere Installationen ein zusätzliches vertrauenswürdig konfiguriertes Proxy-Limit vorsehen.
- CalDAV-Verfügbarkeitsfehler blockieren Buchungen. Ein anschließend fehlgeschlagener Kalender-PUT oder E-Mail-Versand bleibt ein Betriebsfehler nach der lokalen Buchung; es gibt weiterhin keine dauerhafte Versandwarteschlange oder verteilte Transaktion. SMTP besitzt jetzt einen 15-Sekunden-Timeout.
- Live-TLS, Proxy-Konfiguration, Provider-Zugänge, Backups und die reale Nextcloud-Installation wurden nicht aus der lokalen Prüfung heraus verifiziert. Der offene SMTP-Geheimnisbefund oben erfordert eine Änderung beim Anbieter.
