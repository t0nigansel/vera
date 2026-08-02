# learnISTQB – Produktvision

## Vision

learnISTQB ist eine prüfungsorientierte Lernplattform für ISTQB-Zertifizierungen. Sie verbindet die offiziellen Lernquellen mit aktiven Übungsformen und einem gezielt eingesetzten KI-Tutor.

Die Plattform versorgt Lernende nicht nur mit Stoff und möglichst vielen Fragen. Sie erkennt, was bereits verstanden wurde, wo Fehlvorstellungen bestehen und welche nächste Aktivität den größten Fortschritt verspricht — und begleitet vom ersten Überblick bis zur bestandenen Prüfung.

## Positionierung

Ein CTFL-Kurs kostet im deutschsprachigen Raum typischerweise 1.500 bis 2.500 Euro, Advanced-Level-Kurse liegen darüber. Die Prüfungsgebühr von rund 250 Euro kommt in jedem Fall hinzu.

Für viele Kandidaten ist dieser Kurs nicht der beste Weg: feste Termine, das Tempo der Gruppe, derselbe Stoff, der ohnehin frei im Syllabus steht, und ein Format, das über den individuellen Wissensstand nichts weiß. Wer die Prüfung ohne Kurs besteht, tut das wegen der systematischen Eigenarbeit an den offiziellen Unterlagen — nicht trotz des fehlenden Kurses.

**learnISTQB ist die Alternative zum überteuerten Kurs.** Zielgruppe sind Menschen, die ihre Zertifizierung selbst bezahlen oder selbst organisieren: Tester, Testmanager, Entwickler, Business Analysts, Quality Engineers und Wiedereinsteiger. Versprochen wird dasselbe Ergebnis — die bestandene Prüfung — zu einem Bruchteil der Kosten und ohne feste Termine.

Der Zielpreis liegt deutlich unter zehn Prozent eines vergleichbaren Kurses. Das ist keine Marketingaussage, sondern eine Konstruktionsvorgabe: Sie bestimmt, wie viel Rechenleistung pro Nutzer vertretbar ist und welche Funktionen sich lohnen.

## Produktversprechen

learnISTQB beantwortet fortlaufend vier Fragen:

1. Was muss ich für meine Prüfung wissen und anwenden können?
2. Was davon beherrsche ich bereits zuverlässig?
3. Wo habe ich Lücken oder Fehlvorstellungen?
4. Was sollte ich als Nächstes tun?

Ein Syllabus beantwortet keine davon. Eine Musterlösung erkennt nicht, warum eine falsche Antwort plausibel erschien. Ein Chatbot ohne Lernlogik, Quellenbindung und Qualitätskontrolle ist keine verlässliche Prüfungsvorbereitung.

## Unterstützte Zertifizierungen

1. Certified Tester Foundation Level (CTFL) v4.0
2. Certified Tester Advanced Level Test Management (CTAL-TM) v3.0
3. Certified Tester AI Testing (CT-AI) v2.0

CTFL wird zuerst vollständig fertiggestellt — ein Kurs, der tatsächlich zur bestandenen Prüfung führt, ist überzeugender als drei mit lückenhaftem Fragenbestand. CTFL hat zudem die größte Kandidatenzahl.

Kurse und Quelldokumente werden unabhängig versioniert, damit nachvollziehbar bleibt, auf welcher Fassung eine Erklärung oder Bewertung beruht. Weitere Zertifizierungen lassen sich als versionierte Inhalte ergänzen, ohne die Lernplattform neu zu entwickeln.

## Lernmethoden

Die Plattform bietet bewusst wenige, sorgfältig gebaute Lernformen statt eines breiten Werkzeugkastens. Maßstab für jede Methode: Steht sie zwischen dem Nutzer und der bestandenen Prüfung?

### 1. Fragen üben

Der Arbeitspferd-Modus. Eine Frage, sofortiges Feedback, hinterlegte Begründung, Erklärung zu jeder Antwortoption, Quellenangabe.

Zwei Zusatzangaben je Antwort, beide deterministisch und ohne Modellkosten:

