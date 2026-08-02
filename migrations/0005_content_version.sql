CREATE TABLE IF NOT EXISTS content_versions (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    corpus_version TEXT NOT NULL,
    installed_at TEXT NOT NULL
);
