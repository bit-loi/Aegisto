//! Generic OpenAI-compatible chat client.
//!
//! Works with any endpoint that speaks the `/chat/completions` protocol:
//! Ollama `/v1`, Groq, OpenRouter, Google AI Studio, and so on.

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::llm::provider::{ChatRequest, ChatResponse, LlmProvider, TokenUsage};

/// OpenAI-compatible chat completions provider.
#[derive(Debug, Clone)]
pub struct OpenAiCompatProvider {
    pub name: &'static str,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    client: Client,
}

impl OpenAiCompatProvider {
    pub fn new(
        name: &'static str,
        base_url: impl Into<String>,
        api_key: Option<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            name,
            base_url: base_url.into(),
            api_key,
            model: model.into(),
            client: Client::new(),
        }
    }

    /// Build the custom endpoint from `AEGISTO_BASE_URL` / `AEGISTO_API_KEY` /
    /// `AEGISTO_MODEL`.
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("AEGISTO_BASE_URL")
            .with_context(|| "AEGISTO_BASE_URL is required for the openai-compat provider")?;
        let api_key = std::env::var("AEGISTO_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());
        let model = std::env::var("AEGISTO_MODEL")
            .with_context(|| "AEGISTO_MODEL is required for the openai-compat provider")?;
        Ok(Self::new("openai-compat", base_url, api_key, model))
    }

    /// Shared constructor for the provider presets: optional base URL and key
    /// env vars, with sensible defaults and a required model env var override.
    pub(crate) fn from_parts(
        name: &'static str,
        env_base: &str,
        env_key: &str,
        env_model: &str,
        default_base: &str,
        default_model: &str,
    ) -> Self {
        let base_url = std::env::var(env_base).unwrap_or_else(|_| default_base.to_string());
        let api_key = std::env::var(env_key).ok().filter(|k| !k.is_empty());
        let model = std::env::var(env_model).unwrap_or_else(|_| default_model.to_string());
        Self::new(name, base_url, api_key, model)
    }
}

#[derive(Serialize)]
struct WireRequest {
    model: String,
    messages: Vec<WireMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct WireMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct WireResponse {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<WireChoice>,
    #[serde(default)]
    usage: Option<WireUsage>,
}

#[derive(Deserialize)]
struct WireChoice {
    message: WireMessage,
}

#[derive(Deserialize)]
struct WireUsage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

#[async_trait]
impl LlmProvider for OpenAiCompatProvider {
    fn id(&self) -> &'static str {
        self.name
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let messages: Vec<WireMessage> = request
            .messages
            .iter()
            .map(|m| WireMessage {
                role: m.role.as_str().to_string(),
                content: m.content.clone(),
            })
            .collect();

        let mut builder = self.client.post(&url).json(&WireRequest {
            model: request.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        });
        if let Some(key) = &self.api_key {
            builder = builder.header("Authorization", format!("Bearer {key}"));
        }

        let wire: WireResponse = builder
            .send()
            .await
            .context("failed to reach LLM provider")?
            .error_for_status()
            .context("LLM provider returned an error status")?
            .json()
            .await
            .context("failed to decode chat completion response")?;

        let content = wire
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();
        Ok(ChatResponse {
            content,
            model: wire.model.unwrap_or(request.model),
            usage: wire.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_completions_url_is_built_without_double_slash() {
        let p = OpenAiCompatProvider::new("test", "http://localhost:11434/v1/", None, "m");
        let url = format!("{}/chat/completions", p.base_url.trim_end_matches('/'));
        assert_eq!(url, "http://localhost:11434/v1/chat/completions");
    }
}
