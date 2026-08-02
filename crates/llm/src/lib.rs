use async_trait::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub response_schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct EmbeddingRequest {
    pub inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub provider: String,
    pub model: String,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub configured: bool,
    pub reachable: bool,
    pub provider: String,
    pub model: Option<String>,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Kein Chatmodell ist konfiguriert.")]
    NotConfigured,
    #[error("Der Modellprovider ist nicht erreichbar: {0}")]
    Unreachable(String),
    #[error("Der Modellprovider antwortete mit HTTP {status}: {body}")]
    Http { status: StatusCode, body: String },
    #[error("Die Modellantwort hatte ein unerwartetes Format: {0}")]
    InvalidResponse(String),
}

#[async_trait]
pub trait ChatProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn model(&self) -> Option<&str>;
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, LlmError>;
    async fn health(&self) -> ProviderHealth;
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn model(&self) -> Option<&str>;
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, LlmError>;
    async fn health(&self) -> ProviderHealth;
}

#[derive(Clone)]
pub struct OllamaChatProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OllamaChatProvider {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl ChatProvider for OllamaChatProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn model(&self) -> Option<&str> {
        (!self.model.is_empty()).then_some(self.model.as_str())
    }

    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        if self.model.is_empty() {
            return Err(LlmError::NotConfigured);
        }

        let mut payload = json!({
            "model": self.model,
            "messages": request.messages,
            "stream": false,
            "options": { "temperature": request.temperature }
        });
        if let Some(schema) = request.response_schema {
            payload["format"] = schema;
        }

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|error| LlmError::Unreachable(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
        if !status.is_success() {
            return Err(LlmError::Http { status, body });
        }

        let value: Value = serde_json::from_str(&body)
            .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
        let content = value["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::InvalidResponse("message.content fehlt".into()))?
            .to_owned();

        Ok(ChatResponse {
            content,
            provider: self.name().into(),
            model: self.model.clone(),
            prompt_tokens: value["prompt_eval_count"].as_u64(),
            completion_tokens: value["eval_count"].as_u64(),
        })
    }

    async fn health(&self) -> ProviderHealth {
        if self.model.is_empty() {
            return ProviderHealth {
                configured: false,
                reachable: false,
                provider: self.name().into(),
                model: None,
                message: "Ollama ist eingetragen, aber CHAT_MODEL ist leer.".into(),
            };
        }

        match self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let body: Value = response.json().await.unwrap_or_else(|_| json!({}));
                let installed = body["models"]
                    .as_array()
                    .map(|models| {
                        models.iter().any(|entry| {
                            entry["name"].as_str() == Some(self.model.as_str())
                                || entry["model"].as_str() == Some(self.model.as_str())
                        })
                    })
                    .unwrap_or(false);
                ProviderHealth {
                    configured: true,
                    reachable: installed,
                    provider: self.name().into(),
                    model: Some(self.model.clone()),
                    message: if installed {
                        "Ollama und das konfigurierte Modell sind verfügbar.".into()
                    } else {
                        format!(
                            "Ollama ist erreichbar, aber das Modell '{}' ist nicht installiert.",
                            self.model
                        )
                    },
                }
            }
            Ok(response) => ProviderHealth {
                configured: true,
                reachable: false,
                provider: self.name().into(),
                model: Some(self.model.clone()),
                message: format!("Ollama antwortet mit HTTP {}.", response.status()),
            },
            Err(error) => ProviderHealth {
                configured: true,
                reachable: false,
                provider: self.name().into(),
                model: Some(self.model.clone()),
                message: format!("Ollama ist nicht erreichbar: {error}"),
            },
        }
    }
}

#[derive(Clone)]
pub struct OpenAiCompatibleChatProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiCompatibleChatProvider {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    fn authorize(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.api_key.is_empty() {
            request
        } else {
            request.bearer_auth(&self.api_key)
        }
    }
}

