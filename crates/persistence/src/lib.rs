use chrono::{Duration, Utc};
use domain::{
    AnswerConfidence, AttemptDiagnosis, AttemptResult, AttemptSubmission, Chapter,
    ConfusionCluster, ConfusionClusterMember, ContentOrigin, CourseDetail, CourseSummary,
    GlossaryLink, GlossaryTerm, KLevelProgress, LearningObjective, LearningStatus,
    ObjectiveProgress, OptionFeedback, ProgressOverview, Question, QuestionOption, QuestionType,
    ReasoningChoice, RetrievedContext, ReviewItemKind, ReviewState, SourceReference,
    TermAttemptResult, TermAttemptSubmission, TermCandidate, TermCard, TermDirection, TermExercise,
    TermExerciseInput, TermTopic, build_exercise, compress_for_exam, next_interval_days, next_item,
    plan, rotate_direction,
};
use serde::Deserialize;
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("Datenbankfehler: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Datenbankmigration fehlgeschlagen: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("Ungültige Seed-Daten: {0}")]
    Seed(#[from] serde_json::Error),
    #[error("Ressource nicht gefunden: {0}")]
    NotFound(String),
}

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, PersistenceError> {
        let options = sqlx::sqlite::SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        sqlx::migrate!("../../migrations").run(&pool).await?;
        let database = Self { pool };
        database.install_content().await?;
        Ok(database)
    }

    pub async fn in_memory() -> Result<Self, PersistenceError> {
        let options =
            sqlx::sqlite::SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        sqlx::migrate!("../../migrations").run(&pool).await?;
        let database = Self { pool };
        database.install_content().await?;
        Ok(database)
    }

    pub async fn install_content(&self) -> Result<(), PersistenceError> {
        let seed: Seed = serde_json::from_str(include_str!("../../../content/seed.json"))?;
        let installed_version: Option<String> =
            sqlx::query_scalar("SELECT corpus_version FROM content_versions WHERE id = 1")
                .fetch_optional(&self.pool)
                .await?;
        let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM courses")
            .fetch_one(&self.pool)
            .await?;
        if installed_version.as_deref() == Some(seed.corpus_version.as_str()) && existing > 0 {
            return Ok(());
        }

        let mut transaction = self.pool.begin().await?;

        for table in [
            "course_glossary_terms",
            "glossary_term_links",
            "confusion_cluster_members",
            "confusion_clusters",
            "glossary_term_objectives",
            "knowledge_fts",
        ] {
            sqlx::query(&format!("DELETE FROM {table}"))
                .execute(&mut *transaction)
                .await?;
        }

        for course in &seed.courses {
            sqlx::query(
                "INSERT INTO courses (id, code, name, version, description, exam_questions, passing_score, exam_minutes) VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET code = excluded.code, name = excluded.name, version = excluded.version, description = excluded.description, exam_questions = excluded.exam_questions, passing_score = excluded.passing_score, exam_minutes = excluded.exam_minutes",
            )
            .bind(&course.id)
            .bind(&course.code)
            .bind(&course.name)
            .bind(&course.version)
            .bind(&course.description)
            .bind(course.exam_questions)
            .bind(course.passing_score)
            .bind(course.exam_minutes)
            .execute(&mut *transaction)
            .await?;

            for chapter in &course.chapters {
                sqlx::query(
                    "INSERT INTO chapters (id, course_id, position, title, description) VALUES (?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET course_id = excluded.course_id, position = excluded.position, title = excluded.title, description = excluded.description",
                )
                .bind(&chapter.id)
                .bind(&course.id)
                .bind(chapter.position)
                .bind(&chapter.title)
                .bind(&chapter.description)
                .execute(&mut *transaction)
                .await?;

                for objective in &chapter.objectives {
                    sqlx::query(
                        "INSERT INTO learning_objectives (id, course_id, chapter_id, code, k_level, title, summary, source_label, source_url, source_section) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET course_id = excluded.course_id, chapter_id = excluded.chapter_id, code = excluded.code, k_level = excluded.k_level, title = excluded.title, summary = excluded.summary, source_label = excluded.source_label, source_url = excluded.source_url, source_section = excluded.source_section",
                    )
                    .bind(&objective.id)
                    .bind(&course.id)
                    .bind(&chapter.id)
                    .bind(&objective.code)
                    .bind(&objective.k_level)
                    .bind(&objective.title)
                    .bind(&objective.summary)
                    .bind(&objective.source_label)
                    .bind(&objective.source_url)
                    .bind(&objective.source_section)
                    .execute(&mut *transaction)
                    .await?;

                    sqlx::query(
                        "INSERT INTO knowledge_fts (id, course_id, title, text, source_label, source_url, source_section, origin) VALUES (?, ?, ?, ?, ?, ?, ?, 'official_excerpt')",
                    )
                    .bind(&objective.id)
                    .bind(&course.id)
                    .bind(&objective.title)
                    .bind(&objective.summary)
                    .bind(&objective.source_label)
                    .bind(&objective.source_url)
                    .bind(&objective.source_section)
                    .execute(&mut *transaction)
                    .await?;
                }
            }
        }

        for term in &seed.glossary {
            sqlx::query(
                "INSERT INTO glossary_terms (id, term, definition, language, snapshot, origin, source_label, source_url, term_version, reference) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET term = excluded.term, definition = excluded.definition, language = excluded.language, snapshot = excluded.snapshot, origin = excluded.origin, source_label = excluded.source_label, source_url = excluded.source_url, term_version = excluded.term_version, reference = excluded.reference",
            )
            .bind(&term.id)
            .bind(&term.term)
            .bind(&term.definition)
            .bind(&term.language)
            .bind(&term.snapshot)
            .bind(&term.origin)
            .bind(&term.source_label)
            .bind(&term.source_url)
            .bind(&term.term_version)
            .bind(&term.reference)
            .execute(&mut *transaction)
            .await?;
        }

        let existing_course_ids: HashSet<String> = sqlx::query_scalar("SELECT id FROM courses")
            .fetch_all(&mut *transaction)
            .await?
            .into_iter()
            .collect();
        let existing_term_ids: HashSet<String> =
            sqlx::query_scalar("SELECT id FROM glossary_terms")
                .fetch_all(&mut *transaction)
                .await?
                .into_iter()
                .collect();
        let existing_objective_ids: HashSet<String> =
            sqlx::query_scalar("SELECT id FROM learning_objectives")
                .fetch_all(&mut *transaction)
                .await?
                .into_iter()
                .collect();

        for term in &seed.glossary {
            for course_id in &term.courses {
                if !existing_course_ids.contains(course_id) {
                    continue;
                }
                sqlx::query(
                    "INSERT INTO course_glossary_terms (course_id, glossary_term_id) VALUES (?, ?)",
                )
                .bind(course_id)
                .bind(&term.id)
                .execute(&mut *transaction)
                .await?;

                sqlx::query(
                    "INSERT INTO knowledge_fts (id, course_id, title, text, source_label, source_url, source_section, origin) VALUES (?, ?, ?, ?, ?, ?, NULL, ?)",
                )
                .bind(format!("glossary:{}:{course_id}", term.id))
                .bind(course_id)
                .bind(&term.term)
                .bind(&term.definition)
                .bind(&term.source_label)
                .bind(&term.source_url)
                .bind(&term.origin)
                .execute(&mut *transaction)
                .await?;
            }

            for value in &term.synonyms {
                sqlx::query(
                    "INSERT INTO glossary_term_links (glossary_term_id, kind, value, target_term_id) VALUES (?, 'synonym', ?, NULL) ON CONFLICT(glossary_term_id, kind, value) DO UPDATE SET target_term_id = excluded.target_term_id",
                )
                .bind(&term.id)
                .bind(value)
                .execute(&mut *transaction)
                .await?;
            }
            for value in &term.abbreviations {
                sqlx::query(
                    "INSERT INTO glossary_term_links (glossary_term_id, kind, value, target_term_id) VALUES (?, 'abbreviation', ?, NULL) ON CONFLICT(glossary_term_id, kind, value) DO UPDATE SET target_term_id = excluded.target_term_id",
                )
                .bind(&term.id)
                .bind(value)
                .execute(&mut *transaction)
                .await?;
            }
            for link in &term.see_also {
                if link
                    .term_id
                    .as_ref()
                    .is_some_and(|target| !existing_term_ids.contains(target))
                {
                    continue;
                }
                sqlx::query(
                    "INSERT INTO glossary_term_links (glossary_term_id, kind, value, target_term_id) VALUES (?, 'see_also', ?, ?) ON CONFLICT(glossary_term_id, kind, value) DO UPDATE SET target_term_id = excluded.target_term_id",
                )
                .bind(&term.id)
                .bind(&link.value)
                .bind(&link.term_id)
                .execute(&mut *transaction)
                .await?;
            }
            for objective in &term.objectives {
                if !existing_objective_ids.contains(&objective.learning_objective_id) {
                    continue;
                }
                sqlx::query(
                    "INSERT INTO glossary_term_objectives (glossary_term_id, learning_objective_id, relation) VALUES (?, ?, ?)",
                )
                .bind(&term.id)
                .bind(&objective.learning_objective_id)
                .bind(&objective.relation)
                .execute(&mut *transaction)
                .await?;
            }
        }

        for cluster in &seed.confusion_clusters {
            sqlx::query(
                "INSERT INTO confusion_clusters (id, course_id, title, note, origin) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&cluster.id)
            .bind(&cluster.course_id)
            .bind(&cluster.title)
            .bind(&cluster.note)
            .bind(&cluster.origin)
            .execute(&mut *transaction)
            .await?;

            for member in &cluster.members {
                if !existing_term_ids.contains(&member.glossary_term_id) {
                    continue;
                }
                sqlx::query(
                    "INSERT INTO confusion_cluster_members (cluster_id, glossary_term_id, position, distinction) VALUES (?, ?, ?, ?)",
                )
                .bind(&cluster.id)
                .bind(&member.glossary_term_id)
                .bind(member.position)
                .bind(&member.distinction)
                .execute(&mut *transaction)
                .await?;
            }
        }

        for question in &seed.questions {
            sqlx::query(
                "INSERT INTO questions (id, course_id, learning_objective_id, prompt, question_type, origin, points, explanation, source_label, source_url, source_section) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET course_id = excluded.course_id, learning_objective_id = excluded.learning_objective_id, prompt = excluded.prompt, question_type = excluded.question_type, origin = excluded.origin, points = excluded.points, explanation = excluded.explanation, source_label = excluded.source_label, source_url = excluded.source_url, source_section = excluded.source_section, review_status = excluded.review_status, difficulty = excluded.difficulty",
            )
            .bind(&question.id)
            .bind(&question.course_id)
            .bind(&question.learning_objective_id)
            .bind(&question.prompt)
            .bind(&question.question_type)
            .bind(&question.origin)
            .bind(question.points)
            .bind(&question.explanation)
            .bind(&question.source_label)
            .bind(&question.source_url)
            .bind(&question.source_section)
            .execute(&mut *transaction)
            .await?;

            for option in &question.options {
                sqlx::query(
                    "INSERT INTO question_options (id, question_id, position, text, is_correct, feedback, misconception) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET question_id = excluded.question_id, position = excluded.position, text = excluded.text, is_correct = excluded.is_correct, feedback = excluded.feedback, misconception = excluded.misconception",
                )
                .bind(&option.id)
                .bind(&question.id)
                .bind(option.position)
                .bind(&option.text)
                .bind(option.is_correct)
                .bind(&option.feedback)
                .bind(&option.misconception)
                .execute(&mut *transaction)
                .await?;
            }
        }

        sqlx::query(
            "INSERT INTO content_versions (id, corpus_version, installed_at) VALUES (1, ?, ?) ON CONFLICT(id) DO UPDATE SET corpus_version = excluded.corpus_version, installed_at = excluded.installed_at",
        )
        .bind(&seed.corpus_version)
        .bind(Utc::now().to_rfc3339())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn counts(&self) -> Result<(i64, i64), PersistenceError> {
        let courses = sqlx::query_scalar("SELECT COUNT(*) FROM courses")
            .fetch_one(&self.pool)
            .await?;
        let terms = sqlx::query_scalar("SELECT COUNT(*) FROM glossary_terms")
            .fetch_one(&self.pool)
            .await?;
        Ok((courses, terms))
    }

    pub async fn list_courses(&self) -> Result<Vec<CourseSummary>, PersistenceError> {
        let rows = sqlx::query(
            "SELECT c.*, COUNT(DISTINCT lo.id) AS objective_count FROM courses c LEFT JOIN learning_objectives lo ON lo.course_id = c.id GROUP BY c.id ORDER BY CASE c.code WHEN 'CTFL' THEN 0 WHEN 'CTAL-TM' THEN 1 WHEN 'CT-AI' THEN 2 ELSE 3 END, c.code",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut courses = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.get("id");
            let progress = self.progress(&id).await?;
            courses.push(CourseSummary {
                id,
                code: row.get("code"),
                name: row.get("name"),
                version: row.get("version"),
                description: row.get("description"),
                exam_questions: row.get("exam_questions"),
                passing_score: row.get("passing_score"),
                exam_minutes: row.get("exam_minutes"),
                objective_count: row.get("objective_count"),
                progress_percent: progress.percent,
                readiness_label: progress.readiness_label,
                due_reviews: progress.due_reviews,
                confidence_issue_count: progress.confident_wrong_attempts,
            });
        }
        Ok(courses)
    }

    pub async fn course(&self, course_id: &str) -> Result<CourseDetail, PersistenceError> {
        let summary = self
            .list_courses()
            .await?
            .into_iter()
            .find(|course| course.id == course_id)
            .ok_or_else(|| PersistenceError::NotFound(format!("Kurs {course_id}")))?;

        let chapter_rows = sqlx::query(
            "SELECT id, position, title, description FROM chapters WHERE course_id = ? ORDER BY position",
        )
        .bind(course_id)
        .fetch_all(&self.pool)
        .await?;

        let progress = self.progress(course_id).await?;
        let mut chapters = Vec::with_capacity(chapter_rows.len());
        for chapter_row in chapter_rows {
            let chapter_id: String = chapter_row.get("id");
            let objective_rows =
                sqlx::query("SELECT * FROM learning_objectives WHERE chapter_id = ? ORDER BY code")
                    .bind(&chapter_id)
                    .fetch_all(&self.pool)
                    .await?;
            let objectives = objective_rows
                .into_iter()
                .map(|row| {
                    let objective_id: String = row.get("id");
                    let status = progress
                        .objectives
                        .iter()
                        .find(|item| item.objective_id == objective_id)
                        .map(|item| item.status.clone())
                        .unwrap_or(LearningStatus::NotStarted);
                    LearningObjective {
                        id: objective_id,
                        code: row.get("code"),
                        k_level: row.get("k_level"),
                        title: row.get("title"),
                        summary: row.get("summary"),
                        status,
                        source: SourceReference {
                            label: row.get("source_label"),
                            url: row.get("source_url"),
                            section: row.get("source_section"),
                            origin: ContentOrigin::OfficialExcerpt,
                        },
                    }
                })
                .collect();
            chapters.push(Chapter {
                id: chapter_id,
                position: chapter_row.get("position"),
                title: chapter_row.get("title"),
                description: chapter_row.get("description"),
                objectives,
            });
        }

        Ok(CourseDetail { summary, chapters })
    }

    pub async fn glossary(
        &self,
        course_id: Option<&str>,
        query: Option<&str>,
    ) -> Result<Vec<GlossaryTerm>, PersistenceError> {
        let query_pattern = format!("%{}%", query.unwrap_or_default());
        let rows = sqlx::query(
            "SELECT DISTINCT g.* FROM glossary_terms g LEFT JOIN course_glossary_terms cg ON cg.glossary_term_id = g.id WHERE (?1 IS NULL OR cg.course_id = ?1) AND (?2 = '%%' OR g.term LIKE ?2 OR g.definition LIKE ?2) ORDER BY g.term LIMIT 100",
        )
        .bind(course_id)
        .bind(&query_pattern)
        .fetch_all(&self.pool)
        .await?;

        let mut links = self
            .term_links(
                &rows
                    .iter()
                    .map(|row| row.get("id"))
                    .collect::<Vec<String>>(),
            )
            .await?;
        let mut terms = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.get("id");
            let course_rows = sqlx::query(
                "SELECT course_id FROM course_glossary_terms WHERE glossary_term_id = ? ORDER BY course_id",
            )
            .bind(&id)
            .fetch_all(&self.pool)
            .await?;
            let term_links = links.remove(&id).unwrap_or_default();
            terms.push(GlossaryTerm {
                id,
                term: row.get("term"),
                definition: row.get("definition"),
                language: row.get("language"),
                snapshot: row.get("snapshot"),
                origin: parse_origin(row.get::<String, _>("origin").as_str()),
                term_version: row.get("term_version"),
                reference: row.get("reference"),
                synonyms: term_links.synonyms,
                abbreviations: term_links.abbreviations,
                see_also: term_links.see_also,
                courses: course_rows
                    .into_iter()
                    .map(|row| row.get("course_id"))
                    .collect(),
                source: SourceReference {
                    label: row.get("source_label"),
                    url: row.get("source_url"),
                    section: None,
                    origin: parse_origin(row.get::<String, _>("origin").as_str()),
                },
            });
        }
        Ok(terms)
    }

    /// Synonyme, Abkürzungen und See-also-Beziehungen für mehrere Begriffe in
    /// einer Abfrage. Die Platzhalterliste wird aus der Anzahl der IDs gebaut,
    /// die Werte bleiben gebunden.
    async fn term_links(
        &self,
        term_ids: &[String],
    ) -> Result<HashMap<String, TermLinks>, PersistenceError> {
        if term_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders = vec!["?"; term_ids.len()].join(", ");
        let statement = format!(
            "SELECT glossary_term_id, kind, value, target_term_id FROM glossary_term_links WHERE glossary_term_id IN ({placeholders}) ORDER BY glossary_term_id, kind, value"
        );
        let mut query = sqlx::query(&statement);
        for term_id in term_ids {
            query = query.bind(term_id);
        }

        let mut links: HashMap<String, TermLinks> = HashMap::new();
        for row in query.fetch_all(&self.pool).await? {
            let entry = links.entry(row.get("glossary_term_id")).or_default();
            let value: String = row.get("value");
            match row.get::<String, _>("kind").as_str() {
                "synonym" => entry.synonyms.push(value),
                "abbreviation" => entry.abbreviations.push(value),
                _ => entry.see_also.push(GlossaryLink {
                    value,
                    term_id: row.get("target_term_id"),
                }),
            }
        }
        Ok(links)
    }

    pub async fn confusion_clusters(
        &self,
        course_id: &str,
    ) -> Result<Vec<ConfusionCluster>, PersistenceError> {
        let cluster_rows =
            sqlx::query("SELECT * FROM confusion_clusters WHERE course_id = ? ORDER BY id")
                .bind(course_id)
                .fetch_all(&self.pool)
                .await?;

        let mut clusters = Vec::with_capacity(cluster_rows.len());
        for row in cluster_rows {
            let id: String = row.get("id");
            let member_rows = sqlx::query(
                "SELECT m.glossary_term_id, m.position, m.distinction, g.term, g.definition FROM confusion_cluster_members m JOIN glossary_terms g ON g.id = m.glossary_term_id WHERE m.cluster_id = ? ORDER BY m.position",
            )
            .bind(&id)
            .fetch_all(&self.pool)
            .await?;

            clusters.push(ConfusionCluster {
                id,
                course_id: row.get("course_id"),
                title: row.get("title"),
                note: row.get("note"),
                origin: parse_origin(row.get::<String, _>("origin").as_str()),
                members: member_rows
                    .into_iter()
                    .map(|row| ConfusionClusterMember {
                        term_id: row.get("glossary_term_id"),
                        term: row.get("term"),
                        definition: row.get("definition"),
                        position: row.get("position"),
                        distinction: row.get("distinction"),
                    })
                    .collect(),
            });
        }
        Ok(clusters)
    }

    async fn term_exercise_input(
        &self,
        term_id: &str,
    ) -> Result<TermExerciseInput, PersistenceError> {
        let term = sqlx::query("SELECT id, term, definition FROM glossary_terms WHERE id = ?")
            .bind(term_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| PersistenceError::NotFound(format!("Begriff {term_id}")))?;
        let course_id: String = sqlx::query_scalar(
            "SELECT course_id FROM course_glossary_terms WHERE glossary_term_id = ? ORDER BY course_id LIMIT 1",
        )
        .bind(term_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::NotFound(format!("Begriff {term_id}")))?;

        let topic_row = sqlx::query(
            "SELECT DISTINCT c.id, c.title, c.position FROM glossary_term_objectives gto JOIN learning_objectives lo ON lo.id = gto.learning_objective_id JOIN chapters c ON c.id = lo.chapter_id WHERE gto.glossary_term_id = ? AND gto.relation = 'chapter_keyword' AND c.course_id = ? ORDER BY c.position, c.id LIMIT 1",
        )
        .bind(term_id)
        .bind(&course_id)
        .fetch_optional(&self.pool)
        .await?;
        let topic = topic_row.map(|row| TermTopic {
            id: row.get("id"),
            title: row.get("title"),
        });

        let cluster_row = sqlx::query(
            "SELECT cc.id, cc.title, ccm.distinction FROM confusion_cluster_members ccm JOIN confusion_clusters cc ON cc.id = ccm.cluster_id WHERE ccm.glossary_term_id = ? AND cc.course_id = ? ORDER BY cc.id LIMIT 1",
        )
        .bind(term_id)
        .bind(&course_id)
        .fetch_optional(&self.pool)
        .await?;
        let (cluster_id, cluster_title, distinction) = if let Some(row) = cluster_row {
            (
                Some(row.get::<String, _>("id")),
                Some(row.get::<String, _>("title")),
                Some(row.get::<String, _>("distinction")),
            )
        } else {
            (None, None, None)
        };

        let cluster_mates = if let Some(cluster_id) = cluster_id.as_deref() {
            sqlx::query(
                "SELECT DISTINCT g.id, g.term, g.definition FROM confusion_cluster_members ccm JOIN glossary_terms g ON g.id = ccm.glossary_term_id WHERE ccm.cluster_id = ? AND g.id != ? ORDER BY g.id",
            )
            .bind(cluster_id)
            .bind(term_id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| TermCandidate {
                id: row.get("id"),
                term: row.get("term"),
                definition: row.get("definition"),
            })
            .collect()
        } else {
            Vec::new()
        };

        let see_also = sqlx::query(
            "SELECT DISTINCT g.id, g.term, g.definition FROM glossary_term_links gtl JOIN glossary_terms g ON g.id = gtl.target_term_id WHERE gtl.glossary_term_id = ? AND gtl.kind = 'see_also' AND gtl.target_term_id IS NOT NULL ORDER BY g.id",
        )
        .bind(term_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| TermCandidate {
            id: row.get("id"),
            term: row.get("term"),
            definition: row.get("definition"),
        })
        .collect();

        let chapter_mates = if let Some(topic) = topic.as_ref() {
            sqlx::query(
                "SELECT DISTINCT g.id, g.term, g.definition FROM glossary_terms g JOIN course_glossary_terms cgt ON cgt.glossary_term_id = g.id JOIN glossary_term_objectives gto ON gto.glossary_term_id = g.id AND gto.relation = 'chapter_keyword' JOIN learning_objectives lo ON lo.id = gto.learning_objective_id WHERE cgt.course_id = ? AND lo.chapter_id = ? AND g.id != ? ORDER BY g.id LIMIT 20",
            )
            .bind(&course_id)
            .bind(&topic.id)
            .bind(term_id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| TermCandidate {
                id: row.get("id"),
                term: row.get("term"),
                definition: row.get("definition"),
            })
            .collect()
        } else {
            Vec::new()
        };

        let topics =
            sqlx::query("SELECT id, title FROM chapters WHERE course_id = ? ORDER BY position, id")
                .bind(&course_id)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| TermTopic {
                    id: row.get("id"),
                    title: row.get("title"),
                })
                .collect();
        let attempt_count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM term_attempts WHERE glossary_term_id = ? AND profile_id = 'local-default'",
        )
        .bind(term_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TermExerciseInput {
            card: TermCard {
                term_id: term.get("id"),
                term: term.get("term"),
                definition: term.get("definition"),
                topic,
                cluster_title,
                distinction,
            },
            cluster_mates,
            see_also,
            chapter_mates,
            topics,
            attempt_count,
        })
    }

    pub async fn next_term_exercise(
        &self,
        course_id: &str,
        direction: Option<TermDirection>,
    ) -> Result<TermExercise, PersistenceError> {
        let now = Utc::now().to_rfc3339();
        let rows = sqlx::query(
            "SELECT g.id AS item_id, COUNT(ta.id) AS attempts, (SELECT latest.next_review_at FROM term_attempts latest WHERE latest.glossary_term_id = g.id AND latest.profile_id = 'local-default' ORDER BY latest.created_at DESC, latest.id DESC LIMIT 1) AS due_at FROM glossary_terms g JOIN course_glossary_terms cgt ON cgt.glossary_term_id = g.id LEFT JOIN term_attempts ta ON ta.glossary_term_id = g.id AND ta.profile_id = 'local-default' WHERE cgt.course_id = ? GROUP BY g.id",
        )
        .bind(course_id)
        .fetch_all(&self.pool)
        .await?;
        let states = rows
            .into_iter()
            .map(|row| ReviewState {
                item_id: row.get("item_id"),
                kind: ReviewItemKind::Term,
                attempts: row.get("attempts"),
                due_at: row.get("due_at"),
            })
            .collect();
        let review_plan = plan(states, &now);
        let last_item_id: Option<String> = sqlx::query_scalar(
            "SELECT ta.glossary_term_id FROM term_attempts ta JOIN course_glossary_terms cgt ON cgt.glossary_term_id = ta.glossary_term_id WHERE ta.profile_id = 'local-default' AND cgt.course_id = ? ORDER BY ta.created_at DESC, ta.id DESC LIMIT 1",
        )
        .bind(course_id)
        .fetch_optional(&self.pool)
        .await?;
        let first_item = next_item(&review_plan, last_item_id.as_deref())
            .ok_or_else(|| PersistenceError::NotFound("Keine passende Begriffsübung".into()))?;
        let first_index = review_plan
            .iter()
            .position(|item| item.item_id == first_item.item_id)
            .expect("Ein Planeintrag muss in seinem Plan enthalten sein");

        for item in review_plan
            .iter()
            .skip(first_index)
            .chain(review_plan.iter().take(first_index))
            .take(25)
        {
            let input = self.term_exercise_input(&item.item_id).await?;
            let selected_direction = direction.or_else(|| rotate_direction(&input));
            if let Some(exercise) = selected_direction
                .and_then(|selected_direction| build_exercise(&input, selected_direction))
            {
                return Ok(exercise);
            }
        }

        Err(PersistenceError::NotFound(
            "Keine passende Begriffsübung".into(),
        ))
    }

    pub async fn submit_term_attempt(
        &self,
        term_id: &str,
        submission: TermAttemptSubmission,
    ) -> Result<TermAttemptResult, PersistenceError> {
        let input = self.term_exercise_input(term_id).await?;
        let exercise = build_exercise(&input, submission.direction)
            .ok_or_else(|| PersistenceError::NotFound("Keine passende Begriffsübung".into()))?;
        let correct = submission.selected_option_id == exercise.correct_option_id;
        let confidence = submission.confidence.clone();
        let diagnosis =
            AttemptDiagnosis::evaluate(correct, &confidence, ReasoningChoice::NotStated);
        let previous_attempts = sqlx::query(
            "SELECT is_correct, confidence FROM term_attempts WHERE glossary_term_id = ? AND profile_id = 'local-default' ORDER BY created_at DESC, id DESC LIMIT 10",
        )
        .bind(term_id)
        .fetch_all(&self.pool)
        .await?;
        let mastery_streak = previous_attempts
            .into_iter()
            .take_while(|row| {
                let confidence = match row.get::<String, _>("confidence").as_str() {
                    "sure" => AnswerConfidence::Sure,
                    "guessed" => AnswerConfidence::Guessed,
                    _ => AnswerConfidence::Unsure,
                };
                AttemptDiagnosis::evaluate(
                    row.get::<i64, _>("is_correct") == 1,
                    &confidence,
                    ReasoningChoice::NotStated,
                )
                .counts_as_mastery
            })
            .count() as i64;
        let review_interval_days =
            compress_for_exam(next_interval_days(mastery_streak, &diagnosis), None);
        let now = Utc::now();
        let review_due_at = (now + Duration::days(review_interval_days)).to_rfc3339();
        let attempt_id = Uuid::new_v4().to_string();
        let source_row =
            sqlx::query("SELECT source_label, source_url, origin FROM glossary_terms WHERE id = ?")
                .bind(term_id)
                .fetch_one(&self.pool)
                .await?;

        sqlx::query(
            "INSERT INTO term_attempts (id, profile_id, glossary_term_id, direction, selected_option_id, is_correct, confidence, next_review_at, created_at) VALUES (?, 'local-default', ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&attempt_id)
        .bind(term_id)
        .bind(term_direction_value(submission.direction))
        .bind(&submission.selected_option_id)
        .bind(correct)
        .bind(confidence_value(&confidence))
        .bind(&review_due_at)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        let distinction = if !correct || input.card.cluster_title.is_some() {
            input.card.distinction.clone()
        } else {
            None
        };

        Ok(TermAttemptResult {
            attempt_id,
            correct,
            correct_option_id: exercise.correct_option_id,
            term: input.card.term,
            definition: input.card.definition,
            outcome: diagnosis.outcome,
            counts_as_mastery: diagnosis.counts_as_mastery,
            confidence,
            diagnosis: diagnosis.message,
            distinction,
            cluster_title: input.card.cluster_title,
            review_due_at,
            tutor_recommended: diagnosis.tutor_recommended,
            source: SourceReference {
                label: source_row.get("source_label"),
                url: source_row.get("source_url"),
                section: None,
                origin: parse_origin(source_row.get::<String, _>("origin").as_str()),
            },
        })
    }

    pub async fn next_question(
        &self,
        course_id: &str,
        objective_id: Option<&str>,
    ) -> Result<Question, PersistenceError> {
        let now = Utc::now().to_rfc3339();
        let rows = sqlx::query(
            "SELECT q.id AS item_id, COUNT(a.id) AS attempts, (SELECT latest.next_review_at FROM attempts latest WHERE latest.question_id = q.id AND latest.profile_id = 'local-default' ORDER BY latest.created_at DESC, latest.id DESC LIMIT 1) AS due_at FROM questions q LEFT JOIN attempts a ON a.question_id = q.id AND a.profile_id = 'local-default' WHERE q.course_id = ? AND q.review_status = 'approved' AND (? IS NULL OR q.learning_objective_id = ?) GROUP BY q.id",
        )
        .bind(course_id)
        .bind(objective_id)
        .bind(objective_id)
        .fetch_all(&self.pool)
        .await?;
        let states = rows
            .into_iter()
            .map(|row| ReviewState {
                item_id: row.get("item_id"),
                kind: ReviewItemKind::LearningObjective,
                attempts: row.get("attempts"),
                due_at: row.get("due_at"),
            })
            .collect();
        let review_plan = plan(states, &now);
        let last_item_id: Option<String> = sqlx::query_scalar(
            "SELECT a.question_id FROM attempts a JOIN questions q ON q.id = a.question_id WHERE a.profile_id = 'local-default' AND q.course_id = ? ORDER BY a.created_at DESC, a.id DESC LIMIT 1",
        )
        .bind(course_id)
        .fetch_optional(&self.pool)
        .await?;
        let selected_item = next_item(&review_plan, last_item_id.as_deref())
            .ok_or_else(|| PersistenceError::NotFound("Keine passende Frage".into()))?;
        let row = sqlx::query("SELECT * FROM questions WHERE id = ?")
            .bind(&selected_item.item_id)
            .fetch_one(&self.pool)
            .await?;

        let question_id: String = row.get("id");
        let option_rows = sqlx::query(
            "SELECT id, position, text FROM question_options WHERE question_id = ? ORDER BY position",
        )
        .bind(&question_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Question {
            id: question_id,
            course_id: row.get("course_id"),
            learning_objective_id: row.get("learning_objective_id"),
            prompt: row.get("prompt"),
            question_type: match row.get::<String, _>("question_type").as_str() {
                "multiple_choice" => QuestionType::MultipleChoice,
                _ => QuestionType::SingleChoice,
            },
            origin: parse_origin(row.get::<String, _>("origin").as_str()),
            points: row.get("points"),
            review_status: row.get("review_status"),
            difficulty: row.get("difficulty"),
            options: option_rows
                .into_iter()
                .map(|row| QuestionOption {
                    id: row.get("id"),
                    position: row.get("position"),
                    text: row.get("text"),
                })
                .collect(),
            source: SourceReference {
                label: row.get("source_label"),
                url: row.get("source_url"),
                section: row.get("source_section"),
                origin: parse_origin(row.get::<String, _>("origin").as_str()),
            },
        })
    }

    pub async fn submit_attempt(
        &self,
        question_id: &str,
        submission: AttemptSubmission,
    ) -> Result<AttemptResult, PersistenceError> {
        let question = sqlx::query("SELECT * FROM questions WHERE id = ?")
            .bind(question_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| PersistenceError::NotFound(format!("Frage {question_id}")))?;
        let option_rows = sqlx::query(
            "SELECT id, text, is_correct, feedback, misconception FROM question_options WHERE question_id = ? ORDER BY position",
        )
        .bind(question_id)
        .fetch_all(&self.pool)
        .await?;

        let selected: HashSet<&str> = submission
            .selected_option_ids
            .iter()
            .map(String::as_str)
            .collect();
        let expected: HashSet<String> = option_rows
            .iter()
            .filter(|row| row.get::<i64, _>("is_correct") == 1)
            .map(|row| row.get("id"))
            .collect();
        let correct =
            selected.len() == expected.len() && selected.iter().all(|id| expected.contains(*id));
        let objective_id: String = question.get("learning_objective_id");
        let attempt_id = Uuid::new_v4().to_string();
        let confidence = submission.confidence.clone();
        let confidence_value = confidence_value(&confidence);
        let reasoning_choice_value = reasoning_choice_value(&submission.reasoning_choice);
        let now = Utc::now();
        let diagnosis =
            AttemptDiagnosis::evaluate(correct, &confidence, submission.reasoning_choice);
        let previous_attempts = sqlx::query(
            "SELECT a.is_correct, a.confidence, a.reasoning_choice FROM attempts a JOIN questions q ON q.id = a.question_id WHERE q.learning_objective_id = ? AND a.profile_id = 'local-default' ORDER BY a.created_at DESC, a.id DESC LIMIT 10",
        )
        .bind(&objective_id)
        .fetch_all(&self.pool)
        .await?;
        let mastery_streak = previous_attempts
            .into_iter()
            .take_while(|row| {
                let confidence = match row.get::<String, _>("confidence").as_str() {
                    "sure" => AnswerConfidence::Sure,
                    "guessed" => AnswerConfidence::Guessed,
                    _ => AnswerConfidence::Unsure,
                };
                let reasoning_choice = match row.get::<String, _>("reasoning_choice").as_str() {
                    "recalled" => ReasoningChoice::Recalled,
                    "eliminated" => ReasoningChoice::Eliminated,
                    "applied_rule" => ReasoningChoice::AppliedRule,
                    "from_experience" => ReasoningChoice::FromExperience,
                    "guessed" => ReasoningChoice::Guessed,
                    _ => ReasoningChoice::NotStated,
                };
                AttemptDiagnosis::evaluate(
                    row.get::<i64, _>("is_correct") == 1,
                    &confidence,
                    reasoning_choice,
                )
                .counts_as_mastery
            })
            .count() as i64;
        let review_interval_days =
            compress_for_exam(next_interval_days(mastery_streak, &diagnosis), None);
        let review_due_at = (now + Duration::days(review_interval_days)).to_rfc3339();
        let misconception = option_rows
            .iter()
            .find(|row| {
                let id: String = row.get("id");
                selected.contains(id.as_str()) && row.get::<i64, _>("is_correct") == 0
            })
            .and_then(|row| {
                row.get::<Option<String>, _>("misconception")
                    .or_else(|| Some(row.get("feedback")))
            });
        sqlx::query(
            "INSERT INTO attempts (id, profile_id, question_id, selected_option_ids, reasoning, reasoning_choice, is_correct, confidence, next_review_at, created_at) VALUES (?, 'local-default', ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&attempt_id)
        .bind(question_id)
        .bind(serde_json::to_string(&submission.selected_option_ids).unwrap_or_default())
        .bind(&submission.reasoning)
        .bind(reasoning_choice_value)
        .bind(correct)
        .bind(confidence_value)
        .bind(&review_due_at)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        let course_id: String = question.get("course_id");
        let learning_status = self
            .progress(&course_id)
            .await?
            .objectives
            .into_iter()
            .find(|objective| objective.objective_id == objective_id)
            .map(|objective| objective.status)
            .unwrap_or(LearningStatus::Introduced);
        let origin = parse_origin(question.get::<String, _>("origin").as_str());

        Ok(AttemptResult {
            attempt_id,
            correct,
            outcome: diagnosis.outcome,
            counts_as_mastery: diagnosis.counts_as_mastery,
            earned_points: if correct { question.get("points") } else { 0 },
            available_points: question.get("points"),
            explanation: question.get("explanation"),
            option_feedback: option_rows
                .into_iter()
                .map(|row| {
                    let id: String = row.get("id");
                    let option_correct = row.get::<i64, _>("is_correct") == 1;
                    let feedback: String = row.get("feedback");
                    let misconception = row
                        .get::<Option<String>, _>("misconception")
                        .or_else(|| (!option_correct).then(|| feedback.clone()));
                    OptionFeedback {
                        selected: selected.contains(id.as_str()),
                        correct: option_correct,
                        option_id: id,
                        feedback,
                        misconception,
                    }
                })
                .collect(),
            learning_status,
            confidence,
            diagnosis: diagnosis.message,
            misconception,
            review_due_at,
            tutor_recommended: diagnosis.tutor_recommended,
            source: SourceReference {
                label: question.get("source_label"),
                url: question.get("source_url"),
                section: question.get("source_section"),
                origin,
            },
        })
    }

    pub async fn progress(&self, course_id: &str) -> Result<ProgressOverview, PersistenceError> {
        let now = Utc::now().to_rfc3339();
        let rows = sqlx::query(
            "SELECT lo.id, lo.k_level, lo.title, COUNT(a.id) AS total_attempts, COALESCE(SUM(a.is_correct), 0) AS correct_attempts, COALESCE(SUM(CASE WHEN a.is_correct = 1 AND a.confidence = 'sure' THEN 1 ELSE 0 END), 0) AS confident_correct_attempts, COALESCE(SUM(CASE WHEN a.is_correct = 0 AND a.confidence = 'sure' THEN 1 ELSE 0 END), 0) AS confident_wrong_attempts, MAX(a.created_at) AS last_attempt_at, COALESCE(MAX(CASE WHEN a.id = (SELECT a2.id FROM attempts a2 WHERE a2.question_id = q.id AND a2.profile_id = 'local-default' ORDER BY a2.created_at DESC, a2.id DESC LIMIT 1) AND a.next_review_at <= ? THEN 1 ELSE 0 END), 0) AS due_for_review FROM learning_objectives lo LEFT JOIN questions q ON q.learning_objective_id = lo.id LEFT JOIN attempts a ON a.question_id = q.id AND a.profile_id = 'local-default' WHERE lo.course_id = ? GROUP BY lo.id ORDER BY lo.code",
        )
        .bind(&now)
        .bind(course_id)
        .fetch_all(&self.pool)
        .await?;

        let objectives: Vec<ObjectiveProgress> = rows
            .into_iter()
            .map(|row| {
                let total = row.get("total_attempts");
                let correct = row.get("correct_attempts");
                let confident_correct = row.get("confident_correct_attempts");
                let confident_wrong = row.get("confident_wrong_attempts");
                ObjectiveProgress {
                    objective_id: row.get("id"),
                    k_level: row.get("k_level"),
                    title: row.get("title"),
                    correct_attempts: correct,
                    total_attempts: total,
                    confident_correct_attempts: confident_correct,
                    confident_wrong_attempts: confident_wrong,
                    due_for_review: row.get::<i64, _>("due_for_review") > 0,
                    last_attempt_at: row.get("last_attempt_at"),
                    status: LearningStatus::from_evidence(
                        total,
                        correct,
                        confident_correct,
                        confident_wrong,
                    ),
                }
            })
            .collect();
        let total_attempts = objectives.iter().map(|item| item.total_attempts).sum();
        let correct_attempts = objectives.iter().map(|item| item.correct_attempts).sum();
        let confident_wrong_attempts = objectives
            .iter()
            .map(|item| item.confident_wrong_attempts)
            .sum();
        let confident_correct_attempts: i64 = objectives
            .iter()
            .map(|item| item.confident_correct_attempts)
            .sum();
        let due_reviews = objectives.iter().filter(|item| item.due_for_review).count() as i64;
        let covered_objectives = objectives
            .iter()
            .filter(|item| item.total_attempts > 0)
            .count() as i64;
        let percent = if objectives.is_empty() {
            0
        } else {
            objectives
                .iter()
                .map(|item| status_percent(&item.status))
                .sum::<i64>()
                / objectives.len() as i64
        };
        let non_confident_wrong = total_attempts - correct_attempts - confident_wrong_attempts;
        let confidence_alignment_percent = if total_attempts == 0 {
            0
        } else {
            (confident_correct_attempts + non_confident_wrong) * 100 / total_attempts
        };
        let readiness_label = readiness_label(percent).to_owned();
        let mut k_levels: Vec<KLevelProgress> = Vec::new();
        for objective in &objectives {
            if let Some(level) = k_levels
                .iter_mut()
                .find(|level| level.k_level == objective.k_level)
            {
                level.objective_count += 1;
                level.total_attempts += objective.total_attempts;
                level.correct_attempts += objective.correct_attempts;
                level.confident_correct_attempts += objective.confident_correct_attempts;
                level.readiness_percent += status_percent(&objective.status);
            } else {
                k_levels.push(KLevelProgress {
                    k_level: objective.k_level.clone(),
                    objective_count: 1,
                    total_attempts: objective.total_attempts,
                    correct_attempts: objective.correct_attempts,
                    confident_correct_attempts: objective.confident_correct_attempts,
                    readiness_percent: status_percent(&objective.status),
                });
            }
        }
        for level in &mut k_levels {
            level.readiness_percent /= level.objective_count;
        }

        Ok(ProgressOverview {
            course_id: course_id.into(),
            percent,
            correct_attempts,
            total_attempts,
            readiness_percent: percent,
            readiness_label,
            calibration_note: "Konservative Schätzung aus aktiv beantworteten, sicher gewussten Lernzielen. Prüfungssimulationen und echte Prüfungsergebnisse sind noch nicht kalibriert.".into(),
            covered_objectives,
            due_reviews,
            confident_wrong_attempts,
            confidence_alignment_percent,
            objectives,
            k_levels,
        })
    }

    pub async fn search_context(
        &self,
        course_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<RetrievedContext>, PersistenceError> {
        let fts_query = fts_query(query);
        let rows = if fts_query.is_empty() {
            sqlx::query("SELECT * FROM knowledge_fts WHERE course_id = ? ORDER BY rowid LIMIT ?")
                .bind(course_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(
                "SELECT * FROM knowledge_fts WHERE knowledge_fts MATCH ? AND course_id = ? ORDER BY rank LIMIT ?",
            )
            .bind(&fts_query)
            .bind(course_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default()
        };

        Ok(rows
            .into_iter()
            .map(|row| RetrievedContext {
                id: row.get("id"),
                title: row.get("title"),
                text: row.get("text"),
                source: SourceReference {
                    label: row.get("source_label"),
                    url: row.get("source_url"),
                    section: row.get("source_section"),
                    origin: parse_origin(row.get::<String, _>("origin").as_str()),
                },
            })
            .collect())
    }
}

#[derive(Debug, Default)]
struct TermLinks {
    synonyms: Vec<String>,
    abbreviations: Vec<String>,
    see_also: Vec<GlossaryLink>,
}

fn parse_origin(value: &str) -> ContentOrigin {
    match value {
        "official" => ContentOrigin::Official,
        "official_excerpt" => ContentOrigin::OfficialExcerpt,
        "generated" => ContentOrigin::Generated,
        "user" => ContentOrigin::User,
        _ => ContentOrigin::Editorial,
    }
}

fn status_percent(status: &LearningStatus) -> i64 {
    match status {
        LearningStatus::NotStarted => 0,
        LearningStatus::Introduced => 10,
        LearningStatus::Practiced => 30,
        LearningStatus::Understood => 55,
        LearningStatus::ExamReady => 80,
    }
}

fn confidence_value(confidence: &AnswerConfidence) -> &'static str {
    match confidence {
        AnswerConfidence::Sure => "sure",
        AnswerConfidence::Unsure => "unsure",
        AnswerConfidence::Guessed => "guessed",
    }
}

fn term_direction_value(direction: TermDirection) -> &'static str {
    match direction {
        TermDirection::TermToDefinition => "term_to_definition",
        TermDirection::DefinitionToTerm => "definition_to_term",
        TermDirection::ScenarioToTerm => "scenario_to_term",
        TermDirection::TermToTopic => "term_to_topic",
    }
}

fn reasoning_choice_value(reasoning_choice: &ReasoningChoice) -> &'static str {
    match reasoning_choice {
        ReasoningChoice::Recalled => "recalled",
        ReasoningChoice::Eliminated => "eliminated",
        ReasoningChoice::AppliedRule => "applied_rule",
        ReasoningChoice::FromExperience => "from_experience",
        ReasoningChoice::Guessed => "guessed",
        ReasoningChoice::NotStated => "not_stated",
    }
}

fn readiness_label(percent: i64) -> &'static str {
    match percent {
        0..=19 => "Grundlage aufbauen",
        20..=39 => "Im Aufbau",
        40..=59 => "Noch nicht prüfungsreif",
        60..=74 => "Fast prüfungsreif",
        _ => "Konservativ prüfungsreif",
    }
}

