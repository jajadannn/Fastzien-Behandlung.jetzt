# Marketing-Umsetzung – Thilo Seifried

Stand: 7. Oktober 2026. Arbeitsgrundlage: der vom Inhaber übergebene Marketing-Leitfaden. Ansprache bleibt einheitlich „du“. Dieses Dokument enthält vorbereitete Maßnahmen und Texte; es dokumentiert keine bereits veröffentlichten Anzeigen, versendeten Nachrichten oder abgeschlossenen Kooperationen.

## Im Projekt umgesetzt

- Startseite, Metadaten, strukturierte Daten und FAQ ohne Heilversprechen, garantierte Ergebnisse, Krankheitsbehandlung oder unbelegte Überlegenheitsvergleiche.
- Einheitliche Adresse: Sulgauer Straße 24, 78713 Schramberg-Sulgen. Das bestehende Angebot bleibt Einzelbehandlung 195 € / ca. 90 Minuten; Werte sind weiterhin administrierbar.
- Klarer Hinweis: Wellness- und Gesundheitsförderungsleistung, keine Erstattung durch Krankenkassen oder Versicherungen, Rechnung für die eigenen Unterlagen. Vorgespräch als Teil der Sitzung, keine separat versprochene kostenlose Beratung.
- Buchung ohne vorherige Registrierung unter `/buchen`: Terminwahl, Kontaktdaten, Bestätigung per E-Mail. Link 30 Minuten gültig, Termin erst nach Bestätigung gebucht. Guthaben bleibt im Kundenportal. Keine Werbeeinwilligung als Buchungsvoraussetzung.
- Telefon und WhatsApp als Kontaktalternativen; Datenschutzhinweise an den neuen Ablauf angepasst.
- Größeres vorhandenes Porträt; Themen-Unterseiten für Rückenverspannungen, Nacken/Schulter, Haltung und Sport sowie `/preise-und-ablauf`. Sitemap enthält nur echte Routen.
- Originalbewertungen bleiben im Admin-Bereich. Echtheit, Einwilligung und Veröffentlichung müssen dort ausdrücklich bestätigt werden. Neue Datenbanken enthalten keine Musterbewertungen. Automatische Bewertungs-Werbemails mit dem bisherigen Platzhalterlink sind entfernt.
- Feld für den echten Google-Bewertungslink in den Einstellungen. Der Link wird nur angezeigt, wenn er hinterlegt ist. Keine erfundenen Google-Sterne oder Bewertungszahlen.
- Freiwillige Herkunftsangabe bei der Buchung, Auswertung bestätigter Buchungen der letzten 90 Tage im Admin-Bereich. Kein Google Analytics oder Meta Pixel.

## Vor Veröffentlichung und im Tagesbetrieb

1. Sicherung der produktiven SQLite-Datenbank erstellen. Beim Start des neuen Servers werden die bekannten bisherigen FAQ-Fragen einmalig überarbeitet. Die alten Antworten liegen danach in `marketing_content_backup`. Andere FAQ-Fragen bleiben erhalten und müssen redaktionell geprüft werden.
2. Produktive SMTP-Konfiguration testen: Der Bestätigungslink muss auf die öffentliche Domain zeigen, nicht auf die lokale Vorschau. Eine echte eigene Testbuchung durchführen, den Link bestätigen und den Termin danach im Admin prüfen und absagen.
3. In **Einstellungen** Telefonnummer, Öffnungszeiten, Adresse, Dauer und Preise prüfen. Die Texte zu Öffnungszeiten steuern bisher nur die Darstellung; der Buchungskalender arbeitet weiterhin mit Mo–Fr 16–22 Uhr und Sa 9–19 Uhr. Abweichende Buchungszeiten müssen im Kalendercode angepasst werden.
4. In **Bewertungen** nur Originalstimmen über Erlebnis und Service mit vorhandener Einwilligung freigeben. Krankheits- oder Heilversprechen nicht veröffentlichen. Einwilligungsnachweis in den Praxisunterlagen aufbewahren.
5. Den echten Google-Link aus dem Unternehmensprofil unter **Einstellungen → Kontakt** eintragen. Bei Google selbst Name, Kategorie, Adresse, Buchungslink, Öffnungszeiten und tatsächliche Zahlungsarten prüfen. Den Eintrag nicht anhand des zitierten Verzeichnisses ungeprüft ändern.
6. Rechtstexte und Werbetexte vor Veröffentlichung fachlich prüfen lassen. Die Änderungen nehmen keine vollständige rechtliche Freigabe vorweg. Die Rechnungsformulierung folgt der bereits im Impressum angegebenen Kleinunternehmerregelung.