#[async_trait]
impl ChatProvider for OpenAiCompatibleChatProvider {
    fn name(&self) -> &'static str {
        "openai_compatible"
    }

    fn model(&self) -> Option<&str> {
        (!self.model.is_empty()).then_some(self.model.as_str())
    }

    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        if self.model.is_empty() {
            return Err(LlmError::NotConfigured);
        }

        let mut payload = json!({
            "model": self.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "stream": false
        });
        if let Some(schema) = request.response_schema {
            payload["response_format"] = json!({
                "type": "json_schema",
                "json_schema": { "name": "learnistqb_response", "strict": true, "schema": schema }
            });
        }

        let response = self
            .authorize(
                self.client
                    .post(format!("{}/chat/completions", self.base_url)),
            )
            .json(&payload)
            .send()
            .await
            .map_err(|error| LlmError::Unreachable(error.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
        if !status.is_success() {
            return Err(LlmError::Http { status, body });
        }

        let value: Value = serde_json::from_str(&body)
            .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
        let content = value["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::InvalidResponse("choices[0].message.content fehlt".into()))?
            .to_owned();

        Ok(ChatResponse {
            content,
            provider: self.name().into(),
            model: self.model.clone(),
            prompt_tokens: value["usage"]["prompt_tokens"].as_u64(),
            completion_tokens: value["usage"]["completion_tokens"].as_u64(),
        })
    }

    async fn health(&self) -> ProviderHealth {
        if self.model.is_empty() {
            return ProviderHealth {
                configured: false,
                reachable: false,
                provider: self.name().into(),
                model: None,
                message: "Kein gehostetes Chatmodell ist konfiguriert.".into(),
            };
        }

        match self
            .authorize(self.client.get(format!("{}/models", self.base_url)))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => ProviderHealth {
                configured: true,
                reachable: true,
                provider: self.name().into(),
                model: Some(self.model.clone()),
                message: "Der gehostete Modellprovider ist erreichbar.".into(),
            },
            Ok(response) => ProviderHealth {
                configured: true,
                reachable: false,
                provider: self.name().into(),
                model: Some(self.model.clone()),
                message: format!("Der Provider antwortet mit HTTP {}.", response.status()),
            },
            Err(error) => ProviderHealth {
                configured: true,
                reachable: false,
                provider: self.name().into(),
                model: Some(self.model.clone()),
                message: format!("Der Provider ist nicht erreichbar: {error}"),
            },
        }
    }
}

