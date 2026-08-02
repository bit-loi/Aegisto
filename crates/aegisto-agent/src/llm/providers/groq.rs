//! Groq preset.
//!
//! Env:
//! - `GROQ_API_KEY` — required
//! - `GROQ_MODEL` — default `llama-3.3-70b-versatile`

use anyhow::{Context, Result};

use super::openai_compat::OpenAiCompatProvider;

/// Build a Groq provider from the environment.
pub fn from_env() -> Result<OpenAiCompatProvider> {
    let api_key = std::env::var("GROQ_API_KEY").context("GROQ_API_KEY is not set")?;
    let model =
        std::env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.3-70b-versatile".to_string());
    Ok(OpenAiCompatProvider::new(
        "groq",
        "https://api.groq.com/openai/v1",
        Some(api_key),
        model,
    ))
}
