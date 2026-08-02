import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "./api";
import type {
  AnswerConfidence,
  AttemptResult,
  CourseDetail,
  CourseSummary,
  GlossaryTerm,
  LearningObjective,
  LearningStatus,
  ProgressOverview,
  Question,
  ReasoningChoice,
  SystemStatus,
  TutorResponse,
} from "./types";

type Page = "dashboard" | "course" | "glossary" | "system";
type CourseTab = "learn" | "objectives" | "progress" | "glossary";

const statusLabels: Record<LearningStatus, string> = {
  not_started: "Nicht begonnen",
  introduced: "Eingeführt",
  practiced: "Geübt",
  understood: "Verstanden",
  exam_ready: "Prüfungssicher",
};

const originLabels = {
  official: "Offiziell",
  official_excerpt: "Offizieller Auszug",
  editorial: "Redaktionell",
  generated: "KI-generiert",
  user: "Eigener Inhalt",
};

export default function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const [courses, setCourses] = useState<CourseSummary[]>([]);
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [activeCourseId, setActiveCourseId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const refreshShell = useCallback(async () => {
    try {
      const [courseData, statusData] = await Promise.all([api.courses(), api.status()]);
      setCourses(courseData);
      setStatus(statusData);
      setError(null);
    } catch (caught) {
      setError(messageOf(caught));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshShell();
  }, [refreshShell]);

  const openCourse = (courseId: string) => {
    setActiveCourseId(courseId);
    setPage("course");
  };

  return (
    <div className="app-shell">
      <Sidebar
        page={page}
        courses={courses}
        activeCourseId={activeCourseId}
        status={status}
        onNavigate={setPage}
        onCourse={openCourse}
      />
      <main className="main-content">
        {error && <ErrorBanner message={error} onRetry={() => void refreshShell()} />}
        {loading ? (
          <LoadingState label="Lernumgebung wird vorbereitet …" />
        ) : page === "dashboard" ? (
          <Dashboard courses={courses} status={status} onOpenCourse={openCourse} />
        ) : page === "course" && activeCourseId ? (
          <CourseWorkspace
            courseId={activeCourseId}
            chatReady={Boolean(status?.chat.reachable)}
            onProgressChanged={() => void refreshShell()}
          />
        ) : page === "glossary" ? (
          <GlossaryLibrary />
        ) : (
          <SystemPage status={status} onRefresh={() => void refreshShell()} />
        )}
      </main>
    </div>
  );
}

function Sidebar({
  page,
  courses,
  activeCourseId,
  status,
  onNavigate,
  onCourse,
}: {
  page: Page;
  courses: CourseSummary[];
  activeCourseId: string | null;
  status: SystemStatus | null;
  onNavigate: (page: Page) => void;
  onCourse: (courseId: string) => void;
}) {
  return (
    <aside className="sidebar">
      <button className="brand" onClick={() => onNavigate("dashboard")}>
        <span className="brand-mark">L</span>
        <span>
          <strong>learnISTQB</strong>
          <small>Prüfungsvorbereitung</small>
        </span>
      </button>

      <nav aria-label="Hauptnavigation">
        <NavButton active={page === "dashboard"} icon="⌂" onClick={() => onNavigate("dashboard")}>
          Übersicht
        </NavButton>
        <NavButton active={page === "glossary"} icon="A" onClick={() => onNavigate("glossary")}>
          Glossar
        </NavButton>
        <NavButton active={page === "system"} icon="●" onClick={() => onNavigate("system")}>
          System
        </NavButton>
      </nav>

      <div className="sidebar-section">
        <span className="sidebar-label">Deine Kurse</span>
        {courses.map((course) => (
          <button
            key={course.id}
            className={`course-nav ${page === "course" && activeCourseId === course.id ? "active" : ""}`}
            onClick={() => onCourse(course.id)}
          >
            <span className="course-nav-code">{course.code}</span>
            <span className="course-nav-meta">
              <span>{course.version}</span>
              <span>{course.progress_percent}% Reife</span>
            </span>
            <span className="tiny-progress">
              <span style={{ width: `${course.progress_percent}%` }} />
            </span>
          </button>
        ))}
      </div>

      <div className={`provider-card ${status?.chat.reachable ? "online" : "offline"}`}>
        <span className="status-dot" />
        <div>
          <strong>{status?.chat.reachable ? "Tutor bereit" : "Tutor offline"}</strong>
          <small>{status?.chat.model ?? status?.chat.provider ?? "Nicht konfiguriert"}</small>
        </div>
      </div>
    </aside>
  );
}

function NavButton({
  active,
  icon,
  children,
  onClick,
}: {
  active: boolean;
  icon: string;
  children: string;
  onClick: () => void;
}) {
  return (
    <button className={`nav-button ${active ? "active" : ""}`} onClick={onClick}>
      <span className="nav-icon">{icon}</span>
      {children}
    </button>
  );
}

function Dashboard({
  courses,
  status,
  onOpenCourse,
}: {
  courses: CourseSummary[];
  status: SystemStatus | null;
  onOpenCourse: (courseId: string) => void;
}) {
  const nextCourse = courses.find((course) => course.progress_percent > 0) ?? courses[0];

  return (
    <div className="page-wrap">
      <header className="hero">
        <div>
          <span className="eyebrow">Dein Lerncockpit</span>
          <h1>Verstehen. Anwenden. Bestehen.</h1>
          <p>
            Deine Alternative zum starren Schulungskurs: Lerne im eigenen Tempo entlang der
            offiziellen Ziele und übe genau dort, wo dir noch Sicherheit fehlt.
          </p>
        </div>
        <div className="hero-orbit" aria-hidden="true">
          <span className="orbit-ring ring-one" />
          <span className="orbit-ring ring-two" />
          <span className="orbit-core">{courses.length}</span>
          <small>Kurse</small>
        </div>
      </header>

      {nextCourse && (
        <section className="continue-card">
          <div className="continue-icon">↗</div>
          <div className="continue-copy">
            <span className="eyebrow">{nextCourse.due_reviews > 0 ? "Wiederholung fällig" : "Als Nächstes"}</span>
            <h2>{nextCourse.code} {nextCourse.version}</h2>
            <p>{nextCourse.due_reviews > 0 ? `${nextCourse.due_reviews} Lernziel${nextCourse.due_reviews === 1 ? "" : "e"} wartet auf Wiederholung.` : nextCourse.description}</p>
          </div>
          <div className="continue-progress">
            <strong>{nextCourse.progress_percent}%</strong>
            <span>Prüfungsreife</span>
          </div>
          <button className="primary-button" onClick={() => onOpenCourse(nextCourse.id)}>
            Lernen fortsetzen
          </button>
        </section>
      )}

      <section className="section-block">
        <div className="section-heading">
          <div>
            <span className="eyebrow">Kursbibliothek</span>
            <h2>Deine Zertifizierungen</h2>
          </div>
          <span className="muted">{status?.glossary_terms ?? 0} Glossarbegriffe installiert</span>
        </div>
        <div className="course-grid">
          {courses.map((course, index) => (
            <article className={`course-card course-accent-${index + 1}`} key={course.id}>
              <div className="course-card-top">
                <span className="course-code">{course.code}</span>
                <span className="version-chip">v{course.version}</span>
              </div>
              <h3>{course.name}</h3>
              <p>{course.description}</p>
              <div className="exam-facts">
                <span><strong>{course.exam_questions}</strong> Fragen</span>
                <span><strong>{course.exam_minutes}</strong> Min.</span>
                <span><strong>{course.objective_count}</strong> Ziele</span>
              </div>
              <div className="progress-row">
                <span className="progress-track">
                  <span style={{ width: `${course.progress_percent}%` }} />
                </span>
                <strong>{course.progress_percent}%</strong>
              </div>
              <span className="readiness-caption">{course.readiness_label}{course.confidence_issue_count > 0 ? ` · ${course.confidence_issue_count} Fehlvorstellung${course.confidence_issue_count === 1 ? "" : "en"}` : ""}</span>
              <button className="text-button" onClick={() => onOpenCourse(course.id)}>
                Kurs öffnen <span>→</span>
              </button>
            </article>
          ))}
        </div>
      </section>

      <section className="trust-strip">
        <div><strong>Quellengebunden</strong><span>Syllabus und Glossar bleiben sichtbar.</span></div>
        <div><strong>Ohne Braindumps</strong><span>Nur selbst erstellte, redaktionell geprüfte Fragen.</span></div>
        <div><strong>Lokal zuerst</strong><span>Deine Lernhistorie bleibt auf deinem Gerät.</span></div>
      </section>
    </div>
  );
}

function CourseWorkspace({
  courseId,
  chatReady,
  onProgressChanged,
}: {
  courseId: string;
  chatReady: boolean;
  onProgressChanged: () => void;
}) {
  const [course, setCourse] = useState<CourseDetail | null>(null);
  const [progress, setProgress] = useState<ProgressOverview | null>(null);
  const [tab, setTab] = useState<CourseTab>("learn");
  const [objective, setObjective] = useState<LearningObjective | null>(null);
  const [question, setQuestion] = useState<Question | null>(null);
  const [examDate, setExamDate] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadCourse = useCallback(async () => {
    setLoading(true);
    try {
      const [courseData, progressData] = await Promise.all([
        api.course(courseId),
        api.progress(courseId),
      ]);
      setCourse(courseData);
      setProgress(progressData);
      const firstObjective = courseData.chapters[0]?.objectives[0] ?? null;
      setObjective((current) =>
        current
          ? courseData.chapters.flatMap((chapter) => chapter.objectives).find((item) => item.id === current.id) ?? firstObjective
          : firstObjective,
      );
      setError(null);
    } catch (caught) {
      setError(messageOf(caught));
    } finally {
      setLoading(false);
    }
  }, [courseId]);

  useEffect(() => {
    setQuestion(null);
    setObjective(null);
    setExamDate(window.localStorage.getItem(`learnistqb:exam-date:${courseId}`) ?? "");
    void loadCourse();
  }, [loadCourse]);

  const daysUntilExam = useMemo(() => {
    if (!examDate) return null;
    const target = new Date(`${examDate}T12:00:00`);
    return Math.max(0, Math.ceil((target.getTime() - Date.now()) / 86_400_000));
  }, [examDate]);

  const updateExamDate = (value: string) => {
    setExamDate(value);
    if (value) window.localStorage.setItem(`learnistqb:exam-date:${courseId}`, value);
    else window.localStorage.removeItem(`learnistqb:exam-date:${courseId}`);
  };

  const loadQuestion = useCallback(async (objectiveId?: string) => {
    try {
      const next = await api.nextQuestion(courseId, objectiveId);
      setQuestion(next);
      if (!objectiveId) {
        const nextObjective = course?.chapters
          .flatMap((chapter) => chapter.objectives)
          .find((item) => item.id === next.learning_objective_id);
        if (nextObjective) setObjective(nextObjective);
      }
      setError(null);
    } catch (caught) {
      setError(messageOf(caught));
    }
  }, [course, courseId]);

  useEffect(() => {
    if (objective && !question) void loadQuestion(objective.id);
  }, [objective, question, loadQuestion]);

  const selectObjective = (item: LearningObjective) => {
    setObjective(item);
    setQuestion(null);
    setTab("learn");
  };

  const refreshAfterAttempt = async () => {
    await loadCourse();
    onProgressChanged();
  };

  if (loading && !course) return <LoadingState label="Kurs wird geladen …" />;
  if (!course) return <ErrorBanner message={error ?? "Kurs nicht verfügbar."} onRetry={() => void loadCourse()} />;

  return (
    <div className="page-wrap course-workspace">
      {error && <ErrorBanner message={error} onRetry={() => void loadCourse()} />}
      <header className="course-header">
        <div>
          <span className="course-code large">{course.code}</span>
          <span className="version-chip">Version {course.version}</span>
          <h1>{course.name}</h1>
          <p>{course.description}</p>
        </div>
        <div className="score-ring" style={{ "--score": `${progress?.percent ?? 0}%` } as React.CSSProperties}>
          <span>{progress?.percent ?? 0}%</span>
          <small>Prüfungsreife</small>
        </div>
      </header>

      <section className="study-plan-bar">
        <div>
          <span className="eyebrow">Dein Lernplan</span>
          <strong>{examDate ? `${daysUntilExam} Tage bis zur Prüfung` : "Prüfungsdatum optional festlegen"}</strong>
          <small>{examDate ? `Empfohlen: ${Math.max(2, Math.ceil(((progress?.due_reviews ?? 0) + (course.objective_count - (progress?.covered_objectives ?? 0))) / Math.max(1, Math.ceil((daysUntilExam ?? 1) / 7))))} Lernaktivitäten pro Woche` : "Damit verdichten wir neue Themen und Wiederholungen passend zu deinem Zeitbudget."}</small>
        </div>
        <label>
          <span>Prüfungsdatum</span>
          <input type="date" min={new Date().toISOString().slice(0, 10)} value={examDate} onChange={(event) => updateExamDate(event.target.value)} />
        </label>
      </section>

      <div className="tab-bar" role="tablist" aria-label="Kursbereiche">
        {([
          ["learn", "Lernen"],
          ["objectives", "Lernziele"],
          ["progress", "Fortschritt"],
          ["glossary", "Glossar"],
        ] as [CourseTab, string][]).map(([value, label]) => (
          <button key={value} className={tab === value ? "active" : ""} onClick={() => setTab(value)}>
            {label}
          </button>
        ))}
      </div>

      {tab === "learn" && objective && question ? (
        <LearningSession
          course={course}
          objective={objective}
          question={question}
          chatReady={chatReady}
          onNext={() => void loadQuestion()}
          onCompleted={() => void refreshAfterAttempt()}
        />
      ) : tab === "objectives" ? (
        <ObjectivesPanel course={course} onSelect={selectObjective} />
      ) : tab === "progress" && progress ? (
        <ProgressPanel progress={progress} />
      ) : tab === "glossary" ? (
        <GlossaryLibrary courseId={courseId} />
      ) : (
        <LoadingState label="Übung wird ausgewählt …" />
      )}
    </div>
  );
}

function LearningSession({
  course,
  objective,
  question,
  chatReady,
  onNext,
  onCompleted,
}: {
  course: CourseDetail;
  objective: LearningObjective;
  question: Question;
  chatReady: boolean;
  onNext: () => void;
  onCompleted: () => void;
}) {
  const [tutorDraft, setTutorDraft] = useState("");

  return (
    <div className="learning-layout">
      <section className="lesson-column">
        <article className="objective-card">
          <div className="objective-meta">
            <span>{objective.code}</span>
            <span className="k-chip">{objective.k_level}</span>
            <StatusChip status={objective.status} />
          </div>
          <h2>{objective.title}</h2>
          <p>{objective.summary}</p>
          <SourceLink source={objective.source} />
        </article>
        <QuestionCard
          key={question.id}
          question={question}
          onCompleted={onCompleted}
          onNext={onNext}
          onTutor={(message) => setTutorDraft(message)}
        />
      </section>
      <TutorPanel
        courseId={course.id}
        objective={objective}
        chatReady={chatReady}
        suggestedMessage={tutorDraft}
        onSuggestionAccepted={() => setTutorDraft("")}
      />
    </div>
  );
}

function QuestionCard({
  question,
  onCompleted,
  onNext,
  onTutor,
}: {
  question: Question;
  onCompleted: () => void;
  onNext: () => void;
  onTutor: (message: string) => void;
}) {
  const [selected, setSelected] = useState<string[]>([]);
  const [reasoning, setReasoning] = useState("");
  const [confidence, setConfidence] = useState<AnswerConfidence | null>(null);
  const [reasoningChoice, setReasoningChoice] =
    useState<ReasoningChoice>("not_stated");
  const [result, setResult] = useState<AttemptResult | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const toggle = (optionId: string) => {
    if (result) return;
    setSelected((current) =>
      question.question_type === "single_choice"
        ? [optionId]
        : current.includes(optionId)
          ? current.filter((id) => id !== optionId)
          : [...current, optionId],
    );
  };

  const submit = async () => {
    setSubmitting(true);
    try {
      if (!confidence) return;
      setResult(
        await api.submitAttempt(
          question.id,
          selected,
          confidence,
          reasoningChoice,
          reasoning,
        ),
      );
      setError(null);
      onCompleted();
    } catch (caught) {
      setError(messageOf(caught));
    } finally {
      setSubmitting(false);
    }
  };

  const feedbackByOption = useMemo(
    () => new Map(result?.option_feedback.map((item) => [item.option_id, item])),
    [result],
  );

  return (
    <article className="question-card">
      <div className="question-kicker">
        <span>Aktive Übung</span>
        <span>{question.review_status === "approved" ? "Redaktionell freigegeben" : originLabels[question.origin]}</span>
        <span>{question.difficulty === "exam" ? "Prüfungsniveau" : question.difficulty}</span>
        <strong>{question.points} {question.points === 1 ? "Punkt" : "Punkte"}</strong>
      </div>
      <h3>{question.prompt}</h3>
      <div className="option-list">
        {question.options.map((option) => {
          const feedback = feedbackByOption.get(option.id);
          const checked = selected.includes(option.id);
          const state = result
            ? feedback?.correct
              ? "correct"
              : checked
                ? "incorrect"
                : "dimmed"
            : checked
              ? "selected"
              : "";
          return (
            <button key={option.id} className={`option ${state}`} onClick={() => toggle(option.id)}>
              <span className="option-letter">{String.fromCharCode(64 + option.position)}</span>
              <span className="option-copy">
                <span>{option.text}</span>
                {result && feedback && <small>{feedback.feedback}</small>}
              </span>
              <span className="option-control">{feedback?.correct ? "✓" : checked && result ? "×" : checked ? "●" : ""}</span>
            </button>
          );
        })}
      </div>

      {!result && (
        <>
          <fieldset className="confidence-field">
            <legend>Wie sicher bist du?</legend>
            <span>Bewerte deine Sicherheit vor der Auflösung.</span>
            <div className="confidence-options">
              {(["sure", "unsure", "guessed"] as AnswerConfidence[]).map((value) => (
                <button key={value} type="button" className={confidence === value ? "active" : ""} onClick={() => setConfidence(value)}>
                  {value === "sure" ? "Sicher" : value === "unsure" ? "Unsicher" : "Geraten"}
                </button>
              ))}
            </div>
          </fieldset>
          <fieldset className="confidence-field">
            <legend>Wie bist du zu dieser Antwort gekommen?</legend>
            <span>Der Denkweg zeigt, ob du gewusst oder ausgeschlossen hast.</span>
            <div className="confidence-options">
              {([
                ["recalled", "Definition erinnert"],
                ["applied_rule", "Regel angewendet"],
                ["eliminated", "Andere ausgeschlossen"],
                ["from_experience", "Aus Erfahrung"],
                ["guessed", "Geraten"],
              ] as [ReasoningChoice, string][]).map(([value, label]) => (
                <button
                  key={value}
                  type="button"
                  className={reasoningChoice === value ? "active" : ""}
                  onClick={() => setReasoningChoice(value)}
                >
                  {label}
                </button>
              ))}
            </div>
          </fieldset>
          <label className="reasoning-field">
            <span>Warum wählst du diese Antwort? <small>Optional, aber lernwirksam.</small></span>
            <textarea
              value={reasoning}
              onChange={(event) => setReasoning(event.target.value)}
              placeholder="Erkläre deine Überlegung in einem oder zwei Sätzen …"
              rows={3}
            />
          </label>
        </>
      )}

      {error && <p className="inline-error">{error}</p>}

      {result ? (
        <div className={`result-box ${result.correct ? "success" : "needs-work"}`}>
          <div className="result-heading">
            <span className="result-icon">{result.correct ? "✓" : "↻"}</span>
            <div>
              <strong>{result.correct ? "Richtig eingeordnet" : "Noch nicht ganz"}</strong>
              <span>{result.earned_points} von {result.available_points} Punkten</span>
            </div>
            <StatusChip status={result.learning_status} />
          </div>
          <p>{result.explanation}</p>
          <div className={`diagnosis-box ${result.tutor_recommended ? "priority" : ""}`}>
            <strong>{result.diagnosis}</strong>
            {result.correct && !result.counts_as_mastery && (
              <span>Zählt noch nicht als sichere Beherrschung.</span>
            )}
            {result.misconception && <span>Wahrscheinliche Fehlvorstellung: {result.misconception}</span>}
            <small>Nächste Wiederholung: {formatReviewDate(result.review_due_at)}</small>
          </div>
          <SourceLink source={result.source} />
          {result.tutor_recommended && (
            <button className="secondary-button tutor-trigger" onClick={() => onTutor(`Ich war mir bei dieser falschen Antwort sicher. Hilf mir sokratisch, diese Fehlvorstellung zu klären: ${result.misconception ?? question.prompt}`)}>
              Mit Tutor klären
            </button>
          )}
          <button className="primary-button" onClick={onNext}>Nächste empfohlene Übung</button>
        </div>
      ) : (
        <div className="question-actions">
          <SourceLink source={question.source} />
          <button className="primary-button" disabled={!selected.length || !confidence || submitting} onClick={() => void submit()}>
            {submitting ? "Wird bewertet …" : "Antwort prüfen"}
          </button>
        </div>
      )}
    </article>
  );
}

function TutorPanel({
  courseId,
  objective,
  chatReady,
  suggestedMessage,
  onSuggestionAccepted,
}: {
  courseId: string;
  objective: LearningObjective;
  chatReady: boolean;
  suggestedMessage: string;
  onSuggestionAccepted: () => void;
}) {
  const [message, setMessage] = useState("");
  const [response, setResponse] = useState<TutorResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!suggestedMessage) return;
    setMessage(suggestedMessage);
    onSuggestionAccepted();
  }, [suggestedMessage, onSuggestionAccepted]);

  const ask = async (suggestion?: string) => {
    const prompt = suggestion ?? message;
    if (!prompt.trim()) return;
    setLoading(true);
    setResponse(null);
    try {
      setResponse(await api.tutor(courseId, objective.id, prompt));
      setMessage("");
      setError(null);
    } catch (caught) {
      setError(messageOf(caught));
    } finally {
      setLoading(false);
    }
  };

  return (
    <aside className="tutor-panel">
      <div className="tutor-title">
        <span className="tutor-avatar">T</span>
        <div>
          <strong>Dein Tutor</strong>
          <small>{chatReady ? "Quellengebunden · bereit" : "Modell noch offline"}</small>
        </div>
        <span className={`status-dot ${chatReady ? "online" : ""}`} />
      </div>
      <div className="tutor-context">
        <span>Aktueller Fokus</span>
        <strong>{objective.title}</strong>
      </div>

      {!response && !loading && !error && (
        <div className="tutor-empty">
          <div className="spark">✦</div>
          <p>Ich antworte nur auf Basis der Quellen dieses Kurses.</p>
          <div className="suggestion-list">
            {["Erkläre mir das mit einem Beispiel.", "Womit könnte ich diesen Begriff verwechseln?", "Stelle mir eine sokratische Rückfrage."].map((suggestion) => (
              <button key={suggestion} onClick={() => void ask(suggestion)}>{suggestion}</button>
            ))}
          </div>
        </div>
      )}

      {loading && <LoadingState label="Tutor prüft die Quellen …" compact />}
      {error && (
        <div className="tutor-error">
          <strong>Tutor nicht verfügbar</strong>
          <p>{error}</p>
        </div>
      )}
      {response && (
        <div className="tutor-response">
          <p>{response.answer}</p>
          <div className="citation-list">
            {response.citations.map((citation, index) => (
              <a key={`${citation.url}-${index}`} href={citation.url} target="_blank" rel="noreferrer">
                Quelle {index + 1}: {citation.section ?? citation.label}
              </a>
            ))}
          </div>
          <small>{response.provider} · {response.model}</small>
        </div>
      )}

      <div className="tutor-composer">
        <textarea
          value={message}
          onChange={(event) => setMessage(event.target.value)}
          placeholder="Frage zu diesem Lernziel …"
          rows={3}
        />
        <button disabled={!message.trim() || loading} onClick={() => void ask()} aria-label="Frage senden">↑</button>
      </div>
    </aside>
  );
}

