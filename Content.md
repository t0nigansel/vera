# learnISTQB – Inhalte und Wissensmodell

## Zweck dieses Dokuments

Dieses Dokument beschreibt, welche Inhalte learnISTQB verwendet, wie sie strukturiert, versioniert und miteinander verknüpft werden und wie ihre Herkunft sichtbar bleibt.

Die Qualität des Inhaltsmodells hat Vorrang vor der Menge generierter Inhalte.

## Offizielle Lernbasis

Für jeden Kurs werden mindestens folgende offizielle Quellen berücksichtigt:

- Syllabus,
- Learning Objectives und Business Outcomes,
- Kapitel-Keywords,
- ISTQB-Glossar,
- offizielle Beispielprüfungen,
- offizielle Antworten und Begründungen,
- Exam Structure and Rules,
- Exam Structure Tables, sofern für den Kurs relevant.

Die erste Produktgeneration unterstützt:

| Kurs | Kursversion | Primäre Inhalte |
| --- | --- | --- |
| Certified Tester Foundation Level | CTFL v4.0 | Syllabus, Glossar, Sample Exams, Prüfungsstruktur |
| Advanced Level Test Management | CTAL-TM v3.0 | Syllabus, Glossar, Sample Exams, Prüfungsstruktur |
| AI Testing | CT-AI v2.0 | Syllabus, Glossar, Sample Exams, Prüfungsstruktur |

Die genaue Dokumentversion wird im Quellenmanifest festgehalten. Eine Kursversion wie „CTFL v4.0“ darf daher beispielsweise einen Syllabus mit einer aktuelleren Patchversion enthalten.

## Inhaltsklassen

Jeder Inhalt besitzt eine Klasse:

### Official

Unveränderte oder strukturell normalisierte Inhalte aus einer offiziellen Quelle.

Beispiele:

- Syllabus-Abschnitt,
- Glossardefinition,
- offizielle Sample-Exam-Frage,
- offizielle Antwortbegründung.

### Editorial

Von den Produktverantwortlichen erstellter und geprüfter Inhalt.

Beispiele:

- vereinfachte Erklärung,
- zusätzliche Übungsfrage,
- Bewertungsrubrik,
- kuratierte Gegenüberstellung ähnlicher Begriffe.

### Generated

Durch ein Sprachmodell erzeugter, nicht redaktionell freigegebener Inhalt.

Beispiele:

- spontane Transferfrage,
- individuelles Beispiel,
- Tutorantwort,
- temporäre Zusammenfassung.

### User

Vom Nutzer erstellter Inhalt.

Beispiele:

- offene Antwort,
- eigene Zusammenfassung,
- Notiz,
- später eventuell Mindmap oder Karteikarte.

Die Klasse ist in Datenbank, API und Benutzeroberfläche sichtbar. `Generated` darf nicht stillschweigend zu `Editorial` oder `Official` werden.

## Quellenmanifest

Jeder importierbare Corpus besitzt ein Manifest, beispielsweise:

```toml
[course]
id = "ctfl"
version = "4.0"
language = "en"

[[documents]]
id = "ctfl-syllabus"
kind = "syllabus"
version = "4.0.1"
url = "https://..."
sha256 = "..."
license_review = "pending"

[[documents]]
id = "ctfl-sample-a-questions"
kind = "sample_exam_questions"
version = "..."
url = "https://..."
sha256 = "..."
license_review = "pending"

[glossary]
snapshot = "..."
languages = ["en", "de"]
```

Das Manifest dient als reproduzierbare Beschreibung, nicht als Beweis für ein Nutzungsrecht.

## Kernentitäten

### Course

- stabile ID
- offizieller Name
- Kurzbezeichnung
- Zertifizierungsfamilie

### CourseVersion

- Kurs-ID
- Versionsbezeichnung
- Status
- Veröffentlichungs- und Importdatum
- Sprache
- vorausgesetzte Kurse

### SourceDocument

- Dokument-ID
- Dokumenttyp
- Titel
- Version
- Sprache
- URL
- Prüfsumme
- Rechte- und Freigabestatus

