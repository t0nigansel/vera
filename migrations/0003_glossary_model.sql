ALTER TABLE glossary_terms ADD COLUMN term_version TEXT NOT NULL DEFAULT '';
ALTER TABLE glossary_terms ADD COLUMN reference TEXT NOT NULL DEFAULT '';

-- Synonyme, Abkürzungen und See-also-Beziehungen eines Begriffs.
-- Der Wortlaut aus dem Glossar wird immer gespeichert; target_term_id bleibt
-- leer, solange der Zielbegriff nicht installiert ist.
CREATE TABLE IF NOT EXISTS glossary_term_links (
    glossary_term_id TEXT NOT NULL REFERENCES glossary_terms(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK(kind IN ('synonym', 'abbreviation', 'see_also')),
    value TEXT NOT NULL,
    target_term_id TEXT REFERENCES glossary_terms(id) ON DELETE SET NULL,
    PRIMARY KEY(glossary_term_id, kind, value)
);

-- Redaktionell gepflegte Gruppen ähnlicher Begriffe. Sie dienen dem
-- Begriffstraining und später den Distraktoren neuer Fragen.
CREATE TABLE IF NOT EXISTS confusion_clusters (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    origin TEXT NOT NULL DEFAULT 'editorial'
);

CREATE TABLE IF NOT EXISTS confusion_cluster_members (
    cluster_id TEXT NOT NULL REFERENCES confusion_clusters(id) ON DELETE CASCADE,
    glossary_term_id TEXT NOT NULL REFERENCES glossary_terms(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    distinction TEXT NOT NULL DEFAULT '',
    PRIMARY KEY(cluster_id, glossary_term_id)
);

-- Abgeleitet aus Syllabus-Keywords und Lernzielwortlaut, nicht gepflegt.
CREATE TABLE IF NOT EXISTS glossary_term_objectives (
    glossary_term_id TEXT NOT NULL REFERENCES glossary_terms(id) ON DELETE CASCADE,
    learning_objective_id TEXT NOT NULL REFERENCES learning_objectives(id) ON DELETE CASCADE,
    relation TEXT NOT NULL CHECK(relation IN ('chapter_keyword', 'objective_title')),
    PRIMARY KEY(glossary_term_id, learning_objective_id, relation)
);

CREATE INDEX IF NOT EXISTS idx_term_links_target ON glossary_term_links(target_term_id);
CREATE INDEX IF NOT EXISTS idx_cluster_members_term ON confusion_cluster_members(glossary_term_id);
CREATE INDEX IF NOT EXISTS idx_term_objectives_objective ON glossary_term_objectives(learning_objective_id);
