# learnISTQB – Architektur

## Zweck dieses Dokuments

Dieses Dokument beschreibt die technische Zielarchitektur und die verbindlichen Systemgrenzen. Konkrete Implementierungsdetails können sich weiterentwickeln, solange diese Grenzen erhalten bleiben.

## Architekturziele

learnISTQB soll:

- lokal auf einem einzelnen Rechner funktionieren,
- über einen Browser bedient werden,
- später mit derselben Codebasis als Website betrieben werden,
- zunächst Ollama und später gehostete Sprachmodelle verwenden können,
- fachliche Regeln unabhängig vom Sprachmodell ausführen,
- Kurs- und Glossarversionen reproduzierbar verwalten,
- Modellantworten testen und nachvollziehen können.

## Systemübersicht

```text
┌──────────────────────────────────────────────┐
│ Browser                                      │
│ React + TypeScript                           │
└─────────────────────┬────────────────────────┘
                      │ HTTP / SSE
┌─────────────────────▼────────────────────────┐
│ Rust-Server mit Axum                         │
│                                              │
│  API ─ Application ─ Domain                  │
│          │          │                        │
│          ├─ Learning Engine                  │
│          ├─ Question Engine                  │
│          ├─ Retrieval                        │
│          └─ LLM Harness                      │
└───────────┬───────────────┬──────────────────┘
            │               │
┌───────────▼──────┐  ┌─────▼─────────────────┐
│ SQLite           │  │ Modellprovider        │
│ Inhalte          │  │ zuerst: Ollama        │
│ Glossar          │  │ später: gehostet      │
│ Fortschritt      │  └───────────────────────┘
│ Traces           │
└──────────────────┘
```

## Technologieentscheidungen

### Backend

- Rust Stable
- Tokio als asynchrone Laufzeit
- Axum für HTTP-Routing und Server-Sent Events
- Tower und Tower HTTP für Middleware, Tracing, Timeouts und statische Dateien
- Serde für Transport- und Persistenzformate
- Reqwest für LLM- und optionale Downloadzugriffe
- SQLx für SQLite und spätere PostgreSQL-Kompatibilität
- Tracing für strukturierte Logs

### Frontend

- React
- TypeScript
- Vite
- eine kleine, explizit gewählte Bibliothek für Server-State und Requests
- semantisches HTML und zugängliche Komponenten

Das Frontend enthält keine fachliche Bewertungslogik. Es stellt Serverzustand dar und sammelt Nutzereingaben.

### Datenhaltung

SQLite ist der Standard für die lokale Einzelplatzversion. Die Datenbank enthält:

- Kurs- und Dokumentmetadaten,
- normalisierte Syllabus-Abschnitte,
- Learning Objectives und K-Level,
- Glossarbegriffe und Beziehungen,
- Fragen, Rubriken und Freigabestatus,
- Lernversuche und Fortschritt,
- Prompt- und Modelltraces,
- Corpus- und Embeddingversionen.

Migrationen werden versioniert im Repository abgelegt.

Für einen späteren Mehrbenutzerbetrieb wird die Persistenz hinter Repository-Schnittstellen gekapselt. PostgreSQL ist eine mögliche spätere Implementierung, aber kein Bestandteil des MVP.

## Rust-Workspace

```text
crates/
├── domain/          Fachliche Entitäten und Regeln
├── application/     Use Cases und Orchestrierung
├── persistence/     SQLx-Repositories und Migrationen
├── retrieval/       Volltext- und Vektorsuche
├── llm/             Provider, Prompts, Validierung und Traces
├── content-ingest/  Import und Normalisierung
├── evaluation/      Retrieval- und Modelltests
└── server/          Axum-Routen, Konfiguration und Auslieferung
```

Abhängigkeitsregel:

```text
server/infrastructure → application → domain
```

Das Domain-Crate kennt weder Axum noch SQLx, React, Ollama oder einen externen LLM-Anbieter.

## Fachliche Module

### Course Catalog

Verwaltet Kurse, Kursversionen, Kapitel, Abschnitte, Learning Objectives, Business Outcomes und Prüfungsregeln.

### Glossary

