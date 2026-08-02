import type {
  AnswerConfidence,
  AttemptResult,
  CourseDetail,
  CourseSummary,
  GlossaryTerm,
  ProgressOverview,
  Question,
  ReasoningChoice,
  SystemStatus,
  TutorResponse,
} from "./types";

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
  });

  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as { error?: string } | null;
    throw new Error(body?.error ?? `Anfrage fehlgeschlagen (${response.status})`);
  }

  return response.json() as Promise<T>;
}

export const api = {
  status: () => request<SystemStatus>("/api/system/status"),
  courses: () => request<CourseSummary[]>("/api/courses"),
  course: (courseId: string) => request<CourseDetail>(`/api/courses/${courseId}`),
  progress: (courseId: string) =>
    request<ProgressOverview>(`/api/courses/${courseId}/progress`),
  nextQuestion: (courseId: string, objectiveId?: string) => {
    const suffix = objectiveId
      ? `?objective_id=${encodeURIComponent(objectiveId)}`
      : "";
    return request<Question>(`/api/courses/${courseId}/questions/next${suffix}`);
  },
  submitAttempt: (
    questionId: string,
    selectedOptionIds: string[],
    confidence: AnswerConfidence,
    reasoningChoice: ReasoningChoice,
    reasoning: string,
  ) =>
    request<AttemptResult>(`/api/questions/${questionId}/attempts`, {
      method: "POST",
      body: JSON.stringify({
        selected_option_ids: selectedOptionIds,
        confidence,
        reasoning_choice: reasoningChoice,
        reasoning,
      }),
    }),
  glossary: (courseId?: string, query?: string) => {
    const params = new URLSearchParams();
    if (courseId) params.set("course_id", courseId);
    if (query) params.set("query", query);
    const suffix = params.size ? `?${params.toString()}` : "";
    return request<GlossaryTerm[]>(`/api/glossary${suffix}`);
  },
  tutor: (courseId: string, learningObjectiveId: string | null, message: string) =>
    request<TutorResponse>("/api/tutor/messages", {
      method: "POST",
      body: JSON.stringify({
        course_id: courseId,
        learning_objective_id: learningObjectiveId,
        message,
      }),
    }),
};