- **Sicherheit.** Der Nutzer gibt an, ob er sicher war, unsicher oder geraten hat.
- **Begründung als Auswahl.** Nach der Antwort eine zweite Auswahlfrage nach dem Denkweg. Das deckt den Fall auf, den keine Trefferquote sieht: richtige Antwort aus falschem Grund — bei vier Optionen häufig, weil sich zwei oft sicher ausschließen lassen.

Aus Ergebnis und Sicherheit ergeben sich vier Fälle: sicher und richtig (echtes Wissen), unsicher und richtig (vermutlich geraten, zählt nicht als Beherrschung), unsicher und falsch (bekannte Lücke), **sicher und falsch** (echte Fehlvorstellung — der wertvollste Moment im Lernprozess und der bevorzugte Auslöser für eine Tutorinteraktion).

Da jedem Distraktor bereits eine Fehlvorstellung zugeordnet ist, benennt das System die wahrscheinliche Ursache ohne Modellaufruf: Begriff nicht erinnert, ähnliche Begriffe verwechselt, Regel auswendig gelernt statt verstanden, Prinzip nicht übertragen, Frage missverstanden oder geraten.

### 2. Begriffstraining

ISTQB prüft zu einem erheblichen Teil die exakte Beherrschung offizieller Begriffe, besonders im Foundation Level. Trainiert werden ausschließlich die für die Zertifizierung und Version relevanten Begriffe, nicht das Gesamtglossar.

**Abfragerichtungen** mit steigendem Anspruch: Begriff → Definition, Definition → Begriff, **Szenario → passender Begriff** (entspricht der Prüfungssituation und hat das höchste Gewicht), Begriff → zugehöriges Thema.

**Verwechslungscluster** sind der Kern. Die Prüfung testet selten einzelne Begriffe, sondern die Abgrenzung ähnlicher: Fehler / Fehlerzustand / Fehlerwirkung, Verifizierung / Validierung, Testfall / Testszenario / Testsuite, Testüberwachung / Teststeuerung, Testen / Debugging. Genau aus diesen Nachbarschaften werden die Distraktoren der echten Prüfung gebildet. Ein Trainer, der Begriffe isoliert abfragt, übt am Prüfungsmechanismus vorbei — und ist zudem austauschbar, denn kostenlose Karteikartensammlungen gibt es reichlich.

**Prüfungssprache** wird getrennt trainiert. Übersetzungsunschärfen zwischen englischer und deutscher Fassung sind selbst eine bekannte Fehlerquelle.

**Grenze.** Begriffsbeherrschung deckt Wissen und Verständnis ab, nicht Anwendung. Äquivalenzklassen bilden, Grenzwerte bestimmen, Entscheidungstabellen auswerten oder eine Teststrategie beurteilen lässt sich nicht drillen.

### 3. Probetest

Ein Messinstrument, kein Lernwerkzeug — üblicherweise vier bis sechs Durchläufe in der gesamten Vorbereitung. Fragenanzahl, Zeitvorgabe und Kapitelverteilung entsprechen der echten Prüfung, Feedback gibt es erst am Ende.

Jede Simulation braucht Fragen, die der Nutzer noch nie gesehen hat, sonst misst sie Erinnerung statt Wissen. Für vier saubere Durchläufe sind das 160 exklusiv reservierte Fragen zusätzlich zum Übungsbestand — eine harte Untergrenze der Inhaltsplanung.

Die Auswertung zeigt Punkte und Bestehensbewertung, Ergebnisse nach Kapitel, Learning Objective und K-Level, den Abgleich von Sicherheitsangabe und Ergebnis, wiederkehrende Fehlvorstellungen und Empfehlungen für die nächste Lernphase.

### 4. Erklären lassen

Die drei bisherigen Methoden sind Auswahlaufgaben: Der Nutzer erkennt wieder, er ruft nicht ab. Deshalb kann man ISTQB-Fragen auswendig lernen und trotzdem nichts können.

