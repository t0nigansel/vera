#!/usr/bin/env python3
"""Deterministischer Importer für offizielle ISTQB-Quellen.

Liest die PDF-Quellen aus content/ und erzeugt strukturierte JSON-Dateien:

    content/generated/glossary.json   – vollständiges ISTQB-Glossar
    content/generated/ctfl.json       – CTFL-Kapitel, Learning Objectives, Keywords

Anschließend wird content/seed.json aktualisiert: Der CTFL-Kurs erhält die
echten Kapitel und Learning Objectives, das Glossar die tatsächlich für CTFL
relevanten Begriffe (Keyword-Listen des Syllabus).

Kein LLM. Rein regelbasiert und wiederholbar.

Aufruf:  python3 tools/import_content.py
"""

from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass, field, asdict
from pathlib import Path

try:
    from pypdf import PdfReader
except ImportError:  # pragma: no cover
    sys.exit("pypdf fehlt.  Installation:  pip install pypdf")


ROOT = Path(__file__).resolve().parent.parent
CONTENT = ROOT / "content"
GENERATED = CONTENT / "generated"

GLOSSARY_PDF = CONTENT / "search_result_8_2_2026.pdf"
CTFL_PDF = CONTENT / "ISTQB_CTFL_Syllabus_v4.0.1.pdf"

GLOSSARY_URL = "https://glossary.istqb.org/"
CTFL_URL = "https://www.istqb.org/certifications/certified-tester-foundation-level"
CTFL_LABEL = "ISTQB CTFL v4.0.1 Syllabus"
GLOSSARY_LABEL = "ISTQB Standard Glossary of Terms used in Software Testing"


# ---------------------------------------------------------------- Hilfsmittel


def read_pdf(path: Path) -> str:
    if not path.exists():
        sys.exit(f"Quelle fehlt: {path}")
    reader = PdfReader(str(path))
    return "\n".join(page.extract_text() or "" for page in reader.pages)