### Chapter und Section

- stabile ID innerhalb der Dokumentversion
- Nummer und Überschrift
- hierarchische Position
- Seitenbereich
- Rohtext und normalisierter Text

### LearningObjective

- offizielle Kennung
- Wortlaut
- K-Level
- Kapitel und Abschnitt
- zugehörige Business Outcomes
- vorausgesetzte Objectives

### GlossaryTerm

- stabile fachliche ID
- offizielle Benennung
- offizielle Definition
- Sprache
- Glossarsnapshot
- Synonyme und Abkürzungen
- `see also`-Beziehungen
- zugehörige Syllabi und Keywords

### Question

- Herkunftsklasse
- Freigabestatus
- Fragetext und Szenario
- Antwortoptionen
- korrekte Antwort oder korrekte Kombination
- Punkte
- K-Level
- Learning Objectives
- Begründung pro Option
- Quellen
- Eignung für Lernmodus und Prüfungssimulation

### Rubric

- erwartete Konzepte
- optionale Aspekte
- falsche oder widersprüchliche Aussagen
- bekannte Fehlvorstellungen
- Mindestanforderungen nach K-Level
- Quellen

## Glossarmodell

Das Glossar ist kein gewöhnlicher RAG-Chunkbestand. Es besitzt strukturierte Begriffe und Beziehungen.

Zu jedem Begriff können gespeichert werden:

- offizielle Definitionen pro Sprache,
- offizielle und inoffizielle Übersetzungen,
- frühere Bezeichnungen,
- Abkürzungen,
- Synonyme,
- verwandte Begriffe,
- abzugrenzende Begriffe,
- verwendende Kurse und Versionen,
- Kapitel, in denen der Begriff ein Keyword ist.

Bei exakten Begriffen gilt:

```text
offizielle Glossardefinition
> offizieller Syllabus-Kontext
> redaktionelle Erklärung
> generierte Erklärung
```

Eine generierte Übersetzung wird ausdrücklich als solche gespeichert und darf eine offizielle deutsche Definition nicht überschreiben.

### Umgesetzter Stand

Gespeichert werden je Begriff die Glossarversion (`term_version`), die Quellenangabe des Glossars (`reference`) sowie Synonyme, Abkürzungen und `see also`-Beziehungen. Die drei Beziehungsarten liegen gemeinsam in `glossary_term_links`, unterschieden über `kind`.

Der Wortlaut einer Beziehung wird immer gespeichert, auch wenn der Zielbegriff nicht installiert ist; `target_term_id` bleibt dann leer. Das ist der Regelfall und kein Mangel: Von den 36 `see also`-Kanten der CTFL-Begriffe zeigen 20 auf Begriffe außerhalb des kursrelevanten Bestands.

### Verwechslungscluster

Cluster sind redaktionell und liegen in `content/clusters.json`. Diese Datei wird von keinem Werkzeug geschrieben. `tools/import_content.py` liest sie, prüft jedes Mitglied gegen das Glossar — ein unbekannter Begriff bricht den Import ab — und übernimmt sie in den Seed.

Je Mitglied wird neben der Position ein redaktioneller Abgrenzungssatz geführt. Er ist kein Ersatz für die offizielle Definition, sondern benennt, wodurch sich dieses Mitglied von den übrigen des Clusters unterscheidet. Im Begriffstraining ist dieser Satz die eigentliche Rückmeldung nach einer Antwort.

Ein Cluster darf Begriffe enthalten, die kein Syllabus-Keyword sind. Sie werden dann zusätzlich in den Seed aufgenommen, damit die Abgrenzung vollständig bleibt.

### Begriff und Learning Objective

Die Zuordnung wird deterministisch abgeleitet, nicht gepflegt, und in `glossary_term_objectives` mit ihrer Herkunft geführt:

- `chapter_keyword` — der Begriff steht in der Keyword-Liste eines Kapitels und gilt damit für alle Lernziele dieses Kapitels,
- `objective_title` — der Begriff kommt im Wortlaut eines Lernziels vor, geprüft an Wortgrenzen.

