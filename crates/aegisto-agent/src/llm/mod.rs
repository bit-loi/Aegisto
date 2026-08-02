//! LLM provider layer: a small OpenAI-compatible chat interface with
//! provider presets for Ollama, Groq, Google AI Studio and OpenRouter.

pub mod provider;
pub mod providers;

pub use provider::{ChatMessage, ChatRequest, ChatResponse, LlmProvider, Role, TokenUsage};
pub use providers::{ProviderKind, load_provider_from_env};