Verwaltet offizielle Begriffe, Definitionen, Übersetzungen, Synonyme, Beziehungen und die Zuordnung zu Kursen und Syllabus-Abschnitten.

### Question Engine

Verwaltet Fragen, Antwortoptionen, Lösungen, Begründungen, Rubriken, Punkte und Herkunft. Multiple-Choice-Fragen werden hier deterministisch bewertet.

### Learning Engine

Wählt die nächste Lernaktivität aus und aktualisiert den Lernstatus anhand von nachvollziehbaren Lernnachweisen.

### Retrieval

Liefert zu einem klar begrenzten Kurs- und Lernkontext relevante Glossar- und Syllabus-Quellen.

### Tutor

Verwendet Retrieval, Promptvorlagen und den LLM-Harness für Erklärungen, Rückfragen und Analysen. Der Tutor besitzt keine eigene Wahrheit und keine direkte Schreibberechtigung auf den Lernstatus.

### Assessment

Führt Einstufungen und Prüfungssimulationen durch. Die Punkteberechnung ist deterministisch und unabhängig vom LLM.

## LLM-Architektur

Chat und Embeddings sind getrennte Dienste:

```text
ChatProvider
├── OllamaChatProvider
├── OpenAiCompatibleChatProvider
├── FakeChatProvider
└── ReplayChatProvider

EmbeddingProvider
├── OllamaEmbeddingProvider
├── OpenAiCompatibleEmbeddingProvider
└── FakeEmbeddingProvider
```

Ein Wechsel des Chatmodells erfordert keine Neuindexierung. Ein Wechsel des Embeddingmodells erzeugt eine neue Embeddingversion und erfordert eine Neuindexierung.

### Interne Providerverträge

Providerimplementierungen liefern ausschließlich interne Typen zurück. Anbieterabhängige Antwortobjekte dürfen die Infrastrukturgrenze nicht überschreiten.

Der Vertrag unterstützt mindestens:

- einfachen Chatabschluss,
- Streaming,
- strukturierte Ausgabe anhand eines JSON-Schemas,
- Embedding mehrerer Texte,
- Healthcheck,
- Modell- und Capability-Informationen,
- Nutzungs- und Laufzeitmetadaten, soweit vorhanden.

### Capabilities

Nicht jeder Provider unterstützt dieselben Funktionen. Fähigkeiten werden explizit beschrieben, beispielsweise:

- Streaming,
- JSON-Schema-Ausgabe,
- Tool Calling,
- Seed oder reproduzierbare Ausgabe,
- maximale Kontextgröße,
- Nutzungsmetriken.

Fehlende Fähigkeiten führen zu einem kontrollierten Fehler oder einem ausdrücklich konfigurierten Alternativpfad. Es gibt keine stillen Modellwechsel.

### Konfiguration

Die Konfiguration unterscheidet Chat und Embeddings:

```env
CHAT_PROVIDER=ollama
CHAT_BASE_URL=http://localhost:11434
CHAT_MODEL=
CHAT_API_KEY=

EMBEDDING_PROVIDER=ollama
EMBEDDING_BASE_URL=http://localhost:11434
EMBEDDING_MODEL=embeddinggemma
```

Geheimnisse werden ausschließlich serverseitig geladen und niemals an das Frontend gesendet.

## Retrieval-Architektur

Der Corpus ist klein genug, um im MVP ohne separaten Vector-Database-Server auszukommen.

### Indexe

- SQLite FTS5 für Begriffe und exakte Formulierungen
- Embeddings für semantische Ähnlichkeit
- strukturierte Filter für Kurs, Version, Sprache, Kapitel und Learning Objective

Embeddings werden normalisiert gespeichert. Zusätzlich werden Modellname, Dimension, Erzeugungszeitpunkt und Corpusversion gespeichert.

### Retrieval-Reihenfolge

1. Kurs und Kursversion zwingend eingrenzen.
2. Aktuelles Learning Objective und Kapitel berücksichtigen.
3. Exakte Glossarbegriffe und Aliase ermitteln.
4. Volltexttreffer bestimmen.
5. semantische Treffer bestimmen.
6. Ergebnisse zusammenführen und deduplizieren.
7. Kontextbudget anwenden.
8. Quellen mit stabilen IDs an den Tutor übergeben.

