//! OpenRouter preset.
//!
//! Env:
//! - `OPENROUTER_API_KEY` — required
//! - `OPENROUTER_MODEL` — default `openrouter/auto`

use anyhow::{Context, Result};

use super::openai_compat::OpenAiCompatProvider;

/// Build an OpenRouter provider from the environment.
pub fn from_env() -> Result<OpenAiCompatProvider> {
    let api_key = std::env::var("OPENROUTER_API_KEY").context("OPENROUTER_API_KEY is not set")?;
    let model = std::env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "openrouter/auto".to_string());
    Ok(OpenAiCompatProvider::new(
        "openrouter",
        "https://openrouter.ai/api/v1",
        Some(api_key),
        model,
    ))
}
