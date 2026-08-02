# learnISTQB

learnISTQB ist eine lokal und später als Website betreibbare, KI-gestützte Lernplattform für ISTQB-Zertifizierungen.

Die Anwendung verbindet versionierte offizielle Lernquellen, das ISTQB-Glossar, aktive Übungen und einen quellengebundenen persönlichen Tutor; realistische Prüfungssimulationen folgen mit dem vollständigen Fragenkorpus. Das Sprachmodell erklärt und analysiert, während Inhalte, Fragenbewertung und Lernfortschritt unter Kontrolle der Anwendung bleiben.

## Projektstatus

Ein ausführbarer MVP-Vertical-Slice ist implementiert. Er umfasst Dashboard, Kursansicht, versionierte Lernziele und Glossareinträge, redaktionell freigegebene und deterministisch bewertete Multiple-Choice-Fragen, Antwortsicherheit, Fehlvorstellungsdiagnose, verteilte Wiederholungen, eine bewusst konservative Prüfungsreife, einen datumsbezogenen Lernplan, FTS5-Retrieval sowie einen quellengebundenen Tutor mit austauschbarem LLM-Provider.

Der mitgelieferte Inhalt ist bewusst ein kleiner, redaktioneller Starterdatensatz für den technischen MVP. Er ersetzt keinen vollständigen, lizenzierten Import der offiziellen ISTQB-Unterlagen. Alle Demo-Lernziele sind als `*-DEMO-*` gekennzeichnet; Links führen zu den offiziellen Originalquellen.

## Erste unterstützte Kurse

- Certified Tester Foundation Level (CTFL) v4.0
- Certified Tester Advanced Level Test Management (CTAL-TM) v3.0
- Certified Tester AI Testing (CT-AI) v2.0

## Zielarchitektur

- Rust-Backend mit Axum
- React-Frontend mit TypeScript
- SQLite für lokale Inhalte und Lernfortschritt
- SQLite FTS5 und Embeddings für Retrieval
- Ollama als erster lokaler Modellprovider
- austauschbare Chat- und Embeddingprovider für spätere gehostete Modelle
- Browser als Benutzeroberfläche
- dieselbe Codebasis für lokalen und gehosteten Betrieb

## Dokumentation

- [Vision.md](Vision.md) – langfristige Produktvision
- [MVP.md](MVP.md) – Umfang und Abnahmekriterien der ersten Version
- [Roadmap.md](Roadmap.md) – Arbeitsstand, Entscheidungen und nächste Meilensteine
- [Architecture.md](Architecture.md) – technisches Zielbild und Systemgrenzen
- [Content.md](Content.md) – Quellen, Glossar, Versionierung und Inhaltsmodell
- [Quality.md](Quality.md) – Tests, LLM-Evaluationen und Release-Gates

## Lokal starten

Voraussetzungen: Rust 1.85 oder neuer, Node.js mit npm und für den Tutor optional [Ollama](https://ollama.com/).

```bash
cd web
npm install
npm run build
cd ..

cp .env.example .env
```

Trage in `.env` ein lokal installiertes Ollama-Modell ein, zum Beispiel:

```dotenv
CHAT_PROVIDER=ollama
CHAT_BASE_URL=http://127.0.0.1:11434
CHAT_MODEL=<dein-lokal-installiertes-modell>
```

Danach startet ein einziger Rust-Prozess API und Weboberfläche:

```bash
cargo run -p server
```

Die Anwendung ist unter [http://127.0.0.1:8080](http://127.0.0.1:8080) erreichbar. Beim ersten Start werden `data/learnistqb.db`, das Schema und der Starterdatensatz automatisch angelegt.

Für einen vollständig deterministischen Start ohne Ollama können in `.env` `CHAT_PROVIDER=fake` und `EMBEDDING_PROVIDER=fake` gesetzt werden. Die Fake-Provider sind ausschließlich für Entwicklung und Tests gedacht.

## Qualität prüfen

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd web && npm run build
```

Die Persistenztests verwenden isolierte temporäre SQLite-Datenbanken. Der Fake-LLM erlaubt reproduzierbare Tests ohne Netzwerk oder Modellinstallation.

## Leitprinzipien

- Das LLM ist nicht die fachliche Wissensquelle.
- Offizielle Definitionen und Lösungen werden nicht vom Modell verändert.
- Jede fachliche Tutorantwort ist auf konkrete Quellen zurückführbar.
- Offizielle, redaktionelle und KI-generierte Inhalte bleiben unterscheidbar.
- Multiple-Choice- und Prüfungsbewertung sind deterministisch.
- Richtig geratene Antworten gelten nicht als sichere Beherrschung.
- Sicher falsche Antworten werden als priorisierte Fehlvorstellungen behandelt.
- Die Prüfungsreife steigt nur durch aktive Lernnachweise und bleibt bis zur Kalibrierung konservativ.
- Chat- und Embeddingmodelle sind unabhängig austauschbar.
- Der lokale Betrieb schützt Lernhistorie und Antworten vor unnötiger Übertragung an Dritte.
- Modellqualität wird mit reproduzierbaren Evaluationen geprüft.

## Weiterer Entwicklungsablauf

1. Nutzungsrechte klären und vollständige, versionierte Kurs- und Glossarimporte bauen.
2. Embeddingprovider und Vektorsuche ergänzen; FTS5 bleibt als lokaler Fallback bestehen.
3. offene Antworten mit Rubrics und nachvollziehbarer KI-Unterstützung ergänzen.
4. vollständige Prüfungssimulation und Kalibrierung gegen freiwillig gemeldete Prüfungsergebnisse ergänzen.
5. Replay-Provider und fachliche LLM-Evaluationen erweitern.
6. lokale Distribution und späteren gehosteten Betrieb vorbereiten.

## Lizenz und Inhalte

Die MIT-Lizenz dieses Repositorys gilt für den eigenen Quellcode. Sie gilt **nicht** für ISTQB-Inhalte.

- Das ISTQB-Glossar steht unter Creative Commons mit Namensnennung. Übernommene Definitionen führen ihre Quelle in den Daten mit (`source_label`, `source_url`).
- Syllabus-Formulierungen wie Kapitelnamen und Learning Objectives sind Auszüge aus den offiziellen Unterlagen und bleiben Eigentum des ISTQB beziehungsweise der jeweiligen Rechteinhaber.
- Die offiziellen PDF-Quelldokumente werden bewusst **nicht** mit ausgeliefert. Der Import erwartet sie lokal in `content/`; siehe [Content.md](Content.md).

## Rechtlicher Hinweis

learnISTQB ist kein Prüfungsanbieter und derzeit kein akkreditierter ISTQB-Trainingsanbieter. Vor einer öffentlichen oder kommerziellen Bereitstellung müssen die Nutzungsrechte der verwendeten Syllabi, Glossarinhalte und Beispielprüfungen für die konkrete Verwendung geklärt sein.
