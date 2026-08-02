use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use domain::{
    AttemptResult, AttemptSubmission, CourseDetail, CourseSummary, GlossaryTerm, ProgressOverview,
    ProviderStatus, Question, SystemStatus, TermAttemptResult, TermAttemptSubmission,
    TermDirection, TermExercise, TutorRequest, TutorResponse,
};
use llm::{
    ChatMessage, ChatProvider, ChatRequest, EmbeddingProvider, FakeChatProvider,
    FakeEmbeddingProvider, LlmError, OllamaChatProvider, OllamaEmbeddingProvider,
    OpenAiCompatibleChatProvider, OpenAiCompatibleEmbeddingProvider,
};
use persistence::{Database, PersistenceError};
use serde::{Deserialize, Serialize};
use std::{env, path::Path as FilePath, sync::Arc, time::Instant};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    database: Database,
    chat: Arc<dyn ChatProvider>,
    embedding: Arc<dyn EmbeddingProvider>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let bind = env::var("APP_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/learnistqb.db".into());
    ensure_database_parent(&database_url)?;
    let database = Database::connect(&database_url)
        .await
        .context("Datenbank konnte nicht initialisiert werden")?;
    let chat = chat_provider_from_environment();
    let embedding = embedding_provider_from_environment();
    let state = AppState {
        database,
        chat,
        embedding,
    };
    let frontend_dir = env::var("FRONTEND_DIR").unwrap_or_else(|_| "web/dist".into());

    let api = Router::new()
        .route("/system/status", get(system_status))
        .route("/courses", get(list_courses))
        .route("/courses/{course_id}", get(course_detail))
        .route("/courses/{course_id}/progress", get(course_progress))
        .route("/courses/{course_id}/questions/next", get(next_question))
        .route("/courses/{course_id}/terms/next", get(next_term_exercise))
        .route("/questions/{question_id}/attempts", post(submit_attempt))
        .route("/terms/{term_id}/attempts", post(submit_term_attempt))
        .route("/glossary", get(glossary))
        .route("/tutor/messages", post(tutor_message));

    let index_file = format!("{frontend_dir}/index.html");
    let static_files = ServeDir::new(&frontend_dir).fallback(ServeFile::new(index_file));
    let app = Router::new()
        .nest("/api", api)
        .fallback_service(static_files)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("Server konnte nicht an {bind} gebunden werden"))?;
    info!(address = %bind, "learnISTQB ist gestartet");
    axum::serve(listener, app).await?;
    Ok(())
}

fn ensure_database_parent(database_url: &str) -> Result<()> {
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        if path != ":memory:" {
            if let Some(parent) = FilePath::new(path).parent() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }
    Ok(())
}

fn chat_provider_from_environment() -> Arc<dyn ChatProvider> {
    let provider = env::var("CHAT_PROVIDER").unwrap_or_else(|_| "ollama".into());
    let base_url = env::var("CHAT_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".into());
    let model = env::var("CHAT_MODEL").unwrap_or_default();
    match provider.as_str() {
        "fake" => Arc::new(FakeChatProvider),
        "openai_compatible" => Arc::new(OpenAiCompatibleChatProvider::new(
            base_url,
            env::var("CHAT_API_KEY").unwrap_or_default(),
            model,
        )),
        _ => Arc::new(OllamaChatProvider::new(base_url, model)),
    }
}

fn embedding_provider_from_environment() -> Arc<dyn EmbeddingProvider> {
    let provider = env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "ollama".into());
    let base_url =
        env::var("EMBEDDING_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".into());
    let model = env::var("EMBEDDING_MODEL").unwrap_or_default();
    match provider.as_str() {
        "fake" => Arc::new(FakeEmbeddingProvider),
        "openai_compatible" => Arc::new(OpenAiCompatibleEmbeddingProvider::new(
            base_url,
            env::var("EMBEDDING_API_KEY").unwrap_or_default(),
            model,
        )),
        _ => Arc::new(OllamaEmbeddingProvider::new(base_url, model)),
    }
}

async fn system_status(State(state): State<AppState>) -> Result<Json<SystemStatus>, AppError> {
    let (courses, glossary_terms) = state.database.counts().await?;
    let (chat_health, embedding_health) =
        tokio::join!(state.chat.health(), state.embedding.health());
    Ok(Json(SystemStatus {
        app: "learnISTQB".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        database: "ready".into(),
        courses,
        glossary_terms,
        chat: ProviderStatus {
            configured: chat_health.configured,
            reachable: chat_health.reachable,
            provider: chat_health.provider,
            model: chat_health.model,
            message: chat_health.message,
        },
        embedding: ProviderStatus {
            configured: embedding_health.configured,
            reachable: embedding_health.reachable,
            provider: embedding_health.provider,
            model: embedding_health.model,
            message: embedding_health.message,
        },
    }))
}

async fn list_courses(State(state): State<AppState>) -> Result<Json<Vec<CourseSummary>>, AppError> {
    Ok(Json(state.database.list_courses().await?))
}

async fn course_detail(
    State(state): State<AppState>,
    Path(course_id): Path<String>,
) -> Result<Json<CourseDetail>, AppError> {
    Ok(Json(state.database.course(&course_id).await?))
}

async fn course_progress(
    State(state): State<AppState>,
    Path(course_id): Path<String>,
) -> Result<Json<ProgressOverview>, AppError> {
    Ok(Json(state.database.progress(&course_id).await?))
}

#[derive(Debug, Deserialize)]
struct QuestionQuery {
    objective_id: Option<String>,
}

async fn next_question(
    State(state): State<AppState>,
    Path(course_id): Path<String>,
    Query(query): Query<QuestionQuery>,
) -> Result<Json<Question>, AppError> {
    Ok(Json(
        state
            .database
            .next_question(&course_id, query.objective_id.as_deref())
            .await?,
    ))
}

