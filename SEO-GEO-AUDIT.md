# SEO- und GEO-Audit – faszienbehandlung.jetzt

Stand: 17. November 2026. Grundlage: Skills `seo-audit`, `schema` und `ai-seo` (coreyhaines31/marketingskills), Quellcode im Branch `Rust` und die Live-Seite am selben Tag. Ohne Zugang zu Search Console, Google-Unternehmensprofil oder Rankingdaten. Aussagen zu Rankings und KI-Zitaten sind deshalb Hypothesen, keine Messwerte.

Geschäftsmodell laut `PRODUCT.md`: lokale Einzelpraxis mit Hausbesuchen (Ladengeschäft und Einzugsgebiet), Selbstzahler-Wellnessleistung, keine Heilkunde.

## Zusammenfassung

Die technische Basis ist solide: HTTPS mit HSTS, saubere Weiterleitung von http auf https, Canonical-Tags, eine Sitemap nur mit echten Routen, serverseitig gerendertes HTML ohne JavaScript-Abhängigkeit, JSON-LD für das Unternehmen, Bewertungen als echter Text, Alt-Texte und `lang="de"`. KI-Suchcrawler (OAI-SearchBot, PerplexityBot, Claude-SearchBot) erreichen die Seite.

Die fünf wichtigsten Punkte:

1. **`/llms.txt` enthält ein Heilversprechen** („behandelt chronische Schmerzen, Migräne und Gelenkbeschwerden“). Das widerspricht der Website, dem Impressum und den Compliance-Regeln. KI-Systeme lesen genau diese Datei und geben sie wieder. Außerdem verweist der Buchungslink auf `/portal/book` (Login, in robots.txt gesperrt) statt auf `/buchen`.
2. **Themenseiten sind dünn** (ca. 180 Wörter, gleiche Schablone), ohne strukturierte Daten, ohne Social-Media-Vorschau und mit zu langen Titeln.
3. **Strukturierte Daten unvollständig:** Öffnungszeiten, Einzugsgebiet, Koordinaten, Logo, Person (Thilo Seifried) und Verweise auf das Google-Profil fehlen.
4. **`www.faszienbehandlung.jetzt` liefert 200 statt einer 301-Weiterleitung.** Dadurch gibt es eine doppelte Domain, die nur durch das Canonical-Tag abgefangen wird.
5. **Die H1 der Startseite enthält weder Leistung noch Ort** („Endlich wieder freier bewegen.“).

## Technisches SEO

| # | Befund | Auswirkung | Nachweis | Lösung | Priorität |
|---|---|---|---|---|---|
| T1 | `www.` liefert 200 mit identischem Inhalt | Mittel | `curl https://www.faszienbehandlung.jetzt/` → 200, Canonical zeigt auf die Domain ohne www | In Cloudflare eine Redirect-Regel `www.*` → `https://faszienbehandlung.jetzt/$1` (301) anlegen | Hoch |
| T2 | `/buchen` hat `noindex, nofollow` | Niedrig | Meta-Tag in `templates/customer/book_appointment.html:7` | Indexierung kann aus bleiben, `nofollow` auf `follow` ändern | Niedrig |
| T3 | Sitemap ohne `lastmod` | Niedrig | `sitemap.xml` | `lastmod` bei echten Inhaltsänderungen pflegen (nicht bei jedem Build) | Niedrig |
| T4 | `/Portrait.jpg` (14,7 MB) öffentlich ausgeliefert, auf keiner Seite eingebunden | Niedrig | `src/handlers/landing.rs:118`, 4,5 s Ladezeit | Aus der Freigabeliste entfernen oder auf ca. 1600 px verkleinern | Niedrig |
| T5 | Standard-Beschreibung in `base.html` („… in Sulgen“) gilt für mehrere Seiten | Niedrig | `templates/base.html:7` | Auf „Schramberg-Sulgen“ angleichen, für indexierbare Seiten eigene Beschreibungen | Niedrig |
| T6 | `geo.position` 48.2157;8.5033 nicht geprüft | Niedrig | `templates/landing.html:16` | Koordinaten der Adresse in Google Maps prüfen. Sie werden auch für das Schema (S1) gebraucht | Niedrig |

In Ordnung: http → https (301), HSTS, `/index.html` → `/` (308), Canonicals auf allen öffentlichen Seiten, `noindex` für Portal, Admin, Login und Rechtstexte, robots.txt mit Sitemap-Verweis, Komprimierung, vorgeladene Schriften, Bilder mit Breiten- und Höhenangaben, AVIF/WebP.

## On-Page-SEO

