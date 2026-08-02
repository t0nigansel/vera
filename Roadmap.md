# learnISTQB – Roadmap

## Zweck dieses Dokuments

Dieses Dokument hält den Arbeitsstand und die nächsten Meilensteine fest. Es beschreibt keinen Zielzustand — das tun [Vision.md](Vision.md) und [MVP.md](MVP.md) — sondern den Weg dorthin, einschließlich der unterwegs getroffenen Entscheidungen und ihrer Begründung.

Stand: 2026-08-02

## Arbeitsstand

Ein ausführbarer vertikaler Durchstich steht: Dashboard, Kursansicht, Lernziele, Glossar, deterministisch bewertete Multiple-Choice-Fragen mit Sicherheitsangabe, Fehlvorstellungsdiagnose, verteilte Wiederholung, konservative Prüfungsreife, FTS5-Retrieval und ein quellengebundener Tutor mit austauschbarem Provider.

Der Inhaltsimport ist abgeschlossen und LLM-frei:

| Artefakt | Umfang |
| --- | --- |
| `content/generated/glossary.json` | 651 Begriffe mit Synonymen, Abkürzungen, See-also, Reference |
| `content/generated/ctfl.json` | 6 Kapitel, 64 Learning Objectives, Keyword-Listen |
| `content/seed.json` | 97 CTFL-relevante Begriffe, 8 Kapitel, 68 Learning Objectives |

Erzeugt von `tools/import_content.py` aus den PDF-Quellen in `content/`. Der Importer bleibt regelbasiert, wiederholbar und ohne Modellaufruf.

**Das Datenmodell nutzt die neuen Glossarfelder noch nicht.** `glossary_terms` kennt weder `term_version` noch Synonyme, Abkürzungen, See-also oder Reference; Verwechslungscluster und die Zuordnung Begriff → Learning Objective existieren nicht. Das schließen die Meilensteine (a) bis (f).

### Bekannte Stolperstelle beim Inhaltsupdate

