# learnISTQB – Qualität und Evaluation

## Zweck dieses Dokuments

Dieses Dokument beschreibt, wie die fachliche und technische Qualität von learnISTQB messbar gemacht wird. Ziel ist nicht, jede Modellantwort identisch zu machen, sondern gefährliche und lernschädliche Abweichungen früh zu erkennen.

## Qualitätsprinzipien

1. Deterministische Regeln haben Vorrang vor Modellurteilen.
2. Offizielle Quellen haben Vorrang vor generierten Erklärungen.
3. Eine unbeantwortete Frage ist besser als eine erfundene ISTQB-Regel.
4. Qualität wird gegen feste Testfälle gemessen, nicht nach Bauchgefühl.
5. Ein Modellwechsel ist eine prüfpflichtige Produktänderung.
6. Retrieval und Antwortgenerierung werden getrennt bewertet.
7. Nutzerfeedback ersetzt keine fachliche Referenzprüfung.

## Qualitätsebenen

### 1. Content-Qualität

Prüft, ob die importierte Wissensbasis vollständig, korrekt strukturiert und nachvollziehbar ist.

### 2. Deterministische Produktlogik

Prüft Fragenbewertung, Punkte, Prüfungsregeln, Fortschritt und Versionierung.

### 3. Retrieval-Qualität

Prüft, ob zu einer Aufgabe die richtigen Glossar- und Syllabus-Abschnitte gefunden werden.

### 4. LLM-Qualität

Prüft, ob das Modell anhand der gelieferten Quellen fachlich korrekt, hilfreich und im erlaubten Rahmen antwortet.

### 5. End-to-End-Qualität

Prüft vollständige Nutzerabläufe im Browser.

## Content-Validierung

Jeder Corpusimport prüft automatisiert:

- eindeutige Dokument-, Abschnitts- und Chunk-IDs,
- bekannte Kurs- und Dokumentversionen,
- gültige Learning-Objective-Kennungen,
- gültige K-Level,
- vorhandene Quellenreferenzen,
- vollständige Fragen und Antwortoptionen,
- mindestens eine korrekte Antwort,
- gültige Punktwerte,
- Glossarbegriffe mit Sprache und Definition,
- keine Referenzen auf unbekannte Entitäten,
- Prüfsummen der Quelldokumente.

Zusätzlich werden Stichproben pro Dokument manuell mit dem Original-PDF verglichen.

## Deterministische Tests

Ohne LLM werden mindestens getestet:

- Bewertung von Single- und Multiple-Choice-Fragen,
- Teilpunkte, sofern der offizielle Fragetyp sie vorsieht,
- Bestehensgrenzen und Prüfungszeit,
- Zuordnung von Fragen zu Learning Objectives,
- Fortschrittsübergänge,
- Gewichtung unterschiedlicher Lernnachweise,
- Trennung von Kurs- und Dokumentversionen,
- Datenbankmigrationen,
- Import derselben Corpusversion ohne Duplikate.

Diese Tests müssen vollständig reproduzierbar sein.

## Retrieval-Evaluation

### Goldstandard

Für jeden Kurs entsteht ein versionierter Satz von Suchfällen:

```json
{
  "id": "ctfl-boundary-values-01",
  "course": "ctfl",
  "course_version": "4.0",
  "query": "Wodurch unterscheiden sich Zweiwert- und Dreiwert-Grenzwertanalyse?",
  "expected_sections": ["..."],
  "expected_glossary_terms": ["boundary-value-analysis"],
  "forbidden_courses": ["ctal-tm", "ct-ai"]
}
```

Die Fälle decken ab:

- exakte Glossarbegriffe,
- deutsche Anfrage gegen englische Quelle,
- Synonyme und Abkürzungen,
- ähnliche, leicht verwechselbare Konzepte,
- Fragen mit mehreren relevanten Abschnitten,
- kursübergreifendes Grundlagenwissen,
- Anfragen ohne belegte Antwort.