Freies Erklären schließt diese Lücke und ist die einzige Methode, für die ein Sprachmodell unverzichtbar ist. Sie wird deshalb **gezielt ausgelöst** statt dauerhaft angeboten: nach "sicher und falsch", bei wiederholt verfehlten Begriffen eines Verwechslungsclusters und vor dem ersten Probetest als Selbsteinschätzung. Bewertet wird gegen eine hinterlegte Rubrik; das Feedback unterscheidet korrekt genannte Aspekte, fehlende Aspekte, sachlich falsche Aussagen und vermutete Fehlvorstellungen.

### 5. Wiederholung

Kein eigener Modus, sondern ein Planer über den Methoden 1 und 2. Verteiltes Wiederholen ist der wirksamste bekannte Mechanismus, um Wissen bis zum Prüfungstag verfügbar zu halten, und verursacht keine Modellkosten.

Geplant wird auf Ebene einzelner Learning Objectives und Begriffe: Was sicher saß, kommt seltener wieder; was wackelte, früher. Bei gesetztem Prüfungsdatum wird der Plan so verdichtet, dass der gesamte Stoff bis dahin mehrfach abgerufen wurde. Ein Fragenkatalog ohne Wiederholungsplanung ist ein Quiz — erst die Wiederholung macht daraus eine Prüfungsvorbereitung.

### 6. Lesen

Offene Entscheidung mit erheblicher Tragweite. Entweder liefert die App eigene Erklärtexte pro Lernziel — dann ist sie vollständiger Kursersatz, kostet aber deutliche Redaktionsarbeit. Oder sie verweist auf Syllabus-Abschnitte und fragt ab — dann ist sie ein Trainingswerkzeug neben dem Syllabus und nicht die Kursalternative aus der Positionierung.

Angestrebt wird der erste Weg, schrittweise: eigene Erklärungen zuerst dort, wo die Auswertung zeigt, dass Nutzer tatsächlich scheitern. Der Syllabus erklärt vieles ausreichend; Redaktionsarbeit lohnt nur, wo er es nicht tut.

### Nicht vorgesehen

Vorgelesene Inhalte, generierte Lernvideos, erzeugte Bilder und Mind Maps. Hoher Produktionsaufwand, schwache Wirkung, und nichts davon steht zwischen dem Nutzer und der bestandenen Prüfung.

## Lernstand und Prüfungsreife

Der Fortschritt wird pro Kurs, Kapitel, Learning Objective, K-Level und Begriff geführt. Er beruht auf Lernnachweisen, nicht auf Nutzungsdauer — Lesen allein erhöht ihn nicht.

Die zentrale Zahl beantwortet die Frage, auf die alles zuläuft: **Bin ich so weit?** Sie berücksichtigt Abdeckung der Lernziele, Leistung je K-Level, Ergebnisse der Probetests, den Abstand zur letzten Wiederholung und die Übereinstimmung von Sicherheitsangabe und Ergebnis.

Begriffsbeherrschung ist dabei gedeckelt: Solange die Anwendungsziele nicht durch entsprechende Aufgaben belegt sind, bleibt die Prüfungsreife unterhalb der Bestehensschwelle. Ein vollständig grünes Glossar bedeutet nicht prüfungsbereit.

Die Anzeige ist bewusst konservativ. Ein Nutzer, der auf Basis dieser Zahl antritt und durchfällt, ist der schwerste Vertrauensschaden, den das Produkt erleiden kann.

## Fragenkorpus

Der Fragenbestand ist die kritischste Ressource des Produkts. Ohne genügend Fragen auf realistischem Niveau nützt die beste Lernlogik nichts.

### Grundsatz

**learnISTQB verwendet ausschließlich selbst erstellte Fragen.** Geleakte oder aus dem Gedächtnis rekonstruierte Prüfungsfragen werden nicht verwendet — weder als Inhalt noch als Vorlage:

- Sie sind urheberrechtlich geschützt und ihre Veröffentlichung verletzt in aller Regel die Verschwiegenheitsvereinbarung der Prüfungsteilnahme.
- Sie enthalten Übertragungsfehler, falsch markierte Lösungen und veraltete Syllabus-Bezüge.
- Wer sie auswendig lernt, besteht möglicherweise, versteht den Stoff aber nicht — das Gegenteil dessen, wofür learnISTQB gebaut wird.

