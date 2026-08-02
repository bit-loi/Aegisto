//! Chat-completions request/response types and the [`LlmProvider`] trait.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Message role in a chat conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    /// OpenAI-compatible role string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

/// One chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// A chat completion request.
#[derive(Debug, Clone, Default)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl ChatRequest {
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
        }
    }
}

/// Token usage reported by the provider (when available).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// A chat completion response.
#[derive(Debug, Clone, Default)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

/// A chat-completions LLM backend. Every supported provider implements this
/// trait; most are built on the OpenAI-compatible HTTP API.
#[async_trait]
pub trait LlmProvider: Send + Sync + std::fmt::Debug {
    /// Stable provider id, e.g. `"ollama"`, `"groq"`.
    fn id(&self) -> &'static str;

    /// Active model name.
    fn model(&self) -> &str;

    /// Send a chat completion request and return the reply.
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
}