Glossardefinitionen und offizielle Antwortbegründungen erhalten gegenüber semantisch ähnlichen Textstellen eine höhere Priorität.

## Promptarchitektur

Prompts liegen als versionierte Dateien außerhalb der HTTP-Handler. Jeder Prompt definiert:

- Zweck und erlaubte Aktion,
- Eingabevariablen,
- erwartetes Ausgabeschema,
- Regeln für Quellen und Unsicherheit,
- Promptversion,
- zugehörige Evaluationen.

Beispielhafte Use Cases:

```text
ExplainWrongAnswer
EvaluateOpenAnswer
AskSocraticQuestion
ExplainGlossaryDifference
GenerateTransferExercise
SummarizeLearningProgress
```

## API-Oberfläche

Die genaue URL-Struktur darf sich während der Implementierung ändern. Fachlich werden mindestens folgende Ressourcen benötigt:

```text
GET  /api/system/status
GET  /api/courses
GET  /api/courses/{course}/objectives
GET  /api/glossary/terms
GET  /api/learning/next
POST /api/attempts
POST /api/open-answers/evaluate
POST /api/tutor/messages
POST /api/exams
POST /api/exams/{exam}/answers
GET  /api/progress
```

Lang laufende Tutorantworten können über Server-Sent Events übertragen und vom Nutzer abgebrochen werden.

## Content-Import

Der Content-Importer ist ein separates Rust-Binary. Er:

1. liest ein versioniertes Quellenmanifest,
2. prüft Dokumenthashes,
3. extrahiert eingebetteten PDF-Text seitenweise,
4. erkennt und normalisiert die Dokumentstruktur,
5. wendet manuell geprüfte Overrides an,
6. validiert das Ergebnis gegen Schemas,
7. erzeugt Embeddings,
8. importiert ein atomar versioniertes Content-Bundle.

Der Produktionsserver parst keine PDFs während einer Lernsession.

## Auslieferung

### Entwicklung

- Vite-Entwicklungsserver für das Frontend
- Axum-Server für die API
- Proxy von Vite auf `/api`

### Lokales Release

- gebautes Frontend wird vom Rust-Server ausgeliefert
- SQLite und Inhalte liegen in einem konfigurierbaren Datenverzeichnis
- Ollama läuft lokal als separater Dienst

### Gehostetes Release

- dasselbe Server-Image
- statisches Frontend im selben Image
- konfigurierbare persistente Datenbank
- lokaler oder externer Modellprovider
- Reverse Proxy und TLS außerhalb der Anwendung

## Beobachtbarkeit

Strukturierte Logs verwenden eine Trace-ID pro Request und eine eigene Invocation-ID pro Modellaufruf.

Ein LLM-Trace kann enthalten:

- Use Case,
- Modell und Provider,
- Promptversion,
- Quell-IDs,
- Eingabehash oder Eingabe gemäß Datenschutzkonfiguration,
- validierte Ausgabe,
- Laufzeit und Tokenzahlen,
- Fehler und Wiederholungen.

Produktionslogs dürfen keine API-Schlüssel enthalten.

## Sicherheits- und Datenschutzgrenzen

- Der Browser erhält keine Modell- oder Datenbankschlüssel.
- Externe URLs werden nicht aufgrund freier LLM-Ausgaben aufgerufen.
- Importierte Dokumente werden als Daten, nicht als Anweisungen behandelt.
- Ein gehosteter Modellprovider erhält nur den für den Use Case notwendigen Kontext.
- Die lokale Betriebsart benötigt nach Einrichtung keine Übertragung von Lernantworten an Dritte.

## Bewusst aufgeschobene Entscheidungen

- PostgreSQL für Mehrbenutzerbetrieb
- Authentifizierung und Autorisierung
- Mandantentrennung
- Abrechnung
- Hostinganbieter
- konkretes gehostetes LLM
- native Desktopverpackung
- Tool Calling oder agentische Abläufe

Diese Entscheidungen werden erst getroffen, wenn der lokale MVP ihre Anforderungen tatsächlich sichtbar macht.