function ObjectivesPanel({ course, onSelect }: { course: CourseDetail; onSelect: (objective: LearningObjective) => void }) {
  return (
    <div className="content-panel">
      <div className="section-heading">
        <div><span className="eyebrow">Syllabus-Struktur</span><h2>Learning Objectives</h2></div>
        <span className="muted">{course.objective_count} Ziele installiert</span>
      </div>
      {course.chapters.map((chapter) => (
        <section className="chapter-block" key={chapter.id}>
          <div className="chapter-number">{chapter.position}</div>
          <div className="chapter-content">
            <h3>{chapter.title}</h3>
            <p>{chapter.description}</p>
            <div className="objective-list">
              {chapter.objectives.map((objective) => (
                <button key={objective.id} onClick={() => onSelect(objective)}>
                  <span className="k-chip">{objective.k_level}</span>
                  <span><strong>{objective.title}</strong><small>{objective.code}</small></span>
                  <StatusChip status={objective.status} />
                  <span className="arrow">→</span>
                </button>
              ))}
            </div>
          </div>
        </section>
      ))}
    </div>
  );
}

function ProgressPanel({ progress }: { progress: ProgressOverview }) {
  return (
    <div className="content-panel">
      <section className="readiness-hero">
        <div className="score-ring standalone" style={{ "--score": `${progress.readiness_percent}%` } as React.CSSProperties}>
          <span>{progress.readiness_percent}%</span>
          <small>Prüfungsreife</small>
        </div>
        <div>
          <span className="eyebrow">Bin ich so weit?</span>
          <h2>{progress.readiness_label}</h2>
          <p>{progress.calibration_note}</p>
        </div>
      </section>
      <div className="metric-grid four">
        <Metric value={`${progress.covered_objectives}/${progress.objectives.length}`} label="Lernziele aktiv geprüft" />
        <Metric value={String(progress.due_reviews)} label="Wiederholungen fällig" />
        <Metric value={String(progress.confident_wrong_attempts)} label="Sicher, aber falsch" />
        <Metric value={`${progress.confidence_alignment_percent}%`} label="Sicherheit passt zum Ergebnis" />
      </div>
      <div className="section-heading"><div><span className="eyebrow">Anforderungsniveau</span><h2>Leistung nach K-Level</h2></div></div>
      <div className="k-level-grid">
        {progress.k_levels.map((level) => (
          <article key={level.k_level}>
            <span className="k-chip">{level.k_level}</span>
            <strong>{level.readiness_percent}%</strong>
            <span className="progress-track"><span style={{ width: `${level.readiness_percent}%` }} /></span>
            <small>{level.confident_correct_attempts} sicher richtige Antworten · {level.objective_count} Lernziele</small>
          </article>
        ))}
      </div>
      <div className="section-heading"><div><span className="eyebrow">Lernnachweise</span><h2>Fortschritt pro Lernziel</h2></div></div>
      <div className="progress-list">
        {progress.objectives.map((objective) => (
          <div key={objective.objective_id} className={objective.due_for_review ? "review-due" : ""}>
            <span><strong>{objective.title}</strong><small>{objective.confident_correct_attempts} sicher richtige · {objective.correct_attempts} von {objective.total_attempts} insgesamt richtig{objective.due_for_review ? " · Wiederholung fällig" : ""}</small></span>
            <StatusChip status={objective.status} />
          </div>
        ))}
      </div>
    </div>
  );
}

