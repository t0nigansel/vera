# learnISTQB – Minimum Viable Product

## Zweck dieses Dokuments

Dieses Dokument grenzt die erste nutzbare Produktversion von der langfristigen Produktvision ab. Der MVP soll beweisen, dass eine quellengebundene, adaptive Lernbegleitung einen erkennbaren Mehrwert gegenüber einem statischen Syllabus und einem gewöhnlichen Fragenkatalog bietet.

Die langfristige Ausrichtung ist in [Vision.md](Vision.md) beschrieben.

## MVP-Ziel

Ein Nutzer kann learnISTQB lokal starten, im Browser eine ISTQB-Zertifizierung auswählen, entlang der offiziellen Learning Objectives lernen, Fragen beantworten, individuelles Feedback erhalten und seinen Lernfortschritt nachvollziehen.

Der MVP gilt als erfolgreich, wenn ein vollständiger Lernzyklus funktioniert:

```text
Wissensstand feststellen
→ Thema auswählen
→ Quelle und Erklärung bearbeiten
→ Frage beantworten und begründen
→ Fehlvorstellung erkennen
→ gezielt weiterüben
→ Fortschritt aktualisieren
```

## Unterstützte Kurse

Der MVP unterstützt diese drei Zertifizierungen:

1. Certified Tester Foundation Level (CTFL) v4.0
2. Certified Tester Advanced Level Test Management (CTAL-TM) v3.0
3. Certified Tester AI Testing (CT-AI) v2.0

Die Implementierung erfolgt als vertikaler Durchstich zunächst vollständig für CTFL. Danach werden CTAL-TM und CT-AI über dasselbe Inhalts- und Lernmodell ergänzt. Eine MVP-Veröffentlichung umfasst alle drei Kurse.

## Primäre Nutzerabläufe

### 1. Anwendung lokal starten

Der Nutzer startet den Rust-Server und Ollama. Anschließend öffnet er learnISTQB in einem normalen Browser.

Die Anwendung prüft:

- ob die lokale Datenbank verfügbar ist,
- ob ein Chatmodell konfiguriert und erreichbar ist,
- ob ein Embeddingmodell verfügbar ist,
- welche Kurs- und Glossarversionen installiert sind.

Fehlende Voraussetzungen werden verständlich erklärt.

### 2. Kurs auswählen

Die Kursübersicht zeigt:

- Zertifizierung und Version,
- Kapitel und Learning Objectives,
- Prüfungsstruktur, soweit sie aus den offiziellen Unterlagen hervorgeht,
- vorhandene Lern- und Frageninhalte,
- persönlichen Fortschritt.

### 3. Einstufung durchführen

Der Nutzer kann mit einer kurzen Einstufung beginnen oder den Kurs ohne Einstufung starten.

Die Einstufung verwendet ausschließlich offizielle oder redaktionell geprüfte Fragen. Sie liefert keine endgültige Bestehensprognose, sondern einen ersten Lernstand pro Themenbereich.

### 4. Geführte Lerneinheit bearbeiten

Eine Lerneinheit ist einem Kurs, einem Kapitel und mindestens einem Learning Objective zugeordnet. Sie enthält:

- Lernziel und gefordertes K-Level,
- relevante Syllabus-Abschnitte,
- relevante offizielle Glossarbegriffe,
- eine verständliche Erklärung,
- mindestens eine aktive Lernaufgabe,
- mindestens eine überprüfbare Abschlussfrage.

### 5. Multiple-Choice-Fragen beantworten

Der Nutzer kann eine oder mehrere Antworten auswählen und optional seine Entscheidung begründen.

Die Plattform zeigt anschließend:

- ob die Auswahl richtig war,
- die hinterlegte Begründung,
- eine Erklärung zu jeder relevanten Antwortoption,
- die zugehörigen Quellen,
- eine mögliche Fehlvorstellung,
- eine passende nächste Aktivität.