| # | Befund | Auswirkung | Lösung | Priorität |
|---|---|---|---|---|
| O1 | H1 der Startseite ohne Suchbegriff | Mittel | Slogan behalten und die H1 um eine Zeile ergänzen, z. B. „Gantke® Faszienbehandlung in Schramberg-Sulgen“ als H1 und „Endlich wieder freier bewegen“ als Untertitel. Den Slogan auf Heilversprechen prüfen lassen | Mittel |
| O2 | Titel der Themenseiten 68–83 Zeichen, Suchbegriff steht hinten | Mittel | Muster „Faszienbehandlung Rücken – Schramberg-Sulgen \| Thilo Seifried“ (ca. 55–60 Zeichen) | Mittel |
| O3 | H1 der Themenseiten ohne Suchbegriff („Zeit für deinen Rücken“) | Mittel | z. B. „Faszienbehandlung bei Rückenverspannungen“ und den emotionalen Satz als Untertitel | Mittel |
| O4 | Themenseiten mit ca. 180 Wörtern, gleicher Aufbau | Hoch | Je 400–700 Wörter echter, eigener Inhalt: typische Fragen zum Thema, Ablauf am Beispiel, was die Sitzung leistet und was nicht, 3–5 Fragen mit Antworten | Hoch |
| O5 | Themenseiten ohne Open-Graph- oder Twitter-Tags | Niedrig | Dieselben OG-Tags wie auf der Startseite mit seitenspezifischem Titel und Beschreibung | Niedrig |
| O6 | Keine Brotkrumen auf den Themenseiten | Niedrig | Sichtbare Brotkrumen „Startseite › Rücken“ plus `BreadcrumbList` | Niedrig |
| O7 | Karten „Beweglichkeit“ und „Stress & Anspannung“ ohne eigene Seite | Niedrig | Nur anlegen, wenn echter Inhalt entsteht, sonst so lassen | Niedrig |

## Strukturierte Daten (Schema)

Vorhanden: `HealthAndBeautyBusiness` mit Adresse, Telefon, E-Mail und Angebot sowie `FAQPage` auf der Startseite. Beides wird serverseitig gerendert und ist im HTML sichtbar.

Hinweis: Google zeigt seit Mai 2026 keine FAQ-Rich-Results mehr. `FAQPage` bleibt als semantische Auszeichnung für andere KI-Systeme sinnvoll, bringt bei Google aber keine Sonderdarstellung.

| # | Befund | Lösung | Priorität |
|---|---|---|---|
| S1 | Im Unternehmens-Schema fehlen `openingHoursSpecification`, `geo`, `areaServed`, `logo`, `priceRange` und `sameAs` | Aus den Admin-Einstellungen befüllen. `areaServed`: Schramberg, Dunningen, Villingendorf. `sameAs`: Google-Profil-URL, sobald hinterlegt | Hoch |
| S2 | Keine `Person` für Thilo Seifried | `employee` bzw. `founder` als `Person` mit `name`, `jobTitle` „Zertifizierter Gantke® Faszienbehandler“ und `image` | Mittel |
| S3 | Keine `WebSite`-Auszeichnung (Website-Name in Google) | `WebSite` mit `name` „Faszienbehandlung Thilo Seifried“ und `url` | Niedrig |
| S4 | Themenseiten ohne Schema | `Service` (Anbieter = Unternehmen, `areaServed`, `offers`) und `BreadcrumbList` | Mittel |
| S5 | Keine `AggregateRating` | Richtig so: Google wertet selbst veröffentlichte Bewertungen eines LocalBusiness nicht aus. Nicht ergänzen | – |

Hinweis zu S1: Die Öffnungszeiten stehen in den Einstellungen als Freitext („Mo–Fr 16:00–22:00 Uhr“). Für das Schema werden strukturierte Werte gebraucht. Am einfachsten nimmt man dieselben festen Zeiten, mit denen der Buchungskalender arbeitet.