Dass alle Fragen selbst erstellt sind, ist gegenüber Braindump-Angeboten eine Stärke und wird offen kommuniziert.

### Warum Eigenerstellung tragfähig ist

ISTQB-Prüfungen sind stark formalisiert: Jede Frage bildet ein Learning Objective auf einem definierten K-Level ab, die Kapitelverteilung ergibt sich aus dem Syllabus, die Fragetypen wiederholen sich. Was geprüft werden darf, ist öffentlich und präzise spezifiziert. Benötigt wird nicht die Originalfrage, sondern das Bauprinzip.

### Prüfungsniveau als bewusster Parameter

Die veröffentlichten Beispielprüfungen sind **deutlich einfacher als die tatsächliche Prüfung**. Reale Fragen sind länger, stärker in Szenarien eingebettet und weicher formuliert; die Auswahl erfolgt über Abwägung zwischen mehreren vertretbaren Optionen statt über Wiedererkennen einer Definition.

Musterprüfungen taugen daher als Untergrenze, nicht als Maßstab. Der Schwierigkeitsgrad des eigenen Bestands wird bewusst darüber angesetzt, im Register der echten Prüfung.

### Fragen-Stilrichtlinie

Das erfahrungsbasierte Wissen darüber, wie ISTQB-Prüfungen tatsächlich gebaut sind, wird in einer verbindlichen Stilrichtlinie festgehalten. Sie geht in jede Fragenerstellung ein und umfasst mindestens:

- Kapitelgewichtung und typische Fragetypen,
- Abgrenzung der K-Level, insbesondere K2 gegen K3,
- Aufbau und Länge realistischer Szenario-Stämme,
- die wiederkehrenden Distraktor-Muster, etwa benachbarter Glossarbegriff, plausible Alltagslogik oder eine für eine andere Teststufe zutreffende Aussage,
- Formulierungsregeln einschließlich der weichen Frageformen und unzulässiger Muster,
- den Katalog bekannter Fehlvorstellungen aus der Prüfungspraxis.

Diese Richtlinie lässt sich nicht aus öffentlichen Quellen zusammentragen. Sie ist der eigentliche inhaltliche Wettbewerbsvorteil.

### Erstellung und Freigabe

Fragen entstehen in Serien gegen die Stilrichtlinie, werden redaktionell geprüft und erst nach Freigabe ausgeliefert. Jede freigegebene Frage trägt dauerhaft: Zuordnung zu Kurs, Kapitel und Learning Objective, K-Level, Lösung und Begründung, Erklärung je Antwortoption, hinterlegte Fehlvorstellung je Distraktor sowie Herkunft und Erstellungsversion.

### Kalibrierung

Da keine offiziellen Fragen ausgeliefert werden, muss die Angemessenheit des Schwierigkeitsgrads anders belegt werden — über die Auswertung des Antwortverhaltens (Fragen, die praktisch alle richtig beantworten, sind zu leicht) und über freiwillige Rückmeldung nach der tatsächlichen Prüfung. Der Abgleich zwischen angezeigter Prüfungsreife und echtem Ergebnis ist der einzige belastbare Nachweis und zugleich das stärkste Argument gegenüber Interessenten. Bis eine tragfähige Datenbasis vorliegt, wird konservativ ausgewiesen.

## Ökonomie des KI-Einsatzes

Ein Sprachmodell über die gesamte Lernphase mitlaufen zu lassen, ist weder nötig noch wirtschaftlich tragfähig. Bei einem Produkt, dessen Positionierung ein Preisvergleich ist, sind die Betriebskosten pro Nutzer eine Architekturentscheidung.

Leitsatz: **Der überwiegende Teil der KI-Arbeit gehört in die Inhaltserstellung, nicht in die Laufzeit.**

**Einmalig erzeugt, dauerhaft ausgeliefert:** Übungsfragen samt Distraktoren, Begründungen, Erklärungen je Antwortoption, Fehlvorstellungs-Katalog, Standarderklärungen zu Lernzielen und Begriffen, Transferaufgaben.