def slugify(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return slug or "term"


# ------------------------------------------------------------------- Glossar


LABELS = ("Reference:", "Synonyms:", "Synonym:", "Abbreviation:", "Abbreviations:", "See also:")

# Kopf- und Fußzeilen des Glossar-Exports
GLOSSARY_NOISE = re.compile(
    r"^(Standard Glossary of Terms|used in Software Testing|Version \d+\.\d+|"
    r"[A-Z][a-z]{2} \d+, \d{4}|Search results|Exact matches first|All terms|"
    r"Except where otherwise noted|a Creative Commons|Page \d+|\d+\s*/\s*\d+)"
)


@dataclass
class GlossaryTerm:
    term: str
    definition: str
    term_version: str = ""
    reference: str = ""
    synonyms: list[str] = field(default_factory=list)
    abbreviations: list[str] = field(default_factory=list)
    see_also: list[str] = field(default_factory=list)


def parse_glossary(raw: str) -> list[GlossaryTerm]:
    """Der Export ist streng regelmäßig: Begriff, 'Version N', Definition, Labels."""
    lines = [line.strip() for line in raw.splitlines()]
    lines = [line for line in lines if line and not GLOSSARY_NOISE.match(line)]

    version_at = [i for i, line in enumerate(lines) if re.fullmatch(r"Version \d+", line)]
    terms: list[GlossaryTerm] = []

    for pos, idx in enumerate(version_at):
        if idx == 0:
            continue
        term = lines[idx - 1]
        end = version_at[pos + 1] - 1 if pos + 1 < len(version_at) else len(lines)
        body = lines[idx + 1 : end]

        definition_parts: list[str] = []
        buckets: dict[str, list[str]] = {label: [] for label in LABELS}
        current: str | None = None

        for line in body:
            matched = next((label for label in LABELS if line.startswith(label)), None)
            if matched:
                current = matched
                rest = line[len(matched) :].strip()
                if rest:
                    buckets[matched].append(rest)
            elif current:
                buckets[current].append(line)
            else:
                definition_parts.append(line)

        def collect(*labels: str) -> list[str]:
            values: list[str] = []
            for label in labels:
                for chunk in buckets.get(label, []):
                    values.extend(part.strip() for part in chunk.split(",") if part.strip())
            return values

        definition = " ".join(definition_parts).strip()
        if not definition:
            continue

        terms.append(
            GlossaryTerm(
                term=term,
                definition=definition,
                term_version=re.sub(r"\D", "", lines[idx]),
                reference=" ".join(buckets["Reference:"]).strip(),
                synonyms=collect("Synonyms:", "Synonym:"),
                abbreviations=collect("Abbreviation:", "Abbreviations:"),
                see_also=collect("See also:"),
            )
        )

    return terms


# ------------------------------------------------------------------ Syllabus

CHAPTER_RE = re.compile(r"^(\d)\.\s+(.+?)\s+[–-]\s+\d+\s+minutes\s*$")
CHAPTER_WRAPPED_RE = re.compile(r"^(\d)\.\s+([A-Z].+?)\s*$")
DURATION_RE = re.compile(r"^[–-]\s*\d+\s+minutes\s*$")
OBJECTIVE_RE = re.compile(r"^(FL-(\d)\.\d+\.\d+)\s*\((K\d)\)\s*(.+?)\s*$")
SECTION_RE = re.compile(r"^(\d\.\d+)\s+(.+?)\s*$")


def despace(value: str) -> str:
    """PDF-Extraktion zerreißt Wörter ('test proces s'). Für Vergleiche normalisieren."""
    return re.sub(r"[^a-z0-9]", "", value.lower())


def build_vocabulary(raw: str) -> set[str]:
    return {w.lower() for w in re.findall(r"[A-Za-z]{2,}", raw)}


def repair_spacing(text: str, vocabulary: set[str]) -> str:
    """Zerrissene Wörter zusammenfügen – aber nur, wenn das Dokument selbst
    die zusammengefügte Schreibweise kennt und die Bruchstücke nicht.

    'software developm ent' -> 'software development'   (development ist bekannt)
    'test the system'       -> unverändert              (testthe ist unbekannt)
    """
    text = re.sub(r"(?<=[a-z])- (?=[a-z])", "-", text)
    parts = text.split(" ")
    out: list[str] = []
    for part in parts:
        if out:
            left, right = out[-1], part
            merged = left + right
            core = re.sub(r"[^A-Za-z]", "", merged).lower()
            left_core = re.sub(r"[^A-Za-z]", "", left).lower()
            right_core = re.sub(r"[^A-Za-z]", "", right).lower()
            fragment = left_core not in vocabulary or right_core not in vocabulary
            if core in vocabulary and fragment and left_core and right_core:
                out[-1] = merged
                continue
        out.append(part)
    return " ".join(out)


@dataclass
class Objective:
    code: str
    k_level: str
    title: str
    section: str = ""


@dataclass
class Chapter:
    position: int
    title: str
    keywords: list[str] = field(default_factory=list)
    objectives: list[Objective] = field(default_factory=list)


def parse_syllabus(raw: str) -> list[Chapter]:
    vocabulary = build_vocabulary(raw)
    lines = [re.sub(r"\s+", " ", line).strip() for line in raw.splitlines()]
    lines = [repair_spacing(line, vocabulary) for line in lines if line]

    chapters: dict[int, Chapter] = {}
    sections: dict[str, str] = {}

    # Kapitelüberschriften und Keyword-Blöcke.
    # Das Inhaltsverzeichnis wird über die Punktführung ausgeschlossen; lange
    # Überschriften stehen im Fließtext auf zwei Zeilen ("Titel" / "– N minutes").
    for i, line in enumerate(lines):
        if "...." in line:
            continue
        match = CHAPTER_RE.match(line)
        if match:
            number, title = int(match.group(1)), match.group(2).strip()
            after = i + 1
        else:
            wrapped = CHAPTER_WRAPPED_RE.match(line)
            follows_duration = i + 1 < len(lines) and DURATION_RE.match(lines[i + 1])
            if not (wrapped and follows_duration):
                continue
            number, title = int(wrapped.group(1)), wrapped.group(2).strip()
            after = i + 2
        chapter = chapters.setdefault(number, Chapter(position=number, title=title))
        if not chapter.title:
            chapter.title = title
        if chapter.keywords:
            continue
        # Keywords stehen unmittelbar nach der Überschrift
        window = lines[after : after + 12]
        if window and window[0].lower().startswith("keyword"):
            collected: list[str] = []
            for entry in window[1:]:
                if entry.lower().startswith("learning objective") or CHAPTER_RE.match(entry):
                    break
                collected.append(entry)
            chapter.keywords = [
                kw.strip() for kw in " ".join(collected).split(",") if kw.strip()
            ]

    for line in lines:
        match = SECTION_RE.match(line)
        if match and not OBJECTIVE_RE.match(line):
            sections.setdefault(match.group(1), match.group(2).strip())

    seen: set[str] = set()
    for i, line in enumerate(lines):
        match = OBJECTIVE_RE.match(line)
        if not match:
            continue
        code, chapter_no, k_level, title = match.groups()
        if code in seen:
            continue
        seen.add(code)

        # Lange Titel laufen im PDF auf die nächste Zeile über.
        for follow in lines[i + 1 : i + 3]:
            if (
                OBJECTIVE_RE.match(follow)
                or SECTION_RE.match(follow)
                or CHAPTER_RE.match(follow)
                or follow.lower().startswith("learning objective")
                or "...." in follow
                or re.match(r"^(FL-|Page |© |v?\d+\.\d+\.\d+)", follow)
            ):
                break
            if follow[:1].isupper() or follow[:1].isdigit():
                break
            title = f"{title} {follow}".strip()
            break
        chapter = chapters.setdefault(int(chapter_no), Chapter(position=int(chapter_no), title=""))
        section = ".".join(code.replace("FL-", "").split(".")[:2])
        chapter.objectives.append(
            Objective(code=code, k_level=k_level, title=title.strip(), section=section)
        )

    for chapter in chapters.values():
        chapter.objectives.sort(key=lambda o: [int(p) for p in o.code.replace("FL-", "").split(".")])

    return [chapters[key] for key in sorted(chapters)], sections


# --------------------------------------------------------------------- Ausgabe


def build_seed(terms: list[GlossaryTerm], chapters: list[Chapter], sections: dict[str, str]) -> dict:
    seed = json.loads((CONTENT / "seed.json").read_text(encoding="utf-8"))

    # Vergleich über die entzerrte Form, damit PDF-Trennartefakte
    # ("test proces s") trotzdem auf den Glossarbegriff treffen.
    keyword_set = {despace(kw) for chapter in chapters for kw in chapter.keywords}
    relevant = [t for t in terms if despace(t.term) in keyword_set]

    ctfl = next(c for c in seed["courses"] if c["id"] == "ctfl-4")
    ctfl["version"] = "4.0.1"
    ctfl["chapters"] = []

    first_objective_id = None
    for chapter in chapters:
        chapter_id = f"ctfl-ch{chapter.position}"
        objectives = []
        for objective in chapter.objectives:
            objective_id = f"ctfl-lo-{objective.code.replace('FL-', '').replace('.', '-')}"
            first_objective_id = first_objective_id or objective_id
            section_title = sections.get(objective.section, "")
            objectives.append(
                {
                    "id": objective_id,
                    "code": objective.code,
                    "k_level": objective.k_level,
                    "title": objective.title,
                    "summary": objective.title,
                    "source_label": CTFL_LABEL,
                    "source_url": CTFL_URL,
                    "source_section": f"{objective.section} {section_title}".strip(),
                }
            )
        ctfl["chapters"].append(
            {
                "id": chapter_id,
                "position": chapter.position,
                "title": chapter.title,
                "description": ", ".join(chapter.keywords[:12]),
                "objectives": objectives,
            }
        )

    seed["glossary"] = [
        {
            "id": f"glossary-{slugify(term.term)}",
            "term": term.term,
            "definition": term.definition,
            "language": "en",
            "snapshot": "istqb-glossary-4.7.2",
            "origin": "official",
            "source_label": GLOSSARY_LABEL,
            "source_url": GLOSSARY_URL,
            "courses": ["ctfl-4"],
        }
        for term in relevant
    ]

    # Demo-Fragen auf echte Learning Objectives umhängen, damit Fremdschlüssel halten.
    valid = {o["id"] for c in ctfl["chapters"] for o in c["objectives"]}
    for question in seed["questions"]:
        if question["course_id"] == "ctfl-4" and question["learning_objective_id"] not in valid:
            question["learning_objective_id"] = first_objective_id

    return seed, relevant


def main() -> None:
    GENERATED.mkdir(parents=True, exist_ok=True)

    terms = parse_glossary(read_pdf(GLOSSARY_PDF))
    chapters, sections = parse_syllabus(read_pdf(CTFL_PDF))

    (GENERATED / "glossary.json").write_text(
        json.dumps(
            {
                "source_label": GLOSSARY_LABEL,
                "source_url": GLOSSARY_URL,
                "snapshot": "istqb-glossary-4.7.2",
                "license": "CC BY 4.0",
                "terms": [asdict(t) for t in terms],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )

    (GENERATED / "ctfl.json").write_text(
        json.dumps(
            {
                "course": "CTFL",
                "version": "4.0.1",
                "source_label": CTFL_LABEL,
                "source_url": CTFL_URL,
                "chapters": [asdict(c) for c in chapters],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )

    seed, relevant = build_seed(terms, chapters, sections)
    (CONTENT / "seed.json").write_text(
        json.dumps(seed, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    objectives = sum(len(c.objectives) for c in chapters)
    keywords = {despace(kw) for c in chapters for kw in c.keywords}
    print(f"Glossar gesamt      : {len(terms)} Begriffe")
    print(f"CTFL-Keywords       : {len(keywords)}")
    print(f"CTFL-relevant       : {len(relevant)} Begriffe")
    print(f"CTFL-Kapitel        : {len(chapters)}")
    print(f"Learning Objectives : {objectives}")
    for chapter in chapters:
        print(
            f"  {chapter.position}. {chapter.title[:44]:<44} "
            f"LO {len(chapter.objectives):>2}  Keywords {len(chapter.keywords):>2}"
        )

    missing = sorted(keywords - {despace(t.term) for t in terms})
    if missing:
        print(f"\nKeywords ohne Glossartreffer ({len(missing)}): {', '.join(missing)}")


if __name__ == "__main__":
    main()
