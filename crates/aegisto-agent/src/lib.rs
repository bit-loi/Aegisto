//! Aegisto agent layer: multi-agent orchestration (Explorer / Systemizer /
//! Verifier) over pluggable LLM providers — Ollama (including the NVIDIA
//! build), Groq, Google AI Studio, OpenRouter, or any OpenAI-compatible
//! endpoint.
//!
//! This crate is scaffolding: the TUI does not call agents yet. The next
//! step is wiring it into `:analyze` (see the README roadmap).

pub mod agent;
pub mod llm;

pub use agent::{Agent, AgentInput, AgentKind, ExplorerAgent, SystemizerAgent, VerifierAgent};
pub use llm::{
    ChatMessage, ChatRequest, ChatResponse, LlmProvider, ProviderKind, Role, TokenUsage,
    load_provider_from_env,
};