function GlossaryLibrary({ courseId }: { courseId?: string }) {
  const [terms, setTerms] = useState<GlossaryTerm[]>([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setLoading(true);
      api.glossary(courseId, query)
        .then((data) => { setTerms(data); setError(null); })
        .catch((caught) => setError(messageOf(caught)))
        .finally(() => setLoading(false));
    }, 150);
    return () => window.clearTimeout(timeout);
  }, [courseId, query]);

  return (
    <div className="page-wrap glossary-page">
      <div className="section-heading">
        <div><span className="eyebrow">Terminologie</span><h1>ISTQB-Glossar</h1><p>Begriffe werden strukturiert und mit ihrer Herkunft angezeigt.</p></div>
        <label className="search-field"><span>⌕</span><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Begriff suchen …" /></label>
      </div>
      <div className="content-note">
        <strong>Starter-Snapshot</strong>
        <span>Aktuell sind redaktionelle Startertexte mit Links zum offiziellen Glossar installiert. Der vollständige offizielle Snapshot folgt über den versionierten Import.</span>
      </div>
      {error && <ErrorBanner message={error} />}
      {loading ? <LoadingState label="Glossar wird durchsucht …" /> : (
        <div className="glossary-grid">
          {terms.map((term) => (
            <article key={term.id}>
              <div className="term-heading"><h2>{term.term}</h2><span className={`origin-chip ${term.origin}`}>{originLabels[term.origin]}</span></div>
              <p>{term.definition}</p>
              {(term.synonyms.length > 0 || term.abbreviations.length > 0 || term.see_also.length > 0 || term.reference) && (
                <dl className="term-relations">
                  {term.abbreviations.length > 0 && (
                    <><dt>Abkürzung</dt><dd>{term.abbreviations.join(", ")}</dd></>
                  )}
                  {term.synonyms.length > 0 && (
                    <><dt>Synonym</dt><dd>{term.synonyms.join(", ")}</dd></>
                  )}
                  {term.see_also.length > 0 && (
                    <>
                      <dt>Abzugrenzen</dt>
                      <dd>
                        {term.see_also.map((link, index) => (
                          <span key={link.value} className={link.term_id ? "installed" : ""}>
                            {link.value}{index < term.see_also.length - 1 ? ", " : ""}
                          </span>
                        ))}
                      </dd>
                    </>
                  )}
                  {term.reference && (<><dt>Quelle</dt><dd>{term.reference}</dd></>)}
                </dl>
              )}
              <div className="term-meta">
                <span>{term.language.toUpperCase()}</span>
                <span>{term.snapshot}</span>
                {term.term_version && <span>Begriffsversion {term.term_version}</span>}
              </div>
              <SourceLink source={term.source} />
            </article>
          ))}
          {!terms.length && <p className="empty-message">Kein passender Begriff gefunden.</p>}
        </div>
      )}
    </div>
  );
}