fn fts_query(input: &str) -> String {
    input
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.chars().count() >= 3)
        .take(8)
        .map(|token| format!("\"{}\"*", token.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[derive(Debug, Deserialize)]
struct Seed {
    #[serde(default)]
    corpus_version: String,
    courses: Vec<SeedCourse>,
    glossary: Vec<SeedGlossary>,
    questions: Vec<SeedQuestion>,
    #[serde(default)]
    confusion_clusters: Vec<SeedCluster>,
}

#[derive(Debug, Deserialize)]
struct SeedCourse {
    id: String,
    code: String,
    name: String,
    version: String,
    description: String,
    exam_questions: i64,
    passing_score: i64,
    exam_minutes: i64,
    chapters: Vec<SeedChapter>,
}

#[derive(Debug, Deserialize)]
struct SeedChapter {
    id: String,
    position: i64,
    title: String,
    description: String,
    objectives: Vec<SeedObjective>,
}

#[derive(Debug, Deserialize)]
struct SeedObjective {
    id: String,
    code: String,
    k_level: String,
    title: String,
    summary: String,
    source_label: String,
    source_url: String,
    source_section: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SeedGlossary {
    id: String,
    term: String,
    definition: String,
    language: String,
    snapshot: String,
    origin: String,
    source_label: String,
    source_url: String,
    courses: Vec<String>,
    #[serde(default)]
    term_version: String,
    #[serde(default)]
    reference: String,
    #[serde(default)]
    synonyms: Vec<String>,
    #[serde(default)]
    abbreviations: Vec<String>,
    #[serde(default)]
    see_also: Vec<SeedSeeAlso>,
    #[serde(default)]
    objectives: Vec<SeedGlossaryObjective>,
}

#[derive(Debug, Deserialize)]
struct SeedSeeAlso {
    value: String,
    term_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SeedGlossaryObjective {
    learning_objective_id: String,
    relation: String,
}

#[derive(Debug, Deserialize)]
struct SeedCluster {
    id: String,
    course_id: String,
    title: String,
    note: String,
    origin: String,
    members: Vec<SeedClusterMember>,
}

#[derive(Debug, Deserialize)]
struct SeedClusterMember {
    glossary_term_id: String,
    position: i64,
    distinction: String,
}

#[derive(Debug, Deserialize)]
struct SeedQuestion {
    id: String,
    course_id: String,
    learning_objective_id: String,
    prompt: String,
    question_type: String,
    origin: String,
    points: i64,
    explanation: String,
    source_label: String,
    source_url: String,
    source_section: Option<String>,
    options: Vec<SeedOption>,
}

#[derive(Debug, Deserialize)]
struct SeedOption {
    id: String,
    position: i64,
    text: String,
    is_correct: bool,
    feedback: String,
    #[serde(default)]
    misconception: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn seed_exposes_three_courses_and_questions() {
        let database = Database::in_memory().await.unwrap();
        let courses = database.list_courses().await.unwrap();
        assert_eq!(courses.len(), 3);
        let question = database.next_question("ctfl-4", None).await.unwrap();
        assert!(!question.options.is_empty());
    }

    #[tokio::test]
    async fn next_term_exercise_has_options_without_serialized_solution() {
        let database = Database::in_memory().await.unwrap();
        let exercise = database.next_term_exercise("ctfl-4", None).await.unwrap();
        let serialized = serde_json::to_string(&exercise).unwrap();

        assert!(exercise.options.len() >= 2);
        assert!(!serialized.contains("correct_option_id"));
    }

    #[tokio::test]
    async fn cluster_term_uses_only_cluster_distractors() {
        let database = Database::in_memory().await.unwrap();
        let input = database
            .term_exercise_input("glossary-defect")
            .await
            .unwrap();
        let exercise = build_exercise(&input, TermDirection::TermToDefinition).unwrap();
        let cluster_member_ids = input
            .cluster_mates
            .iter()
            .map(|candidate| candidate.id.as_str())
            .collect::<HashSet<_>>();

        assert_eq!(
            exercise.distractor_source,
            domain::DistractorSource::Cluster
        );
        assert!(exercise.options.iter().all(|option| {
            option.id == exercise.correct_option_id
                || cluster_member_ids.contains(option.id.as_str())
        }));
    }

    #[tokio::test]
    async fn term_attempts_are_scored_and_each_adds_one_row() {
        let database = Database::in_memory().await.unwrap();
        let input = database
            .term_exercise_input("glossary-defect")
            .await
            .unwrap();
        let exercise = build_exercise(&input, TermDirection::TermToDefinition).unwrap();
        let wrong_option_id = exercise
            .options
            .iter()
            .find(|option| option.id != exercise.correct_option_id)
            .unwrap()
            .id
            .clone();
        let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM term_attempts")
            .fetch_one(&database.pool)
            .await
            .unwrap();

        let correct = database
            .submit_term_attempt(
                "glossary-defect",
                TermAttemptSubmission {
                    direction: TermDirection::TermToDefinition,
                    selected_option_id: exercise.correct_option_id,
                    confidence: AnswerConfidence::Sure,
                },
            )
            .await
            .unwrap();
        let after_correct: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM term_attempts")
            .fetch_one(&database.pool)
            .await
            .unwrap();
        let wrong = database
            .submit_term_attempt(
                "glossary-defect",
                TermAttemptSubmission {
                    direction: TermDirection::TermToDefinition,
                    selected_option_id: wrong_option_id,
                    confidence: AnswerConfidence::Unsure,
                },
            )
            .await
            .unwrap();
        let after_wrong: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM term_attempts")
            .fetch_one(&database.pool)
            .await
            .unwrap();

        assert!(correct.correct);
        assert!(!wrong.correct);
        assert_eq!(after_correct, before + 1);
        assert_eq!(after_wrong, after_correct + 1);
    }

    #[tokio::test]
    async fn next_term_exercise_is_stable_without_an_attempt() {
        let database = Database::in_memory().await.unwrap();
        let first = database.next_term_exercise("ctfl-4", None).await.unwrap();
        let second = database.next_term_exercise("ctfl-4", None).await.unwrap();

        assert_eq!(first.term_id, second.term_id);
        assert_eq!(first.direction, second.direction);
        assert_eq!(
            first
                .options
                .iter()
                .map(|option| (option.id.as_str(), option.text.as_str()))
                .collect::<Vec<_>>(),
            second
                .options
                .iter()
                .map(|option| (option.id.as_str(), option.text.as_str()))
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn next_term_changes_after_an_answer() {
        let database = Database::in_memory().await.unwrap();
        let first = database.next_term_exercise("ctfl-4", None).await.unwrap();
        database
            .submit_term_attempt(
                &first.term_id,
                TermAttemptSubmission {
                    direction: first.direction,
                    selected_option_id: first.correct_option_id.clone(),
                    confidence: AnswerConfidence::Sure,
                },
            )
            .await
            .unwrap();

        let second = database.next_term_exercise("ctfl-4", None).await.unwrap();
        assert_ne!(second.term_id, first.term_id);
    }

    #[tokio::test]
    async fn confidently_wrong_term_is_due_but_not_repeated_when_another_exists() {
        let database = Database::in_memory().await.unwrap();
        let first = database.next_term_exercise("ctfl-4", None).await.unwrap();
        let wrong_option_id = first
            .options
            .iter()
            .find(|option| option.id != first.correct_option_id)
            .unwrap()
            .id
            .clone();
        database
            .submit_term_attempt(
                &first.term_id,
                TermAttemptSubmission {
                    direction: first.direction,
                    selected_option_id: wrong_option_id,
                    confidence: AnswerConfidence::Sure,
                },
            )
            .await
            .unwrap();

        let due: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM term_attempts WHERE glossary_term_id = ? AND next_review_at <= created_at",
        )
        .bind(&first.term_id)
        .fetch_one(&database.pool)
        .await
        .unwrap();
        let second = database.next_term_exercise("ctfl-4", None).await.unwrap();

        assert_eq!(due, 1);
        assert_ne!(second.term_id, first.term_id);
    }

    #[tokio::test]
    async fn only_trainable_term_is_repeated_immediately() {
        let database = Database::in_memory().await.unwrap();
        let term_id = "glossary-defect";
        sqlx::query(
            "DELETE FROM course_glossary_terms WHERE course_id = 'ctfl-4' AND glossary_term_id != ?",
        )
        .bind(term_id)
        .execute(&database.pool)
        .await
        .unwrap();
        let input = database.term_exercise_input(term_id).await.unwrap();
        let exercise = build_exercise(&input, TermDirection::TermToDefinition).unwrap();
        let wrong_option_id = exercise
            .options
            .iter()
            .find(|option| option.id != exercise.correct_option_id)
            .unwrap()
            .id
            .clone();
        database
            .submit_term_attempt(
                term_id,
                TermAttemptSubmission {
                    direction: exercise.direction,
                    selected_option_id: wrong_option_id,
                    confidence: AnswerConfidence::Sure,
                },
            )
            .await
            .unwrap();

        let next = database.next_term_exercise("ctfl-4", None).await.unwrap();

        assert_eq!(next.term_id, term_id);
    }

    #[tokio::test]
    async fn attempts_are_scored_deterministically() {
        let database = Database::in_memory().await.unwrap();
        let question = database.next_question("ctfl-4", None).await.unwrap();
        let result = database
            .submit_attempt(
                &question.id,
                AttemptSubmission {
                    selected_option_ids: vec!["ctfl-q1-o2".into()],
                    confidence: AnswerConfidence::Sure,
                    reasoning_choice: ReasoningChoice::Recalled,
                    reasoning: "Der Test zeigt vorhandene Fehlerwirkungen.".into(),
                },
            )
            .await
            .unwrap();
        assert!(result.correct);
        assert_eq!(result.earned_points, 1);
        assert_eq!(result.confidence, AnswerConfidence::Sure);
        assert!(!result.tutor_recommended);
    }

    #[tokio::test]
    async fn consecutive_mastery_answers_use_seven_then_fourteen_days() {
        let database = Database::in_memory().await.unwrap();
        let submission = || AttemptSubmission {
            selected_option_ids: vec!["ctfl-q1-o2".into()],
            confidence: AnswerConfidence::Sure,
            reasoning_choice: ReasoningChoice::Recalled,
            reasoning: String::new(),
        };

        let first = database
            .submit_attempt("ctfl-q1", submission())
            .await
            .unwrap();
        let second = database
            .submit_attempt("ctfl-q1", submission())
            .await
            .unwrap();

        for (result, expected_days) in [(&first, 7), (&second, 14)] {
            let row = sqlx::query("SELECT created_at, next_review_at FROM attempts WHERE id = ?")
                .bind(&result.attempt_id)
                .fetch_one(&database.pool)
                .await
                .unwrap();
            let created_at =
                chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at")).unwrap();
            let next_review_at =
                chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("next_review_at"))
                    .unwrap();

            assert_eq!(
                next_review_at.signed_duration_since(created_at).num_days(),
                expected_days
            );
        }
    }

    #[tokio::test]
    async fn due_question_is_selected_before_a_new_question() {
        let database = Database::in_memory().await.unwrap();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO attempts (id, profile_id, question_id, selected_option_ids, reasoning, reasoning_choice, is_correct, confidence, next_review_at, created_at) VALUES ('due-attempt', 'local-default', 'ctfl-q1', '[]', '', 'not_stated', 0, 'unsure', ?, ?), ('upcoming-attempt', 'local-default', 'ctfl-q2', '[]', '', 'not_stated', 1, 'sure', ?, ?)",
        )
        .bind((now - Duration::days(1)).to_rfc3339())
        .bind((now - Duration::minutes(2)).to_rfc3339())
        .bind((now + Duration::days(30)).to_rfc3339())
        .bind((now - Duration::minutes(1)).to_rfc3339())
        .execute(&database.pool)
        .await
        .unwrap();

        let question = database.next_question("ctfl-4", None).await.unwrap();

        assert_eq!(question.id, "ctfl-q1");
    }

    #[tokio::test]
    async fn confident_wrong_answer_is_scheduled_and_diagnosed() {
        let database = Database::in_memory().await.unwrap();
        let result = database
            .submit_attempt(
                "ctfl-q1",
                AttemptSubmission {
                    selected_option_ids: vec!["ctfl-q1-o1".into()],
                    confidence: AnswerConfidence::Sure,
                    reasoning_choice: ReasoningChoice::NotStated,
                    reasoning: String::new(),
                },
            )
            .await
            .unwrap();
        assert!(!result.correct);
        assert!(result.tutor_recommended);
        assert!(result.misconception.is_some());

        let progress = database.progress("ctfl-4").await.unwrap();
        assert_eq!(progress.confident_wrong_attempts, 1);
        assert_eq!(progress.due_reviews, 1);
    }

    #[tokio::test]
    async fn eliminated_correct_answer_is_reviewed_before_recalled_answer() {
        let database = Database::in_memory().await.unwrap();
        let eliminated = database
            .submit_attempt(
                "ctfl-q1",
                AttemptSubmission {
                    selected_option_ids: vec!["ctfl-q1-o2".into()],
                    confidence: AnswerConfidence::Sure,
                    reasoning_choice: ReasoningChoice::Eliminated,
                    reasoning: String::new(),
                },
            )
            .await
            .unwrap();
        let recalled = database
            .submit_attempt(
                "ctfl-q1",
                AttemptSubmission {
                    selected_option_ids: vec!["ctfl-q1-o2".into()],
                    confidence: AnswerConfidence::Sure,
                    reasoning_choice: ReasoningChoice::Recalled,
                    reasoning: String::new(),
                },
            )
            .await
            .unwrap();

        assert!(!eliminated.counts_as_mastery);
        assert!(recalled.counts_as_mastery);
        assert!(
            chrono::DateTime::parse_from_rfc3339(&eliminated.review_due_at).unwrap()
                < chrono::DateTime::parse_from_rfc3339(&recalled.review_due_at).unwrap()
        );
    }

    #[tokio::test]
    async fn reasoning_choice_is_stored() {
        let database = Database::in_memory().await.unwrap();
        let result = database
            .submit_attempt(
                "ctfl-q1",
                AttemptSubmission {
                    selected_option_ids: vec!["ctfl-q1-o2".into()],
                    confidence: AnswerConfidence::Sure,
                    reasoning_choice: ReasoningChoice::FromExperience,
                    reasoning: String::new(),
                },
            )
            .await
            .unwrap();

        let stored: String =
            sqlx::query_scalar("SELECT reasoning_choice FROM attempts WHERE id = ?")
                .bind(&result.attempt_id)
                .fetch_one(&database.pool)
                .await
                .unwrap();
        assert_eq!(stored, "from_experience");
    }

    #[tokio::test]
    async fn glossary_terms_carry_links_and_glossary_fields() {
        let database = Database::in_memory().await.unwrap();
        sqlx::query(
            "INSERT INTO glossary_term_links (glossary_term_id, kind, value, target_term_id) VALUES ('glossary-defect', 'see_also', 'not installed', NULL)",
        )
        .execute(&database.pool)
        .await
        .unwrap();

        let terms = database
            .glossary(Some("ctfl-4"), Some("defect"))
            .await
            .unwrap();
        let defect = terms
            .iter()
            .find(|term| term.id == "glossary-defect")
            .expect("Begriff 'defect' fehlt im Seed");

        assert_eq!(defect.term_version, "3");
        assert_eq!(defect.reference, "After ISO 24765");
        assert_eq!(defect.synonyms, vec!["bug", "fault", "flaw"]);
        assert!(defect.abbreviations.is_empty());
        assert_eq!(defect.see_also.len(), 3);
        // Ein nicht installierter Zielbegriff bleibt als Wortlaut erhalten.
        let unresolved = defect
            .see_also
            .iter()
            .find(|link| link.value == "not installed")
            .unwrap();
        assert!(unresolved.term_id.is_none());
        assert_eq!(
            defect
                .see_also
                .iter()
                .find(|link| link.value == "failure")
                .and_then(|link| link.term_id.as_deref()),
            Some("glossary-failure")
        );

        // Begriffe ohne Beziehungen bleiben leer statt zu fehlen.
        let terms = database
            .glossary(Some("ctfl-4"), Some("test basis"))
            .await
            .unwrap();
        let test_basis = terms
            .iter()
            .find(|term| term.id == "glossary-test-basis")
            .unwrap();
        assert!(test_basis.synonyms.is_empty());
        assert!(test_basis.see_also.is_empty());
    }

    #[tokio::test]
    async fn confusion_clusters_are_read_in_member_order() {
        let database = Database::in_memory().await.unwrap();
        let clusters = database.confusion_clusters("ctfl-4").await.unwrap();
        assert_eq!(clusters.len(), 5);
        let cluster = clusters
            .iter()
            .find(|cluster| cluster.id == "ctfl-cluster-error-defect-failure")
            .unwrap();
        assert_eq!(cluster.origin, ContentOrigin::Editorial);
        assert_eq!(
            cluster
                .members
                .iter()
                .map(|member| member.term_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "glossary-error",
                "glossary-defect",
                "glossary-failure",
                "glossary-root-cause"
            ]
        );
        assert_eq!(
            cluster
                .members
                .iter()
                .map(|member| member.position)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        assert!(!cluster.members[0].definition.is_empty());
        assert!(
            database
                .confusion_clusters("ctal-tm-3")
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn glossary_objectives_contain_both_relations() {
        let database = Database::in_memory().await.unwrap();
        for relation in ["chapter_keyword", "objective_title"] {
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM glossary_term_objectives WHERE relation = ?",
            )
            .bind(relation)
            .fetch_one(&database.pool)
            .await
            .unwrap();
            assert!(count > 0, "Relation {relation} fehlt");
        }
    }

    #[tokio::test]
    async fn installing_the_same_content_twice_does_not_duplicate_rows() {
        let database = Database::in_memory().await.unwrap();
        let before = (
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM glossary_terms")
                .fetch_one(&database.pool)
                .await
                .unwrap(),
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM glossary_term_links")
                .fetch_one(&database.pool)
                .await
                .unwrap(),
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM confusion_cluster_members")
                .fetch_one(&database.pool)
                .await
                .unwrap(),
        );

        database.install_content().await.unwrap();

        let after = (
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM glossary_terms")
                .fetch_one(&database.pool)
                .await
                .unwrap(),
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM glossary_term_links")
                .fetch_one(&database.pool)
                .await
                .unwrap(),
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM confusion_cluster_members")
                .fetch_one(&database.pool)
                .await
                .unwrap(),
        );
        assert_eq!(after, before);
    }

    #[tokio::test]
    async fn reinstalling_content_preserves_attempts_and_updates_questions() {
        let database = Database::in_memory().await.unwrap();
        let question = database.next_question("ctfl-4", None).await.unwrap();
        let original_prompt = question.prompt.clone();
        let result = database
            .submit_attempt(
                &question.id,
                AttemptSubmission {
                    selected_option_ids: vec!["ctfl-q1-o2".into()],
                    confidence: AnswerConfidence::Sure,
                    reasoning_choice: ReasoningChoice::Recalled,
                    reasoning: String::new(),
                },
            )
            .await
            .unwrap();

        sqlx::query("UPDATE questions SET prompt = 'Veraltet' WHERE id = ?")
            .bind(&question.id)
            .execute(&database.pool)
            .await
            .unwrap();
        sqlx::query("UPDATE content_versions SET corpus_version = 'veraltet'")
            .execute(&database.pool)
            .await
            .unwrap();

        database.install_content().await.unwrap();

        let attempt_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attempts WHERE id = ?")
            .bind(&result.attempt_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
        let installed_prompt: String =
            sqlx::query_scalar("SELECT prompt FROM questions WHERE id = ?")
                .bind(&question.id)
                .fetch_one(&database.pool)
                .await
                .unwrap();
        assert_eq!(attempt_count, 1);
        assert_eq!(installed_prompt, original_prompt);
    }

    #[test]
    fn fts_input_is_sanitized() {
        assert_eq!(
            fts_query("Testen & Qualität"),
            "\"Testen\"* OR \"Qualität\"*"
        );
    }
}