#[derive(Clone)]
pub struct OllamaEmbeddingProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OllamaEmbeddingProvider {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn model(&self) -> Option<&str> {
        (!self.model.is_empty()).then_some(self.model.as_str())
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, LlmError> {
        if self.model.is_empty() {
            return Err(LlmError::NotConfigured);
        }
        let response = self
            .client
            .post(format!("{}/api/embed", self.base_url))
            .json(&json!({ "model": self.model, "input": request.inputs }))
            .send()
            .await
            .map_err(|error| LlmError::Unreachable(error.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
        if !status.is_success() {
            return Err(LlmError::Http { status, body });
        }
        let value: Value = serde_json::from_str(&body)
            .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
        let embeddings = serde_json::from_value(value["embeddings"].clone())
            .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
        Ok(EmbeddingResponse {
            embeddings,
            provider: self.name().into(),
            model: self.model.clone(),
            total_tokens: value["prompt_eval_count"].as_u64(),
        })
    }

    async fn health(&self) -> ProviderHealth {
        provider_health_from_ollama(
            &self.client,
            &self.base_url,
            &self.model,
            self.name(),
            "EMBEDDING_MODEL",
        )
        .await
    }
}

#[derive(Clone)]
pub struct OpenAiCompatibleEmbeddingProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiCompatibleEmbeddingProvider {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    fn authorize(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.api_key.is_empty() {
            request
        } else {
            request.bearer_auth(&self.api_key)
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiCompatibleEmbeddingProvider {
    fn name(&self) -> &'static str {
        "openai_compatible"
    }

    fn model(&self) -> Option<&str> {
        (!self.model.is_empty()).then_some(self.model.as_str())
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, LlmError> {
        if self.model.is_empty() {
            return Err(LlmError::NotConfigured);
        }
        let response = self
            .authorize(self.client.post(format!("{}/embeddings", self.base_url)))
            .json(&json!({ "model": self.model, "input": request.inputs }))
            .send()
            .await
            .map_err(|error| LlmError::Unreachable(error.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
        if !status.is_success() {
            return Err(LlmError::Http { status, body });
        }
        let value: Value = serde_json::from_str(&body)
            .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
        let data = value["data"]
            .as_array()
            .ok_or_else(|| LlmError::InvalidResponse("data fehlt".into()))?;
        let mut embeddings = Vec::with_capacity(data.len());
        for entry in data {
            embeddings.push(
                serde_json::from_value(entry["embedding"].clone())
                    .map_err(|error| LlmError::InvalidResponse(error.to_string()))?,
            );
        }
        Ok(EmbeddingResponse {
            embeddings,
            provider: self.name().into(),
            model: self.model.clone(),
            total_tokens: value["usage"]["total_tokens"].as_u64(),
        })
    }

    async fn health(&self) -> ProviderHealth {
        if self.model.is_empty() {
            return ProviderHealth {
                configured: false,
                reachable: false,
                provider: self.name().into(),
                model: None,
                message: "Kein gehostetes Embeddingmodell ist konfiguriert.".into(),
            };
        }
        match self
            .authorize(self.client.get(format!("{}/models", self.base_url)))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => ProviderHealth {
                configured: true,
                reachable: true,
                provider: self.name().into(),
                model: Some(self.model.clone()),
                message: "Der gehostete Embeddingprovider ist erreichbar.".into(),
            },
            Ok(response) => ProviderHealth {
                configured: true,
                reachable: false,
                provider: self.name().into(),
                model: Some(self.model.clone()),
                message: format!("Der Provider antwortet mit HTTP {}.", response.status()),
            },
            Err(error) => ProviderHealth {
                configured: true,
                reachable: false,
                provider: self.name().into(),
                model: Some(self.model.clone()),
                message: format!("Der Provider ist nicht erreichbar: {error}"),
            },
        }
    }
}

async fn provider_health_from_ollama(
    client: &reqwest::Client,
    base_url: &str,
    model: &str,
    provider: &str,
    setting: &str,
) -> ProviderHealth {
    if model.is_empty() {
        return ProviderHealth {
            configured: false,
            reachable: false,
            provider: provider.into(),
            model: None,
            message: format!("Ollama ist eingetragen, aber {setting} ist leer."),
        };
    }
    match client.get(format!("{base_url}/api/tags")).send().await {
        Ok(response) if response.status().is_success() => {
            let body: Value = response.json().await.unwrap_or_else(|_| json!({}));
            let installed = body["models"]
                .as_array()
                .map(|models| {
                    models.iter().any(|entry| {
                        entry["name"].as_str() == Some(model)
                            || entry["model"].as_str() == Some(model)
                    })
                })
                .unwrap_or(false);
            ProviderHealth {
                configured: true,
                reachable: installed,
                provider: provider.into(),
                model: Some(model.into()),
                message: if installed {
                    "Ollama und das konfigurierte Modell sind verfügbar.".into()
                } else {
                    format!(
                        "Ollama ist erreichbar, aber das Modell '{model}' ist nicht installiert."
                    )
                },
            }
        }
        Ok(response) => ProviderHealth {
            configured: true,
            reachable: false,
            provider: provider.into(),
            model: Some(model.into()),
            message: format!("Ollama antwortet mit HTTP {}.", response.status()),
        },
        Err(error) => ProviderHealth {
            configured: true,
            reachable: false,
            provider: provider.into(),
            model: Some(model.into()),
            message: format!("Ollama ist nicht erreichbar: {error}"),
        },
    }
}

#[derive(Clone, Default)]
pub struct FakeChatProvider;

#[async_trait]
impl ChatProvider for FakeChatProvider {
    fn name(&self) -> &'static str {
        "fake"
    }

    fn model(&self) -> Option<&str> {
        Some("deterministic-test-model")
    }

    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        let has_question = request
            .messages
            .last()
            .is_some_and(|message| !message.content.trim().is_empty());
        let content = if has_question {
            "Im Testmodus arbeite ich mit dem gefundenen Kontext: Ordne zuerst die Kernaussage aus [Quelle 1] ein und übertrage sie dann auf ein eigenes, konkretes Beispiel. Wichtig ist, aus der Quelle keine allgemeinere Regel abzuleiten, als dort tatsächlich belegt ist.\n\nPrüffrage: Welche Formulierung in [Quelle 1] trägt deine Antwort?"
        } else {
            "Im Testmodus fehlt eine Frage des Lernenden."
        };
        Ok(ChatResponse {
            content: content.into(),
            provider: self.name().into(),
            model: self.model().unwrap_or_default().into(),
            prompt_tokens: Some(1),
            completion_tokens: Some(1),
        })
    }

    async fn health(&self) -> ProviderHealth {
        ProviderHealth {
            configured: true,
            reachable: true,
            provider: self.name().into(),
            model: self.model().map(str::to_owned),
            message: "Der Testprovider ist verfügbar.".into(),
        }
    }
}

#[derive(Clone, Default)]
pub struct FakeEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for FakeEmbeddingProvider {
    fn name(&self) -> &'static str {
        "fake"
    }

    fn model(&self) -> Option<&str> {
        Some("deterministic-test-embedding")
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, LlmError> {
        let embeddings = request
            .inputs
            .iter()
            .map(|input| {
                let mut vector = vec![0.0_f32; 8];
                for (index, byte) in input.bytes().enumerate() {
                    vector[index % 8] += f32::from(byte) / 255.0;
                }
                vector
            })
            .collect();
        Ok(EmbeddingResponse {
            embeddings,
            provider: self.name().into(),
            model: self.model().unwrap_or_default().into(),
            total_tokens: Some(1),
        })
    }

    async fn health(&self) -> ProviderHealth {
        ProviderHealth {
            configured: true,
            reachable: true,
            provider: self.name().into(),
            model: self.model().map(str::to_owned),
            message: "Der Test-Embeddingprovider ist verfügbar.".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_provider_is_deterministic() {
        let provider = FakeChatProvider;
        let response = provider
            .complete(ChatRequest {
                messages: vec![ChatMessage::user("Was ist Testen?")],
                temperature: 0.0,
                response_schema: None,
            })
            .await
            .unwrap();

        assert_eq!(response.provider, "fake");
        assert!(response.content.contains("[Quelle 1]"));
    }

    #[tokio::test]
    async fn fake_embedding_provider_preserves_input_count() {
        let provider = FakeEmbeddingProvider;
        let response = provider
            .embed(EmbeddingRequest {
                inputs: vec!["Testen".into(), "Qualitätsrisiko".into()],
            })
            .await
            .unwrap();

        assert_eq!(response.embeddings.len(), 2);
        assert!(response.embeddings.iter().all(|vector| vector.len() == 8));
    }
}
