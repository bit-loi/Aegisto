//! Google AI Studio (Gemini) preset via the OpenAI-compatible endpoint.
//!
//! Env:
//! - `GOOGLE_API_KEY` (or `GEMINI_API_KEY`) — required
//! - `GOOGLE_MODEL` — default `gemini-2.0-flash`

use anyhow::{Context, Result};

use super::openai_compat::OpenAiCompatProvider;

/// Build a Google AI Studio provider from the environment.
pub fn from_env() -> Result<OpenAiCompatProvider> {
    let api_key = std::env::var("GOOGLE_API_KEY")
        .or_else(|_| std::env::var("GEMINI_API_KEY"))
        .context("GOOGLE_API_KEY (or GEMINI_API_KEY) is not set")?;
    let model = std::env::var("GOOGLE_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".to_string());
    Ok(OpenAiCompatProvider::new(
        "google",
        "https://generativelanguage.googleapis.com/v1beta/openai",
        Some(api_key),
        model,
    ))
}
