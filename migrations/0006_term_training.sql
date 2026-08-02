CREATE TABLE IF NOT EXISTS term_attempts (
    id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL DEFAULT 'local-default',
    glossary_term_id TEXT NOT NULL REFERENCES glossary_terms(id) ON DELETE CASCADE,
    direction TEXT NOT NULL,
    selected_option_id TEXT NOT NULL,
    is_correct INTEGER NOT NULL CHECK(is_correct IN (0, 1)),
    confidence TEXT NOT NULL,
    next_review_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_term_attempts_review ON term_attempts(profile_id, next_review_at);
CREATE INDEX IF NOT EXISTS idx_term_attempts_term ON term_attempts(glossary_term_id);