**Zur Laufzeit unvermeidbar:** Bewertung frei formulierter Antworten gegen eine Rubrik, sokratischer Dialog zu einer konkreten Lücke, Analyse einer geschriebenen Begründung, Zusammenfassung des Lernstands.

**Vollständig deterministisch:** Navigation, Multiple-Choice-Bewertung, Punkteberechnung, Prüfungsregeln, Fortschritt, Wiederholungsplanung, Auswahl der nächsten Aktivität, Zuordnung von Distraktor zu Fehlvorstellung.

Ein Nutzer, der liest, Fragen beantwortet und wiederholt, verursacht damit keine laufenden Modellkosten. Der Tutor ist eine gezielte Vertiefung, kein Dauerbegleiter.

## Rolle der künstlichen Intelligenz

Das Sprachmodell ist nicht die Wissensquelle und nicht der Besitzer des Lernprozesses. Seine Aufgabe ist die sprachliche und pädagogische Anpassung.

Der Tutor arbeitet sokratisch und zielorientiert: Er gibt nicht automatisch die Lösung aus, sondern hilft dem Nutzer, den nächsten gedanklichen Schritt selbst zu machen. Jede Interaktion hat einen definierten Zweck — einen Irrtum klären, einen Begriff abgrenzen, eine Begründung vervollständigen, ein Beispiel entwickeln, einen Transfer prüfen. Die Anwendung bestimmt Thema, Lernziel und zulässige nächste Aktionen; das Modell gestaltet Erklärung und Gespräch. Endlose, unstrukturierte Chats gibt es nicht.

Unter Kontrolle der Anwendung bleiben: Inhalte, Definitionen und freigegebene Lösungen, Punkteberechnung und Prüfungsregeln, Lernfortschritt, Wiederholungsplanung und Kursversionen, Gewichtung der Lernziele, Zuordnung von Distraktoren zu Fehlvorstellungen sowie Quellenangaben.

Jede inhaltliche KI-Antwort lässt erkennen, auf welchen Syllabus-Abschnitten und Glossarbegriffen sie beruht. Tragen die Quellen eine Aussage nicht, wird das offen kommuniziert.

## Rolle des ISTQB-Glossars

Das offizielle Glossar ist ein eigenständiger Bestandteil des Produkts, nicht bloß Textmaterial für die semantische Suche. Je Begriff werden geführt: offizielle Bezeichnung und Definition, Sprache und Glossarversion, Synonyme und Abkürzungen, verwandte und abzugrenzende Begriffe, Verbindungen zu Kursen, Kapiteln und Learning Objectives sowie die Kennzeichnung offizieller gegenüber maschineller Übersetzung.

Zusätzlich werden Begriffe zu **Verwechslungsclustern** gruppiert. Diese Cluster sind redaktionell gepflegt und dienen doppelt: als Material für das Begriffstraining und als Grundlage für die Distraktoren neuer Fragen.

Betrifft eine Frage einen ISTQB-Begriff, hat die offizielle Definition Vorrang vor einer allgemeinen Erklärung des Sprachmodells.

## Vertrauenswürdigkeit und Qualität

learnISTQB trennt zwischen offiziellen Quellen, redaktionell freigegebenen und ungeprüft generierten Inhalten. Die Herkunft bleibt sichtbar; ungeprüft generierte Inhalte gehen nicht in die Prüfungsreife ein.

Wiederholbare Qualitätsprüfungen decken ab: korrekte Zuordnung von Quellen zu Lernzielen, keine Vermischung von Kursversionen, Übereinstimmung mit offiziellen Glossardefinitionen, belastbare Quellen für Erklärungen, konsistente Bewertung offener Antworten, Erkennung nicht belegter Aussagen, Einhaltung der Fragen-Stilrichtlinie und Vergleich verschiedener Sprachmodelle anhand derselben Testfälle.

Ein Modellwechsel darf die fachliche Qualität nicht ungeprüft verändern.

## Technisches Zielbild