Nach dem Einbau mit dem [Rich Results Test](https://search.google.com/test/rich-results) und dem [Schema.org Validator](https://validator.schema.org/) prüfen.

## GEO / KI-Suche

### Crawler-Zugang (Stichprobe mit gefälschtem User-Agent, nur ein Hinweis)

| Crawler | Zweck | Antwort |
|---|---|---|
| OAI-SearchBot | ChatGPT-Suche | 200 |
| PerplexityBot | Perplexity | 200 |
| Claude-SearchBot | Claude-Suche | 200 |
| Googlebot | Google und AI Overviews | 200 |
| GPTBot | OpenAI-Training | 403 (Cloudflare) |
| ClaudeBot | Anthropic-Training | 403 (Cloudflare) |

Suche und Zitate sind erlaubt, Trainingscrawler werden von Cloudflare blockiert. Das ist eine vertretbare Entscheidung. Die `robots.txt` sagt aber „alles erlaubt“. Wenn das Blockieren gewollt ist, sollte es auch dort stehen (`User-agent: GPTBot` / `Disallow: /` usw.), damit beide Signale übereinstimmen. Wenn es nicht gewollt ist: In Cloudflare unter „AI Crawl Control“ freigeben.

### Befunde

| # | Befund | Auswirkung | Lösung | Priorität |
|---|---|---|---|---|
| G1 | `/llms.txt`: „Er behandelt chronische Schmerzen, Verspannungen, Migräne und Gelenkbeschwerden“ | **Kritisch**: rechtliches Risiko (Heilkunde, HWG) und falsche Antworten in KI-Systemen | Text an die Positionierung anpassen: Wellness- und Körperarbeit, keine Heilkunde, ersetzt keine ärztliche Behandlung (`src/handlers/api.rs:44`) | Kritisch |
| G2 | `/llms.txt`: Buchungslink `/portal/book` (Login, robots-gesperrt) | Hoch: KI schickt Interessenten auf eine Anmeldeseite | Auf `https://faszienbehandlung.jetzt/buchen` ändern | Hoch |
| G3 | `/llms.txt`: Adresse falsch formatiert („Sulgauer Straße 24 78713, Schramberg-Sulgen“), alter Standardwert „Sulgen (Schramberg)“ | Mittel: uneinheitliche NAP-Daten | Format „Sulgauer Straße 24, 78713 Schramberg-Sulgen“ und Standardwert angleichen | Mittel |
| G4 | `/llms.txt` ohne Öffnungszeiten, Einzugsgebiet, Hausbesuche, Zahlungsarten, Kassenhinweis und Links zu den Themenseiten | Mittel | Ergänzen, Abschnitt „Seiten“ mit allen Sitemap-URLs und je einem Satz | Mittel |
| G5 | `/llms.txt` wird nirgends verlinkt | Niedrig–Mittel: Agenten finden die Datei meist nur über Links | `<link rel="alternate" type="text/plain" href="/llms.txt">` im `<head>` und ein Fußzeilen-Link | Niedrig |
| G6 | Kein Datum („Stand“ bzw. „zuletzt aktualisiert“) auf Start- und Themenseiten | Mittel: KI-Systeme bevorzugen datierte Inhalte | Sichtbares „Stand: Monat Jahr“ bei Preisen und auf den Themenseiten. Nur ändern, wenn sich wirklich etwas ändert | Mittel |
| G7 | Wenig zitierfähige Antwortblöcke auf den Themenseiten | Mittel | Jede Themenseite mit einem Absatz von 40–60 Wörtern beginnen, der die Kernfrage direkt beantwortet (wie der `quick-answer`-Block der Startseite) | Mittel |
| G8 | Qualifikation nicht nachprüfbar („Gantke® zertifiziert“ ohne Details) | Mittel: E-E-A-T | Jahr der Zertifizierung und ausbildende Stelle nennen. Falls es eine offizielle Liste zertifizierter Gantke®-Behandler gibt: Eintrag sicherstellen und darauf verlinken | Mittel |
| G9 | Präsenz außerhalb der eigenen Seite unbekannt | Hoch für lokale KI-Empfehlungen | Google-Unternehmensprofil vollständig pflegen (siehe `MARKETING.md`) und den Bewertungslink in den Einstellungen hinterlegen. Gleiche Angaben in wenigen relevanten Verzeichnissen (z. B. Gantke-Behandlerliste, Gelbe Seiten, Stadt- oder Vereinsseiten). Keine gekauften oder erfundenen Einträge | Hoch |

### KI-Sichtbarkeit messen (manuell, monatlich)

Diese Anfragen je 3–5 Mal in ChatGPT, Perplexity und Google prüfen und notieren, ob die Praxis genannt wird und welche Quelle zitiert ist:

- „Faszienbehandlung Schramberg“
- „Gantke Faszienbehandlung in der Nähe von Rottweil“
- „Was kostet eine Gantke Faszienbehandlung?“
- „Faszienbehandlung Sulgen Abendtermin“
- „Was ist die Gantke Methode?“
- „Wer bietet Faszienbehandlung mit Hausbesuch in Dunningen an?“

Ergebnisse als Quote festhalten (z. B. „genannt 2/5“), nicht als einzelnen Lauf.

## Maßnahmenplan

**Sofort (kritisch)**
1. `/llms.txt` korrigieren: Heilversprechen entfernen, Buchungslink auf `/buchen`, Adresse angleichen (G1–G3).

**Hoher Nutzen**
2. Redirect `www` → ohne www in Cloudflare (T1).
3. Unternehmens-Schema vervollständigen (S1, S2).
4. Themenseiten ausbauen: Inhalt, Antwortblock, Fragen und Antworten, Datum, Titel und H1 (O2–O4, G6, G7).
5. Google-Unternehmensprofil und Bewertungslink pflegen (G9).

**Schnelle Gewinne**
6. H1 der Startseite (O1), OG-Tags auf den Themenseiten (O5), `llms.txt` ergänzen und verlinken (G4, G5), `Service`- und `BreadcrumbList`-Schema (S4, O6).

**Langfristig**
7. Qualifikation belegen (G8), monatliche KI-Sichtbarkeitsprüfung, Ortsseiten nur für tatsächlich bediente Orte mit echtem Inhalt.

Inhaltliche Änderungen vor Veröffentlichung wie bisher auf Heilversprechen und rechtliche Zulässigkeit prüfen lassen.