function SystemPage({ status, onRefresh }: { status: SystemStatus | null; onRefresh: () => void }) {
  return (
    <div className="page-wrap system-page">
      <div className="section-heading">
        <div><span className="eyebrow">Lokale Umgebung</span><h1>Systemstatus</h1><p>Inhalte und deterministische Übungen funktionieren auch ohne aktives Chatmodell.</p></div>
        <button className="secondary-button" onClick={onRefresh}>Neu prüfen</button>
      </div>
      <div className="system-grid">
        <SystemCard title="Anwendung" ready value={`Version ${status?.version ?? "–"}`} detail="Rust-Server und Browser-Client" />
        <SystemCard title="Datenbank" ready={status?.database === "ready"} value={status?.database ?? "Unbekannt"} detail={`${status?.courses ?? 0} Kurse · ${status?.glossary_terms ?? 0} Begriffe`} />
        <SystemCard title="Chatmodell" ready={Boolean(status?.chat.reachable)} value={status?.chat.model ?? "Nicht gewählt"} detail={status?.chat.message ?? "Status wird geladen"} />
        <SystemCard title="Embeddingmodell" ready={Boolean(status?.embedding.reachable)} value={status?.embedding.model ?? "Nicht gewählt"} detail={status?.embedding.message ?? "FTS5 bleibt ohne Modell verfügbar"} />
      </div>
      <article className="config-card">
        <span className="eyebrow">Provider wechseln</span>
        <h2>Keine Fachlogik ist an Ollama gebunden</h2>
        <p>Über <code>CHAT_PROVIDER</code>, <code>CHAT_BASE_URL</code>, <code>CHAT_MODEL</code> und <code>CHAT_API_KEY</code> kann später ein gehosteter OpenAI-kompatibler Provider verwendet werden.</p>
      </article>
    </div>
  );
}