learnISTQB ist eine Client-Server-Webanwendung: Browser als Oberfläche, Rust-Server, lokale Datenbank für Inhalte und Lernfortschritt. Im Produktivbetrieb liefert der Rust-Server das Frontend aus. Für den ersten Betrieb dient ein lokales Sprachmodell über Ollama; Chat- und Embeddingmodelle sind über definierte Provider-Schnittstellen austauschbar, sodass später ein gehostetes Modell ohne Umbau der Kurs-, Retrieval- oder Lernlogik genutzt werden kann.

Drei Betriebsformen sind vorgesehen: lokale Entwicklungsumgebung, lokal installierte Einzelplatzanwendung mit Browserzugriff, zentral gehostete Website für mehrere Nutzer. Konten, Synchronisation und Bezahlung können später ergänzt werden; der fachliche Kern setzt keine bestimmte Authentifizierungs- oder Hostinglösung voraus.

Deterministische Aktionen warten nie auf eine Modellantwort.

## Datenschutz

Der lokale Betrieb ist ein eigenständiges Produktmerkmal: Antworten, Lernhistorie und Modellinteraktionen können vollständig auf dem eigenen Gerät bleiben. Im gehosteten Betrieb gelten minimale Speicherung personenbezogener Daten, transparente Angabe des verwendeten Modells, keine versteckte Nutzung von Lernantworten zum Training externer Modelle, löschbare Lernhistorie und klare Trennung zwischen fachlichen Inhalten und Nutzerdaten.

## Abgrenzung und Rechte

learnISTQB ist kein Prüfungsanbieter, garantiert kein Bestehen und gibt keine ISTQB-Akkreditierung vor, sofern eine solche nicht vorliegt. Es ist ausdrücklich keine Braindump-Plattform: Geleakte Prüfungsfragen werden nicht verwendet, angeboten oder verlinkt.

Die Nutzungsrechte an Syllabi und Glossar werden je Quelle dokumentiert. Die Copyright-Hinweise der Syllabi erlauben Vervielfältigung und Auszüge bei Nennung der Quelle und unterscheiden zwischen akkreditierter und nicht akkreditierter Verwendung; das Glossar steht unter einer Creative-Commons-Lizenz mit Namensnennung. Vor öffentlicher oder kommerzieller Bereitstellung wird je Dokument geprüft, was die konkrete Verwendung abdeckt — insbesondere der Unterschied zwischen Nutzung als Quelle und Mitauslieferung im Produkt. Herkunft, Marken und Rechteinhaber werden transparent ausgewiesen.

## Erfolgskriterien

Fachlich erfolgreich, wenn Nutzer ihre Prüfung ohne Kurs bestehen, jederzeit wissen, was ihnen noch fehlt, falsche Antworten als Lerngelegenheit erleben, offizielle Begriffe sicher verwenden, Wissen auf Prüfungsszenarien anwenden können und der ausgewiesenen Prüfungsreife vertrauen können, weil sie sich als zutreffend erwiesen hat.

Technisch erfolgreich, wenn die Anwendung lokal mit Ollama zuverlässig läuft, dieselbe Codebasis als Website betrieben werden kann, ein Modellwechsel die fachliche Logik unberührt lässt, neue Kurse als versionierte Inhalte ergänzbar sind, die Modellkosten pro Nutzer zur Preispositionierung passen und Modellantworten reproduzierbar geprüft werden können.

## Langfristiges Ziel

learnISTQB soll kein weiterer Fragenkatalog und kein Chatbot mit hochgeladenen PDFs sein, sondern ein Lernsystem, das Prüfungsanforderungen, aktive Lernmethoden, nachvollziehbare Diagnose und individuelle Betreuung verbindet — zu einem Preis, der die Zertifizierung für alle erreichbar macht, die sie sich sonst nicht leisten oder zeitlich nicht einrichten können.

Der Nutzer soll am Ende nicht möglichst viele Fragen gesehen haben. Er soll wissen, warum eine Antwort richtig ist, verwandte Konzepte unterscheiden können, sein Wissen unter Prüfungsbedingungen anwenden — und bestehen.