Beide Relationen können für dasselbe Paar bestehen. Da die Ableitung reproduzierbar ist, wird sie bei jedem Inhaltsupdate neu gebildet statt fortgeschrieben.

## Sprachstrategie

Die Benutzeroberfläche und Tutorinteraktion können deutsch sein, auch wenn eine Quelle nur auf Englisch vorliegt.

Es wird zwischen drei Ebenen unterschieden:

1. Sprache der offiziellen Quelle
2. Sprache einer offiziellen Übersetzung
3. Sprache einer generierten oder redaktionellen Erläuterung

Offizielle Terminologie wird bevorzugt. Wenn keine offizielle deutsche Fassung vorhanden ist, zeigt die Anwendung auf Wunsch den englischen Originalbegriff zusätzlich zur deutschen Erklärung.

## Normalisierung

Der Importprozess bewahrt den Originaltext und erzeugt zusätzlich eine normalisierte Darstellung.

Normalisierung darf:

- wiederkehrende Kopf- und Fußzeilen entfernen,
- Silbentrennungen und Unicode-Probleme beheben,
- Absätze und Listen rekonstruieren,
- Überschriften hierarchisch zuordnen,
- Tabellen in strukturierte Daten überführen.

Normalisierung darf nicht:

- fachliche Formulierungen umschreiben,
- Definitionen vereinfachen,
- Beispiele entfernen, ohne dies zu dokumentieren,
- verschiedene Dokumentversionen vermischen.

Manuelle Korrekturen werden als versionierte Overrides gespeichert und nicht direkt in generierte Importdateien geschrieben.

## Chunking

Chunks werden an fachlichen Grenzen gebildet, bevorzugt:

- Definition,
- Absatzgruppe innerhalb eines Abschnitts,
- Aufzählung mit einleitendem Satz,
- vollständiges Beispiel,
- Learning Objective mit zugehörigem Kontext.

Ein Chunk besitzt:

- stabile Chunk-ID,
- Dokument- und Versions-ID,
- Kapitel und Abschnitt,
- Seite oder Seitenbereich,
- Sprache,
- Learning Objectives,
- Glossarbegriffe,
- Text,
- Prüfsumme.

Starres Trennen nach einer festen Zeichenzahl ist nur ein Fallback. Überlappungen müssen nachvollziehbar und klein bleiben.

## Fragen und Antworten

### Offizielle Fragen

- werden inhaltlich nicht verändert,
- behalten ursprüngliche Nummerierung und Version,
- verwenden die offizielle Lösung,
- zeigen die offizielle Begründung getrennt von Tutorfeedback,
- werden nur entsprechend der geklärten Nutzungsrechte ausgeliefert.

### Redaktionelle Fragen

- benötigen mindestens eine Quelle,
- benötigen Learning Objective und K-Level,
- benötigen Begründungen für richtige und relevante falsche Optionen,
- durchlaufen eine fachliche Freigabe,
- werden auf Mehrdeutigkeit geprüft.

### Generierte Fragen

- sind sichtbar als KI-generiert markiert,
- erhalten eine Ablauf- oder Cache-Strategie,
- müssen Quellen und erwartete Konzepte nennen,
- dürfen keine offizielle Frage imitieren oder als solche erscheinen,
- werden im MVP nicht für realistische Prüfungssimulationen verwendet.

## Quellenanzeige

Eine Quellenreferenz enthält mindestens:

- Kurztitel,
- Dokumentversion,
- Abschnitt,
- Seite, wenn verfügbar,
- Link zur offiziellen Quelle, wenn zulässig.

Der Nutzer kann zwischen der erklärenden Antwort und der dafür verwendeten Passage wechseln.

## Inhaltliche Abhängigkeiten

Ein Kurs kann Wissen aus einem anderen Kurs voraussetzen. Diese Abhängigkeit wird explizit modelliert.

Beispiel:

```text
CTAL-TM v3.0
└── setzt Foundation-Wissen voraus
    └── verweist auf passende CTFL-Objectives und Glossarbegriffe
```