function SystemCard({ title, ready, value, detail }: { title: string; ready: boolean; value: string; detail: string }) {
  return <article className="system-card"><span className={`status-dot ${ready ? "online" : ""}`} /><span>{title}</span><strong>{value}</strong><p>{detail}</p></article>;
}

function StatusChip({ status }: { status: LearningStatus }) {
  return <span className={`learning-status ${status}`}>{statusLabels[status]}</span>;
}

function SourceLink({ source }: { source: { label: string; url: string; section: string | null } }) {
  return (
    <a className="source-link" href={source.url} target="_blank" rel="noreferrer">
      <span>↗</span>
      <span>{source.section ?? source.label}</span>
    </a>
  );
}

function Metric({ value, label }: { value: string; label: string }) {
  return <div className="metric"><strong>{value}</strong><span>{label}</span></div>;
}

function LoadingState({ label, compact = false }: { label: string; compact?: boolean }) {
  return <div className={`loading-state ${compact ? "compact" : ""}`}><span className="spinner" /><p>{label}</p></div>;
}

function ErrorBanner({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return <div className="error-banner" role="alert"><span>!</span><p>{message}</p>{onRetry && <button onClick={onRetry}>Erneut versuchen</button>}</div>;
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : "Ein unbekannter Fehler ist aufgetreten.";
}

function formatReviewDate(value: string): string {
  const date = new Date(value);
  if (date.getTime() <= Date.now() + 60_000) return "jetzt";
  return new Intl.DateTimeFormat("de-DE", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
  }).format(date);
}