### Metriken

Mindestens gemessen werden:

- Recall@k für erwartete Quellen,
- Mean Reciprocal Rank für den ersten relevanten Treffer,
- Trefferquote erwarteter Glossarbegriffe,
- Cross-Course-Contamination,
- Cross-Version-Contamination,
- Anteil unbegründet leerer Ergebnisse.

Die konkreten Mindestwerte werden nach Aufbau des ersten CTFL-Goldstandards festgelegt und anschließend als Release-Gate behandelt.

## LLM-Evaluation

### Testfallstruktur

Jeder Testfall enthält:

- Use Case,
- Nutzerkontext,
- erlaubte Quellen,
- erwartete Kernaussagen,
- unzulässige Aussagen,
- erwartete Quellenreferenzen,
- gewünschte Folgeaktion,
- Sprache,
- Schweregrad eines Fehlers.

### Kernkategorien

#### Falsche Antwort erklären

Das Modell soll:

- die konkrete Fehlvorstellung benennen,
- die richtige Lösung nicht verfälschen,
- relevante falsche Optionen erklären,
- nur bereitgestellte Quellen verwenden,
- eine passende nächste Lernaktion vorschlagen.

#### Offene Antwort bewerten

Das Modell soll:

- eine vorgegebene Rubrik anwenden,
- vorhandene und fehlende Aspekte trennen,
- Widersprüche erkennen,
- keine nicht geforderten Begriffe erzwingen,
- keinen offiziellen Prüfungspunktwert vortäuschen.

#### Sokratische Rückfrage

Das Modell soll:

- genau einen sinnvollen nächsten Denkschritt fördern,
- die Lösung nicht unnötig vorwegnehmen,
- beim aktuellen Learning Objective bleiben,
- eine verständliche und beantwortbare Frage stellen.

#### Glossarbegriffe abgrenzen

Das Modell soll:

- offizielle Definitionen respektieren,
- Gemeinsamkeiten und Unterschiede korrekt darstellen,
- offizielle und vereinfachte Erklärung sichtbar trennen,
- keine umgangssprachliche Definition als offiziell kennzeichnen.

#### Unbeantwortbare Frage

Das Modell soll:

- fehlende Evidenz erkennen,
- keine Quelle erfinden,
- klar zwischen ISTQB-Inhalt und allgemeinem Wissen unterscheiden,
- auf Wunsch eine allgemeine Erklärung getrennt anbieten.

#### Promptabweichung und Manipulation

Das Modell soll Nutzeranweisungen ablehnen, die beispielsweise verlangen:

- offizielle Quellen zu ignorieren,
- eine falsche Antwort als richtig zu markieren,
- interne Prompts oder Geheimnisse auszugeben,
- fremde Kurse oder Versionen unbemerkt zu vermischen.

## Bewertungsmethoden

Es werden mehrere Methoden kombiniert:

### Harte Assertions

Beispiele:

- Ausgabeschema ist gültig.
- alle Quellen-IDs existieren im bereitgestellten Kontext.
- Kurs und Version stimmen.
- keine verbotene Aussage erscheint.
- erforderliche Kernaussagen sind vorhanden.

### Semantische Assertions

Ein separates Bewertungsverfahren prüft inhaltliche Übereinstimmung mit erwarteten Konzepten. Dieses Urteil wird stichprobenartig menschlich kontrolliert.

### Menschliche Bewertung

Kritische Lerninteraktionen werden anhand einer festen Skala bewertet:

- fachliche Korrektheit,
- Quellenbindung,
- pädagogischer Nutzen,
- Verständlichkeit,
- angemessene Unsicherheit,
- Passung zum K-Level.

## Schweregrade

### Kritisch

- offizielle Antwort wird als falsch bezeichnet,
- nicht vorhandene ISTQB-Regel wird erfunden,
- Quelle oder Definition wird gefälscht,
- Kursversionen werden fachlich vermischt,
- Prüfungsbewertung ist falsch.