Die Richtigkeit der Antwort wird nicht vom Sprachmodell bestimmt.

### 6. Offene Antwort geben

Der Nutzer kann Begriffe erklären, Konzepte vergleichen oder ein Szenario analysieren. Die Antwort wird gegen eine hinterlegte Rubrik geprüft.

Das Feedback unterscheidet:

- korrekt genannte Aspekte,
- fehlende Aspekte,
- sachlich falsche Aussagen,
- vermutete Fehlvorstellungen,
- eine empfohlene Rückfrage oder Übung.

Das Ergebnis dient als Lernfeedback und wird nicht wie eine offizielle Prüfungsfrage behandelt.

### 7. Tutor nutzen

Der Tutor beantwortet Fragen nur innerhalb des gewählten Kurses und des aktuellen Lernkontexts. Er verwendet die bereitgestellten Quellen und nennt sie sichtbar.

Der Tutor kann:

- einen Begriff erklären oder abgrenzen,
- mit einer sokratischen Rückfrage weiterführen,
- ein Beispiel oder Gegenbeispiel bilden,
- eine Antwort des Nutzers analysieren,
- eine zusätzliche Transferaufgabe erzeugen.

Der Tutor darf keine offizielle Lösung verändern und keine nicht belegte Aussage als ISTQB-Regel ausgeben.

### 8. Prüfungssimulation durchführen

Die Prüfungssimulation verwendet ausschließlich offizielle oder redaktionell freigegebene Fragen. Sie berücksichtigt die für den Kurs hinterlegte Prüfungsstruktur.

Die Auswertung zeigt:

- erreichte Punkte,
- bestanden oder nicht bestanden gemäß hinterlegter Regel,
- Ergebnis pro Kapitel und Learning Objective,
- Ergebnis pro K-Level,
- Themen mit weiterem Lernbedarf.

KI-generierte Fragen werden nicht in einer als realistisch bezeichneten Prüfungssimulation verwendet.

### 9. Fortschritt ansehen

Der Nutzer sieht seinen Lernstand mindestens auf diesen Ebenen:

- Kurs,
- Kapitel,
- Learning Objective,
- Glossarbegriff.

Der Fortschritt basiert auf Lernnachweisen und nicht auf der reinen Nutzungsdauer. Lesen allein führt nicht zur Einstufung als „prüfungssicher“.

## Lernstatus im MVP

Für jedes Learning Objective wird ein verständlicher Status verwendet:

```text
Nicht begonnen
→ Eingeführt
→ Geübt
→ Verstanden
→ Prüfungssicher
```

Als Lernnachweise zählen beispielsweise:

- offizielle Frage richtig beantwortet,
- Antwort korrekt begründet,
- offene Rubrik weitgehend erfüllt,
- Transferfrage erfolgreich gelöst,
- Wissen nach einer zeitlichen Pause erneut abgerufen.

Offizielle, redaktionelle und KI-generierte Aktivitäten erhalten unterschiedliche Gewichte.

## Muss-Funktionen

- Browserbasierte Oberfläche
- Rust-Server
- lokale SQLite-Datenbank
- lokaler Betrieb mit Ollama
- austauschbarer Chat- und Embeddingprovider
- versionierte Kursinhalte
- versionierter Glossarbestand
- Kapitel und Learning Objectives
- exakte Quellenanzeige
- Multiple-Choice-Fragen mit deterministischer Bewertung
- offene Antworten mit rubric-basiertem LLM-Feedback
- geführter Tutor innerhalb eines Lernkontexts
- lokaler Lernfortschritt ohne Login
- Prüfungssimulation
- Systemseite für Modell- und Corpusstatus
- automatisierte Tests und LLM-Evaluationen

## Kann-Funktionen

Diese Funktionen sind willkommen, sofern sie den MVP nicht verzögern:

- Streaming von Tutorantworten
- Wiederholungsplanung nach Spaced-Repetition-Prinzipien
- Export des persönlichen Lernstands
- dunkles Farbschema
- Tastaturbedienung für Fragen
- anpassbare tägliche Lernziele
- Umschaltung zwischen deutscher und englischer Darstellung

## Nicht Bestandteil des MVP

- Benutzerkonten und Anmeldung
- Synchronisation zwischen mehreren Geräten
- Bezahlsystem
- öffentliche Kursverwaltung
- Autorentool für externe Trainer
- Community und Ranglisten
- Audiolektionen und Text-to-Speech
- automatisch erzeugte Lernvideos
- Bildgenerierung
- kollaborative Mindmaps
- native Desktop- oder Mobile-App
- Websuche durch den Tutor
- autonome Agenten mit freien Werkzeugaufrufen
- offizielle Akkreditierung als Trainingsanbieter

## Nichtfunktionale Anforderungen

### Lokalität

Nach Installation der Inhalte und Modelle muss die Anwendung vollständig lokal funktionieren. Ein Internetzugang darf für den normalen Lernbetrieb nicht erforderlich sein.

### Austauschbarkeit

Ein Wechsel von Ollama zu einem gehosteten Chatmodell darf keine Änderung an Kurslogik, Fortschrittsberechnung oder Inhaltsmodell erfordern.

### Nachvollziehbarkeit

Jede fachliche Tutorantwort muss auf gespeicherte Quellen zurückführbar sein. Modell, Promptversion, Retrievaltreffer und verwendete Dokumentversionen müssen für Diagnose und Evaluation protokollierbar sein.

### Reaktionsfähigkeit

Deterministische Aktionen wie Navigation, Fragenbewertung und Fortschrittsanzeige dürfen nicht auf eine LLM-Antwort warten. Lang laufende Modellaufrufe werden in der Oberfläche sichtbar gemacht und können abgebrochen werden.

### Barrierearme Nutzung

Die zentralen Lern- und Prüfungsabläufe müssen per Tastatur bedienbar sein und semantische HTML-Elemente verwenden.

## Abnahmekriterien

Der MVP ist fachlich abgenommen, wenn:

- alle drei Kurse mit den festgelegten Versionen auswählbar sind,
- Syllabus, Learning Objectives und Glossarbegriffe miteinander verknüpft sind,
- jede offizielle Frage eine bekannte Lösung, Herkunft und Begründung besitzt,
- eine vollständige Lernsession ohne manuelle Eingriffe durchgeführt werden kann,
- eine falsche Antwort zu belegtem und hilfreichem Feedback führt,
- eine Prüfungssimulation reproduzierbar bewertet wird,
- der Fortschritt nach einem Neustart erhalten bleibt.

Der MVP ist technisch abgenommen, wenn:

- ein dokumentierter lokaler Start auf einer frischen Entwicklungsumgebung funktioniert,
- Ollama durch einen Fake-Provider in Tests ersetzt werden kann,
- der Chatprovider allein über Konfiguration austauschbar ist,
- der Retrieval-Evaluationssatz die festgelegten Mindestwerte erreicht,
- kritische End-to-End-Abläufe automatisiert getestet werden,
- keine direkte LLM-Verbindung aus dem Browser besteht.

## Wichtigste offene Produktfragen

Diese Fragen werden während des CTFL-Durchstichs anhand eines funktionierenden Produkts entschieden:

- Wie lang soll eine typische Lerneinheit sein?
- Wie stark sollen offene Antworten den Lernstatus beeinflussen?
- Wie wird „geraten“ gegenüber „gewusst“ zuverlässig erkannt?
- Wann soll der Tutor erklären und wann nur eine Rückfrage stellen?
- Welche Form der Prüfungsreife ist verständlich, ohne falsche Sicherheit zu erzeugen?
- Welche offiziellen Inhalte dürfen in einer späteren öffentlichen oder kommerziellen Version direkt ausgeliefert werden?