## Google-Unternehmensprofil – vorbereiteter Text

> Gantke® Faszienbehandlung bei Thilo Seifried in Schramberg-Sulgen. Persönliche Wellness- und Körperarbeit für Erwachsene, die sich verspannt fühlen und sich Zeit für Beweglichkeit, Haltung und Entspannung nehmen möchten. Eine Sitzung dauert ca. 90 Minuten und umfasst Vorgespräch, manuelle Faszienbehandlung und individuelle Übungen für den Alltag. Abendtermine und Termine am Samstag. Hausbesuche im bisherigen Einzugsgebiet nach Vereinbarung. Einzelbehandlung 195 €, keine Erstattung durch Krankenkassen oder Versicherungen. Online-Terminwahl ohne vorherige Registrierung auf faszienbehandlung.jetzt.

Name an der tatsächlichen Geschäftsbezeichnung orientieren; keine Suchwortliste in den Namen aufnehmen. Preis, Dauer, Zeiten und Zahlungsarten mit der Website abgleichen. Fotos: tatsächlicher Behandler, Behandlungsraum, Eingang, Parkplatz und Anfahrt. Keine Stockfotos als Praxisfotos ausgeben.

## Bewertungen – persönliche, neutrale Bitte

> Danke für deinen Besuch. Wenn du möchtest, kannst du deine Erfahrung mit meiner Praxis auf Google teilen. Jede ehrliche Rückmeldung ist willkommen. Den Link findest du hier: [echten Bewertungslink einsetzen]. Vielen Dank, Thilo.

