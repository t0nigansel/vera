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
| `content/seed.json` | 98 CTFL-relevante Begriffe, 8 Kapitel, 68 Learning Objectives, 5 Verwechslungscluster |
| `content/clusters.json` | redaktionell gepflegt, 5 Cluster mit 14 Mitgliedern |

Erzeugt von `tools/import_content.py` aus den PDF-Quellen in `content/`, außer `clusters.json` — die wird gelesen, nie geschrieben. Der Importer bleibt regelbasiert, wiederholbar und ohne Modellaufruf.

**Die Meilensteine (a) bis (f) sind abgeschlossen.** Das Datenmodell führt Begriffsversion, Reference, Synonyme, Abkürzungen und See-also-Beziehungen, kennt Verwechslungscluster und die Zuordnung Begriff → Learning Objective. Antwortdiagnose und Wiederholungsplanung liegen als eigene Module im domain-Crate, das Begriffstraining ist über vier Abfragerichtungen bedienbar.

### Inhaltsupdate

`install_content` vergleicht die Corpusversion aus `seed.json` mit der installierten und aktualisiert bei Abweichung per Upsert. Ein Lauf von `import_content.py` genügt also; beim nächsten Serverstart zieht die Datenbank nach, ohne dass die Lernhistorie verloren geht.

Läuft der Server noch, hält er seine geöffnete Datenbankdatei weiter fest und liefert den alten Bestand aus. Vor dem Neustart deshalb beenden:

```bash
pkill -f 'target/debug/server'
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

Quality.md fordert „Import derselben Corpusversion ohne Duplikate". `seed.json` erhält deshalb eine Corpusversion. Weicht sie von der installierten ab, wird der Inhalt per Upsert aktualisiert; vollständig neu aufgebaut werden nur die reinen Ableitungstabellen. `attempts` und `term_attempts` bleiben unangetastet. Damit wird ein Inhaltsupdate ein normaler Vorgang statt eines manuellen Eingriffs.

Ursprünglich war ein Neuaufbau über `DELETE` geplant. Das geht nicht: `attempts.question_id` verweist mit `ON DELETE CASCADE` auf `questions`, ein Neuaufbau nähme die Lernhistorie stillschweigend mit.

## Migrationen

Additiv. Bestehende Lernhistorie bleibt erhalten.

| Migration | Meilenstein | Inhalt |
| --- | --- | --- |
| `0003_glossary_model.sql` | (a) | `glossary_terms.term_version`, `glossary_terms.reference`, `glossary_term_links`, `confusion_clusters`, `confusion_cluster_members`, `glossary_term_objectives` |
| `0004_attempt_reasoning.sql` | (b) | `attempts.reasoning_choice` |
| `0005_content_version.sql` | (c) | `content_versions` |
| `0006_term_training.sql` | (d) | `term_attempts` |
| `0007_commitments_and_mock_exams.sql` | — | `commitments`, `commitment_revisions`, `mock_exams`, `mock_exam_questions`, `questions.reserved_for_exam`; entstanden außerhalb dieser Meilensteine, siehe offene Punkte |

`attempts.confidence` und `attempts.next_review_at` bestehen bereits seit `0002_adaptive_learning.sql`.

`glossary_term_links` führt Synonyme, Abkürzungen und See-also-Beziehungen in einer Tabelle mit `kind`-Unterscheidung. Der Wortlaut aus dem Glossar wird immer gespeichert; `target_term_id` bleibt leer, solange der Zielbegriff nicht installiert ist. Von 36 See-also-Kanten der installierten 98 CTFL-Begriffe lösen sich derzeit 16 innerhalb des Bestands auf — die Distraktorauswahl braucht deshalb den Kapitel-Fallback.

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

### (c) Import und Seed — erledigt

`import_content.py` liefert die neuen Glossarfelder, liest `content/clusters.json`, berechnet die Begriff→Lernziel-Kanten und schreibt alles nach `seed.json`. `persistence` importiert sie und behandelt eine geänderte Corpusversion gemäß Entscheidung 5. Der Importer bleibt LLM-frei und idempotent.

Ergebnis: 98 Begriffe (97 Syllabus-Keywords plus `test suite` als Cluster-Mitglied), 5 Cluster mit 14 Mitgliedern, 86 Beziehungen, davon 16 aufgelöste und 20 offene See-also-Kanten, sowie 1413 Begriff→Lernziel-Kanten (1340 `chapter_keyword`, 73 `objective_title`). Corpusversion `ctfl-4.0.1-glossary-4.7.2-6bed28403813`, abgeleitet aus einem SHA-256 über den Seed-Inhalt — sie ändert sich genau dann, wenn sich der Inhalt ändert.

Aus `seed_if_empty` wurde `install_content`. Aktualisiert wird per Upsert, gelöscht werden nur reine Ableitungstabellen. Das ist kein Stilfrage: `attempts.question_id` hat `ON DELETE CASCADE`, ein Neuaufbau über `DELETE FROM questions` würde die Lernhistorie stillschweigend mitnehmen. Belegt durch `reinstalling_content_preserves_attempts_and_updates_questions` und zusätzlich an einer echten Bestandsdatenbank geprüft: 7 Übungsversuche vor und nach dem Inhaltswechsel.

Begriffe und Lernziele, die ein neuer Corpus nicht mehr enthält, bleiben als Altbestand stehen. Die Lernhistorie hat Vorrang vor einem aufgeräumten Bestand.

### (d) Begriffstraining-Backend — erledigt

Migration `0005`. Neues Modul `domain::term_training` mit vier Abfragerichtungen:

1. Begriff → Definition
2. Definition → Begriff
3. Szenario → Begriff
4. Begriff → Thema

Distraktoren bevorzugt aus demselben Verwechslungscluster, sonst aus `see_also`, sonst aus demselben Kapitel. Die Auswahl ist eine reine Funktion über einen von `persistence` gefüllten Kandidatenpool und wird im domain-Crate unit-getestet. Neue Routen `GET /api/courses/{id}/terms/next` und `POST /api/terms/{id}/attempts`. Kein Modellaufruf.

Die Lösung verlässt den Server nicht: `correct_option_id` ist `#[serde(skip_serializing)]`, und beim Absenden erzeugt der Server dieselbe Übung erneut und vergleicht. Möglich ist das nur, weil die Erzeugung vollständig deterministisch ist — die Optionsreihenfolge entsteht aus einer Rotation um einen FNV-1a-Hash über `(term_id, direction)`.

