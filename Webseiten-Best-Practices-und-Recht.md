# Webseiten: Best Practices, rechtliche Anforderungen und praktische Checklisten

**Stand:** 9. Oktober 2026  
**Rechtlicher Schwerpunkt:** Deutschland und Europäische Union  
**Zielgruppe:** Website-Betreiber, Selbstständige, Unternehmen, Designer, Entwickler, Agenturen und Projektverantwortliche  
**Anwendungsbereich:** Unternehmenswebsites, Portfolios, Blogs, Landingpages, Onlineshops und Webanwendungen

> Dieser Leitfaden verbindet allgemeine Qualitätsstandards mit einem Überblick über rechtliche Prüffelder. Die rechtlichen Pflichten hängen unter anderem von Betreiber, Zielmarkt, Branche, Funktionen und Geschäftsmodell ab. Er ist keine individuelle Rechtsberatung und ersetzt keine Prüfung des konkreten Angebots. Empfehlungen, Beispielwerte und organisatorische Vorschläge sind als solche gekennzeichnet. Quellen stehen bei den jeweiligen Themen und gesammelt am Ende.

## Inhaltsverzeichnis

1. [Was Best Practices sind](#kapitel-01)
2. [Ziele, Zielgruppen und Planung](#kapitel-02)
3. [Informationsarchitektur und Navigation](#kapitel-03)
4. [Gestaltung, Typografie und visuelle Hierarchie](#kapitel-04)
5. [Mobile Nutzung und responsive Gestaltung](#kapitel-05)
6. [Inhalte, Sprache und Vertrauen](#kapitel-06)
7. [Formulare, Interaktionen und Nutzerführung](#kapitel-07)
8. [Barrierefreiheit in der Umsetzung](#kapitel-08)
9. [Performance und Ladegeschwindigkeit](#kapitel-09)
10. [SEO und Auffindbarkeit](#kapitel-10)
11. [Technische Architektur und Entwicklung](#kapitel-11)
12. [Sicherheit und Schutz vor Missbrauch](#kapitel-12)
13. [Datenschutz als technisches und organisatorisches Konzept](#kapitel-13)
14. [Impressum und Anbieterkennzeichnung](#kapitel-14)
15. [Datenschutzerklärung und Datenschutzpflichten](#kapitel-15)
16. [Cookies, Tracking und externe Dienste](#kapitel-16)
17. [Gesetzliche Barrierefreiheit](#kapitel-17)
18. [Urheberrecht, Bilder, Marken und Lizenzen](#kapitel-18)
19. [Onlineshops, Verträge und Verbraucherrechte](#kapitel-19)
20. [Werbung, Newsletter und Bewertungen](#kapitel-20)
21. [Weitere rechtliche Prüffelder und KI](#kapitel-21)
22. [Hosting, Domains und Betrieb](#kapitel-22)
23. [Qualitätssicherung, Tests und Abnahme](#kapitel-23)
24. [Projektablauf, Übergabe und Verantwortlichkeiten](#kapitel-24)
25. [Prioritäten und Entscheidungshilfen](#kapitel-25)
26. [Ausführliche Checklisten](#kapitel-26)
27. [Vorlagen für die eigene Dokumentation](#kapitel-27)
28. [Häufige Irrtümer](#kapitel-28)
29. [Glossar](#kapitel-29)
30. [Quellen und weiterführende Grundlagen](#kapitel-30)

---

<a id="kapitel-01"></a>
## 1. Was Best Practices sind

Best Practices sind bewährte Vorgehensweisen, die unter typischen Bedingungen zu guten Ergebnissen führen. Bei Webseiten bedeutet das: Menschen finden Informationen, können Aufgaben erledigen, verstehen das Angebot und können sich auf die technische Umsetzung verlassen.

Eine Website ist kein fertiges Qualitätsprodukt, nur weil sie schön aussieht oder einen hohen automatisierten Prüfwert erreicht. Ihre Qualität zeigt sich im tatsächlichen Gebrauch: Kann jemand am Smartphone einen Termin vereinbaren? Lässt sich ein Formular mit der Tastatur bedienen? Kommt eine Bestellung tatsächlich an? Werden private Informationen geschützt? Bleibt das Angebot nach einem Update funktionsfähig?

### 1.1 Vier Arten von Anforderungen unterscheiden

| Kategorie | Bedeutung | Beispiel |
|---|---|---|
| Gesetzliche Pflicht | Bei erfülltem Anwendungsbereich verbindlich | Anbieterinformationen, Datenschutzpflichten, Verbraucherinformationen |
| Technischer Standard | Definierte Regeln oder Prüfkriterien | Semantisches HTML, WCAG-Erfolgskriterien |
| Best Practice | Bewährte Empfehlung, situationsabhängig | Gastbestellung, kurze Formulare, lokale Schriftdateien |
| Projektziel | Für das konkrete Projekt vereinbart | Formularabschluss in unter drei Minuten, Wiederanlauf innerhalb von vier Stunden |

Diese Kategorien dürfen nicht vermischt werden. Ein empfohlenes Schriftmaß ist keine gesetzliche Pflicht. Ein Cookie-Banner ist nicht automatisch notwendig. Eine Datenschutzpflicht verschwindet nicht, weil ein Besucher die Website kostenlos nutzt.

### 1.2 Die wichtigsten Qualitätsdimensionen

- **Nutzen:** Die Website beantwortet echte Fragen und erfüllt konkrete Aufgaben.
- **Verständlichkeit:** Angebot, Kosten und nächste Schritte sind klar.
- **Bedienbarkeit:** Navigation und Funktionen arbeiten vorhersehbar.
- **Zugänglichkeit:** Menschen mit unterschiedlichen Fähigkeiten können das Angebot nutzen.
- **Geschwindigkeit:** Inhalte und Interaktionen reagieren zügig.
- **Sicherheit:** Zugänge, Systeme und Daten sind angemessen geschützt.
- **Datenschutz:** Datenverarbeitung bleibt nachvollziehbar und auf das Erforderliche begrenzt.
- **Rechtliche Sorgfalt:** Die tatsächlichen Funktionen werden rechtlich bewertet.
- **Wartbarkeit:** Inhalte, Software und Verträge können dauerhaft gepflegt werden.
- **Messbarkeit:** Verbesserungen beruhen auf Beobachtungen und belastbaren Kennzahlen.

### 1.3 Best Practices brauchen Kontext

Ein Museum, ein lokaler Handwerksbetrieb und eine Banking-Webanwendung benötigen unterschiedliche Funktionen. Eine einfache Informationsseite kann mit wenigen statischen Dateien hervorragend funktionieren. Ein komplexer Kundenbereich braucht dagegen ein sorgfältiges Berechtigungs-, Sicherheits- und Betriebskonzept.

**Entscheidungsregel:** Jede zusätzliche Funktion sollte einen erkennbaren Nutzen haben, einen Verantwortlichen besitzen und ihren Aufwand für Datenschutz, Sicherheit, Performance und Wartung rechtfertigen.

<a id="kapitel-02"></a>
## 2. Ziele, Zielgruppen und Planung

### 2.1 Vor dem Design die Aufgabe klären

Beantworte vor Projektbeginn mindestens diese Fragen:

1. Was bietet die Website an?
2. Wer soll sie nutzen?
3. Welche drei bis fünf Aufgaben sind für diese Menschen besonders wichtig?
4. Welche Informationen brauchen sie vor einer Entscheidung?
5. Welcher nächste Schritt ist sinnvoll?
6. Welche Daten und externen Dienste sind dafür wirklich erforderlich?
7. Wer pflegt Inhalte, bearbeitet Anfragen und betreibt die Technik?

Eine gute Zielbeschreibung lautet beispielsweise: „Interessenten verstehen unsere Leistungen, sehen unser Einsatzgebiet und können eine Anfrage mit Fotos senden.“ Weniger brauchbar ist: „Wir brauchen eine moderne Website mit vielen Animationen.“

### 2.2 Aufgaben statt bloßer Besucherzahlen messen

Geeignete Kennzahlen hängen vom Geschäftsmodell ab:

| Website-Typ | Sinnvolle Kennzahl | Ergänzende Beobachtung |
|---|---|---|
| Dienstleister | Qualifizierte Anfragen | Anteil unpassender Anfragen, Bearbeitungszeit |
| Shop | Erfolgreiche Bestellungen | Checkout-Abbrüche, Zahlungsfehler, Rücksendungsgründe |
| Wissensportal | Gefundene Antworten | Suchanfragen ohne Treffer, wiederholte Supportfragen |
| Bewerbungssystem | Vollständig eingegangene Bewerbungen | Abbrüche und Upload-Probleme |
| Kundenportal | Erfolgreich erledigte Vorgänge | Fehlerquote, Supportkontakte, Zeitaufwand |

Eine hohe Klickzahl kann auch auf Verwirrung hinweisen. Messe deshalb Ergebnisse und Qualität, nicht nur Aktivität.

### 2.3 Umfang und Abhängigkeiten dokumentieren

Erstelle eine Übersicht der Seiten, Seitentypen und Funktionen. Bei einer Unternehmenswebsite können das Startseite, Leistungsübersicht, Leistungsdetails, Über-uns-Seite, Referenzen, Kontakt und rechtliche Seiten sein. Ein Shop benötigt zusätzlich Produktdaten, Warenkorb, Checkout, Zahlungen, Versand, Widerruf und gegebenenfalls weitere Verbraucherfunktionen.

Dokumentiere auch unsichtbare Aufgaben: Mailversand, Spam-Abwehr, Datensicherung, Zertifikate, Rollen, Datenlöschung und Überwachung. Diese Aufgaben brauchen ebenso Budget wie das sichtbare Design.

### 2.4 Mit realistischen Nutzungssituationen planen

Berücksichtige langsame Verbindungen, ältere Geräte, helle Umgebung, kleine Displays, vergrößerte Schrift, Tastaturnutzung und Menschen, die die Fachsprache nicht kennen. Ein Kunde kann mitten im Bestellvorgang unterbrochen werden. Eine Bewerberin kann ihre Dateien nur am Smartphone verfügbar haben.

**Praktischer Start:** Schreibe fünf typische Nutzungsszenarien auf und prüfe sie bereits mit einer einfachen Skizze. So werden Probleme sichtbar, bevor ihre Behebung teuer wird.

<a id="kapitel-03"></a>
## 3. Informationsarchitektur und Navigation

### 3.1 Inhalte nach Nutzerfragen ordnen

Die Struktur sollte sich daran orientieren, was Menschen suchen: Leistungen, Preise, Produkte, Einsatzgebiet, Ansprechpartner und Hilfe. Die interne Organisationsstruktur eines Unternehmens ist dafür selten das beste Vorbild.

Benutze verständliche Menünamen. „Leistungen“ oder „Preise“ sind oft klarer als kreative Begriffe wie „Unsere Welt“. Fachbegriffe sind sinnvoll, wenn die Zielgruppe sie tatsächlich verwendet.

### 3.2 Hauptnavigation übersichtlich halten

- Die wichtigsten Bereiche erhalten sichtbare Einstiege.
- Ähnliche Inhalte werden sinnvoll zusammengefasst.
- Die aktive Seite ist erkennbar.
- Das Logo führt zuverlässig zur Startseite.
- Untermenüs funktionieren mit Tastatur und Touch.
- Menüeinträge behalten auf vergleichbaren Seiten dieselbe Bedeutung.
- Bei tieferen Strukturen helfen Breadcrumbs als ergänzende Orientierung.

Eine feste Zahl maximal erlaubter Menüpunkte ist keine allgemeine Regel. Entscheidend sind verständliche Gruppierung und Auffindbarkeit. Die häufig zitierte „Drei-Klick-Regel“ ersetzt keine Nutzerprüfung.

### 3.3 Suche nur dort einsetzen, wo sie hilft

Bei großen Katalogen und Wissensbeständen kann eine Suche wichtiger sein als ein umfangreiches Menü. Bei fünf kurzen Seiten ist sie meist überflüssig.

Eine brauchbare Suche bietet relevante Treffer, verständliche Seitentitel, klare Filter und hilfreiche Hinweise bei erfolglosen Anfragen. Suchbegriffe können persönliche Informationen enthalten; plane ihre Protokollierung daher bewusst.

### 3.4 Jede Seite braucht einen klaren Zweck

Für jede wichtige Seite sollten diese Punkte feststehen:

- Welche Frage beantwortet sie?
- Welche Informationen gehören zwingend darauf?
- Was ist der passende nächste Schritt?
- Welche verwandten Inhalte unterstützen diesen Schritt?
- Wer ist für die Aktualität verantwortlich?

Eine Leistungsseite sollte beispielsweise erklären, was enthalten ist, für wen das Angebot passt, welche Grenzen bestehen und wie eine Anfrage abläuft.

### 3.5 Fehlerseiten und Sackgassen vermeiden

Eine 404-Seite erklärt, dass die gewünschte Seite nicht gefunden wurde, und bietet Navigation oder Suche an. Sie muss technisch mit dem passenden Fehlerstatus ausgeliefert werden. Leere Suchergebnisse und ausverkaufte Produkte sollten Alternativen anbieten, ohne den eigentlichen Zustand zu verschleiern.

Bei entfernten Inhalten ist zu entscheiden, ob ein passender Nachfolger existiert. Eine pauschale Weiterleitung aller gelöschten Seiten auf die Startseite hilft Besuchern meistens wenig.

<a id="kapitel-04"></a>
## 4. Gestaltung, Typografie und visuelle Hierarchie

### 4.1 Hierarchie vor Dekoration

Die wichtigsten Inhalte sollen zuerst auffallen: Was wird angeboten, welchen Nutzen hat es, welche Informationen sind relevant und welcher nächste Schritt ist möglich? Farbe, Größe, Position und Abstand unterstützen diese Reihenfolge.

Wenn jede Fläche gleich auffällig ist, verliert die Seite Orientierung. Verwende Akzentfarben gezielt und lasse genügend Abstand zwischen unabhängigen Inhalten.

### 4.2 Ein konsistentes Gestaltungssystem verwenden

Definiere wiederverwendbare Regeln für:

- Farben und Kontraste;
- Schriftfamilien, Schriftgrößen und Zeilenabstände;
- Abstände, Inhaltsbreiten und Raster;
- Buttons, Links und Formularfelder;
- Hinweise, Fehler, Warnungen und Erfolgsmeldungen;
- Karten, Tabellen, Navigation und Dialoge;
- Fokus-, Hover-, Lade- und deaktivierte Zustände.

Diese Regeln erleichtern spätere Änderungen. Wenn ein Formularfeld verbessert wird, sollte die Verbesserung alle Formulare erreichen.

### 4.3 Lesbarkeit praktisch prüfen

Als **Ausgangspunkt**, nicht als verpflichtende Norm, eignen sich für viele textorientierte Websites etwa 16–18 CSS-Pixel Fließtext, ein Zeilenabstand um 1,5 und eine Textbreite von ungefähr 45–80 Zeichen. Die geeigneten Werte hängen von Schrift, Sprache, Display und Inhalt ab.

Vermeide sehr kleine Schrift, überlange Textzeilen, lange Absätze in Großbuchstaben und dünne Schrift auf schwachem Hintergrund. Prüfe echte Texte: Namen, lange deutsche Wörter, Preise, Zahlen und mehrzeilige Überschriften.

### 4.4 Farben und Zustände müssen verständlich sein

Ein Fehler darf nicht nur durch Rot erkennbar sein. Ergänze eine Nachricht oder ein Symbol mit verständlicher Bedeutung. Links sollten als interaktive Elemente erkennbar bleiben. Ein deaktivierter Button braucht gegebenenfalls eine Erklärung, wie er aktiviert werden kann.

Gestalte Fokuszustände genauso bewusst wie Hoverzustände. Menschen, die mit Tastatur navigieren, müssen jederzeit sehen können, wo sie sind.

### 4.5 Animationen sparsam und zweckgebunden einsetzen

Animation kann Zustandswechsel erklären oder eine Handlung bestätigen. Sie sollte Bedienung und Lesen nicht verzögern. Vermeide automatisch laufende Effekte, die Aufmerksamkeit dauerhaft binden, und respektiere die Einstellung für reduzierte Bewegung.

Ein Seitenwechsel muss nicht durch eine lange Animation angekündigt werden. Bewegte Hintergründe dürfen den Textkontrast nicht wechselnd verschlechtern.

### 4.6 Vertrauen durch konkrete Gestaltung stärken

Echte Ansprechpartner, verständliche Prozesse, nachvollziehbare Preise und überprüfbare Referenzen schaffen Vertrauen. Dekorative Siegel, erfundene Zahlen und anonyme Testimonials ohne Herkunft können das Gegenteil bewirken.

Zeige Qualität mit relevanten Details: Beispiele, Leistungsumfang, Qualifikationen und verlässliche Kontaktwege. Behauptungen sollten belegt werden können.

<a id="kapitel-05"></a>
## 5. Mobile Nutzung und responsive Gestaltung

### 5.1 Kleine Bildschirme von Anfang an berücksichtigen

„Mobile first“ bedeutet, Inhalt und Bedienung bereits für enge Platzverhältnisse sinnvoll zu planen. Es bedeutet nicht, dass Desktop-Nutzung unwichtig ist. Beide müssen ihre Aufgaben gut erfüllen können.

Prüfe die Website auch in Zwischenbreiten und im Querformat. Ein Layout darf nicht nur auf einem schmalen Smartphone und einem großen Monitor funktionieren.

### 5.2 Inhalte dürfen nicht abgeschnitten werden

- Text muss umbrechen können.
- Bilder skalieren innerhalb ihrer Container.
- Tabellen brauchen eine verständliche Darstellung auf schmalen Displays.
- Dialoge bleiben vollständig bedienbar.
- Navigation und Formularfelder passen zu vergrößerter Schrift.
- Wichtige Funktionen sind ohne Hover erreichbar.
- Fixierte Elemente verdecken weder Inhalte noch Eingabefelder.

Breite technische Tabellen können horizontal scrollbar sein, wenn die Zusammenhänge eine zweidimensionale Darstellung erfordern. Die gesamte Seite sollte dadurch nicht unbeabsichtigt seitlich scrollen.

### 5.3 Touch-Bedienung erleichtern

Großzügige Klickflächen und ausreichender Abstand reduzieren Fehlbedienung. Kleine Symbole können eine größere unsichtbare Bedienfläche bekommen. Als Komfortziel sind häufig etwa 44 × 44 CSS-Pixel sinnvoll; die genaue WCAG-Anforderung samt Ausnahmen wird im Barrierefreiheitskapitel erläutert.

Ziehgesten sollten eine Alternative haben, etwa Plus-/Minus-Schaltflächen oder eine Auswahl. Wichtige Handlungen brauchen eine verlässliche Bestätigung, wenn ein Fehlklick erhebliche Folgen hätte.

### 5.4 Formulare auf echten Geräten testen

Passende Eingabetypen erleichtern die Nutzung: E-Mail-Tastatur bei E-Mail-Adressen, Telefonnummerneingabe bei Telefonnummern und passende Autovervollständigung für Adressdaten. Eine Telefonnummer ist kein mathematischer Zahlenwert; führende Nullen und internationale Vorwahlen müssen erhalten bleiben.

Prüfe, ob die Bildschirmtastatur den nächsten Schritt verdeckt, ob Autofill Daten korrekt zuordnet und ob Dateiauswahl mit Kamera und Galerie funktioniert.

<a id="kapitel-06"></a>
## 6. Inhalte, Sprache und Vertrauen

### 6.1 Die wichtigsten Fragen früh beantworten

Ein neuer Besucher sollte schnell erkennen:

1. Was wird angeboten?
2. Für wen ist es geeignet?
3. Welches Problem löst es?
4. Welche Kosten oder Voraussetzungen gibt es?
5. Wie geht es weiter?

Beginne mit der Antwort. Firmenhistorie und ausführliche Selbstdarstellung können später folgen, wenn sie für die Entscheidung relevant sind.

### 6.2 Verständlich und konkret schreiben

Nutze kurze, eindeutige Sätze und erläutere Fachbegriffe. „Wir reparieren Ihre Heizung im Raum Köln“ ist informativer als „Wir liefern innovative Lösungen mit ganzheitlichem Ansatz“.

Trenne Versprechen, Fakten und Bedingungen. Bei einer Aussage wie „Antwort innerhalb eines Werktags“ müssen Team und Prozess dieses Versprechen tragen können.

### 6.3 Gute Texte haben eine erkennbare Struktur

Verwende aussagekräftige Überschriften, kurze Absätze, Listen für parallele Informationen und Tabellen für Vergleiche. Vermeide Text in Bildern, wenn die Information auch als echter Text verfügbar sein kann.

Linktexte beschreiben das Ziel: „Preise für Wartungsverträge“ ist klarer als mehrfaches „Hier klicken“. Buttontexte beschreiben eine Handlung: „Anfrage senden“ ist klarer als „Weiter“, wenn tatsächlich eine Anfrage verschickt wird.

### 6.4 Inhalte regelmäßig pflegen

Jede wichtige Seite sollte einen Verantwortlichen und einen Prüftermin haben. Besonders anfällig für Veraltung sind Preise, Öffnungszeiten, Ansprechpartner, Stellenangebote, Lieferzeiten, Softwareanleitungen und rechtliche Texte.

Ein sichtbares Aktualisierungsdatum ist nur sinnvoll, wenn die Aktualisierung inhaltlich stattgefunden hat. Automatisch das heutige Datum einzusetzen schafft keine Aktualität.

### 6.5 Medien sinnvoll einsetzen

Ein Bild sollte erklären, belegen oder Orientierung geben. Prüfe Bildrechte, erkennbare Personen, Dateigröße und benötigte Beschreibung. Videos brauchen je nach Inhalt Untertitel und weitere zugängliche Alternativen.

Anleitungen und wichtige Informationen sollten nicht ausschließlich als PDF angeboten werden. Ein zugänglicher HTML-Text ist häufig einfacher nutzbar, besser durchsuchbar und leichter aktuell zu halten.

### 6.6 Mehrsprachigkeit planen

Übersetze Navigation, Formulare, Fehlermeldungen, Bestätigungen und relevante Rechtstexte konsistent. Prüfe Zahlenformate, Datumsangaben, Währungen und Textlängen. Eine Sprachauswahl sollte Sprache und Land nicht durch Flaggen allein gleichsetzen.

Kennzeichne die Dokument- und gegebenenfalls Abschnittssprache technisch. Automatische Übersetzungen sollten bei entscheidenden Inhalten fachlich kontrolliert werden.

<a id="kapitel-07"></a>
## 7. Formulare, Interaktionen und Nutzerführung

### 7.1 Nur notwendige Angaben abfragen

Jedes zusätzliche Feld kostet Aufmerksamkeit und erhöht die Abbruchwahrscheinlichkeit. Frage nur Daten ab, die für die Aufgabe benötigt werden. Für eine einfache Rückfrage ist eine Postanschrift häufig unnötig.

Pflichtfelder müssen erkennbar sein. Erläutere ungewöhnliche Anforderungen, beispielsweise warum ein Geburtsdatum benötigt wird. Freitextfelder sollten keine sensiblen Angaben unnötig nahelegen.

### 7.2 Felder verständlich beschriften

Jedes Feld braucht eine dauerhaft sichtbare, technisch zugeordnete Beschriftung. Ein Platzhalter allein reicht nicht: Er verschwindet beim Tippen und kann schwer lesbar sein.

Ergänze Beispiele nur, wenn sie helfen. Für ein Datum ist eine Formatangabe sinnvoll. Für einen Namen sind starre Regeln wie „nur Buchstaben ohne Leerzeichen“ oft ungeeignet.

### 7.3 Eingabefehler hilfreich behandeln

Eine gute Fehlermeldung beantwortet drei Fragen: Was ist falsch? Wo ist es falsch? Wie kann es korrigiert werden?

Beispiel: „Bitte geben Sie eine E-Mail-Adresse im Format name@beispiel.de ein.“ Schlechter: „Ungültige Eingabe“.

Nach einem Fehler bleiben bereits eingegebene Daten erhalten, soweit das sicher und angemessen ist. Bei mehreren Fehlern hilft eine Zusammenfassung mit Links zu den Feldern. Neue Fehlermeldungen müssen auch für assistive Technik erkennbar sein.

### 7.4 Alle Zustände gestalten

Plane für jede zentrale Funktion:

- Ausgangszustand;
- Eingabe und Fokus;
- Verarbeitung;
- Erfolg;
- fachlichen Fehler;
- technischen Fehler;
- leeren Zustand;
- fehlende Berechtigung;
- abgelaufene Sitzung;
- wiederholte oder verspätete Anfrage.

Ein Ladeindikator ohne Ende ist keine brauchbare Fehlerbehandlung. Gib bei längerem Warten Rückmeldung und einen nachvollziehbaren nächsten Schritt.

### 7.5 Erfolg muss den tatsächlichen Prozess widerspiegeln

„Nachricht gesendet“ darf erst erscheinen, wenn die Anfrage serverseitig angenommen wurde. „Termin gebucht“ und „Terminanfrage erhalten“ sind unterschiedliche Aussagen.

Bei wichtigen Vorgängen sind Referenznummer, Zusammenfassung und passende Bestätigung sinnvoll. Prüfe den vollständigen Weg bis zum zuständigen Team. Ein funktionierendes Formular, dessen E-Mails im Spam landen, erfüllt seinen Zweck nicht.

### 7.6 Datenschutzinformation und Einwilligung auseinanderhalten

Eine Datenschutzerklärung informiert. Sie muss bei einem gewöhnlichen Kontaktformular nicht pauschal durch eine verpflichtende „Ich akzeptiere den Datenschutz“-Checkbox in eine Einwilligung verwandelt werden. Die Rechtsgrundlage der Bearbeitung hängt vom Anliegen ab.

Wenn zusätzlich Newsletterwerbung gewünscht ist, braucht diese einen getrennten, eindeutig beschriebenen Ablauf. Eine Kontaktanfrage darf nicht stillschweigend zum Newsletterabo werden.

### 7.7 Faire Nutzerführung

Vermeide vorangekreuzte Zusatzleistungen, versteckte Kosten, irreführende Knöpfe, künstliche Countdown-Timer und erschwerte Abmeldung. Gute Nutzerführung erleichtert eine informierte Entscheidung und hält Versprechen ein.

Wenn ein Vorgang Daten löscht oder Kosten auslöst, zeige die Konsequenz rechtzeitig und eindeutig. Die rechtlichen Anforderungen an Bestell-, Kündigungs- und Widerrufsfunktionen stehen in Kapitel 19.

<a id="kapitel-08"></a>
## 8. Barrierefreiheit in der Umsetzung

Barrierefreiheit berücksichtigt unter anderem visuelle, motorische, auditive und kognitive Einschränkungen. Viele Maßnahmen helfen zugleich bei vorübergehenden Einschränkungen, schlechter Umgebung oder ungewohnter Technik.

### 8.1 Ein überprüfbares Ziel festlegen

**Empfohlenes technisches Projektziel:** WCAG 2.2 auf Stufe AA, sofern keine abweichenden oder ergänzenden Anforderungen gelten. WCAG ist ein Kriterienkatalog; rechtliche Verpflichtungen ergeben sich aus den jeweils anwendbaren Gesetzen und Normbezügen.

WCAG 2.2 AA allein ist kein pauschaler Nachweis sämtlicher BFSG-, BITV- oder EN-301-549-Anforderungen. Definiere für ein gesetzlich erfasstes Angebot eine gesonderte Anforderungsmatrix.

Quellen: [WCAG 2.2](https://www.w3.org/TR/WCAG22/) und [WAI-Referenz mit Kriterien und Techniken](https://www.w3.org/WAI/WCAG22/quickref/).

### 8.2 Semantisches HTML als Grundlage

Verwende Überschriften für die Inhaltsstruktur, Links für Navigation, Buttons für Aktionen und echte Formularelemente für Eingaben. Bereiche wie Hauptinhalt und Navigation sollten technisch erkennbar sein.

Ein optisch klickbares `div` ersetzt einen Button nicht ohne zusätzliche Arbeit an Tastaturbedienung, Rolle, Fokus und Zuständen. Native Elemente ersparen Fehler und erleichtern Wartung.

### 8.3 Tastaturbedienung und Fokus

Alle zentralen Funktionen müssen ohne Maus bedienbar sein. Teste Tabulator, Umschalt+Tabulator, Eingabetaste, Leertaste und Escape dort, wo sie zum jeweiligen Element passen.

Ein Dialog braucht einen verständlichen Namen, sinnvolle Fokusführung und eine erreichbare Schließfunktion. Nach dem Schließen sollte der Fokus an einen passenden Ausgangspunkt zurückkehren. Fixierte Kopfzeilen und Banner dürfen fokussierte Elemente nicht verdecken.

### 8.4 Kontraste, Größe und Vergrößerung

Für WCAG-AA-Textkontrast gelten grundsätzlich mindestens **4,5:1** für normalen Text und **3:1** für großen Text; für bestimmte Nicht-Text-Elemente gilt ebenfalls ein Verhältnis von **3:1**. Die Definitionen und Ausnahmen stehen im Standard.

WCAG 2.2 AA verlangt für Zeigerziele grundsätzlich mindestens **24 × 24 CSS-Pixel** oder eine der vorgesehenen Alternativen bzw. Ausnahmen. Größere Ziele sind häufig angenehmer.

Prüfe Textvergrößerung und Umbruch. Ein Test bei 200 % Textvergrößerung und Reflow in einer Breite von 320 CSS-Pixeln deckt viele Probleme auf. Eine bloße feste Mindestbreite für die gesamte Seite verhindert diesen Umbruch.

Quelle: [WCAG-Quickref, insbesondere Kontrast, Reflow, Fokus und Zielgröße](https://www.w3.org/WAI/WCAG22/quickref/).

### 8.5 Bilder und Medien

Informative Bilder brauchen eine Beschreibung, die ihren Zweck im Kontext wiedergibt. Rein dekorative Bilder erhalten einen leeren Alternativtext. Ein Diagramm benötigt gegebenenfalls eine ausführliche Erklärung oder eine Datentabelle.

Bei Videos und Audio ist zu prüfen, welche Informationen visuell bzw. akustisch vermittelt werden und welche Alternativen erforderlich sind. Ein automatisch erzeugter Untertitel sollte auf Namen, Fachwörter und Sinnfehler kontrolliert werden.

### 8.6 Formulare und Authentifizierung

Beschriftungen, Hilfen und Fehler werden technisch mit den jeweiligen Feldern verbunden. Autovervollständigung und Passwortmanager sollen funktionieren. Authentifizierung sollte nicht unnötig ausschließlich auf Gedächtnisaufgaben, Bilderrätseln oder manueller Übertragung beruhen.

Eine barrierearme Spam-Abwehr kann mit abgestuften Maßnahmen arbeiten: serverseitige Begrenzung, unauffällige Fallen für einfache Bots und alternative Kontaktwege. Prüfe jede Lösung auf unerwünschte Sperren legitimer Nutzer.

### 8.7 Automatische und manuelle Prüfungen kombinieren

Automatische Werkzeuge erkennen einen Teil der Fehler. Sie beurteilen nicht zuverlässig, ob Alternativtexte sinnvoll sind, eine Navigation verständlich ist oder ein vollständiger Checkout mit Screenreader funktioniert.

**Empfohlene Abnahme:** repräsentative Seitentypen automatisch prüfen, Kernabläufe vollständig per Tastatur testen, Screenreader-Stichproben durchführen und bei relevantem Risiko Menschen mit Behinderungen in Nutzungstests einbeziehen. Befunde und Nachprüfungen dokumentieren.

<a id="kapitel-09"></a>
## 9. Performance und Ladegeschwindigkeit

### 9.1 Geschwindigkeit als Nutzererlebnis messen

Eine Website kann schnell mit dem Download beginnen und trotzdem schlecht reagieren. Unterscheide daher Ladezeit, Interaktionsgeschwindigkeit und visuelle Stabilität.

Die Core Web Vitals bewerten diese Aspekte mit folgenden „gut“-Grenzwerten am 75. Perzentil realer Besuche:

| Metrik | Was sie beschreibt | Guter Wert |
|---|---|---|
| LCP – Largest Contentful Paint | Zeitpunkt, zu dem ein großer relevanter Inhalt sichtbar wird | höchstens 2,5 Sekunden |
| INP – Interaction to Next Paint | Reaktion auf Interaktionen im Verlauf eines Besuchs | höchstens 200 Millisekunden |
| CLS – Cumulative Layout Shift | Unerwartete Layoutverschiebungen | höchstens 0,1 |

Quelle: [Google/web.dev: Web Vitals](https://web.dev/articles/vitals).

Ein einzelner Labortest ersetzt keine Messung realer Nutzung. Ein hoher Lighthouse-Wert garantiert auch keine guten Formulare, rechtliche Konformität oder zufriedenstellende Inhalte.

### 9.2 Die größten Ursachen zuerst beheben

Häufige Ansatzpunkte sind zu große Bilder, automatisch gestartete Videos, viele Drittanbieter-Skripte, unnötiges JavaScript, langsame Datenbankabfragen, fehlendes Caching und übermäßig viele Schriftdateien.

Miss die wichtigen Seitentypen: Startseite, Produktseite, Suche, Warenkorb und Checkout können sehr unterschiedliche Engpässe haben.

### 9.3 Bilder und Videos optimieren

Liefere passende Bildgrößen für das jeweilige Gerät aus. Verwende geeignete moderne Formate und kontrolliere die sichtbare Qualität. Lege Bildabmessungen fest, damit Platz vor dem Download reserviert wird.

Lade Medien unterhalb des sichtbaren Bereichs bei Bedarf nach. Das zentrale erste Bild sollte dadurch nicht unnötig verzögert werden. Große Hintergrundvideos benötigen eine belastbare Begründung, ein angemessenes Datenbudget und eine reduzierte Alternative.

### 9.4 JavaScript und Schriften begrenzen

Eine statische Textseite benötigt nicht automatisch ein großes Frontend-Framework. Lade Funktionen dann, wenn sie gebraucht werden. Entferne ungenutzte Bibliotheken und überprüfe den Nutzen jedes eingebundenen Widgets.

Beschränke Schriften auf benötigte Schnitte und Zeichensätze. Prüfe, ob der Wechsel von Ersatzschrift zu Webfont das Layout verschiebt. Lokale Auslieferung kann Datenschutz und Ausfallsicherheit vereinfachen, setzt aber passende Nutzungsrechte voraus.

### 9.5 Server, Cache und Auslieferung

Kompression, gezieltes Caching und schnelle Datenbankzugriffe können viel bewirken. Ein CDN ist hilfreich, wenn es zum Nutzerkreis und Datenmodell passt.

Cache-Regeln müssen private und öffentliche Inhalte unterscheiden. Ein Kundenkonto darf nicht versehentlich als öffentliche Seite zwischengespeichert und einem anderen Nutzer angezeigt werden.

### 9.6 Performancebudgets festlegen

**Beispielhafte interne Projektziele:** keine automatischen Videos auf Mobilgeräten, begrenzte Zahl externer Skripte und ein vereinbartes Gesamtgewicht zentraler Informationsseiten. Konkrete Zahlen sind passend zum Inhalt festzulegen; es gibt kein universell sinnvolles Kilobyte-Limit.

Jede neue Einbindung wird gegen das Budget geprüft. Ein Chatwidget, ein Werbepixel und ein Terminmodul können einzeln klein wirken und gemeinsam erhebliche Verzögerungen verursachen.

<a id="kapitel-10"></a>
## 10. SEO und Auffindbarkeit

### 10.1 Mit hilfreichen Inhalten beginnen

Suchmaschinenoptimierung soll relevante Inhalte auffindbar machen. Eine klare Antwort auf eine echte Suchfrage ist wertvoller als künstlich wiederholte Schlüsselwörter.

Beschreibe Leistungen und Produkte konkret, halte Informationen aktuell und mache Herkunft und Verantwortlichkeit bei fachlichen Inhalten nachvollziehbar.

Quelle für die technischen Grundlagen: [Google Search Central: SEO-Startleitfaden](https://developers.google.com/search/docs/fundamentals/seo-starter-guide?hl=de).

### 10.2 Technische Grundlagen

- Aussagekräftige und unterscheidbare Seitentitel.
- Logische Überschriften und echte Textinhalte.
- Crawlbare interne Links und verständliche URLs.
- Korrekte HTTP-Statuscodes.
- Passende Weiterleitungen nach Umzügen.
- Canonical-Angaben bei tatsächlich gleichwertigen Duplikaten.
- XML-Sitemap für relevante indexierbare Inhalte.
- Bewusst konfigurierte Indexierungsanweisungen.

Eine Sitemap garantiert keine Indexierung. Eine Meta-Description kann die Darstellung unterstützen, wird aber nicht zwingend unverändert angezeigt.

### 10.3 robots.txt ist kein Zugangsschutz

Eine `robots.txt` steuert kooperative Crawler. Sie verhindert nicht, dass ein Mensch oder ein anderer Client eine URL aufruft. Vertrauliche Inhalte brauchen Authentifizierung und Berechtigungsprüfungen.

Auch `noindex` ist kein Ersatz für Schutz privater Daten. Verhindere sensible öffentliche URLs bereits im Systemdesign.

### 10.4 Strukturierte Daten und lokale Auffindbarkeit

Strukturierte Daten sollten zu den sichtbaren Inhalten passen und die geltenden Richtlinien für den jeweiligen Typ erfüllen. Sie garantieren keine besondere Ergebnisdarstellung.

Lokale Unternehmen sollten Name, Kontakt, Anschrift, Einsatzgebiet und Öffnungszeiten konsistent pflegen. Mehrere nahezu identische Ortsseiten ohne örtlichen Mehrwert sind keine überzeugende Inhaltsstrategie.

### 10.5 Migrationen sorgfältig planen

Erfasse bestehende URLs vor einem Relaunch. Ordne wichtige alte Inhalte ihren neuen Zielen zu, kontrolliere Weiterleitungen und prüfe nach dem Wechsel Fehler, Indexierung und Anfrageaufkommen.

Eine technisch gelungene neue Website kann trotzdem Anfragen verlieren, wenn bewährte Inhalte verschwinden oder alte Einstiege ins Leere führen.

<a id="kapitel-11"></a>
## 11. Technische Architektur und Entwicklung

### 11.1 Die einfachste tragfähige Lösung wählen

Wähle Technik nach Anforderungen und Betriebsmöglichkeiten. Eine statische Website, ein klassisches CMS, ein Shop-System und eine individuelle Webanwendung lösen unterschiedliche Aufgaben.

Bewerte redaktionellen Bedarf, Integrationen, Rollen, Datenvolumen, Betriebskosten und vorhandene Fähigkeiten. Eine aufwendige Architektur ist kein Qualitätsbeweis.

### 11.2 Progressive Verbesserung

Wo möglich, funktionieren grundlegende Inhalte und Aktionen bereits mit einer robusten Basis. Zusätzliche Technik verbessert das Erlebnis. Berücksichtige verzögerte oder fehlerhafte Skripte und blockierte Drittanbieter.

Falls eine Funktion JavaScript zwingend benötigt, behandle dessen Ausfall nachvollziehbar. Eine leere Fläche ohne Erklärung hilft niemandem.

### 11.3 Inhalte und Komponenten strukturiert halten

Gemeinsame Komponenten sorgen für einheitliche Bedienung. Strukturierte Felder für Preise, Produktdaten und Ansprechpartner sind meist pflegeleichter als frei zusammengesetzte Textblöcke.

Dokumentiere wichtige Datenmodelle, Schnittstellen und fachliche Regeln. Verwende für kritische Vorgänge serverseitig geprüfte Zustände statt bloßer Anzeigeentscheidungen im Browser.

### 11.4 Entwicklungs-, Test- und Produktivumgebung trennen

Teste Änderungen vor der Veröffentlichung. Die Testumgebung braucht Zugangsschutz; reine Nichtindexierung genügt nicht. Verwende möglichst synthetische oder angemessen anonymisierte Testdaten.

Testzahlungen dürfen keine echten Käufe auslösen. Test-E-Mails dürfen nicht versehentlich an Kunden gehen. Prüfe, ob ein Testsystem Zugangsdaten zur Produktion besitzt, die es nicht benötigt.

### 11.5 Versionierung und Veröffentlichung

Änderungen sollten nachvollziehbar versioniert werden. Lege fest, wer veröffentlichen darf und wie eine fehlerhafte Version zurückgenommen wird. Automatisierte Prüfungen können zentrale Fehler vor der Freigabe erkennen.

Ein Rollback muss auch Änderungen an Daten und Datenbankschemata berücksichtigen. Nur alte Programmdateien zurückzukopieren reicht nicht immer.

### 11.6 Externe Abhängigkeiten begrenzen

Pflege eine Liste verwendeter Plugins, Bibliotheken und Dienste. Dokumentiere Zweck, Verantwortliche, Aktualisierung, Lizenz und Ersatzmöglichkeit. Entferne ungenutzte Komponenten.

Ein aufgegebenes Plugin kann zum Wartungsproblem werden, obwohl es heute noch funktioniert. Kalkuliere regelmäßige Pflege als Bestandteil des Produkts.

<a id="kapitel-12"></a>
## 12. Sicherheit und Schutz vor Missbrauch

### 12.1 Sicherheit nach Risiko planen

Bewerte, welche Daten und Funktionen besonders schützenswert sind und was bei Missbrauch passieren könnte. Ein öffentlicher Blog, ein Kundenportal und ein Shop benötigen unterschiedliche Prüftiefe.

Als strukturierte Grundlage eignet sich der [OWASP Application Security Verification Standard und sein Leitfaden](https://devguide.owasp.org/en/06-verification/01-guides/03-asvs/). Eine kurze Liste ersetzt keine umfassende Sicherheitsprüfung komplexer Anwendungen.

### 12.2 Transport, Administration und Geheimnisse

Setze durchgehend HTTPS ein und überwache Zertifikatsablauf. Schütze Administratorzugänge mit starken individuellen Zugangsdaten und möglichst Mehrfaktor-Authentifizierung oder passenden Passkeys.

Zugangsschlüssel, Datenbankpasswörter und private Tokens gehören weder in öffentlich ausgelieferte Dateien noch in ein öffentliches Repository. Verwende geeignete Geheimnisverwaltung und widerrufe kompromittierte Schlüssel.

Mitarbeiter erhalten eigene Konten. Bei Austritt werden Berechtigungen und aktive Sitzungen entzogen. Geteilte Administratorkonten erschweren Nachvollziehbarkeit und sichere Übergabe.

### 12.3 Berechtigungen auf dem Server durchsetzen

Prüfe bei jeder geschützten Anfrage, ob die aktuelle Person die konkrete Aktion auf dem konkreten Objekt ausführen darf. Ein ausgeblendeter Button schützt keine API.

**Beispiel:** Kunde A darf eine Rechnungsnummer im Aufruf nicht verändern können, um die Rechnung von Kunde B herunterzuladen. Unerratbare IDs können ergänzen, ersetzen aber keine Berechtigungsprüfung.

Quelle: [OWASP: Authorization](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html).

### 12.4 Passwörter und Kontowiederherstellung

Wenn Passwörter selbst gespeichert werden, verwende geeignete Passwort-Hashverfahren mit Salt und angemessenen Parametern; für neue Systeme ist Argon2id eine verbreitete Empfehlung. Klartext, einfache allgemeine Hashes oder reversible Speicherung sind dafür ungeeignet.

Rücksetzlinks sollten zeitlich begrenzt und nur einmal nutzbar sein. Konto-Wiederherstellung darf die Sicherheit des normalen Logins nicht umgehen. Passwortmanager, Einfügen und lange Passphrasen sollten unterstützt werden.

Quellen: [OWASP: Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) und [Authentication](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html).

### 12.5 Sitzungen und Cookies schützen

Sitzungen müssen bei Anmeldung und relevanten Sicherheitsänderungen angemessen erneuert werden. Logout und Kontosperrung müssen serverseitig wirken. Plane passende Ablaufzeiten und eine sichere erneute Anmeldung.

Für Sitzungscookies sind insbesondere `Secure`, `HttpOnly` und eine zum Ablauf passende `SameSite`-Einstellung zu prüfen. Behandle Cross-Site-Request-Forgery zusätzlich dort, wo es erforderlich ist; ein einzelnes Attribut ist kein vollständiges Sicherheitskonzept.

Quelle: [OWASP: Session Management](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html).

### 12.6 Eingaben, Ausgaben und Datenbankzugriffe

Serverseitige Eingabeprüfung schützt fachliche Regeln. Für SQL-Abfragen sind parametrisierte Abfragen statt zusammengebauter Abfragetexte zu verwenden. Daten aus Nutzereingaben werden für den jeweiligen Ausgabekontext sicher behandelt, damit sie nicht als ausführbarer Inhalt interpretiert werden.

Rich-Text-Inhalte benötigen eine geeignete Bereinigung erlaubter Elemente und Attribute. Verlasse dich nicht darauf, dass eine Browserprüfung manipulierte Anfragen verhindert.

Quellen: [OWASP: SQL Injection Prevention](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html) und [Cross Site Scripting Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html).

### 12.7 Uploads und Downloads

Beschränke erlaubte Dateitypen, Größen und Anzahl. Prüfe Dateien serverseitig und speichere sie so, dass hochgeladener Inhalt nicht als Programm ausgeführt werden kann. Der vom Nutzer gelieferte Dateiname ist kein vertrauenswürdiger Pfad.

Vertrauliche Uploads brauchen geschützte Downloads, passende Löschfristen und je nach Risiko eine Schadsoftwareprüfung. Ungeprüfte SVG- oder HTML-Dateien können aktive Inhalte enthalten.

Quelle: [OWASP: File Upload](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html).

### 12.8 Sicherheitsheader

Eine Content Security Policy kann die Ausführung unerwünschter Inhalte begrenzen. Entwickle sie passend zur Anwendung und teste sie, bevor sie legitime Funktionen blockiert. Sie ergänzt sichere Verarbeitung, ersetzt sie aber nicht.

Prüfe außerdem geeignete Regeln für Einbettung in Frames, Inhaltstypen, Referrer und Browserberechtigungen. HSTS sollte bewusst eingeführt werden, insbesondere wenn Subdomains oder langfristige Browserregeln betroffen sind.

Quelle: [OWASP: Content Security Policy](https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html).

### 12.9 Missbrauch und Geschäftslogik

Begrenze übermäßige Loginversuche, automatisierte Anfragen und teure Aktionen. Schutzmaßnahmen dürfen legitime Nutzer nicht ohne nachvollziehbaren Ausweg aussperren.

Bei Bestellungen berechnet der Server Preise und Rabatte aus vertrauenswürdigen Daten. Er darf keine vom Browser mitgesendete Gesamtsumme ungeprüft übernehmen. Wiederholte Zahlungsbenachrichtigungen dürfen nicht mehrfach liefern oder buchen.

### 12.10 Vorfälle vorbereiten

Dokumentiere, wer bei einem Sicherheitsvorfall handelt, wo Protokolle liegen und wie der Dienst gesichert oder vorübergehend eingeschränkt wird. Bei einer Kompromittierung sind Ursache, Zugangsdaten, Datenabfluss und saubere Wiederherstellung zu prüfen.

Ein Backup auf einen weiterhin kompromittierten Server zurückzuspielen kann den Angriff fortsetzen. Datenschutzbezogene Meldepflichten werden in Kapitel 15 behandelt.

<a id="kapitel-13"></a>
## 13. Datenschutz als technisches und organisatorisches Konzept

### 13.1 Mit einer Datenlandkarte beginnen

Erfasse für jede Funktion, welche Daten wo anfallen und wohin sie gehen. Betrachte Browser, Webserver, Hosting, Datenbank, E-Mail, CRM, Zahlungsdienste, Analyse und Support gemeinsam.

Eine lokale Website kann trotzdem externe Verbindungen auslösen, etwa durch Schriftdateien, Karten, Bilder, Captchas oder Fehleranalyse. Auch Informationen in URLs und Protokollen gehören in die Betrachtung.

**Praktischer Ablauf:** Website in einem frischen Browserprofil öffnen, Netzwerkverbindungen und Speicher prüfen, anschließend Kontakt, Anmeldung, Kauf und eingebettete Inhalte auslösen. Vergleiche die Beobachtungen mit der dokumentierten Diensteliste.

### 13.2 Datenminimierung konkret umsetzen

- Nur erforderliche Pflichtfelder verwenden.
- Sensible Daten nicht in Seitenadressen oder Analyseereignisse schreiben.
- Produktionsdaten nicht routinemäßig in Testumgebungen kopieren.
- Protokolle auf den erforderlichen Inhalt begrenzen.
- Zugriff auf Anfragen und Bewerbungen auf zuständige Personen beschränken.
- Test- und Demokonten rechtzeitig entfernen.
- Exporte und lokale Kopien in das Löschkonzept einbeziehen.

Diese Punkte sind Umsetzungsempfehlungen für ein datensparsames System. Die konkrete rechtliche Bewertung erfolgt anhand der jeweiligen Verarbeitung.

### 13.3 Aufbewahrung als Prozess planen

„Wir löschen irgendwann“ ist kein Konzept. Lege für Datenarten fest, welcher Zweck die Speicherung trägt, wann dieser endet, ob gesetzliche Aufbewahrungspflichten bestehen und wie die Löschung technisch erfolgt.

Kontaktnachrichten, Rechnungen, Bewerbungen, Konten und Sicherheitsprotokolle haben unterschiedliche Zwecke. Eine allgemeine Aufbewahrungsfrist für alles ist selten sinnvoll.

Backups brauchen eine gesonderte Betrachtung: begrenzte Vorhaltezeit, eingeschränkter Zugriff und ein Verfahren, damit nach einer Wiederherstellung bereits umgesetzte Löschungen erneut berücksichtigt werden.

### 13.4 Dienstleister vor der Einbindung prüfen

Frage bei jedem Dienst:

1. Welche konkrete Funktion erfüllt er?
2. Welche Daten verarbeitet er?
3. Nutzt er die Daten auch für eigene Zwecke?
4. Welche Rolle hat er datenschutzrechtlich?
5. Welche Länder und Unterauftragnehmer sind beteiligt?
6. Welche Verträge und Schutzmaßnahmen werden benötigt?
7. Wie lassen sich Daten exportieren und löschen?
8. Was passiert bei Ausfall oder Anbieterwechsel?

Eine Werbeaussage wie „DSGVO-konform“ beantwortet diese Fragen nicht. Ebenso ist ein europäischer Serverstandort allein noch keine vollständige Bewertung aller Zugriffe und Datenflüsse.

### 13.5 Datenschutzfreundliche Ausgangskonfiguration

Für eine kleine Unternehmenswebsite ist häufig ein guter Ausgangspunkt: möglichst wenige externe Dienste, eigene Bild- und Schriftdateien mit passenden Rechten, ein sparsames Kontaktformular, begrenzte Serverprotokolle und ein dokumentierter Hostingdienst.

Dadurch kann ein Einwilligungsbanner entbehrlich sein, **wenn** tatsächlich keine einwilligungspflichtigen Speicherungen, Zugriffe oder Verarbeitungen stattfinden. Das muss anhand der Umsetzung geprüft werden.

<a id="kapitel-14"></a>
## 14. Impressum und Anbieterkennzeichnung

### 14.1 Anwendungsbereich prüfen

Für geschäftsmäßige, in der Regel gegen Entgelt angebotene digitale Dienste gilt § 5 DDG. Eine Unternehmenswebsite kann darunter fallen, auch wenn ihr Aufruf kostenlos ist. Für Angebote außerhalb ausschließlich persönlicher oder familiärer Zwecke können zusätzlich Informationspflichten nach § 18 MStV relevant sein.

Quellen: [§ 5 DDG](https://www.gesetze-im-internet.de/ddg/__5.html) und [Medienstaatsvertrag, insbesondere § 18](https://www.die-medienanstalten.de/fileadmin/user_upload/Rechtsgrundlagen/Gesetze_Staatsvertraege/Medienstaatsvertrag_MStV.pdf).

### 14.2 Angaben nach dem konkreten Betreiber bestimmen

Zu prüfen sind insbesondere:

- Name und Anschrift des Anbieters;
- bei juristischen Personen Rechtsform und Vertretungsberechtigte;
- E-Mail-Adresse und Angaben für schnelle elektronische Kontaktaufnahme und unmittelbare Kommunikation;
- Register und Registernummer, sofern vorhanden;
- zuständige Aufsichtsbehörde bei erlaubnispflichtiger Tätigkeit;
- besondere Berufsangaben bei reglementierten Berufen;
- vorhandene Umsatzsteuer-Identifikationsnummer oder Wirtschafts-Identifikationsnummer;
- weitere Angaben für besondere Anbieter oder Unternehmenssituationen.

Quelle: [§ 5 DDG, vollständiger Pflichtenkatalog](https://www.gesetze-im-internet.de/ddg/__5.html).

Die gewöhnliche persönliche Steuernummer ist nicht mit diesen Identifikationsnummern gleichzusetzen. Kopiere keine Angaben eines fremden Unternehmens. Eine reine Markenbezeichnung oder ein Postfach ersetzt die erforderlichen Betreiberangaben nicht.

### 14.3 Erreichbarkeit und Redaktion

Das Impressum sollte unter einer eindeutigen Bezeichnung dauerhaft erreichbar sein, auch auf Mobilgeräten. Teste den Zugriff ohne Login und ohne Zustimmung zu optionalem Tracking.

Journalistisch-redaktionelle Angebote können zusätzlich eine verantwortliche Person mit Namen und Anschrift erfordern. Die Voraussetzungen und Anforderungen an diese Person ergeben sich aus § 18 MStV.

Quelle: [MStV § 18](https://www.die-medienanstalten.de/fileadmin/user_upload/Rechtsgrundlagen/Gesetze_Staatsvertraege/Medienstaatsvertrag_MStV.pdf).

### 14.4 Praktische Pflege

Verknüpfe Änderungen der Geschäftsanschrift, Rechtsform, Geschäftsführung oder Registerdaten mit einer Aufgabe zur Aktualisierung der Website. Prüfe auch Social-Media-Profile und weitere geschäftliche Onlineauftritte auf passende Anbieterinformationen.

Ein pauschaler Haftungsausschluss macht unzulässige Inhalte oder fehlende Angaben nicht zulässig. Verwende Rechtstexte, die zum tatsächlichen Angebot passen.

<a id="kapitel-15"></a>
## 15. Datenschutzerklärung und Datenschutzpflichten

### 15.1 Rechtsgrundlagen und Grundsätze

Personenbezogene Daten dürfen nur auf einer passenden Rechtsgrundlage verarbeitet werden. Je nach Vorgang kommen beispielsweise Vertrag bzw. vorvertragliche Maßnahmen, gesetzliche Pflichten, berechtigte Interessen oder Einwilligung infrage. Eine Einwilligung ist nicht automatisch für jede Websitefunktion erforderlich.

Beachte Zweckbindung, Datenminimierung, Speicherbegrenzung und angemessene Sicherheit. Besondere Datenkategorien, etwa Gesundheitsdaten, erfordern zusätzlich die Prüfung der besonderen Voraussetzungen.

Quellen: [DSGVO, insbesondere Art. 5, 6 und 9](https://eur-lex.europa.eu/legal-content/DE/TXT/?uri=CELEX:32016R0679) und [EU-Kommission: Datenschutz erklärt](https://commission.europa.eu/law/law-topic/data-protection/data-protection-explained_en).

### 15.2 Informationspflichten

Bei der Erhebung sind die jeweils erforderlichen Informationen nach Art. 13 bzw. 14 DSGVO bereitzustellen. Dazu gehören je nach Fall Verantwortlicher, Datenschutzkontakt, Zwecke und Rechtsgrundlagen, Empfänger, Drittlandbezug, Speicherdauer bzw. Kriterien sowie einschlägige Betroffenenrechte. Hinzu kommen gegebenenfalls Angaben zur Pflicht der Bereitstellung und zu automatisierten Entscheidungen.

Eine Datenschutzerklärung muss die **tatsächliche** Website abbilden. Nenne keine Analyse- oder Marketingdienste, die nicht verwendet werden, und verschweige keine tatsächlich eingebundenen Dienste.

Quellen: [DSGVO Art. 12–14](https://eur-lex.europa.eu/legal-content/DE/TXT/?uri=CELEX:32016R0679) und [BfDI: DSGVO und BDSG](https://www.bfdi.bund.de/SharedDocs/Downloads/DE/Broschueren/INFO1.pdf?__blob=publicationFile&v=24).

### 15.3 Auftragsverarbeitung und Rollen

Verarbeitet ein Dienstleister personenbezogene Daten weisungsgebunden für den Betreiber, kann ein Vertrag nach Art. 28 DSGVO erforderlich sein. Hosting, Formular- oder Versanddienste sind typische Prüffälle; die tatsächlichen Rollen entscheiden.

Ein Dienstleister kann für andere Vorgänge eigenständig verantwortlich sein. Bei gemeinsamer Festlegung von Zwecken und Mitteln ist gemeinsame Verantwortlichkeit zu prüfen. Ein AV-Vertrag ist daher keine universelle Lösung für jede Einbindung.

Quelle: [BfDI-Muster zur Auftragsverarbeitung mit Bezug auf Art. 28 DSGVO](https://www.bfdi.bund.de/SharedDocs/Downloads/DE/Muster/Muster_zur_Auftragsverarbeitung.pdf?__blob=publicationFile&v=2).

**Empfohlene Dokumentation:** Dienst, Leistung, Rolle, Vertragsstand, Unterauftragnehmer, Speicherorte, Sicherheitsmaßnahmen, Lösch- und Exportmöglichkeit. Prüfe, ob tatsächlicher Betrieb und Vertrag zusammenpassen.

### 15.4 Übermittlungen in Drittländer

Bei Übermittlungen außerhalb des Europäischen Wirtschaftsraums sind zusätzlich die Voraussetzungen aus Kapitel V DSGVO zu prüfen. Möglich sind unter anderem einschlägige Angemessenheitsbeschlüsse oder geeignete Garantien. Bei Standarddatenschutzklauseln kann eine zusätzliche Bewertung der Situation im Empfängerland und ergänzender Maßnahmen erforderlich sein.

Prüfe die konkret empfangende Organisation, den Anwendungsbereich eines verwendeten Instruments und aktuelle Voraussetzungen. Ein Cookie-Banner oder ein Serverstandort in der EU löst diese Fragen nicht automatisch.

Quellen: [BayLDA: Übermittlungen in Drittländer](https://www.lda.bayern.de/de/thema_uebermittlung_personenbezogener_daten_in_drittlaender.html) und [LfDI Baden-Württemberg: Handreichung zu Kapitel V](https://www.baden-wuerttemberg.datenschutz.de/drittstaatentransfer-handreichung-kap-v-dsgvo/).

### 15.5 Betroffenenrechte organisatorisch erfüllen

Je nach Voraussetzungen bestehen unter anderem Rechte auf Auskunft, Berichtigung, Löschung, Einschränkung, Datenübertragbarkeit und Widerspruch sowie Beschwerde. Für Anträge gilt grundsätzlich eine Monatsfrist; mögliche Verlängerungen und Ausnahmen müssen korrekt behandelt werden.

Quelle: [EU-Kommission: Informationen zu Betroffenenrechten](https://commission.europa.eu/law/law-topic/data-protection/information-individuals_en).

**Empfohlener Prozess:** Eingang erfassen, zuständige Person bestimmen, erforderliche Identitätsprüfung angemessen durchführen, betroffene Systeme ermitteln, Antwort dokumentieren und Fristen überwachen. Verlange nicht standardmäßig eine vollständige Ausweiskopie für jedes Anliegen.

Eine Löschanfrage bedeutet nicht automatisch, dass gesetzlich aufzubewahrende Rechnungen vollständig entfernt werden dürfen. Prüfe Zweck und Aufbewahrung für die betroffene Datenart.

### 15.6 Weitere interne Pflichten

Prüfe die Anforderungen an Verzeichnis von Verarbeitungstätigkeiten, Datenschutz-Folgenabschätzung und Datenschutzbeauftragten. Umfang, Art und Risiko der Verarbeitung entscheiden; geringe Unternehmensgröße befreit nicht pauschal von allen Pflichten.

Quellen: [DSGVO Art. 30, 35 und 37](https://eur-lex.europa.eu/legal-content/DE/TXT/?uri=CELEX:32016R0679) und [§ 38 BDSG für zusätzliche deutsche Benennungspflichten](https://www.gesetze-im-internet.de/bdsg_2018/__38.html).

Die Website allein bildet nur einen Teil des Datenschutzes ab. Auch E-Mail-Bearbeitung, CRM, Bewerbungen, Exporte und Dienstleister müssen berücksichtigt werden.

### 15.7 Datenschutzverletzungen

Eine Verletzung des Schutzes personenbezogener Daten kann eine Meldung an die zuständige Aufsicht erfordern. Diese erfolgt unverzüglich und möglichst binnen 72 Stunden ab Bekanntwerden, sofern die gesetzliche Risikoausnahme nicht greift. Bei voraussichtlich hohem Risiko kann zusätzlich die Benachrichtigung Betroffener erforderlich sein. Vorfälle sind zu dokumentieren.

Quelle: [BfDI: DSGVO in der Bundesverwaltung, Erläuterungen zu Datenschutzverletzungen](https://www.bfdi.bund.de/SharedDocs/Downloads/DE/Broschueren/INFO6.pdf?__blob=publicationFile&v=9).

**Praktische Vorbereitung:** Halte Ansprechpartner, Entscheidungsbefugnisse, Vorfalldokumentation und Zugänge bereit. Ein über das Wochenende unbeachtetes Postfach ist kein verlässlicher Eskalationsweg.

<a id="kapitel-16"></a>
## 16. Cookies, Tracking und externe Dienste

### 16.1 Zwei Prüfungen sind erforderlich

§ 25 TDDDG betrifft die Speicherung von Informationen auf Endeinrichtungen und den Zugriff auf dort gespeicherte Informationen. Grundsätzlich ist eine informierte Einwilligung erforderlich. Das Gesetz enthält Ausnahmen insbesondere für Übertragungsvorgänge und für unbedingt erforderliche Vorgänge zur Bereitstellung eines ausdrücklich gewünschten digitalen Dienstes.

Unabhängig davon braucht die Verarbeitung personenbezogener Daten eine DSGVO-Rechtsgrundlage. Die TDDDG-Ausnahme beantwortet diese zweite Frage nicht automatisch.

Quelle: [§ 25 TDDDG](https://www.gesetze-im-internet.de/ttdsg/__25.html).

### 16.2 Technik und Zweck prüfen

Die Regeln betreffen nicht ausschließlich klassische Cookies. Auch andere Speicher- und Zugriffstechniken müssen bewertet werden. „Ohne Cookies“, „serverseitig“ oder „First Party“ sind keine pauschalen Freistellungen.

Bewerte den tatsächlichen Ablauf, den Zweck und die übermittelten Daten. Analyse und Werbung sind nicht deshalb unbedingt erforderlich, weil sie wirtschaftlich nützlich sind.

Quelle: [Datenschutzkonferenz: Orientierungshilfe Digitale Dienste](https://datenschutzkonferenz-online.de/media/oh/OH_Digitale_Dienste.pdf).

### 16.3 Einwilligungsmanagement wirksam umsetzen

Wenn Einwilligungen erforderlich sind, sollten Nutzer eine informierte, freiwillige und zweckbezogene Wahl treffen können. Die Ablehnung darf nicht durch vermeidbare Umwege oder irreführende Gestaltung erschwert werden. Der Widerruf muss so einfach möglich sein wie die Erteilung.

Quelle: [DSK-Orientierungshilfe, insbesondere Gestaltung von Auswahl und Widerruf](https://datenschutzkonferenz-online.de/media/oh/OH_Digitale_Dienste.pdf).

**Empfohlene Umsetzung:** verständliche erste Ebene mit Zustimmung, gleichwertiger Ablehnung und Einstellungen; optionale Zwecke zunächst deaktiviert; dauerhaft erreichbare Änderung der Auswahl; nachvollziehbare Protokollierung mit begrenzter Speicherung.

Eine pauschale Wiederabfrage nach einer festen Zahl von Tagen ist kein universelles gesetzliches Rezept. Berücksichtige Gültigkeit, Änderungen der Zwecke und Anbieter sowie die tatsächlich verwendete Lösung.

### 16.4 Vor der Einwilligung wirklich blockieren

Prüfe in einem frischen Browserprofil:

1. Welche externen Verbindungen entstehen unmittelbar beim Aufruf?
2. Welche Speicherungen und Zugriffe passieren vor einer Entscheidung?
3. Bleiben optionale Dienste nach Ablehnung blockiert?
4. Starten sie nur nach passender Auswahl?
5. Was passiert nach Widerruf und beim nächsten Seitenaufruf?
6. Funktioniert die Kernleistung weiterhin?

Ein Banner allein beweist keine wirksame technische Sperre. Ein eingebettetes Vorschaubild, ein `preconnect` oder ein Chatwidget kann bereits Verbindungen herstellen, bevor die sichtbare Funktion geöffnet wird.

### 16.5 Typische Drittanbieter prüfen

| Funktion | Typische Prüffrage | Häufig datensparsamere Gestaltung |
|---|---|---|
| Schriften | Externe Abrufe und Rechte? | Zulässige lokale Auslieferung |
| Video | Verbindung schon vor Wiedergabe? | Lokales Vorschaubild, bewusstes Aktivieren |
| Karten | Werden Daten sofort übertragen? | Adresse und Anfahrtstext, optionaler Kartenaufruf |
| Social Feed | Plattformzugriffe und Tracking? | Gewöhnliche Links statt Live-Feed |
| Captcha | Tracking, Drittland und Zugänglichkeit? | Abgestufte serverseitige Spam-Abwehr |
| Chat | Datenweitergabe und Gesprächsspeicherung? | Erst bei Bedarf laden, Aufbewahrung begrenzen |
| Analyse | Personenbezug, Gerätezugriff, Zwecke? | Sparsame aggregierte Auswertung nach Prüfung |
| Fehleranalyse | Enthalten Berichte Formulardaten? | Maskierung und gezielte Datenerhebung |

Diese Alternativen sind keine automatischen Rechtsfreistellungen. Sie reduzieren häufig Komplexität und sollten dennoch konkret geprüft werden.

### 16.6 Consent Mode und serverseitiges Tracking

Technische Betriebsarten eines Analyseprodukts sind anhand ihres tatsächlichen Verhaltens zu bewerten. Prüfe insbesondere, ob auch ohne Zustimmung Anfragen oder Kennungen übertragen werden.

Ein serverseitiger Vermittler kann Datenflüsse verändern. Er ersetzt weder die Rechtsgrundlagenprüfung noch Informationspflichten, wirksame Einwilligung oder die Bewertung von Empfängern und Drittländern.

<a id="kapitel-17"></a>
## 17. Gesetzliche Barrierefreiheit

### 17.1 Öffentlich und privat unterscheiden

Öffentliche Stellen unterliegen je nach Ebene dem BGG, der BITV 2.0 und entsprechenden Landesregelungen. Für bestimmte Produkte und für Verbraucher erbrachte Dienstleistungen privater Anbieter ist das BFSG mit der BFSGV relevant.

Das BFSG gilt seit dem **28. Juni 2025**. Es erfasst unter anderem Dienstleistungen im elektronischen Geschäftsverkehr. Die Aussage „jede Website muss wegen des BFSG barrierefrei sein“ ist zu pauschal.

Quelle: [BFSG, insbesondere §§ 1–3](https://www.gesetze-im-internet.de/bfsg/BJNR297010021.html).

### 17.2 Anwendungsbereich und Ausnahmen prüfen

Prüfe, ob über das Angebot eine erfasste Dienstleistung für Verbraucher erbracht wird, beispielsweise ein Onlineshop oder ein einschlägiger Buchungsablauf. Auch der Verkauf von Waren, die selbst nicht zu den vom BFSG erfassten Produkten gehören, kann einen erfassten elektronischen Geschäftsverkehr darstellen.

Kleinstunternehmen, die Dienstleistungen anbieten oder erbringen, sind ausgenommen. Die Definition verbindet **weniger als zehn Beschäftigte** mit **höchstens 2 Millionen Euro Jahresumsatz oder höchstens 2 Millionen Euro Jahresbilanzsumme**. Die Ausnahme ist nicht pauschal auf das Inverkehrbringen erfasster Produkte übertragbar.

Quellen: [BFSG §§ 2 und 3](https://www.gesetze-im-internet.de/bfsg/BJNR297010021.html) und [Bundesfachstelle: Überblick zum BFSG](https://www.bundesfachstelle-barrierefreiheit.de/DE/Barrierefreiheitsstaerkungsgesetz).

Eine reine Unternehmensdarstellung ist anders zu bewerten als ein Vertragsabschluss. Bei Terminbuchungen, gemischten B2B-/B2C-Angeboten und komplexen Plattformen ist die tatsächliche Funktion zu untersuchen.

### 17.3 Mehr als sichtbare Startseiten prüfen

Erfasste Abläufe können Navigation, Identifizierung, Authentifizierung, Sicherheits- und Zahlungsfunktionen betreffen. Ein zugänglicher Shop mit unbedienbarem Zahlungsdialog löst das Problem nicht.

Quelle: [§ 19 BFSGV: zusätzliche Anforderungen für elektronischen Geschäftsverkehr](https://www.gesetze-im-internet.de/bfsgv/__19.html).

Übernimm Zugänglichkeit deshalb in die Auswahlkriterien für Shop-, Buchungs- und Zahlungsanbieter. Vereinbare prüfbare Leistungen mit einer Agentur.

### 17.4 Informationen und Erklärungen

Erfasste Dienstleistungserbringer müssen die vorgeschriebenen Informationen nach § 14 BFSG und Anlage 3 bereitstellen. Diese Informationspflicht ist von der Erklärung zur Barrierefreiheit öffentlicher Stellen zu unterscheiden. Prüfe das passende Format, die Inhalte und die Angaben zur zuständigen Marktüberwachung.

Quelle: [§ 14 BFSG](https://www.gesetze-im-internet.de/bfsg/__14.html).

### 17.5 Keine pauschale Übergangsfrist für alle Websites

Das BFSG enthält bestimmte Übergangsregeln und begrenzte Ausnahmen. Daraus folgt keine allgemeine Erlaubnis, einen erfassten alten Onlineshop bis 2030 unverändert unzugänglich weiterzubetreiben.

Quelle: [Bundesfachstelle: BFSG-Newsletter mit Erläuterungen zu bestehenden Websites](https://www.bundesfachstelle-barrierefreiheit.de/SharedDocs/Downloads/DE/BFSG-Newsletter/bfsg-newsletter-1-2025.pdf?__blob=publicationFile).

Begründungen wie unverhältnismäßige Belastung müssen anhand der gesetzlichen Voraussetzungen geprüft und gegebenenfalls dokumentiert werden. Hoher Aufwand allein ist keine automatisch geltende Ausnahme.

### 17.6 Nachweis und dauerhafte Pflege

Dokumentiere Geltungsprüfung, Anforderungskatalog, geprüfte Seiten und Abläufe, Testmethoden, Befunde und Korrekturen. Eine automatisierte Bewertung oder ein Overlay ist allein kein Nachweis, dass sämtliche Anforderungen erfüllt sind.

Rechtsbezogene Nachweise sollten die jeweils einschlägigen technischen Anforderungen abbilden. Der allgemeine technische Zielwert WCAG 2.2 AA aus Kapitel 8 bleibt dafür ein Ausgangspunkt, kein vollständiges Rechtsgutachten.

<a id="kapitel-18"></a>
## 18. Urheberrecht, Bilder, Marken und Lizenzen

### 18.1 Rechte vor Veröffentlichung klären

Texte, Fotos, Illustrationen, Musik, Videos, Software und Gestaltungselemente können geschützt sein. Öffentlich im Internet auffindbar bedeutet nicht frei verwendbar. Für die Onlinebereitstellung sind passende Rechte erforderlich.

Quelle: [§ 19a UrhG: öffentliche Zugänglichmachung](https://www.gesetze-im-internet.de/urhg/__19a.html).

**Empfohlene Rechteakte:** Dateiname, Urheber bzw. Anbieter, Kauf- oder Downloadnachweis, Lizenzfassung, erlaubte Nutzung, Einschränkungen, Namensnennung und Veröffentlichungsorte.

### 18.2 Lizenzbedingungen genau lesen

Prüfe bei Stockmedien und anderen Ressourcen insbesondere kommerzielle Nutzung, Bearbeitung, Weitergabe an Auftraggeber, Nutzung in Werbung, zeitliche oder räumliche Beschränkung und erforderliche Quellenangaben.

Schriftdateien können eigene Webfont-Lizenzen benötigen. Eine Lizenz für die Installation am Arbeitsplatz ist nicht automatisch eine Lizenz zur Auslieferung an Websitebesucher.

Open-Source-Komponenten und frei verfügbare Icons können Pflichten enthalten. Dokumentiere die konkret verwendete Lizenz und erfülle die für diese Nutzung geltenden Bedingungen.

### 18.3 Personen auf Bildern

Für erkennbare Personen sind die Voraussetzungen für die Veröffentlichung zu prüfen. § 22 KunstUrhG sieht grundsätzlich Einwilligung für Verbreitung und öffentliche Zurschaustellung vor; Ausnahmen und die datenschutzrechtliche Bewertung dürfen nicht pauschal übergangen werden.

Quelle: [§ 22 KunstUrhG](https://www.gesetze-im-internet.de/kunsturhg/__22.html).

Bei Mitarbeiterfotos und Testimonials sollten Zweck, Umfang und Veröffentlichungsorte nachvollziehbar geregelt sein. Bei Minderjährigen und sensiblen Situationen ist besondere Sorgfalt erforderlich.

### 18.4 Marken, Logos und Referenzen

Prüfe, ob Kundenlogos, Markenzeichen und Referenzprojekte in der vorgesehenen Weise veröffentlicht werden dürfen. Eine Zusammenarbeit bedeutet nicht automatisch, dass jede öffentliche Darstellung abgestimmt ist.

Lege auch fest, ob eine Agentur das Projekt in ihrem Portfolio zeigen darf. Geheimhaltung, personenbezogene Daten und Rechte an zugelieferten Materialien können eine Rolle spielen.

### 18.5 KI-generierte Inhalte

Überprüfe Herkunft, Nutzungsbedingungen und mögliche Rechte Dritter. Achte besonders auf realistische Personen, erkennbare Marken und Nachbildungen vorhandener Werke. Dass ein Werkzeug einen Inhalt erzeugt hat, garantiert keine rechtlich unproblematische Nutzung.

Die urheberrechtliche Schutzfähigkeit rein KI-generierter Inhalte ist nicht mit derjenigen menschlicher Werke gleichzusetzen. Verbindliche Zusagen wie „exklusive weltweite Rechte an jedem KI-Bild“ brauchen eine tragfähige Grundlage.

<a id="kapitel-19"></a>
## 19. Onlineshops, Verträge und Verbraucherrechte

### 19.1 Geschäftsmodell zuerst einordnen

Unterscheide B2C und B2B sowie Waren, Dienstleistungen, digitale Inhalte, Abonnements und Finanzdienstleistungen. Diese Einordnung beeinflusst Informationspflichten, Widerruf und weitere Anforderungen.

Ein Hinweis „nur für Geschäftskunden“ sollte zum tatsächlichen Vertriebsprozess passen. Wenn Verbraucher faktisch bestellen können, ist eine bloße Formulierung keine verlässliche Lösung.

### 19.2 Vorvertragliche Informationen

Vor der verbindlichen Erklärung müssen einschlägige Informationen klar bereitstehen. Dazu gehören je nach Vertrag unter anderem wesentliche Eigenschaften, Unternehmerangaben, Preis, weitere Kosten, Zahlungs- und Lieferbedingungen, Beschwerdeverfahren, Vertragslaufzeit und Widerrufsangaben.

Für digitale Angebote kommen zusätzliche Informationen etwa zur Funktionalität, Kompatibilität oder Interoperabilität infrage. Prüfe den vollständigen Katalog für die konkrete Leistung.

Quellen: [Art. 246a § 1 EGBGB](https://www.gesetze-im-internet.de/bgbeg/art_246a__1.html) und [Art. 246a § 4 EGBGB: Zeitpunkt und Form](https://www.gesetze-im-internet.de/bgbeg/art_246a__4.html).

### 19.3 Preise korrekt und nachvollziehbar darstellen

Bei Verbraucherangeboten ist grundsätzlich der Gesamtpreis anzugeben. Prüfe gegebenenfalls Versandkosten, weitere Preisangaben und Grundpreise. Bei Abonnements muss verständlich sein, welche Kosten pro Abrechnungszeitraum entstehen.

Bei angekündigten Preisermäßigungen für Waren ist grundsätzlich der niedrigste Gesamtpreis der letzten 30 Tage vor der Ermäßigung relevant; Ausnahmen und Sonderfälle sind gesondert zu prüfen.

Quellen: [§ 3 PAngV](https://www.gesetze-im-internet.de/pangv_2022/__3.html), [PAngV einschließlich Grundpreisregeln](https://www.gesetze-im-internet.de/pangv_2022/) und [§ 11 PAngV](https://www.gesetze-im-internet.de/pangv_2022/__11.html).

**Praktische Umsetzung:** Berechne Preisbestandteile aus einer konsistenten Datenquelle und prüfe Produktseite, Warenkorb, Checkout und Bestätigung gemeinsam. Versteckte Aufschläge erst am letzten Schritt untergraben Vertrauen und können rechtlich problematisch sein.

### 19.4 Bestellbutton und letzte Bestellseite

Bei einschlägigen zahlungspflichtigen Verbraucherverträgen muss der abschließende Schritt die Zahlungspflicht eindeutig erkennen lassen. „Zahlungspflichtig bestellen“ ist die gesetzlich genannte Formulierung; eine entsprechend eindeutige Formulierung ist ebenfalls vorgesehen. Unklare Begriffe wie bloßes „Weiter“ reichen dafür nicht.

Die gesetzlich verlangten Kerninformationen müssen unmittelbar vor der Bestellung hervorgehoben bereitgestellt werden. Bei Verstoß gegen die Button-Anforderung kann der Vertrag nach der gesetzlichen Regelung nicht zustande kommen.

Quelle: [§ 312j BGB](https://www.gesetze-im-internet.de/bgb/__312j.html).

### 19.5 AGB passend gestalten

AGB sind nicht für jede Website verpflichtend. Wenn sie verwendet werden, müssen sie zum Geschäftsmodell passen und wirksam einbezogen werden. Unzulässige Klauseln werden nicht dadurch wirksam, dass der Kunde ein Kästchen aktiviert.

Eine reine Informationswebsite benötigt nicht automatisch Shop-AGB. Individuelle Vertragsmodelle, Abonnements und internationale Verkäufe sollten mit geeigneten Rechtstexten geprüft werden.

### 19.6 Widerruf und Ausnahmen

Für viele Fernabsatzverträge mit Verbrauchern besteht ein Widerrufsrecht mit grundsätzlich 14-tägiger Frist. Beginn und Berechnung hängen vom Vertragstyp und der ordnungsgemäßen Information ab. Fehlende Belehrung kann die Frist erheblich verlängern.

Für bestimmte Verträge bestehen Ausnahmen. Bei digitalen Inhalten und Dienstleistungen sind besondere Voraussetzungen für einen vorzeitigen Beginn bzw. das Erlöschen des Widerrufsrechts zu prüfen. Ein pauschales „Kein Widerruf bei Downloads“ ist kein ausreichender Ablauf.

Quellen: [§ 355 BGB](https://www.gesetze-im-internet.de/bgb/__355.html), [§ 356 BGB](https://www.gesetze-im-internet.de/bgb/__356.html) und [§ 312g BGB](https://www.gesetze-im-internet.de/bgb/__312g.html).

### 19.7 Elektronische Widerrufsfunktion: aktuelles Prüffeld

Nach § 356a BGB muss bei den erfassten, über eine Online-Benutzeroberfläche geschlossenen Fernabsatzverträgen eine elektronische Widerrufsfunktion zur Verfügung stehen. Sie muss während der Widerrufsfrist ständig verfügbar, hervorgehoben und leicht zugänglich sein.

Der Ablauf umfasst eine eindeutig beschriftete Funktion, die nötigen Angaben zur Person und Vertragszuordnung sowie eine Bestätigungsfunktion. Nach Abgabe ist unverzüglich eine Eingangsbestätigung auf dauerhaftem Datenträger mit Inhalt, Datum und Uhrzeit zu übermitteln.

Quelle: [§ 356a BGB: elektronische Widerrufsfunktion](https://www.gesetze-im-internet.de/bgb/__356a.html).

**Praktische Umsetzung:** Prüfe Auffindbarkeit, Gastbestellungen, Teilwiderrufe, Bestätigungsversand und tatsächliche Bearbeitung. Ein Link zur Widerrufsbelehrung ist nicht dieselbe Funktion. Teste den Ablauf vollständig auf Mobilgeräten und per Tastatur.

### 19.8 Kündigungsbutton bei erfassten Dauerschuldverhältnissen

Für bestimmte über Webseiten abschließbare entgeltliche Verbraucherdauerschuldverhältnisse schreibt § 312k BGB eine Kündigungsschaltfläche und Bestätigungsseite vor. Gesetzliche Ausnahmen, etwa für bestimmte Finanzdienstleistungen, sind zu prüfen.

Die Funktionen müssen ständig verfügbar und unmittelbar sowie leicht zugänglich sein. Vorgesehen sind eindeutige Beschriftungen wie „Verträge hier kündigen“ und „jetzt kündigen“, eine speicherbare Erklärung und sofortige elektronische Bestätigung in Textform.

Quelle: [§ 312k BGB](https://www.gesetze-im-internet.de/bgb/__312k.html).

Kündigung und Widerruf sind unterschiedliche Vorgänge. Eine gemeinsame unklare Schaltfläche oder eine normale Supportanfrage bildet die gesetzlichen Abläufe nicht automatisch ab.

### 19.9 Bestätigung, Zahlung und Support

Stimme Zeitpunkt des Vertragsschlusses, Bestellbestätigung und Zahlungsprozess miteinander ab. „Bestellung eingegangen“ kann etwas anderes bedeuten als die verbindliche Annahme.

Verwende bewährte Zahlungsdienste und speichere Zahlungsdaten nur, wenn das notwendig und angemessen abgesichert ist. Prüfe fehlgeschlagene Zahlungen, doppelte Meldungen, Rückerstattungen und erneute Versuche.

Der Support braucht einen klaren Zugang zu Bestellstatus, Widerruf und Kündigung. Dokumentierte Bearbeitung verhindert, dass rechtzeitig abgegebene Erklärungen im Posteingang liegen bleiben.

### 19.10 Verbraucherstreitbeilegung und alte OS-Links

Die frühere europäische Online-Streitbeilegungsplattform wurde am **20. Juli 2025** eingestellt. Übernehme aus alten Vorlagen keinen Hinweis, der eine noch aktive OS-Plattform behauptet.

Quelle: [EU-Kommission: Einstellung der ODR-/OS-Plattform](https://consumer-redress.ec.europa.eu/site-relocation_en).

Davon unabhängig sind einschlägige Informationspflichten zur Verbraucherstreitbeilegung nach dem VSBG zu prüfen. § 36 enthält unter anderem eine Ausnahme von der Informationspflicht nach Absatz 1 Nummer 1 für Unternehmer mit zehn oder weniger Beschäftigten zum maßgeblichen Stichtag; daraus folgt keine pauschale Ausnahme von sämtlichen Streitbeilegungspflichten.

Quelle: [§ 36 VSBG](https://www.gesetze-im-internet.de/vsbg/__36.html).

### 19.11 Produktsicherheit und besondere Waren

Für erfasste Verbraucherprodukte verlangt die GPSR bei Fernabsatzangeboten bestimmte sichtbare Informationen, unter anderem zur Identifizierung, zum Hersteller bzw. gegebenenfalls zur verantwortlichen Person in der EU sowie zu relevanten Warn- und Sicherheitsinformationen.

Quelle: [EU-Produktsicherheitsverordnung, insbesondere Art. 19](https://eur-lex.europa.eu/eli/reg/2023/988/oj).

Prüfe abhängig vom Sortiment weitere Produktvorschriften, etwa für Lebensmittel, Kosmetik, Elektrogeräte, Batterien oder Textilien. Diese Themen brauchen eine eigene Sortimentsprüfung; eine technisch vollständige Produktseite ist nicht automatisch rechtlich vollständig.

### 19.12 Neue Verbraucherinformationen seit September 2026

Die Regeln aus der Richtlinie (EU) 2024/825 betreffen seit **27. September 2026** unter anderem Verbraucherinformationen zu Gewährleistung und bestimmten Haltbarkeitsgarantien. Für erfasste Warenangebote sind der harmonisierte Hinweis und bei erfüllten Voraussetzungen das entsprechende Garantielabel zu prüfen. Maßgeblich sind die nationale Umsetzung und das konkrete Produkt.

Quelle: [EU-Kommission: nachhaltiger Konsum und harmonisierte Garantieinformationen](https://commission.europa.eu/topics/consumers/consumer-rights-and-complaints/sustainable-consumption_en).

Alte Shop-Vorlagen sollten deshalb nicht nur hinsichtlich Cookie- und Widerrufstexten geprüft werden. Auch Produktdaten und Verbraucherhinweise können Aktualisierungen benötigen.

<a id="kapitel-20"></a>
## 20. Werbung, Newsletter und Bewertungen

### 20.1 Werbliche Kommunikation kenntlich machen

Trenne Werbung und redaktionelle Informationen erkennbar. Affiliate-Links, bezahlte Kooperationen und gesponserte Inhalte sind nach ihrem tatsächlichen Charakter zu bewerten und entsprechend transparent zu gestalten.

Verwende keine erfundenen Auszeichnungen, angeblichen Kundenstimmen oder falschen Verfügbarkeitsangaben. Halte Nachweise für wesentliche Werbeaussagen bereit.

### 20.2 Newsletter und Werbe-E-Mails

E-Mail-Werbung benötigt grundsätzlich vorherige ausdrückliche Einwilligung. Die Bestandskundenausnahme nach § 7 Abs. 3 UWG gilt nur bei Erfüllung sämtlicher Voraussetzungen; eine früher erhaltene Anfrage genügt nicht automatisch.

Quelle: [§ 7 UWG](https://www.gesetze-im-internet.de/uwg_2004/__7.html).

**Empfohlener Ablauf:** getrennte Anmeldung, verständliche Angaben zum Absender und Inhalt, dokumentierter Double-Opt-in-Prozess als Nachweis- und Missbrauchsschutz, eindeutige Bestätigung und einfache Abmeldung. Das Bestätigungsverfahren selbst sollte nicht zu einer Werbenachricht ohne Grundlage werden.

Tracking von Öffnungen und Klicks ist gesondert zu bewerten. Eine Newsletteranmeldung umfasst nicht automatisch jede Analyse des Leseverhaltens.

### 20.3 Bewertungen und Testimonials

Zeige Herkunft und Kontext nachvollziehbar. Wenn Verbraucherbewertungen zugänglich gemacht werden, sind einschlägige Transparenzpflichten über die Prüfung ihrer Echtheit zu beachten. Erfundene oder manipulierte Bewertungen sind kein zulässiges Mittel zur Vertrauensbildung.

Quelle: [UWG, insbesondere § 5b Abs. 3 und Anhang](https://www.gesetze-im-internet.de/uwg_2004/).

**Praktische Umsetzung:** Dokumentiere Herkunft, Freigabe und gegebenenfalls Gegenleistungen. Wähle Ausschnitte so, dass die ursprüngliche Aussage nicht verfälscht wird. Prüfe Rechte an Namen, Bildern und verwendeten Logos.

### 20.4 Umwelt- und Nachhaltigkeitswerbung

Die Richtlinie (EU) 2024/825 verschärft den Rahmen gegen irreführende Umweltwerbung. Seit dem 27. September 2026 ist die Anwendung der umgesetzten Regeln zu berücksichtigen. Allgemeine Umweltbehauptungen, Nachhaltigkeitssiegel und Aussagen zur Klimawirkung müssen entsprechend den jeweiligen Voraussetzungen geprüft werden.

Quellen: [Richtlinie (EU) 2024/825](https://eur-lex.europa.eu/eli/dir/2024/825/oj) und [EU-Kommission: nachhaltiger Konsum](https://commission.europa.eu/topics/consumers/consumer-rights-and-complaints/sustainable-consumption_en).

**Empfehlung:** Benenne konkret, worauf sich eine Aussage bezieht, wie sie belegt wird und welche Grenzen sie hat. Eine pauschale Behauptung wie „unsere Website ist klimaneutral“ braucht eine belastbare rechtliche und sachliche Prüfung. Verwende keine selbst erfundenen Gütesiegel als Ersatz für Nachweise.

<a id="kapitel-21"></a>
## 21. Weitere rechtliche Prüffelder und KI

### 21.1 Plattformen und Nutzerinhalte

Foren, Kommentarbereiche, Marktplätze und andere Angebote mit Nutzerinhalten können zusätzliche Pflichten aus dem Digital Services Act und weiteren Vorschriften auslösen. Die Einordnung als Vermittlungs-, Hosting- oder Plattformdienst sowie mögliche Ausnahmen müssen konkret geprüft werden.

Quelle: [Digital Services Act, Verordnung (EU) 2022/2065](https://eur-lex.europa.eu/eli/reg/2022/2065/oj).

**Organisatorische Empfehlung:** verständliche Nutzungsregeln, erreichbarer Meldeweg, zuständige Moderation und dokumentierte Entscheidungen. Definiere auch, wie personenbezogene Daten in Kommentaren, beleidigende Beiträge und urheberrechtswidrige Uploads bearbeitet werden.

### 21.2 Angebote für Kinder und Jugendliche

Prüfe altersbezogene Anforderungen, Jugendschutz und besondere Risiken für Minderjährige. Ein allgemein zugänglicher Hinweis „ab 18“ ist nicht für jede Art von Inhalt oder Verkauf ausreichend.

Wenn eine Datenverarbeitung bei direkt an Kinder angebotenen Diensten auf Einwilligung gestützt werden soll, sind insbesondere die Voraussetzungen von Art. 8 DSGVO zu prüfen. Verwechsle diese Frage nicht mit jeder sonstigen Nutzung eines Angebots durch Minderjährige.

Quelle: [DSGVO Art. 8](https://eur-lex.europa.eu/legal-content/DE/TXT/?uri=CELEX:32016R0679).

### 21.3 Branchen und regulierte Leistungen

Bei Gesundheit, Finanzen, Versicherungen, Rechtsberatung, Glücksspiel und ähnlichen Bereichen können besondere Werbe-, Informations-, Zulassungs- und Berufsregeln gelten. Diese müssen gesondert geprüft werden.

Behandle diesen Leitfaden als Grundlage für die Websitequalität. Ein allgemeines Impressum und eine Datenschutzerklärung ersetzen keine Branchenprüfung.

### 21.4 KI-Chatbots und Transparenz

Die Transparenzpflichten nach Art. 50 der KI-Verordnung gelten grundsätzlich seit **2. August 2026**. Insbesondere bei direkter Interaktion mit KI und bestimmten synthetischen Inhalten sind die einschlägigen Pflichten, Rollen und Ausnahmen zu prüfen. Anbieter- und Betreiberpflichten sind auseinanderzuhalten.

Quellen: [EU-Kommission: Leitlinien zu Art. 50](https://digital-strategy.ec.europa.eu/en/policies/guidelines-ai-transparency-obligations) und [Fragen zu Transparenzpflichten](https://digital-strategy.ec.europa.eu/en/faqs/transparency-obligations-under-article-50-ai-act).

**Empfehlung für Website-Chatbots:** KI-Nutzung von Beginn an verständlich kennzeichnen, menschlichen Kontakt anbieten und Grenzen erklären. Ein Bot sollte keine verbindlichen Zusagen machen, die der Betreiber nicht einhalten will oder kann.

Prüfe außerdem Gesprächsspeicherung, Datenverwendung beim Anbieter, Zugriff auf vertrauliche Dokumente und fehlerhafte Antworten. Ein Chatbot mit Bestellzugriff braucht dieselben Berechtigungsgrenzen wie ein normaler Kundenbereich.

### 21.5 Internationale Angebote

Ein Angebot für mehrere Länder kann zusätzliche Verbraucher-, Datenschutz-, Steuer-, Sprach- und Produktanforderungen auslösen. Eine deutsche Rechtsvorlage ist keine weltweite Freigabe.

Definiere Zielmärkte, Lieferländer, Kundengruppen und Sprachen. Prüfe neue Märkte vor ihrer Freischaltung und halte Versand- und Vertragsbedingungen konsistent.

### 21.6 Verträge mit Agenturen und Dienstleistern

Regle Leistungsumfang, Nutzungsrechte, Abnahme, Pflege, Datenverarbeitung, Zugangsdaten, Quellcodeübergabe, Export und Unterstützung beim Wechsel. Beschreibe Barrierefreiheit und Sicherheit mit prüfbaren Anforderungen.

Eine Vereinbarung „Website fertig“ ist zu ungenau. Sinnvoller sind definierte Seiten, Funktionen, unterstützte Nutzungssituationen und überprüfbare Abnahmekriterien.

<a id="kapitel-22"></a>
## 22. Hosting, Domains und Betrieb

### 22.1 Hosting nach Anforderungen auswählen

Bewerte Verfügbarkeit, Support, Backup, Wiederherstellung, Sicherheitsupdates, Datenschutz, Export und Kosten. Der günstigste Tarif kann teuer werden, wenn Wiederherstellung und Support fehlen.

Klärungspunkte sind unter anderem:

- Wer aktualisiert Betriebssystem, CMS, Plugins und Bibliotheken?
- Wer überwacht Zertifikate, Speicherplatz und Ausfälle?
- Was umfasst ein Backup genau?
- Wie schnell ist ein Restore möglich?
- Wo werden Daten und Sicherungen verarbeitet?
- Wie werden Zugänge abgesichert?
- Welche Unterstützung gibt es außerhalb üblicher Arbeitszeiten?

### 22.2 Domains und Konten unter eigener Kontrolle halten

Domaininhaberschaft und kritische Konten sollten zum Betreiber passen. Lege fest, welche Personen administrieren dürfen und wie der Zugang bei Ausfall einer Person erhalten bleibt.

Überwache Domainverlängerung, Zahlungsdaten und Sicherheitskontakte. Hinterlege Wiederherstellungsinformationen, die nicht ausschließlich von einer einzelnen ehemaligen Mitarbeiterin oder Agentur abhängen.

### 22.3 E-Mail-Zustellung prüfen

Formulare, Bestellbestätigungen und Passwort-Rücksetzungen brauchen zuverlässige Zustellung. Nutze einen geeigneten Versandweg und prüfe Domainauthentifizierung mit SPF, DKIM und einer passend eingeführten DMARC-Richtlinie.

Teste mehrere Empfängerdienste und überwache Fehler. Eine technische Erfolgsmeldung vom Webserver bestätigt nicht, dass die E-Mail beim Empfänger angekommen ist.

### 22.4 Backups und Wiederherstellung

Sichere die tatsächlich benötigten Bestandteile: Dateien, Datenbank, Konfiguration und relevante Informationen zu externen Diensten. Sicherungen müssen vom Ausfall oder Angriff auf das Hauptsystem möglichst unabhängig sein.

**Empfehlung:** mehrere Kopien mit geeigneter Trennung und mindestens einer isolierten bzw. gegen unbefugte Veränderung geschützten Variante. Verschlüsselung, Zugriffsschutz und Aufbewahrungsdauer gehören zum Konzept.

Führe Wiederherstellungstests durch. Ein erfolgreich gemeldeter Backupjob beweist nicht, dass eine Bestellung oder das ganze System daraus wiederhergestellt werden kann.

### 22.5 RPO und RTO festlegen

| Begriff | Leitfrage | Beispiel für ein internes Ziel |
|---|---|---|
| RPO – Recovery Point Objective | Wie viel Datenverlust ist tolerierbar? | Höchstens eine Stunde Bestelldaten |
| RTO – Recovery Time Objective | Wie lange darf die Wiederherstellung dauern? | Kernbetrieb innerhalb von vier Stunden |

Diese Werte sind Beispiele. Ein kleiner Informationsauftritt und ein umsatzstarker Shop brauchen unterschiedliche Ziele. Vergleiche vereinbarte Ziele mit der tatsächlich getesteten Wiederherstellung.

### 22.6 Überwachung mit Handlung verbinden

Überwache mindestens zentrale Erreichbarkeit, Formularannahme, kritische Fehler, Ablauf von Zertifikaten und Sicherungen. Bei Shops kommen Bestellung und Zahlungsintegration hinzu.

Eine Warnung hilft nur, wenn sie jemanden erreicht, der reagieren kann. Lege Verantwortliche und Eskalation fest. Begrenze unnötige personenbezogene Inhalte in Überwachungsdaten.

### 22.7 Updates und Wartung

Plane regelmäßige Updates und einen schnelleren Weg für akute Sicherheitsprobleme. Teste nach Änderungen die wichtigsten Abläufe. Entferne veraltete Plugins und ungenutzte Konten.

Eine Website ist nach dem Launch nicht wartungsfrei. Budgetiere Inhaltsprüfung, technische Pflege, Backups, rechtliche Aktualisierung und kleinere Verbesserungen dauerhaft.

### 22.8 Nachhaltigkeit praktisch verbessern

Weniger Datenvolumen, weniger unnötige Verarbeitung und langlebige Architektur können Ressourcen sparen. Optimiere Medien, reduziere automatische Wiedergabe und vermeide unnötige Abrufe.

Mache daraus keine unbelegte Umweltwerbung. Beschreibe lieber überprüfbare Maßnahmen, etwa Bildoptimierung oder den Verzicht auf automatisch gestartete Videos.

<a id="kapitel-23"></a>
## 23. Qualitätssicherung, Tests und Abnahme

### 23.1 Nach Risiko und Nutzung testen

Die intensivsten Prüfungen gehören zu Vorgängen mit Kosten, Datenverlust, vertraulichen Informationen oder wichtigen Anfragen. Teste zentrale Nutzerwege vollständig, nicht nur einzelne sichtbare Komponenten.

Für eine Kontaktwebsite kann das bedeuten: Leistung finden, Formular ausfüllen, Fehler korrigieren, absenden und tatsächlichen Eingang prüfen. Für einen Shop: Produkt auswählen, Warenkorb ändern, bezahlen, Bestätigung erhalten und Widerruf auslösen.

### 23.2 Repräsentative Testmatrix

| Bereich | Automatisch sinnvoll | Manuell erforderlich oder besonders hilfreich |
|---|---|---|
| Links und Statuscodes | Crawl, tote Links, Weiterleitungen | Ziel und Sinn der Verlinkung |
| Barrierefreiheit | Erkennbare Markup- und Kontrastfehler | Tastatur, Screenreader, sinnvolle Alternativen |
| Formulare | Verarbeitung und Fehlerfälle | Verständlichkeit, Autofill, echte Zustellung |
| Performance | Labormessung, Größen- und Laufzeitkontrolle | Reale Geräte und langsame Verbindung |
| Sicherheit | Abhängigkeiten und gezielte Prüfungen | Berechtigungen, Geschäftslogik, fachliche Prüfung |
| Datenschutz | Netzwerk- und Speicherbeobachtung | Zweck, Rechtsgrundlage, Auswahlverständlichkeit |
| Inhalte | Fehlende Felder und Konsistenzchecks | Fachliche Richtigkeit und Rechte |
| Shop | Bestell- und Integrationsabläufe | Kostenklarheit und Verbraucherfunktionen |

### 23.3 Positive und negative Fälle

Prüfe neben dem Idealablauf:

- leere oder ungewöhnlich lange Eingaben;
- Sonderzeichen und internationale Namen;
- verspätete und doppelte Anfragen;
- Netzwerkunterbrechung;
- abgelaufene Sitzung;
- fehlende Berechtigung;
- gesperrte Drittanbieter;
- abgelehntes Tracking;
- fehlgeschlagene Zahlung;
- erneutes Absenden nach einem Fehler;
- fehlende oder zu große Uploads;
- vergrößerte Schrift und reduzierte Bewegung.

### 23.4 Browser und Geräte bewusst auswählen

Teste die für die Zielgruppe relevanten Browser und mindestens reale Smartphone-Nutzung. Ein Browserfenster auf schmaler Breite bildet weder Touchverhalten noch Bildschirmtastatur vollständig ab.

Dokumentiere unterstützte Versionen und Grenzen. Eine pauschale Zusage „funktioniert überall“ ist selten überprüfbar.

### 23.5 Rechtliche Texte gegen das System abgleichen

Vergleiche die Datenschutzerklärung mit tatsächlich auftretenden Verbindungen und Datenverarbeitungen. Vergleiche Preise und Vertragsinformationen mit dem Checkout. Prüfe, ob Impressum und zuständige Personen aktuell sind.

Bei Verbraucherfunktionen zählt auch die Technik: Eine passende Erklärung ersetzt keinen funktionierenden Widerrufs- oder Kündigungsablauf.

### 23.6 Befunde dokumentieren und priorisieren

Jeder Fehlerbericht sollte Ort, Schritte, erwartetes Verhalten, tatsächliches Verhalten, Auswirkung und verantwortliche Person enthalten. Ein Screenshot allein erklärt einen Fehler oft nicht vollständig.

Behebe zuerst Blockaden wichtiger Vorgänge, Datenoffenlegung, irreführende Kosten und erhebliche Zugänglichkeitsprobleme. Rein dekorative Abweichungen können danach folgen.

### 23.7 Aussagekräftige Abnahme

Eine Abnahme bestätigt die vereinbarten Leistungen und Kriterien. Halte offene Punkte, Verantwortliche und Termine fest. Unterscheide fertige Funktion, bekannte Einschränkung und noch fehlenden Nachweis.

Ein einzelner Prüfwert ist keine vollständige Abnahme. Entscheidend ist, ob zentrale Aufgaben und vereinbarte Anforderungen nachweislich erfüllt sind.

<a id="kapitel-24"></a>
## 24. Projektablauf, Übergabe und Verantwortlichkeiten

### 24.1 Empfohlener Ablauf

1. **Bestandsaufnahme:** Ziele, Inhalte, bestehende URLs, Funktionen und Dienstleister erfassen.
2. **Anforderungsprüfung:** Datenschutz, Barrierefreiheit, Recht, Sicherheit und Betrieb früh einordnen.
3. **Struktur und Inhalte:** Navigation, Seitenzwecke und zentrale Texte entwickeln.
4. **Prototyp:** Kernabläufe und mobile Nutzung prüfen.
5. **Umsetzung:** Wiederverwendbare Komponenten und belastbare Datenprozesse bauen.
6. **Verifikation:** Technik, Inhalte, Zugänglichkeit und rechtliche Umsetzung abgleichen.
7. **Launchvorbereitung:** Domain, Weiterleitungen, Sicherung, Zustellung und Rücknahmeplan prüfen.
8. **Veröffentlichung:** Zuständigkeiten und Überwachung aktivieren.
9. **Nachkontrolle:** Reale Fehler und Anfragewege prüfen.
10. **Dauerbetrieb:** Inhalte, Software, Verträge und Prüfungen pflegen.

### 24.2 Beispielhafte Zuständigkeitsmatrix

| Aufgabe | Federführung | Beteiligte |
|---|---|---|
| Inhalte und Leistungen | Betreiber/Fachteam | Redaktion, Design |
| Anbieter- und Vertragsangaben | Betreiber | Qualifizierte Rechtsberatung bei Bedarf |
| Datenschutzkonzept | Verantwortlicher/Datenschutzfunktion | Technik, Dienstleister |
| Zugänglichkeit | Projektverantwortlicher | Design, Entwicklung, Redaktion, Prüfung |
| Sicherheitskonzept | Technische Verantwortung | Hosting, Entwicklung, Betreiber |
| Backups und Wiederherstellung | Betrieb/Hosting | Betreiber, Technik |
| Anfragen und Beschwerden | Zuständiges Fachteam | Support, Datenschutzkontakt |

Eine externe Agentur entlastet bei der Umsetzung. Der Betreiber muss dennoch wissen, wer welche Aufgabe übernimmt und welche Nachweise vorliegen.

### 24.3 Übergabeunterlagen

- Übersicht der Domains, Konten und administrativen Zuständigkeiten.
- Dokumentation der Technik, Integrationen und Versionen.
- Bearbeitungsanleitung für Inhalte und Medien.
- Rechte- und Lizenznachweise.
- Datenlandkarte und Diensteliste.
- Verträge und relevante Datenschutzunterlagen.
- Prüfberichte und bekannte Einschränkungen.
- Backup-, Wiederherstellungs- und Notfallbeschreibung.
- Wartungsplan und Kontaktwege.
- Export- und Wechselverfahren.

Zugangsdaten selbst gehören in eine geeignete sichere Übergabe, nicht als offene Tabelle in diese Dokumentation.

### 24.4 Beispielhafter Wartungsrhythmus

| Rhythmus | Empfohlene Aufgabe |
|---|---|
| Laufend | Ausfälle, kritische Fehler, Anfragen und Sicherheitsmeldungen bearbeiten |
| Nach jeder relevanten Änderung | Kernabläufe, Datenschutzverhalten und betroffene Zugänglichkeit prüfen |
| Monatlich | Updates, Links, Formulare, offene Befunde und zentrale Inhalte kontrollieren |
| Quartalsweise | Diensteliste, Berechtigungen, Nutzungsprobleme und ausgewählte Wiederherstellung prüfen |
| Mindestens bei Anlass, ergänzt durch planmäßige Prüfung | Rechtstexte, Verträge, Branchenanforderungen und Marktänderungen prüfen |

Die Intervalle sind organisatorische Empfehlungen, keine allgemeingültigen gesetzlichen Fristen. Eine kritische Schwachstelle wartet nicht auf den nächsten Monatscheck.

<a id="kapitel-25"></a>
## 25. Prioritäten und Entscheidungshilfen

### 25.1 Was zuerst erledigt werden sollte

| Priorität | Typische Themen | Ziel |
|---|---|---|
| Sofort | Datenoffenlegung, unsichere Adminzugänge, falsche Zahlungsbeträge, zentrale Ausfälle | Schaden und akute Blockaden beheben |
| Vor Veröffentlichung | Anwendbare Rechtspflichten, wesentliche Zugänglichkeit, reale Anfragewege, Backup und Zustellung | Tragfähigen Betrieb ermöglichen |
| Kurz danach | Weitere Performanceprobleme, Inhaltslücken, Komfort und Suchprobleme | Nutzung verbessern |
| Fortlaufend | Pflege, fachliche Aktualität, Tests nach Änderungen, Dienstleisterprüfung | Qualität erhalten |

Die konkrete Einordnung hängt von Risiko und Anwendungsbereich ab. Eine gesetzliche Pflicht darf nicht allein deshalb nach hinten verschoben werden, weil ihre Umsetzung unbequem ist.

### 25.2 Prüffelder nach Website-Typ

| Website-Typ | Besonders relevante Prüffelder |
|---|---|
| Privates Angebot | Einordnung persönlicher/familiärer Zweck, Datenschutz, Bild- und Inhaltsrechte |
| Unternehmenswebsite | Anbieterinformationen, Kontaktprozess, Datenschutz, Inhalte, Sicherheit |
| Landingpage | Kosten- und Leistungsversprechen, Lead-Formular, Werbung, Tracking |
| Blog/Redaktion | Verantwortlichkeit, Sorgfalt, Rechte, Kommentare, Werbekennzeichnung |
| Onlineshop | Verbraucherinformationen, Preise, Bestellung, Widerruf, Barrierefreiheit, Produktsicherheit |
| Abonnement | Wiederkehrende Kosten, Laufzeit, Kündigungsfunktion, Widerruf, Kontosicherheit |
| Kundenportal | Berechtigungen, Datenzugriff, Sitzungen, Wiederherstellung, Protokolle |
| Öffentliche Stelle | Einschlägige gesetzliche Barrierefreiheit, Datenschutz, öffentliche Informationspflichten |

### 25.3 Dienst oder Funktion hinzufügen: acht Fragen

1. Welches konkrete Nutzerproblem wird gelöst?
2. Gibt es eine einfachere Lösung?
3. Welche Daten fließen wohin?
4. Welche Rechtsgrundlage und Informationen werden benötigt?
5. Ist die Funktion zugänglich?
6. Was kostet sie an Ladezeit und Betrieb?
7. Was passiert bei Fehlern und Ausfall?
8. Wer pflegt sie und wie lässt sie sich wieder entfernen?

Wenn mehrere Antworten fehlen, sollte die Integration vor ihrer Veröffentlichung geklärt werden.

### 25.4 Bewusst begrenzter Ausgangspunkt

Für viele kleine Unternehmensauftritte ist ein guter Start: wenige gut geschriebene Seiten, klares Kontaktangebot, responsive Gestaltung, zugängliche Komponenten, sparsame Technik und geregelter Betrieb.

Weitere Funktionen können hinzukommen, wenn ein belegter Bedarf besteht. Ein Kundenkonto, Live-Chat oder Analysepaket sollte nicht allein aufgrund seiner Verfügbarkeit eingebaut werden.

<a id="kapitel-26"></a>
## 26. Ausführliche Checklisten

Die folgenden Listen dienen als Arbeitsinstrument. „Nicht anwendbar“ sollte begründet dokumentiert werden. Ein gesetztes Häkchen ersetzt bei rechtlichen oder komplexen technischen Fragen keine fachliche Prüfung.

### 26.1 Strategie und Inhalte

- [ ] Zweck und wichtigste Aufgaben sind beschrieben.
- [ ] Zielgruppen und relevante Nutzungssituationen sind bekannt.
- [ ] Seiteninventar und Seitentypen sind festgelegt.
- [ ] Navigation verwendet verständliche Begriffe.
- [ ] Angebot, Nutzen und nächste Schritte sind eindeutig.
- [ ] Preise und Einschränkungen sind nachvollziehbar.
- [ ] Fachliche Aussagen sind überprüft.
- [ ] Referenzen und Zahlen sind echt und belegbar.
- [ ] Medien helfen beim Verständnis.
- [ ] Jede wichtige Seite hat eine zuständige Person.
- [ ] Übersetzungen umfassen auch Formulare und Meldungen.
- [ ] Veraltete Inhalte werden entfernt oder sinnvoll ersetzt.

### 26.2 Design und mobile Nutzung

- [ ] Typografie ist auf echten Geräten gut lesbar.
- [ ] Visuelle Hierarchie unterstützt die wichtigsten Aufgaben.
- [ ] Komponenten und Zustände sind konsistent.
- [ ] Klickflächen sind ausreichend groß und voneinander getrennt.
- [ ] Navigation funktioniert per Touch und Tastatur.
- [ ] Zwischenbreiten und Querformat sind geprüft.
- [ ] Vergrößerte Schrift wird nicht abgeschnitten.
- [ ] Fixierte Elemente verdecken keine Inhalte oder Eingaben.
- [ ] Tabellen und Dialoge sind auf kleinen Displays nutzbar.
- [ ] Reduzierte Bewegung wird berücksichtigt.

### 26.3 Formulare und Nutzerwege

- [ ] Nur erforderliche Daten sind Pflichtangaben.
- [ ] Felder haben sichtbare und technische Beschriftungen.
- [ ] Autofill und passende Eingabetypen funktionieren.
- [ ] Fehler sind verständlich und den Feldern zugeordnet.
- [ ] Eingaben bleiben nach korrigierbaren Fehlern erhalten.
- [ ] Lade-, Erfolgs- und Fehlerzustände sind geprüft.
- [ ] Doppeltes Absenden verursacht keine Doppelverarbeitung.
- [ ] Tatsächlicher Anfrageeingang ist nachgewiesen.
- [ ] Bestätigungen machen keine falschen Zusagen.
- [ ] Spam-Abwehr ist verhältnismäßig und zugänglich.
- [ ] Newsletter und Kontaktanfrage sind getrennt.

### 26.4 Barrierefreiheit

- [ ] Rechtlicher Anwendungsbereich ist dokumentiert.
- [ ] Technisches Prüfziel und ergänzende Anforderungen sind festgelegt.
- [ ] Überschriften und Bereiche sind sinnvoll strukturiert.
- [ ] Linktexte und Buttonnamen sind verständlich.
- [ ] Alle Kernabläufe funktionieren per Tastatur.
- [ ] Fokus ist sichtbar und wird nicht verdeckt.
- [ ] Dialoge haben eine nachvollziehbare Fokusführung.
- [ ] Bilder besitzen passende Alternativen.
- [ ] Kontraste und nicht nur farbliche Zustände sind geprüft.
- [ ] Textvergrößerung und Reflow funktionieren.
- [ ] Formulare und Fehler sind zugänglich.
- [ ] Medienalternativen sind anhand des Inhalts geprüft.
- [ ] Automatische und manuelle Prüfungen sind dokumentiert.
- [ ] Eingebundene Buchungs- und Zahlungsabläufe sind mitgeprüft.
- [ ] Erforderliche Informationen zur Barrierefreiheit sind vorhanden.
- [ ] Ein erreichbarer Weg für Rückmeldungen ist eingerichtet.

### 26.5 Datenschutz und Einwilligung

- [ ] Verarbeitungen und Datenflüsse sind erfasst.
- [ ] Zwecke und Rechtsgrundlagen sind geprüft.
- [ ] Dienstleisterrollen und notwendige Verträge sind geklärt.
- [ ] Drittländer und tatsächliche Empfänger sind bewertet.
- [ ] Datenschutzerklärung entspricht der Umsetzung.
- [ ] Erforderliche Informationen erscheinen rechtzeitig.
- [ ] Löschfristen und gesetzliche Aufbewahrung sind unterschieden.
- [ ] Betroffenenanfragen haben einen zuständigen Prozess.
- [ ] Bedarf für Datenschutzbeauftragten und Folgenabschätzung ist geprüft.
- [ ] Tracking und Gerätezugriffe sind nach TDDDG und DSGVO bewertet.
- [ ] Ein erforderliches Banner ermöglicht wirksame Auswahl.
- [ ] Optionale Dienste starten nicht vor erforderlicher Zustimmung.
- [ ] Ablehnung und späterer Widerruf sind getestet.
- [ ] Protokolle und Analyseereignisse enthalten keine unnötigen Daten.
- [ ] Datenschutzvorfälle haben einen vorbereiteten Meldeprozess.

### 26.6 Anbieter, Inhalte und Werbung

- [ ] Betreiber und Anbieterpflichten sind korrekt eingeordnet.
- [ ] Impressum enthält die einschlägigen aktuellen Angaben.
- [ ] Journalistisch-redaktionelle Verantwortlichkeit ist geprüft.
- [ ] Bilder, Texte, Schriften, Icons und Software haben passende Rechte.
- [ ] Lizenznachweise und erforderliche Namensnennungen liegen vor.
- [ ] Personenfotos und Kundenreferenzen sind rechtlich geprüft.
- [ ] Werbung und Kooperationen sind transparent.
- [ ] Newslettergrundlage und Nachweise sind dokumentiert.
- [ ] Abmeldung funktioniert tatsächlich.
- [ ] Bewertungen und Echtheitsinformationen sind geprüft.
- [ ] Umweltbehauptungen und Siegel sind belastbar geprüft.
- [ ] KI-Funktionen sind auf einschlägige Transparenzpflichten geprüft.

### 26.7 Shop und Verträge, soweit anwendbar

- [ ] Kundengruppe und Vertragstypen sind eingeordnet.
- [ ] Vorvertragliche Informationen sind vollständig und rechtzeitig.
- [ ] Gesamtpreise, weitere Kosten und Grundpreise sind korrekt.
- [ ] Preisermäßigungen werden korrekt dargestellt.
- [ ] Laufzeiten und wiederkehrende Kosten sind eindeutig.
- [ ] Letzte Bestellseite und Bestellbutton sind geprüft.
- [ ] AGB passen und werden wirksam einbezogen.
- [ ] Widerrufsbelehrung und Musterinformationen sind passend.
- [ ] Ausnahmen bzw. Erlöschen sind korrekt umgesetzt.
- [ ] Elektronische Widerrufsfunktion ist bei Anwendbarkeit vorhanden.
- [ ] Kündigungsfunktion ist bei Anwendbarkeit vorhanden.
- [ ] Bestätigungen sind vollständig und werden zugestellt.
- [ ] Zahlung, erneuter Versuch und Rückerstattung sind getestet.
- [ ] Produktsicherheits- und Sortimentspflichten sind geprüft.
- [ ] Aktuelle Garantie- und Gewährleistungsinformationen sind geprüft.
- [ ] Verbraucherstreitbeilegung ist korrekt behandelt.
- [ ] Veraltete Hinweise auf die geschlossene OS-Plattform sind entfernt.

### 26.8 Technik, Sicherheit und Betrieb

- [ ] HTTPS und Zertifikatsüberwachung funktionieren.
- [ ] Adminzugänge sind individuell und angemessen abgesichert.
- [ ] Rechte werden serverseitig geprüft.
- [ ] Private Dateien und APIs sind geschützt.
- [ ] Passwörter und Sitzungen sind sicher behandelt.
- [ ] Eingaben, Ausgaben und Datenbankzugriffe sind abgesichert.
- [ ] Uploads haben geprüfte Grenzen und sicheren Speicher.
- [ ] Sicherheitsheader sind bewusst konfiguriert und getestet.
- [ ] Geheimnisse liegen nicht in öffentlichen Dateien.
- [ ] Testumgebung ist geschützt und von Produktion getrennt.
- [ ] Abhängigkeiten und Updateverantwortung sind dokumentiert.
- [ ] Backups sind unabhängig geschützt.
- [ ] Wiederherstellung ist tatsächlich getestet.
- [ ] Domain, DNS und relevante Konten sind kontrolliert.
- [ ] Mailzustellung und Fehlermeldungen sind geprüft.
- [ ] Überwachung erreicht zuständige Personen.
- [ ] Rücknahmeplan und Notfallkontakte sind vorhanden.

### 26.9 Performance, SEO und Launch

- [ ] Wichtige Seitentypen sind auf Geschwindigkeit geprüft.
- [ ] Bilder besitzen passende Größen und reservierte Abmessungen.
- [ ] Zentrale erste Inhalte werden nicht unnötig verzögert.
- [ ] Externe Skripte haben einen begründeten Nutzen.
- [ ] Cache-Regeln schützen private Inhalte.
- [ ] Titel, Links und URLs sind sinnvoll.
- [ ] Statuscodes und Weiterleitungen sind korrekt.
- [ ] Indexierungsanweisungen der Produktion sind kontrolliert.
- [ ] Sitemap und Canonicals passen zum Inhalt.
- [ ] Bestehende wichtige URLs sind beim Relaunch berücksichtigt.
- [ ] Finale Inhalte und tatsächliche Datenflüsse sind abgeglichen.
- [ ] Kernabläufe wurden nach Veröffentlichung nochmals geprüft.
- [ ] Übergabe und Wartungsbudget sind geklärt.

<a id="kapitel-27"></a>
## 27. Vorlagen für die eigene Dokumentation

### 27.1 Seiteninventar

| Seite/URL | Aufgabe der Besucher | Zentrale Inhalte | Zuständig | Letzte Prüfung | Nächste Prüfung |
|---|---|---|---|---|---|
| Startseite | Angebot verstehen | Nutzen, Leistungen, Einstieg | Ausfüllen | Ausfüllen | Ausfüllen |
| Kontakt | Anfrage stellen | Kontaktwege, Formular, Hinweise | Ausfüllen | Ausfüllen | Ausfüllen |
| Leistungsseite | Eignung beurteilen | Umfang, Voraussetzungen, Ablauf | Ausfüllen | Ausfüllen | Ausfüllen |

### 27.2 Diensteregister

| Dienst | Zweck | Datenarten | Rolle/Vertrag | Länder/Empfänger | Einwilligungsprüfung | Löschung | Zuständig |
|---|---|---|---|---|---|---|---|
| Hosting | Website bereitstellen | Tatsächlich erfasste Daten | Prüfen | Prüfen | Prüfen | Festlegen | Ausfüllen |
| E-Mail-Versand | Nachrichten zustellen | Tatsächlich übermittelte Daten | Prüfen | Prüfen | Prüfen | Festlegen | Ausfüllen |
| Analyse, falls genutzt | Nutzung auswerten | Tatsächlich erhobene Daten | Prüfen | Prüfen | Prüfen | Festlegen | Ausfüllen |

### 27.3 Rechteverzeichnis

| Datei/Inhalt | Rechteinhaber/Quelle | Lizenznachweis | Erlaubte Nutzung | Pflichtangaben | Einschränkungen |
|---|---|---|---|---|---|
| Foto | Ausfüllen | Hinterlegen | Prüfen | Prüfen | Prüfen |
| Schrift | Ausfüllen | Hinterlegen | Webnutzung prüfen | Prüfen | Prüfen |
| Kundenreferenz | Ausfüllen | Freigabe hinterlegen | Umfang klären | Prüfen | Prüfen |

### 27.4 Prüfprotokoll

| Datum | Version/Umgebung | Geprüfter Ablauf | Methode/Gerät | Ergebnis | Befund/Aufgabe | Verantwortlich |
|---|---|---|---|---|---|---|
| Ausfüllen | Ausfüllen | Kontaktanfrage | Echte Zustellung | Ausfüllen | Ausfüllen | Ausfüllen |
| Ausfüllen | Ausfüllen | Navigation und Dialog | Tastatur | Ausfüllen | Ausfüllen | Ausfüllen |
| Ausfüllen | Ausfüllen | Einwilligung ablehnen | Frisches Browserprofil | Ausfüllen | Ausfüllen | Ausfüllen |

### 27.5 Entscheidung über eine neue Integration

```text
Name der Funktion:
Nutzerproblem und erwarteter Nutzen:
Geprüfte einfachere Alternativen:
Anbieter und technische Abhängigkeiten:
Verarbeitete Daten und Empfänger:
Rechtsgrundlagen und erforderliche Informationen:
Einwilligung/Endgerätezugriff geprüft:
Dienstleisterrolle und Vertragsstatus:
Drittlandprüfung:
Barrierefreiheit und Kernabläufe geprüft:
Auswirkung auf Ladezeit und Betrieb:
Fehler- und Ausfallverhalten:
Kosten und laufende Zuständigkeit:
Löschung, Export und Entfernung:
Prüfergebnis und offene Aufgaben:
Freigabe durch zuständige Person am:
```

### 27.6 Kurzbeschreibung für einen Vorfall

```text
Zeitpunkt der Entdeckung:
Meldende Person und Kontakt:
Betroffenes System und beobachtetes Verhalten:
Bekannter Zeitraum:
Betroffene Daten/Funktionen:
Bereits ergriffene Maßnahmen:
Gesicherte Nachweise und Protokolle:
Technisch verantwortliche Person:
Datenschutz-/Rechtsbewertung zuständig:
Fristen und erforderliche Entscheidungen:
Wiederherstellung und Nachprüfung:
Ursache und dauerhafte Verbesserung:
```

<a id="kapitel-28"></a>
## 28. Häufige Irrtümer

| Irrtum | Korrektur |
|---|---|
| Jede Website braucht ein Cookie-Banner. | Entscheidend sind die tatsächlich eingesetzten Speicher-, Zugriffs- und Verarbeitungsvorgänge. |
| Ein Cookie-Banner macht die Website DSGVO-konform. | Rechtsgrundlagen, Informationen, Verträge, Löschung, Sicherheit und technische Sperren sind separat zu prüfen. |
| Ohne Cookies ist Tracking immer frei zulässig. | Auch andere Techniken und die Verarbeitung personenbezogener Daten können Regeln auslösen. |
| Die Datenschutzerklärung muss immer akzeptiert werden. | Sie dient der Information; eine notwendige Einwilligung muss gesondert begründet und gestaltet werden. |
| Ein EU-Server löst jede Drittlandfrage. | Weitere Empfänger, Supportzugriffe und Unterauftragnehmer sind zu betrachten. |
| BFSG gilt automatisch für jede Internetseite. | Anwendungsbereich, Dienstleistung und Ausnahmen sind zu prüfen. |
| Kleine Unternehmen sind von allen Anforderungen ausgenommen. | Ausnahmen beziehen sich auf konkrete Vorschriften und Voraussetzungen. |
| Ein Accessibility-Overlay garantiert Rechtskonformität. | Die gesamte tatsächliche Nutzung und einschlägige Anforderungen müssen geprüft werden. |
| Lighthouse 100 bedeutet eine perfekte Website. | Automatische Werte decken nur ausgewählte Aspekte ab. |
| HTTPS macht jede Webanwendung sicher. | Transportverschlüsselung löst keine Berechtigungs- oder Programmfehler. |
| Ein versteckter Button verhindert unzulässige Aktionen. | Berechtigungen müssen auf dem Server geprüft werden. |
| robots.txt schützt private Dokumente. | Sie ist kein Zugangsschutz. |
| Ein tägliches Backup garantiert Wiederherstellung. | Restore, Vollständigkeit und Unabhängigkeit müssen getestet werden. |
| Bilder aus der Suchmaschine sind frei nutzbar. | Die Rechte müssen für die konkrete Nutzung geklärt sein. |
| Ein Download hat grundsätzlich kein Widerrufsrecht. | Die gesetzlichen Voraussetzungen und der konkrete Ablauf sind maßgeblich. |
| Kündigung und Widerruf sind dasselbe. | Es sind unterschiedliche Rechtsvorgänge mit unterschiedlichen Voraussetzungen. |
| Alte Shop-Rechtstexte bleiben dauerhaft passend. | Funktionen, Anbieter und geltende Regeln ändern sich. |
| Serverseitiges Tracking braucht keine Prüfung. | Datenflüsse, Zwecke, Endgerätezugriffe und Rechtsgrundlagen bleiben relevant. |
| Mehr Plugins machen die Website besser. | Jede Abhängigkeit erhöht Pflege-, Ausfall- und gegebenenfalls Datenschutzaufwand. |
| Nach dem Launch ist das Projekt abgeschlossen. | Betrieb, Inhalte, Sicherheit und rechtliche Aktualität benötigen laufende Verantwortung. |

<a id="kapitel-29"></a>
## 29. Glossar

| Begriff | Bedeutung |
|---|---|
| Accessibility/Barrierefreiheit | Zugänglichkeit und Bedienbarkeit für Menschen mit unterschiedlichen Fähigkeiten |
| API | Schnittstelle, über die Systeme oder Programmteile Daten und Funktionen austauschen |
| Auftragsverarbeitung | Verarbeitung personenbezogener Daten im Auftrag und nach Weisung eines Verantwortlichen |
| BFSG/BFSGV | Deutsches Barrierefreiheitsstärkungsgesetz und zugehörige Verordnung |
| BITV | Verordnung zur barrierefreien Informationstechnik für ihren jeweiligen Anwendungsbereich |
| CDN | Verteilte Infrastruktur zur Auslieferung von Inhalten |
| CLS | Metrik für unerwartete Layoutverschiebungen |
| CMS | System zum Erstellen und Verwalten von Websiteinhalten |
| CSP | Browserregelwerk zur Begrenzung bestimmter Inhaltsquellen und Ausführungen |
| CSRF | Angriff, bei dem ein Browser zu einer unerwünschten Anfrage im Kontext eines Nutzers gebracht wird |
| DDG | Digitale-Dienste-Gesetz |
| DNS | System zur Zuordnung von Domainnamen zu technischen Zielen und weiteren Einträgen |
| DSGVO | Datenschutz-Grundverordnung |
| EWR | Europäischer Wirtschaftsraum: EU sowie Island, Liechtenstein und Norwegen |
| INP | Metrik für die Reaktionsfähigkeit bei Interaktionen |
| LCP | Metrik für das Sichtbarwerden eines großen relevanten Inhalts |
| MFA | Anmeldung mit mehreren unabhängigen Faktoren |
| MStV | Medienstaatsvertrag |
| Progressive Enhancement | Robuste Grundfunktion mit schrittweise ergänzten Verbesserungen |
| Responsive Design | Anpassung an unterschiedliche Bildschirm- und Inhaltsbedingungen |
| RPO/RTO | Ziele für tolerierbaren Datenverlust und Wiederherstellungsdauer |
| SEO | Maßnahmen zur Auffindbarkeit relevanter Inhalte in Suchmaschinen |
| TDDDG | Telekommunikation-Digitale-Dienste-Datenschutz-Gesetz |
| UX | Gesamtes Nutzungserlebnis eines Angebots |
| WCAG | Web Content Accessibility Guidelines |
| XSS | Angriff durch unerwünschte Ausführung eingeschleuster Inhalte im Browserkontext |

<a id="kapitel-30"></a>
## 30. Quellen und weiterführende Grundlagen

Die Quellenliste besteht überwiegend aus Gesetzestexten, offiziellen Behördeninformationen und Originaldokumentation. Sie dient zur Vertiefung und Aktualisierung. Die rechtliche Einordnung des konkreten Angebots kann zusätzliche Quellen und aktuelle Rechtsprechung erfordern. Nicht jede verlinkte Übersicht beantwortet sämtliche Sonderfälle.

### 30.1 Anbieter, Datenschutz und Endgeräte

- [DDG § 5 – Anbieterinformationen](https://www.gesetze-im-internet.de/ddg/__5.html)
- [Medienstaatsvertrag – Medienanstalten](https://www.die-medienanstalten.de/fileadmin/user_upload/Rechtsgrundlagen/Gesetze_Staatsvertraege/Medienstaatsvertrag_MStV.pdf)
- [DSGVO – EUR-Lex](https://eur-lex.europa.eu/legal-content/DE/TXT/?uri=CELEX:32016R0679)
- [EU-Kommission – Datenschutz erklärt](https://commission.europa.eu/law/law-topic/data-protection/data-protection-explained_en)
- [EU-Kommission – Rechte von Einzelpersonen](https://commission.europa.eu/law/law-topic/data-protection/information-individuals_en)
- [BfDI – DSGVO und BDSG](https://www.bfdi.bund.de/SharedDocs/Downloads/DE/Broschueren/INFO1.pdf?__blob=publicationFile&v=24)
- [BfDI – Muster zur Auftragsverarbeitung](https://www.bfdi.bund.de/SharedDocs/Downloads/DE/Muster/Muster_zur_Auftragsverarbeitung.pdf?__blob=publicationFile&v=2)
- [BfDI – Erläuterungen zur DSGVO in der Bundesverwaltung](https://www.bfdi.bund.de/SharedDocs/Downloads/DE/Broschueren/INFO6.pdf?__blob=publicationFile&v=9)
- [BayLDA – Drittlandübermittlungen](https://www.lda.bayern.de/de/thema_uebermittlung_personenbezogener_daten_in_drittlaender.html)
- [LfDI Baden-Württemberg – Drittstaatentransfer](https://www.baden-wuerttemberg.datenschutz.de/drittstaatentransfer-handreichung-kap-v-dsgvo/)
- [BDSG § 38 – Datenschutzbeauftragte nichtöffentlicher Stellen](https://www.gesetze-im-internet.de/bdsg_2018/__38.html)
- [TDDDG § 25 – Speicherung und Zugriff auf Endeinrichtungen](https://www.gesetze-im-internet.de/ttdsg/__25.html)
- [DSK – Orientierungshilfe Digitale Dienste](https://datenschutzkonferenz-online.de/media/oh/OH_Digitale_Dienste.pdf)

### 30.2 Barrierefreiheit

- [BFSG – vollständiger Gesetzestext](https://www.gesetze-im-internet.de/bfsg/BJNR297010021.html)
- [BFSG § 14 – Dienstleistungserbringer](https://www.gesetze-im-internet.de/bfsg/__14.html)
- [BFSGV § 19 – elektronischer Geschäftsverkehr](https://www.gesetze-im-internet.de/bfsgv/__19.html)
- [Bundesfachstelle Barrierefreiheit – BFSG](https://www.bundesfachstelle-barrierefreiheit.de/DE/Barrierefreiheitsstaerkungsgesetz)
- [Bundesfachstelle – Erläuterungen zu bestehenden Websites](https://www.bundesfachstelle-barrierefreiheit.de/SharedDocs/Downloads/DE/BFSG-Newsletter/bfsg-newsletter-1-2025.pdf?__blob=publicationFile)
- [W3C – WCAG 2.2](https://www.w3.org/TR/WCAG22/)
- [W3C/WAI – WCAG-Quickref](https://www.w3.org/WAI/WCAG22/quickref/)
- [BITV 2.0](https://www.gesetze-im-internet.de/bitv_2_0/)

### 30.3 Verbraucher, Werbung und Inhalte

- [EGBGB Art. 246a § 1 – Verbraucherinformationen](https://www.gesetze-im-internet.de/bgbeg/art_246a__1.html)
- [EGBGB Art. 246a § 4 – formale Anforderungen](https://www.gesetze-im-internet.de/bgbeg/art_246a__4.html)
- [BGB § 312j – Bestellsituation](https://www.gesetze-im-internet.de/bgb/__312j.html)
- [BGB § 312k – Kündigungsfunktion](https://www.gesetze-im-internet.de/bgb/__312k.html)
- [BGB § 355 – Widerruf](https://www.gesetze-im-internet.de/bgb/__355.html)
- [BGB § 356 – Fristbeginn und Erlöschen](https://www.gesetze-im-internet.de/bgb/__356.html)
- [BGB § 356a – elektronische Widerrufsfunktion](https://www.gesetze-im-internet.de/bgb/__356a.html)
- [BGB § 312g – Widerrufsrecht und Ausnahmen](https://www.gesetze-im-internet.de/bgb/__312g.html)
- [Preisangabenverordnung](https://www.gesetze-im-internet.de/pangv_2022/)
- [VSBG § 36 – allgemeine Informationspflicht](https://www.gesetze-im-internet.de/vsbg/__36.html)
- [EU-Kommission – eingestellte OS-Plattform](https://consumer-redress.ec.europa.eu/site-relocation_en)
- [UWG § 7 – unzumutbare Belästigungen](https://www.gesetze-im-internet.de/uwg_2004/__7.html)
- [UWG – vollständiger Gesetzestext](https://www.gesetze-im-internet.de/uwg_2004/)
- [UrhG § 19a – öffentliche Zugänglichmachung](https://www.gesetze-im-internet.de/urhg/__19a.html)
- [KunstUrhG § 22 – Bildnisse](https://www.gesetze-im-internet.de/kunsturhg/__22.html)
- [GPSR – EU-Produktsicherheitsverordnung](https://eur-lex.europa.eu/eli/reg/2023/988/oj)
- [Richtlinie (EU) 2024/825 – Verbraucher und ökologische Transformation](https://eur-lex.europa.eu/eli/dir/2024/825/oj)
- [EU-Kommission – nachhaltiger Konsum und Garantieinformationen](https://commission.europa.eu/topics/consumers/consumer-rights-and-complaints/sustainable-consumption_en)
- [Digital Services Act](https://eur-lex.europa.eu/eli/reg/2022/2065/oj)
- [EU-Kommission – KI-Transparenzleitlinien](https://digital-strategy.ec.europa.eu/en/policies/guidelines-ai-transparency-obligations)

### 30.4 Technik, Performance und Sicherheit

- [Google/web.dev – Web Vitals](https://web.dev/articles/vitals)
- [Google Search Central – SEO-Startleitfaden](https://developers.google.com/search/docs/fundamentals/seo-starter-guide?hl=de)
- [OWASP – ASVS](https://devguide.owasp.org/en/06-verification/01-guides/03-asvs/)
- [OWASP – Berechtigungen](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html)
- [OWASP – Authentifizierung](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [OWASP – Passwortspeicherung](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [OWASP – Sitzungsverwaltung](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [OWASP – Datei-Uploads](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html)
- [OWASP – Content Security Policy](https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html)
- [OWASP – XSS-Prävention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
- [OWASP – SQL-Injection-Prävention](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html)

### 30.5 Wie dieser Leitfaden aktuell gehalten werden sollte

Prüfe bei Änderungen der Website erneut die betroffenen Funktionen, Datenflüsse und Texte. Besondere Anlässe sind neue Trackingdienste, Shopfunktionen, KI-Systeme, internationale Märkte, Produktgruppen oder Betreiberangaben.

Bei rechtlichen Änderungen zählt die tatsächlich geltende Fassung einschließlich Übergangsregeln. Ein Entwurf oder eine angekündigte Reform ist nicht automatisch geltendes Recht. Dokumentiere die verwendeten Quellen und den Zeitpunkt der Prüfung.