Den echten Link einsetzen, bevor dieser Entwurf verwendet wird. Keine Belohnungen, keine Vorgabe einer Sternezahl, keine ausschließlich an zufriedene Kunden gerichtete Bitte. Niemanden zur Rezension drängen. Keine automatische Werbenachricht ohne passende Einwilligung. [Google: Tipps für Rezensionen](https://support.google.com/business/answer/3474122?hl=de), [Google: Regeln zu Bewertungen](https://support.google.com/contributionpolicy/answer/7400114?hl=de).

Antwort auf positives Feedback:

> Danke für deine Rückmeldung und dein Vertrauen. Es freut mich, dass du die Zeit in meiner Praxis angenehm erlebt hast. Viele Grüße, Thilo.

Antwort auf kritisches Feedback:

> Danke, dass du deine Erfahrung schilderst. Ich möchte dein Anliegen gern persönlich verstehen. Bitte melde dich telefonisch oder per E-Mail bei mir, damit wir es in Ruhe besprechen können. Viele Grüße, Thilo.

In öffentlichen Antworten keine Behandlung, Kundeneigenschaft oder Gesundheitsangaben bestätigen, die die Person nicht selbst veröffentlicht hat.

## Suchanzeigen – vorbereitete Kampagne

Noch nicht gestartet. Budgetentwurf aus dem Leitfaden: 200–400 € pro Monat, erst nach einer gesonderten Budgetentscheidung. Ziel ist eine bestätigte Erstbuchung; Umsatz allein ist kein Gewinn und rechtfertigt keine festen Akquisekosten.

| Bestandteil | Entwurf |
|---|---|
| Zielregion | Zunächst Schramberg und tatsächlich bediente Umgebung; Radius erst nach Prüfung der Anfahrten festlegen |
| Standortoption | Menschen in der Zielregion statt bloßem Interesse daran; verfügbare Einstellung beim Anlegen prüfen |
| Suchbegriffe | „faszienbehandlung schramberg“, „gantke faszienbehandlung“, „faszienbehandlung sulgen“ |
| Anzeigentitel | „Gantke® Faszienbehandlung“ / „Schramberg-Sulgen“ / „90 Minuten für deinen Körper“ / „Termin ohne Konto wählen“ |
| Beschreibung | „Persönliche Faszien- und Körperarbeit. Ca. 90 Min., 195 €. Auch abends. Termin online wählen.“ |
| Zweite Beschreibung | „Vorgespräch und Übungen inklusive. Selbstzahlerleistung ohne Kassenerstattung.“ |
| Zielseite | `https://faszienbehandlung.jetzt/preise-und-ablauf` |
| Ausschlüsse prüfen | Ausbildung, Job, kostenlose Behandlung, Kassenrezept und Geräteverkauf |

Keine krankheitsbezogenen Zielgruppenlisten, kein Gesundheits-Retargeting und keine Heilversprechen. Weitere Ortsbegriffe erst nutzen, wenn Kunden aus diesen Orten tatsächlich bedient werden. [Google Ads: Standortausrichtung](https://support.google.com/google-ads/answer/1722043?hl=de).

## Social Media – vier Wochen vorbereitet

Alle Beiträge bleiben Entwürfe. Nur reale eigene Fotos oder Videos mit den nötigen Nutzungsrechten verwenden. Keine fremden Kunden zeigen und keine individuelle medizinische Anleitung anbieten.

| Woche | Beitrag 1 | Beitrag 2 |
|---|---|---|
| 1 | Persönliche Vorstellung mit vorhandenem Porträt | Ablauf einer Sitzung: Gespräch, Körperarbeit, Übungen |
| 2 | Was im Preis von 195 € enthalten ist | Praxis zeigen: Raum, Eingang und Parkmöglichkeiten |
| 3 | Eine bewusste Bewegungspause im Büro als Alltagsthema | Regeneration im Sport: auch Pausen einplanen |
| 4 | Buchung ohne Konto erklären | Transparente Antwort: keine Erstattung durch Versicherungen |

Vorstellung:

> Hallo, ich bin Thilo. In meiner Praxis in Schramberg-Sulgen nehme ich mir Zeit für persönliche Gantke® Faszien- und Körperarbeit. Für Menschen, die sich verspannt fühlen und Beweglichkeit und Entspannung bewusster in ihren Alltag bringen möchten. Eine Sitzung dauert ca. 90 Minuten. Mehr zu mir, Ablauf und Preis findest du auf faszienbehandlung.jetzt.

Preisbeitrag:

> Was bekommst du für 195 €? Ca. 90 Minuten persönliche Aufmerksamkeit: Vorgespräch, Gantke® Faszienbehandlung und individuelle Übungen für zu Hause. Die Sitzung ist eine Wellnessleistung und wird nicht von Krankenkassen oder Versicherungen erstattet. Du erhältst eine Rechnung und kannst bar oder per Überweisung zahlen.

Buchungsbeitrag:

> Dein Termin beginnt mit einer einfachen Auswahl: Tag und Uhrzeit wählen, Kontaktdaten angeben, Buchung per E-Mail bestätigen. Du musst vorher kein Konto anlegen. Erst nach deiner Bestätigung ist der Termin verbindlich gebucht. Fragen vorab? Ruf mich gern an.

## Netzwerk, Presse und Unternehmen – Entwürfe

Für persönliche Gespräche mit Trainern, Vereinen und lokalen Netzwerken:

> Ich bin Thilo Seifried, zertifizierter Gantke® Faszienbehandler in Schramberg-Sulgen. Mein Angebot ist persönliche Wellness- und Körperarbeit für Beweglichkeit, Körperwahrnehmung und Entspannung. Es ergänzt den Alltag und ersetzt keine medizinische Versorgung. Ich würde mich gern vorstellen und besprechen, ob sachliche Informationen zu meiner Praxis für eure Mitglieder interessant sind.

Presseidee:

> Persönliche Faszien- und Körperarbeit in Schramberg-Sulgen: Wer Thilo Seifried ist, wie eine Sitzung abläuft und wie Erwachsene Bewegung und Entspannung im Alltag Raum geben können. Keine medizinischen Erfolgsberichte oder erfundenen Kundenzahlen.

B2B-Kontaktentwurf, erst nach Definition des Angebots verwenden:

> Ich würde gern erfahren, ob Körperwahrnehmung, Bewegungspausen und persönliche Wellnessangebote für eure betriebliche Gesundheitsförderung relevant sind. Mein bestehendes Angebot sind individuelle Gantke® Sitzungen in Schramberg-Sulgen. Falls Interesse besteht, können wir gemeinsam prüfen, welches Format organisatorisch passt.

Keine kostenlosen Vorträge, Firmenrabatte oder Kontingente zusagen, bevor Leistung, Aufwand und Konditionen entschieden sind. Es wurden keine Nachrichten versendet.

## 90-Tage-Arbeitsplan: 8. Oktober 2026 bis 5. Januar 2027

| Zeitraum | Konkretes Ergebnis |
|---|---|
| 8.–21. Oktober | Website technisch und redaktionell freigeben; eigene Testbuchung; Google-Angaben prüfen; echte Bewertungs-URL hinterlegen; Fotoauswahl erstellen |
| 22. Oktober–4. November | Berechtigte Kundenstimmen freigeben; neutrale Bewertungsbitte vorbereiten; erste vier Social-Beiträge veröffentlichen; Empfehlungskarte textlich fertigstellen |
| 5. November–4. Dezember | Zehn passende Netzwerkpartner auswählen und persönlich kennenlernen; eine sachliche Presseidee vorstellen; bei Budgetfreigabe eine kleine Suchkampagne starten |
| 5. Dezember–5. Januar | Erstbuchungen und Herkunftskanäle auswerten; Werbekosten mit tatsächlichen Buchungen vergleichen; B2B-Format entscheiden; nächsten Zeitraum anhand echter Daten planen |

Dieser Plan ist keine Automation und versendet keine Erinnerungen oder Werbenachrichten.

## Kennzahlen und Definitionen

| Kennzahl | Messung | Orientierung aus dem Leitfaden |
|---|---|---|
| Neukunden | Kunden mit erstem nicht storniertem Termin im Monat; keine bloßen Registrierungen | 10–20 pro Monat, nach Kapazität anpassen |
| Buchungskanal | Freiwillige Herkunftsangaben aus bestätigten Gastbuchungen, im Admin ausgewiesen | Empfehlungen als eigenständigen Kanal beobachten |
| Werbekosten pro Erstkunde | Tatsächliche Werbeausgaben / eindeutig zugeordnete neue Kunden | Keine verbindliche Grenze ohne Kostenrechnung |
| Folgekontakte | Tatsächlich vereinbarte weitere Termine | Keine fixe Behandlungszahl als Versprechen |
| Google-Rezensionen | Tatsächliche Einträge im Unternehmensprofil | Entwicklung beobachten; Sternezahl und Inhalt niemals vorgeben |

Die Website zählt Buchungen, keine Besucher und keine Werbeklicks. Kanalangaben sind freiwillig und liefern keine vollständige Attribution. Lokale Ortsseiten für Rottweil/Oberndorf oder Firmenangebote werden erst nach Bestätigung des tatsächlich bedienten Gebiets bzw. Angebots veröffentlicht.

## Neue Angebote – ausdrücklich nicht eingeführt

- **3er-Paket 545 €:** möglicher Entwurf; Preis, Gültigkeit, Zahlungszeitpunkt, Stornierung und Guthabenverwaltung vor Einführung festlegen.
- **Kostenloses 15-Minuten-Telefonat:** möglicher Entwurf; keine separate kostenlose Beratung wird derzeit beworben.
- **Kartenzahlung / PayPal:** Anbieter, Gebühren und Prozess auswählen; bisher ausschließlich bar oder Überweisung.
- **Nachfrage nach sieben Tagen:** Umfang und passende Einwilligung vor Einführung klären; keine automatische Nachricht eingerichtet.
- **Geschenkgutscheine / B2B:** Konditionen, Abwicklung und Einlösbarkeit festlegen; nicht als verfügbar beworben.

## Quellen für die überprüften Änderungen

- [§ 5 DDG – Anbieterinformationen](https://www.gesetze-im-internet.de/ddg/__5.html)
- [§ 1 Heilpraktikergesetz – Abgrenzung der Heilkunde](https://www.gesetze-im-internet.de/heilprg/__1.html)
- [EU-Kommission: Einstellung der OS-Plattform](https://consumer-redress.ec.europa.eu/site-relocation_en)
- [Google: Rezensionen und Bewertungsregeln](https://support.google.com/business/answer/3474122?hl=de)

Die regionalen Marktstatistiken, Wettbewerberpreise und extern behaupteten Bewertungszahlen des eingefügten Leitfadens wurden nicht als eigene öffentliche Aussagen übernommen.
