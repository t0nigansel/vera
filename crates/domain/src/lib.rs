mod attempt;
mod review;
mod term_training;
pub use attempt::*;
pub use review::*;
pub use term_training::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentOrigin {
    Official,
    OfficialExcerpt,
    Editorial,
    Generated,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReference {
    pub label: String,
    pub url: String,
    pub section: Option<String>,
    pub origin: ContentOrigin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSummary {
    pub id: String,
    pub code: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub exam_questions: i64,
    pub passing_score: i64,
    pub exam_minutes: i64,
    pub objective_count: i64,
    pub progress_percent: i64,
    pub readiness_label: String,
    pub due_reviews: i64,
    pub confidence_issue_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseDetail {
    #[serde(flatten)]
    pub summary: CourseSummary,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub position: i64,
    pub title: String,
    pub description: String,
    pub objectives: Vec<LearningObjective>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningObjective {
    pub id: String,
    pub code: String,
    pub k_level: String,
    pub title: String,
    pub summary: String,
    pub status: LearningStatus,
    pub source: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearningStatus {
    NotStarted,
    Introduced,
    Practiced,
    Understood,
    ExamReady,
}

impl LearningStatus {
    pub fn from_evidence(
        total: i64,
        correct: i64,
        confident_correct: i64,
        confident_wrong: i64,
    ) -> Self {
        if total == 0 {
            return Self::NotStarted;
        }
        if total == 1 {
            return Self::Introduced;
        }

        let accuracy_percent = correct * 100 / total;
        if confident_correct >= 4 && confident_wrong == 0 && accuracy_percent >= 80 {
            Self::ExamReady
        } else if confident_correct >= 2 && accuracy_percent >= 70 {
            Self::Understood
        } else {
            Self::Practiced
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnswerConfidence {
    Sure,
    Unsure,
    Guessed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryTerm {
    pub id: String,
    pub term: String,
    pub definition: String,
    pub language: String,
    pub snapshot: String,
    pub origin: ContentOrigin,
    pub term_version: String,
    pub reference: String,
    pub synonyms: Vec<String>,
    pub abbreviations: Vec<String>,
    pub see_also: Vec<GlossaryLink>,
    pub courses: Vec<String>,
    pub source: SourceReference,
}

/// Verweis auf einen abzugrenzenden Begriff. `term_id` bleibt leer, solange der
/// Zielbegriff im installierten Bestand fehlt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryLink {
    pub value: String,
    pub term_id: Option<String>,
}

/// Redaktionell gepflegte Gruppe ähnlicher Begriffe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionCluster {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub note: String,
    pub origin: ContentOrigin,
    pub members: Vec<ConfusionClusterMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionClusterMember {
    pub term_id: String,
    pub term: String,
    pub definition: String,
    pub position: i64,
    /// Redaktioneller Abgrenzungssatz gegenüber den übrigen Mitgliedern.
    pub distinction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub course_id: String,
    pub learning_objective_id: String,
    pub prompt: String,
    pub question_type: QuestionType,
    pub origin: ContentOrigin,
    pub points: i64,
    pub review_status: String,
    pub difficulty: String,
    pub options: Vec<QuestionOption>,
    pub source: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    SingleChoice,
    MultipleChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub id: String,
    pub position: i64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptSubmission {
    pub selected_option_ids: Vec<String>,
    pub confidence: AnswerConfidence,
    #[serde(default)]
    pub reasoning_choice: ReasoningChoice,
    #[serde(default)]
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionFeedback {
    pub option_id: String,
    pub selected: bool,
    pub correct: bool,
    pub feedback: String,
    pub misconception: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptResult {
    pub attempt_id: String,
    pub correct: bool,
    pub outcome: AttemptOutcome,
    pub counts_as_mastery: bool,
    pub earned_points: i64,
    pub available_points: i64,
    pub explanation: String,
    pub option_feedback: Vec<OptionFeedback>,
    pub learning_status: LearningStatus,
    pub confidence: AnswerConfidence,
    pub diagnosis: String,
    pub misconception: Option<String>,
    pub review_due_at: String,
    pub tutor_recommended: bool,
    pub source: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressOverview {
    pub course_id: String,
    pub percent: i64,
    pub correct_attempts: i64,
    pub total_attempts: i64,
    pub readiness_percent: i64,
    pub readiness_label: String,
    pub calibration_note: String,
    pub covered_objectives: i64,
    pub due_reviews: i64,
    pub confident_wrong_attempts: i64,
    pub confidence_alignment_percent: i64,
    pub objectives: Vec<ObjectiveProgress>,
    pub k_levels: Vec<KLevelProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveProgress {
    pub objective_id: String,
    pub k_level: String,
    pub title: String,
    pub correct_attempts: i64,
    pub total_attempts: i64,
    pub confident_correct_attempts: i64,
    pub confident_wrong_attempts: i64,
    pub due_for_review: bool,
    pub last_attempt_at: Option<String>,
    pub status: LearningStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLevelProgress {
    pub k_level: String,
    pub objective_count: i64,
    pub total_attempts: i64,
    pub correct_attempts: i64,
    pub confident_correct_attempts: i64,
    pub readiness_percent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedContext {
    pub id: String,
    pub title: String,
    pub text: String,
    pub source: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorRequest {
    pub course_id: String,
    pub learning_objective_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorResponse {
    pub answer: String,
    pub citations: Vec<SourceReference>,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub configured: bool,
    pub reachable: bool,
    pub provider: String,
    pub model: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub app: String,
    pub version: String,
    pub database: String,
    pub courses: i64,
    pub glossary_terms: i64,
    pub chat: ProviderStatus,
    pub embedding: ProviderStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learning_status_is_evidence_based() {
        assert_eq!(
            LearningStatus::from_evidence(0, 0, 0, 0),
            LearningStatus::NotStarted
        );
        assert_eq!(
            LearningStatus::from_evidence(1, 0, 0, 0),
            LearningStatus::Introduced
        );
        assert_eq!(
            LearningStatus::from_evidence(2, 2, 0, 0),
            LearningStatus::Practiced
        );
        assert_eq!(
            LearningStatus::from_evidence(3, 3, 2, 0),
            LearningStatus::Understood
        );
        assert_eq!(
            LearningStatus::from_evidence(5, 4, 4, 0),
            LearningStatus::ExamReady
        );
        assert_eq!(
            LearningStatus::from_evidence(5, 4, 4, 1),
            LearningStatus::Understood
        );
    }
}
