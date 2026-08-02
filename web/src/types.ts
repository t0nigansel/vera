export type ContentOrigin =
  | "official"
  | "official_excerpt"
  | "editorial"
  | "generated"
  | "user";

export type LearningStatus =
  | "not_started"
  | "introduced"
  | "practiced"
  | "understood"
  | "exam_ready";

export type AnswerConfidence = "sure" | "unsure" | "guessed";

export type ReasoningChoice =
  | "recalled"
  | "eliminated"
  | "applied_rule"
  | "from_experience"
  | "guessed"
  | "not_stated";

export type AttemptOutcome =
  | "confident_correct"
  | "unsure_correct"
  | "unsure_wrong"
  | "confident_wrong";

export interface SourceReference {
  label: string;
  url: string;
  section: string | null;
  origin: ContentOrigin;
}

export interface CourseSummary {
  id: string;
  code: string;
  name: string;
  version: string;
  description: string;
  exam_questions: number;
  passing_score: number;
  exam_minutes: number;
  objective_count: number;
  progress_percent: number;
  readiness_label: string;
  due_reviews: number;
  confidence_issue_count: number;
}

export interface LearningObjective {
  id: string;
  code: string;
  k_level: string;
  title: string;
  summary: string;
  status: LearningStatus;
  source: SourceReference;
}

export interface Chapter {
  id: string;
  position: number;
  title: string;
  description: string;
  objectives: LearningObjective[];
}

export interface CourseDetail {
  id: string;
  code: string;
  name: string;
  version: string;
  description: string;
  exam_questions: number;
  passing_score: number;
  exam_minutes: number;
  objective_count: number;
  progress_percent: number;
  chapters: Chapter[];
}

export interface QuestionOption {
  id: string;
  position: number;
  text: string;
}

export interface Question {
  id: string;
  course_id: string;
  learning_objective_id: string;
  prompt: string;
  question_type: "single_choice" | "multiple_choice";
  origin: ContentOrigin;
  points: number;
  review_status: string;
  difficulty: string;
  options: QuestionOption[];
  source: SourceReference;
}

export interface OptionFeedback {
  option_id: string;
  selected: boolean;
  correct: boolean;
  feedback: string;
  misconception: string | null;
}

export interface AttemptResult {
  attempt_id: string;
  correct: boolean;
  outcome: AttemptOutcome;
  counts_as_mastery: boolean;
  earned_points: number;
  available_points: number;
  explanation: string;
  option_feedback: OptionFeedback[];
  learning_status: LearningStatus;
  confidence: AnswerConfidence;
  diagnosis: string;
  misconception: string | null;
  review_due_at: string;
  tutor_recommended: boolean;
  source: SourceReference;
}

export interface ObjectiveProgress {
  objective_id: string;
  k_level: string;
  title: string;
  correct_attempts: number;
  total_attempts: number;
  confident_correct_attempts: number;
  confident_wrong_attempts: number;
  due_for_review: boolean;
  last_attempt_at: string | null;
  status: LearningStatus;
}

export interface ProgressOverview {
  course_id: string;
  percent: number;
  correct_attempts: number;
  total_attempts: number;
  readiness_percent: number;
  readiness_label: string;
  calibration_note: string;
  covered_objectives: number;
  due_reviews: number;
  confident_wrong_attempts: number;
  confidence_alignment_percent: number;
  objectives: ObjectiveProgress[];
  k_levels: KLevelProgress[];
}

export interface KLevelProgress {
  k_level: string;
  objective_count: number;
  total_attempts: number;
  correct_attempts: number;
  confident_correct_attempts: number;
  readiness_percent: number;
}

export interface GlossaryLink {
  value: string;
  term_id: string | null;
}

export interface GlossaryTerm {
  id: string;
  term: string;
  definition: string;
  language: string;
  snapshot: string;
  origin: ContentOrigin;
  term_version: string;
  reference: string;
  synonyms: string[];
  abbreviations: string[];
  see_also: GlossaryLink[];
  courses: string[];
  source: SourceReference;
}

export interface ConfusionClusterMember {
  term_id: string;
  term: string;
  definition: string;
  position: number;
  distinction: string;
}

export interface ConfusionCluster {
  id: string;
  course_id: string;
  title: string;
  note: string;
  origin: ContentOrigin;
  members: ConfusionClusterMember[];
}

export interface ProviderStatus {
  configured: boolean;
  reachable: boolean;
  provider: string;
  model: string | null;
  message: string;
}

export interface SystemStatus {
  app: string;
  version: string;
  database: string;
  courses: number;
  glossary_terms: number;
  chat: ProviderStatus;
  embedding: ProviderStatus;
}

export interface TutorResponse {
  answer: string;
  citations: SourceReference[];
  provider: string;
  model: string;
}