async fn submit_attempt(
    State(state): State<AppState>,
    Path(question_id): Path<String>,
    Json(submission): Json<AttemptSubmission>,
) -> Result<Json<AttemptResult>, AppError> {
    if submission.selected_option_ids.is_empty() {
        return Err(AppError::BadRequest(
            "Wähle mindestens eine Antwortoption aus.".into(),
        ));
    }
    Ok(Json(
        state
            .database
            .submit_attempt(&question_id, submission)
            .await?,
    ))
}

#[derive(Debug, Deserialize)]
struct TermExerciseQuery {
    direction: Option<TermDirection>,
}

async fn next_term_exercise(
    State(state): State<AppState>,
    Path(course_id): Path<String>,
    query: std::result::Result<Query<TermExerciseQuery>, QueryRejection>,
) -> Result<Json<TermExercise>, AppError> {
    let Query(query) =
        query.map_err(|_| AppError::BadRequest("Unbekannte Abfragerichtung.".into()))?;
    Ok(Json(
        state
            .database
            .next_term_exercise(&course_id, query.direction)
            .await?,
    ))
}

async fn submit_term_attempt(
    State(state): State<AppState>,
    Path(term_id): Path<String>,
    Json(submission): Json<TermAttemptSubmission>,
) -> Result<Json<TermAttemptResult>, AppError> {
    if submission.selected_option_id.trim().is_empty() {
        return Err(AppError::BadRequest("Wähle eine Antwortoption aus.".into()));
    }
    Ok(Json(
        state
            .database
            .submit_term_attempt(&term_id, submission)
            .await?,
    ))
}

#[derive(Debug, Deserialize)]
struct GlossaryQuery {
    course_id: Option<String>,
    query: Option<String>,
}

async fn glossary(
    State(state): State<AppState>,
    Query(query): Query<GlossaryQuery>,
) -> Result<Json<Vec<GlossaryTerm>>, AppError> {
    Ok(Json(
        state
            .database
            .glossary(query.course_id.as_deref(), query.query.as_deref())
            .await?,
    ))
}

async fn tutor_message(
    State(state): State<AppState>,
    Json(request): Json<TutorRequest>,
) -> Result<Json<TutorResponse>, AppError> {
    if request.message.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Die Nachricht an den Tutor ist leer.".into(),
        ));
    }

    let started = Instant::now();
    let contexts = state
        .database
        .search_context(&request.course_id, &request.message, 5)
        .await?;
    if contexts.is_empty() {
        return Err(AppError::Unprocessable(
            "In diesem Kurs wurden keine tragfähigen Quellen für die Frage gefunden.".into(),
        ));
    }

    let context_text = contexts
        .iter()
        .enumerate()
        .map(|(index, context)| {
            format!(
                "[Quelle {} | {} | {}]\n{}",
                index + 1,
                context.source.label,
                context
                    .source
                    .section
                    .as_deref()
                    .unwrap_or("ohne Abschnitt"),
                context.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "KURS: {}\nLERNZIEL: {}\n\nQUELLEN:\n{}\n\nFRAGE DES LERNENDEN:\n{}",
        request.course_id,
        request
            .learning_objective_id
            .as_deref()
            .unwrap_or("nicht festgelegt"),
        context_text,
        request.message
    );
    let response = state
        .chat
        .complete(ChatRequest {
            messages: vec![
                ChatMessage::system(
                    "Du bist der lernzielorientierte Tutor von learnISTQB. Antworte auf Deutsch, knapp und verständlich. Verwende ausschließlich die bereitgestellten Quellen für Aussagen über ISTQB. Erfinde keine offiziellen Regeln oder Definitionen. Wenn die Quellen nicht ausreichen, sage das ausdrücklich. Verweise im Text mit [Quelle 1], [Quelle 2] usw. auf die bereitgestellten Quellen. Stelle höchstens eine sokratische Rückfrage.",
                ),
                ChatMessage::user(prompt),
            ],
            temperature: 0.2,
            response_schema: None,
        })
        .await;

    match response {
        Ok(response) => {
            info!(
                use_case = "tutor_message",
                provider = %response.provider,
                model = %response.model,
                duration_ms = started.elapsed().as_millis(),
                source_count = contexts.len(),
                "LLM-Aufruf abgeschlossen"
            );
            Ok(Json(TutorResponse {
                answer: response.content,
                citations: contexts.into_iter().map(|context| context.source).collect(),
                provider: response.provider,
                model: response.model,
            }))
        }
        Err(error) => {
            error!(
                use_case = "tutor_message",
                duration_ms = started.elapsed().as_millis(),
                %error,
                "LLM-Aufruf fehlgeschlagen"
            );
            Err(AppError::Llm(error))
        }
    }
}

#[derive(Debug)]
enum AppError {
    Persistence(PersistenceError),
    Llm(LlmError),
    BadRequest(String),
    Unprocessable(String),
}

impl From<PersistenceError> for AppError {
    fn from(value: PersistenceError) -> Self {
        Self::Persistence(value)
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Persistence(PersistenceError::NotFound(message)) => {
                (StatusCode::NOT_FOUND, message)
            }
            Self::Persistence(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            Self::Llm(LlmError::NotConfigured) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Der Tutor ist noch nicht konfiguriert. Trage CHAT_MODEL ein und starte Ollama."
                    .into(),
            ),
            Self::Llm(error) => (StatusCode::SERVICE_UNAVAILABLE, error.to_string()),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Unprocessable(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
        };
        (status, Json(ErrorBody { error: message })).into_response()
    }
}