Dadurch kann die Anwendung erkennen, ob eine Schwierigkeit im aktuellen Kurs oder in fehlendem Grundlagenwissen liegt.

## Import- und Aktualisierungsprozess

1. Offizielle Quelle in einem Manifest registrieren.
2. Rechte- und Verwendungshinweise erfassen.
3. Datei herunterladen und Prüfsumme festhalten.
4. Rohtext extrahieren.
5. Struktur normalisieren.
6. Learning Objectives, Keywords und Glossarbegriffe verknüpfen.
7. automatisierte Konsistenztests durchführen.
8. manuelle Stichprobe und Korrekturen durchführen.
9. Embeddings für eine neue Indexversion erzeugen.
10. Corpus atomar veröffentlichen.

Eine neue Dokumentversion überschreibt keine alte. Bereits erzeugte Lern- und Modelltraces bleiben auf ihre ursprüngliche Corpusversion beziehbar.

### Umgesetzter Stand

`tools/import_content.py` deckt die Schritte 4 bis 6 ab. Es liest die PDF-Quellen aus `content/`, erzeugt `content/generated/glossary.json` und `content/generated/ctfl.json` und schreibt daraus zusammen mit `content/clusters.json` den Seed `content/seed.json`. Der Lauf ist regelbasiert, ohne Modellaufruf und wiederholbar: Zwei aufeinanderfolgende Läufe erzeugen byte-identische Dateien.

```bash
python3 tools/import_content.py
```

Der Seed trägt eine Corpusversion, gebildet aus Kurs- und Glossarversion sowie den ersten Stellen eines SHA-256 über den Seed-Inhalt. Sie ändert sich genau dann, wenn sich der Inhalt ändert, und muss nicht manuell gepflegt werden.

Beim Serverstart vergleicht `install_content` diese Version mit der installierten. Bei Abweichung werden Kurse, Kapitel, Lernziele, Begriffe, Fragen und Antwortoptionen per Upsert aktualisiert; vollständig neu aufgebaut werden nur die reinen Ableitungstabellen — Kurszuordnungen, Begriffsbeziehungen, Cluster, Begriff-Lernziel-Kanten und der Volltextindex.

Gelöscht wird dabei nichts, was die Lernhistorie trägt. Das ist keine Vorsichtsmaßnahme, sondern notwendig: `attempts.question_id` verweist mit `ON DELETE CASCADE` auf `questions`, ein Neuaufbau über `DELETE` nähme die Versuche des Nutzers stillschweigend mit. Begriffe und Lernziele, die ein neuer Corpus nicht mehr enthält, bleiben deshalb als Altbestand stehen.

## Rechte und Veröffentlichung

Offizielle Verfügbarkeit im Internet bedeutet nicht automatisch, dass vollständige Inhalte in einer eigenen öffentlichen oder kommerziellen Anwendung weiterverbreitet werden dürfen.

Vor einer öffentlichen Bereitstellung werden für jede Dokumentklasse geprüft:

- Urheber- und Markenhinweise,
- erlaubte Nutzung von Auszügen,
- Nutzung offizieller Beispielprüfungen,
- Übersetzungsrechte,
- Anforderungen an Quellenangaben,
- mögliche Akkreditierungs- oder Genehmigungspflichten.

Der Freigabestatus wird im Manifest und in der Datenbank geführt:

```text
unknown
→ reviewed_for_local_prototype
→ approved_for_public_use
→ approved_for_commercial_use
→ prohibited
```

Der MVP für die lokale Entwicklung darf keine stillschweigende Annahme über spätere kommerzielle Nutzungsrechte treffen.

## Content-Qualitätsregeln

- Kein prüfungsrelevanter Inhalt ohne Herkunft.
- Keine offizielle Frage ohne dokumentierte Lösung.
- Keine redaktionelle Frage ohne Learning Objective und K-Level.
- Keine Glossardefinition ohne Sprache und Snapshot.
- Keine Übersetzung ohne Kennzeichnung ihrer Herkunft.
- Keine neue Corpusversion ohne Konsistenz- und Retrievaltests.
- Keine KI-generierten Inhalte in der realistischen Prüfungssimulation.