`Szenario → Begriff` beruht auf den redaktionellen Abgrenzungssätzen aus `clusters.json`, weil im Bestand keine Szenariotexte existieren. Die Richtung ist deshalb nur für Cluster-Mitglieder verfügbar. Echte Szenariostämme im Prüfungsregister gehören zur Fragen-Stilrichtlinie und damit in eine spätere Ausbaustufe.

Bewertung, Intervall und Diagnosetext stammen aus `AttemptDiagnosis::evaluate` — es gibt keine zweite Diagnoselogik neben der aus Meilenstein (b).

### (e) Wiederholungsplanung — erledigt

Neues Modul `domain::review`, einheitlich über Learning Objectives und Begriffe: Intervallleiter, Vorrücken bei sicher richtig, Halten bei unsicher richtig, Rücksetzen bei falsch, Verdichtung bei gesetztem Prüfungsdatum. `persistence` liefert nur Zähler und Zeitpunkte. Deterministisch und unit-getestet.

Die Leiter lautet 7, 14, 30, 60 Tage, indiziert über die Serie unmittelbar vorangehender Antworten, die als Beherrschung zählten. Was nicht als Beherrschung zählt — auch sicher richtig durch Ausschluss — wird nicht gestreckt, sondern behält das kurze Intervall aus der Diagnose.

Die Rangfolge ist jetzt für beide Lernmodi dieselbe: fällige Wiederholungen, dann noch nicht Geübtes, dann alles Weitere. Die eigenen `ORDER BY CASE`-Konstrukte in `next_question` und `next_term_exercise` sind entfallen. Weil eine sicher falsche Antwort das Intervall 0 ergibt und damit sofort wieder fällig ist, liefert `next_item` den ersten Eintrag, der **nicht** der zuletzt beantwortete ist — außer es gibt keinen anderen. Ohne diese Regel klebte der Nutzer an dem Eintrag fest, an dem er gerade scheitert.

`compress_for_exam` ist gerechnet und getestet, wird aber mit `None` aufgerufen: Das Prüfungsdatum liegt bislang nur im Browser. Die Anbindung braucht eine Schemaänderung und bleibt bewusst offen.

### (f) Frontend — erledigt

Tab „Begriffe" im Kurs-Arbeitsbereich mit Richtungswahl, Sicherheitsangabe und Clusterabgrenzung im Feedback. Begründungsauswahl im bestehenden Fragenmodus.

### Abschluss

`Content.md` um Glossarmodell, Cluster, Begriff→Lernziel-Kanten und `clusters.json` im Importprozess ergänzen. `README.md` auf Projektstatus, Lernmodi und Importaufruf bringen.

## Nicht Bestandteil dieser Meilensteine

- Probetest und Prüfungssimulation
- Tutor-Erweiterungen und freie Antworten
- Embeddings und semantische Suche
- Prüfungssprache-Training — setzt eine deutsche Glossarfassung voraus, die nicht importiert ist

## Offene Punkte

- Migration `0007` legt Tabellen für Commitments und Probetests an und ergänzt `questions.reserved_for_exam`. Sie wendet sich sauber an, aber **kein Code liest oder schreibt diese Tabellen**. Insbesondere filtert `next_question` nicht auf `reserved_for_exam = 0`: Für den Probetest reservierte Fragen erschienen damit weiterhin im Übungsmodus, was ihre Reservierung entwertet. Das ist zu schließen, bevor der Probetest gebaut wird.
- Das Prüfungsdatum liegt nur im Browser (`localStorage`). Solange der Server es nicht kennt, bleibt `compress_for_exam` ohne Wirkung und der Lernplan verdichtet sich vor dem Termin nicht. Die Anbindung braucht eine Schemaänderung und eine Route.

- Die harte Obergrenze `LIMIT 100` in der Glossarabfrage ([crates/persistence/src/lib.rs:316](crates/persistence/src/lib.rs#L316)) schneidet größere Bestände stillschweigend ab.
- Der Glossarhinweis im Frontend nennt den kursrelevanten Ausschnitt jetzt korrekt, ist aber weiterhin fest verdrahtet und nicht aus dem installierten Bestand abgeleitet.
- Von 651 Glossarbegriffen sind 97 als CTFL-relevant installiert. Die Abgrenzung folgt Vision.md, ist im Produkt aber nirgends sichtbar.
- Die Demo-Fragen hängen an echten Lernzielen, sind aber weiterhin ein Starterbestand ohne Stilrichtlinie.