`seed_if_empty` importiert nur in eine leere Datenbank ([crates/persistence/src/lib.rs:59-64](crates/persistence/src/lib.rs#L59-L64)). Nach einem Lauf von `import_content.py` liefert eine bestehende Datenbank deshalb weiter den alten Bestand aus, und ein noch laufender Serverprozess hält zusätzlich seine geöffnete Datei fest. Behelf bis Meilenstein (c):

```bash
pkill -f 'target/debug/server'
mv data/learnistqb.db data/learnistqb.db.alt
cargo run -p server
```

## Grundsatzentscheidungen

### 1. Verwechslungscluster sind redaktionell

Vision.md beschreibt die Cluster als redaktionell gepflegt. Sie liegen deshalb in einer handgepflegten Datei `content/clusters.json`, die kein Werkzeug überschreibt. `import_content.py` liest sie, prüft jedes Mitglied gegen das Glossar und übernimmt sie nach `seed.json`. Damit bleibt der Importer wiederholbar, ohne die redaktionelle Arbeit zu zerstören.

### 2. Startcluster

Aus Vision.md, auf die englischen Glossarbegriffe abgebildet:

- `error` / `defect` / `failure` / `root cause`
- `verification` / `validation`
- `test case` / `test condition` / `test procedure` / `test suite`
- `test monitoring` / `test control`
- `testing` / `debugging`

`test suite` ist kein Syllabus-Keyword. Cluster-Mitglieder werden deshalb zusätzlich zu den Keywords in den Seed aufgenommen — eine bewusste, hier dokumentierte Ausnahme, die derzeit ein bis zwei Begriffe betrifft.

### 3. Begriff → Learning Objective wird abgeleitet, nicht gepflegt

Zwei deterministische Relationen:

- `chapter_keyword` — der Begriff steht in der Keyword-Liste des Kapitels und gilt damit für alle Lernziele dieses Kapitels,
- `objective_title` — der Begriff kommt im Wortlaut des Lernziels vor.

Kein Modellaufruf, reproduzierbar aus Syllabus und Glossar.

### 4. Determinismus ohne Zufallsgenerator

Reihenfolge der Antwortoptionen und Position der richtigen Option werden aus einem stabilen FNV-1a-Hash über `(term_id, direction)` abgeleitet. `DefaultHasher` aus der Standardbibliothek ist über Rust-Versionen hinweg ausdrücklich nicht stabil und scheidet damit aus. FNV-1a sind rund zehn Zeilen im domain-Crate und ersparen eine neue Abhängigkeit.

### 5. Seed-Import wird versionierbar

Quality.md fordert „Import derselben Corpusversion ohne Duplikate". `seed.json` erhält deshalb eine Corpusversion. Weicht sie von der installierten ab, werden die Inhaltstabellen neu aufgebaut; `attempts` und `term_attempts` bleiben erhalten. Damit wird ein Inhaltsupdate ein normaler Vorgang statt eines manuellen Eingriffs.

## Migrationen

Additiv. Bestehende Lernhistorie bleibt erhalten.

| Migration | Meilenstein | Inhalt |
| --- | --- | --- |
| `0003_glossary_model.sql` | (a) | `glossary_terms.term_version`, `glossary_terms.reference`, `glossary_term_links`, `confusion_clusters`, `confusion_cluster_members`, `glossary_term_objectives` |
| `0004_attempt_reasoning.sql` | (b) | `attempts.reasoning_choice` |
| `0005_term_training.sql` | (d) | `term_attempts` |

`attempts.confidence` und `attempts.next_review_at` bestehen bereits seit `0002_adaptive_learning.sql`.

`glossary_term_links` führt Synonyme, Abkürzungen und See-also-Beziehungen in einer Tabelle mit `kind`-Unterscheidung. Der Wortlaut aus dem Glossar wird immer gespeichert; `target_term_id` bleibt leer, solange der Zielbegriff nicht installiert ist. Von 36 See-also-Kanten der 97 CTFL-Begriffe lösen sich derzeit 16 innerhalb des installierten Bestands auf — die Distraktorauswahl braucht deshalb den Kapitel-Fallback.

`term_attempts` ist im ursprünglichen Auftrag nicht genannt, aber ohne sie kann das Begriffstraining kein Ergebnis festhalten und die Wiederholungsplanung nichts über Begriffe planen.

## Meilensteine

Jeder Meilenstein ist erst abgeschlossen, wenn alle vier Prüfungen grün sind:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd web && npm run build
```

### (a) Glossarmodell — erledigt

Migration `0003`. `domain::GlossaryTerm` um `term_version`, `reference`, `synonyms`, `abbreviations` und `see_also` erweitert, neue Typen für Cluster und Cluster-Mitglieder, Lesepfade in `persistence`. `GET /api/glossary` liefert die Felder mit, das Frontend zeigt sie an.

Die Tabellen sind angelegt, aber noch leer — gefüllt werden sie in Meilenstein (c). Die Lesepfade liefern bis dahin leere Listen statt zu fehlen; genau das prüfen die Tests. Die Migration wurde zusätzlich gegen eine Bestandsdatenbank mit 7 Übungsversuchen und dem alten Starterglossar geprüft: Daten unverändert, Spalten und Tabellen ergänzt.

### (b) Antwortlogik im domain-Crate — erledigt

Migration `0004`. Neues Modul `domain::attempt`:

- `ReasoningChoice` — `recalled`, `eliminated`, `applied_rule`, `from_experience`, `guessed`, `not_stated`,
- `AttemptOutcome` — die vier Fälle aus Vision.md („Fragen üben"); `guessed` fällt in den unsicheren Zweig, bleibt im Diagnosetext aber unterscheidbar,
- daraus deterministisch: gilt als Beherrschung, Tutor empfohlen, Wiederholungsintervall, Diagnosetext.

Erfasst wird damit auch der Fall „richtig, aber durch Ausschluss", den eine Trefferquote nicht sieht.

`attempt_diagnosis` und die Intervalltabelle verschwinden aus `persistence`; dort bleibt nur der Aufruf. Unit-Tests im domain-Crate über die Matrix Ergebnis × Sicherheit × Begründung.

Die Begründungsauswahl wird zusammen mit der Sicherheitsangabe erhoben, also nach der Wahl der Antwortoption und vor der Auflösung. Vision.md sagt „nach der Antwort"; sobald das Ergebnis sichtbar ist, wäre die Angabe durch das Ergebnis verfälscht. Die Angabe bleibt freiwillig — Pflicht sind nur Auswahl und Sicherheit.

### (c) Import und Seed

`import_content.py` liefert die neuen Glossarfelder, liest `content/clusters.json`, berechnet die Begriff→Lernziel-Kanten und schreibt alles nach `seed.json`. `persistence` importiert sie und behandelt eine geänderte Corpusversion gemäß Entscheidung 5. Der Importer bleibt LLM-frei und idempotent.

### (d) Begriffstraining-Backend

Migration `0005`. Neues Modul `domain::term_training` mit vier Abfragerichtungen:

1. Begriff → Definition
2. Definition → Begriff
3. Szenario → Begriff
4. Begriff → Thema

Distraktoren bevorzugt aus demselben Verwechslungscluster, sonst aus `see_also`, sonst aus demselben Kapitel. Die Auswahl ist eine reine Funktion über einen von `persistence` gefüllten Kandidatenpool und wird im domain-Crate unit-getestet. Neue Routen `GET /api/courses/{id}/terms/next` und `POST /api/terms/{id}/attempts`. Kein Modellaufruf.

### (e) Wiederholungsplanung

Neues Modul `domain::review`, einheitlich über Learning Objectives und Begriffe: Intervallleiter, Vorrücken bei sicher richtig, Halten bei unsicher richtig, Rücksetzen bei falsch, Verdichtung bei gesetztem Prüfungsdatum. `persistence` liefert nur Zähler und Zeitpunkte. Deterministisch und unit-getestet.

### (f) Frontend

Tab „Begriffe" im Kurs-Arbeitsbereich mit Richtungswahl, Sicherheitsangabe und Clusterabgrenzung im Feedback. Begründungsauswahl im bestehenden Fragenmodus.

### Abschluss

`Content.md` um Glossarmodell, Cluster, Begriff→Lernziel-Kanten und `clusters.json` im Importprozess ergänzen. `README.md` auf Projektstatus, Lernmodi und Importaufruf bringen.

## Nicht Bestandteil dieser Meilensteine

- Probetest und Prüfungssimulation
- Tutor-Erweiterungen und freie Antworten
- Embeddings und semantische Suche
- Prüfungssprache-Training — setzt eine deutsche Glossarfassung voraus, die nicht importiert ist

## Offene Punkte

- Die harte Obergrenze `LIMIT 100` in der Glossarabfrage ([crates/persistence/src/lib.rs:316](crates/persistence/src/lib.rs#L316)) schneidet größere Bestände stillschweigend ab.
- Das Frontend zeigt den Hinweis „Starter-Snapshot" fest verdrahtet an ([web/src/App.tsx:830-832](web/src/App.tsx#L830-L832)), unabhängig vom tatsächlich installierten Bestand.
- Von 651 Glossarbegriffen sind 97 als CTFL-relevant installiert. Die Abgrenzung folgt Vision.md, ist im Produkt aber nirgends sichtbar.
- Die Demo-Fragen hängen an echten Lernzielen, sind aber weiterhin ein Starterbestand ohne Stilrichtlinie.