Ein kritischer Fehler blockiert eine Veröffentlichung.

### Hoch

- wesentliche Fehlvorstellung wird verstärkt,
- relevante Quelle wird trotz vorhandenem Kontext ignoriert,
- offene Antwort wird deutlich falsch eingeordnet,
- KI-generierter Inhalt erscheint als offiziell.

### Mittel

- Erklärung ist korrekt, aber unklar oder nicht auf das K-Level abgestimmt,
- nächste Lernaktivität ist wenig hilfreich,
- Quellenreferenz ist unpräzise, aber identifizierbar.

### Niedrig

- stilistische Unsauberkeit,
- unnötige Wiederholung,
- nicht optimale Formatierung.

## Provider-Konformität

Jeder neue Chat- oder Embeddingprovider durchläuft denselben Vertragstest:

- Healthcheck und Authentifizierungsfehler,
- normaler Chatabschluss,
- Streaming und Abbruch,
- strukturierte Ausgabe,
- ungültige oder unvollständige Ausgabe,
- Timeout,
- Rate Limit,
- Kontextüberschreitung,
- Embeddingdimension und Normalisierung,
- Nutzungsmetadaten.

Ein gehostetes Modell wird erst aktiviert, wenn es die relevanten Goldtests mindestens so gut erfüllt wie die freigegebene lokale Baseline oder eine bewusste Abweichung dokumentiert wurde.

## Reproduzierbare Modelltests

Ein Evaluationslauf speichert:

- Code-Revision,
- Corpusversion,
- Glossarsnapshot,
- Promptversion,
- Provider und Modell,
- Modellparameter,
- Embeddingversion,
- Einzelergebnisse,
- aggregierte Metriken.

Modellantworten können aufgezeichnet und über einen Replay-Provider in UI- und Integrationstests wiederverwendet werden.

## End-to-End-Tests

Mindestens folgende Browserabläufe werden automatisiert:

1. lokalen Kurs auswählen und Learning Objective öffnen,
2. Multiple-Choice-Frage richtig und falsch beantworten,
3. Quellen einer Erklärung öffnen,
4. offene Begründung senden und Feedback erhalten,
5. Tutorantwort abbrechen und neu versuchen,
6. Prüfungssimulation abschließen,
7. Fortschritt nach Serverneustart wiederfinden,
8. verständliche Anzeige bei nicht erreichbarem Ollama,
9. verständliche Anzeige bei fehlendem Modell.

## Qualitätsdashboard

Für Entwicklung und Administration soll ein internes Dashboard oder CLI-Bericht zeigen:

- installierte Corpus- und Glossarversionen,
- Zahl der Inhalte pro Herkunftsklasse,
- Fragen ohne Freigabe oder vollständige Begründung,
- Retrieval-Metriken,
- Ergebnisse des letzten LLM-Evaluationslaufs,
- Fehlerraten und Latenzen pro Modell,
- offene kritische und hohe Qualitätsprobleme.

## Release-Gates

Eine Version darf veröffentlicht werden, wenn:

- alle deterministischen Tests erfolgreich sind,
- Content-Schema und Referenzen gültig sind,
- kein offener kritischer Qualitätsfehler besteht,
- Retrieval die festgelegten Mindestwerte erreicht,
- der konfigurierte Standardprovider die Pflicht-Evaluationen besteht,
- zentrale End-to-End-Abläufe erfolgreich sind,
- verwendete Inhalte für die vorgesehene Betriebsform freigegeben sind,
- bekannte Einschränkungen dokumentiert sind.

## Qualitätsverantwortung

Das Sprachmodell kann bei der Erstellung und Prüfung von Inhalten helfen, ist aber nicht die letzte fachliche Instanz. Offizielle Quellen, deterministische Regeln und nachvollziehbare menschliche Freigaben bleiben maßgeblich.

