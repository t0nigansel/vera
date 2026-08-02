PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS courses (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT NOT NULL,
    exam_questions INTEGER NOT NULL,
    passing_score INTEGER NOT NULL,
    exam_minutes INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chapters (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    UNIQUE(course_id, position)
);

CREATE TABLE IF NOT EXISTS learning_objectives (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    chapter_id TEXT NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    k_level TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    source_label TEXT NOT NULL,
    source_url TEXT NOT NULL,
    source_section TEXT,
    UNIQUE(course_id, code)
);

CREATE TABLE IF NOT EXISTS glossary_terms (
    id TEXT PRIMARY KEY,
    term TEXT NOT NULL,
    definition TEXT NOT NULL,
    language TEXT NOT NULL,
    snapshot TEXT NOT NULL,
    origin TEXT NOT NULL,
    source_label TEXT NOT NULL,
    source_url TEXT NOT NULL,
    UNIQUE(term, language, snapshot)
);

CREATE TABLE IF NOT EXISTS course_glossary_terms (
    course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    glossary_term_id TEXT NOT NULL REFERENCES glossary_terms(id) ON DELETE CASCADE,
    PRIMARY KEY(course_id, glossary_term_id)
);

CREATE TABLE IF NOT EXISTS questions (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    learning_objective_id TEXT NOT NULL REFERENCES learning_objectives(id) ON DELETE CASCADE,
    prompt TEXT NOT NULL,
    question_type TEXT NOT NULL,
    origin TEXT NOT NULL,
    points INTEGER NOT NULL,
    explanation TEXT NOT NULL,
    source_label TEXT NOT NULL,
    source_url TEXT NOT NULL,
    source_section TEXT
);

CREATE TABLE IF NOT EXISTS question_options (
    id TEXT PRIMARY KEY,
    question_id TEXT NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    text TEXT NOT NULL,
    is_correct INTEGER NOT NULL CHECK(is_correct IN (0, 1)),
    feedback TEXT NOT NULL,
    UNIQUE(question_id, position)
);

CREATE TABLE IF NOT EXISTS attempts (
    id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL DEFAULT 'local-default',
    question_id TEXT NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    selected_option_ids TEXT NOT NULL,
    reasoning TEXT NOT NULL DEFAULT '',
    is_correct INTEGER NOT NULL CHECK(is_correct IN (0, 1)),
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS llm_traces (
    id TEXT PRIMARY KEY,
    use_case TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    course_id TEXT,
    prompt_version TEXT NOT NULL,
    source_ids TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    success INTEGER NOT NULL CHECK(success IN (0, 1)),
    error TEXT,
    created_at TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_fts USING fts5(
    id UNINDEXED,
    course_id UNINDEXED,
    title,
    text,
    source_label UNINDEXED,
    source_url UNINDEXED,
    source_section UNINDEXED,
    origin UNINDEXED,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE INDEX IF NOT EXISTS idx_objectives_course ON learning_objectives(course_id);
CREATE INDEX IF NOT EXISTS idx_questions_course ON questions(course_id);
CREATE INDEX IF NOT EXISTS idx_attempts_question ON attempts(question_id);

