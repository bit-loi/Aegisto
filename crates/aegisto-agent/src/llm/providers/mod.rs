//! Provider presets and the env-based factory.

pub mod google;
pub mod groq;
pub mod ollama;
pub mod openai_compat;
pub mod openrouter;

use std::str::FromStr;

use anyhow::{Context, Result};

use crate::llm::provider::LlmProvider;

/// Supported provider backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Ollama,
    Groq,
    GoogleAiStudio,
    OpenRouter,
    /// Any OpenAI-compatible endpoint via `AEGISTO_BASE_URL`.
    OpenAiCompat,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::Ollama => "ollama",
            ProviderKind::Groq => "groq",
            ProviderKind::GoogleAiStudio => "google",
            ProviderKind::OpenRouter => "openrouter",
            ProviderKind::OpenAiCompat => "openai-compat",
        }
    }
}

impl FromStr for ProviderKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "ollama" => Ok(ProviderKind::Ollama),
            "groq" => Ok(ProviderKind::Groq),
            "google" | "google-ai-studio" | "gemini" => Ok(ProviderKind::GoogleAiStudio),
            "openrouter" => Ok(ProviderKind::OpenRouter),
            "openai" | "openai-compat" | "custom" => Ok(ProviderKind::OpenAiCompat),
            other => anyhow::bail!(
                "unknown provider '{other}' (expected: ollama, groq, google, openrouter, openai-compat)"
            ),
        }
    }
}

/// Load the provider selected by the `AEGISTO_PROVIDER` environment variable.
///
/// Returns `Ok(None)` when the variable is unset (no AI configured yet).
pub fn load_provider_from_env() -> Result<Option<Box<dyn LlmProvider>>> {
    let Ok(raw) = std::env::var("AEGISTO_PROVIDER") else {
        return Ok(None);
    };
    let kind: ProviderKind = raw.parse().with_context(|| "invalid AEGISTO_PROVIDER")?;
    let provider: Box<dyn LlmProvider> = match kind {
        ProviderKind::Ollama => Box::new(ollama::from_env()?),
        ProviderKind::Groq => Box::new(groq::from_env()?),
        ProviderKind::GoogleAiStudio => Box::new(google::from_env()?),
        ProviderKind::OpenRouter => Box::new(openrouter::from_env()?),
        ProviderKind::OpenAiCompat => Box::new(openai_compat::OpenAiCompatProvider::from_env()?),
    };
    Ok(Some(provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_provider_kinds() {
        assert_eq!(
            "ollama".parse::<ProviderKind>().unwrap(),
            ProviderKind::Ollama
        );
        assert_eq!("Groq".parse::<ProviderKind>().unwrap(), ProviderKind::Groq);
        assert_eq!(
            "gemini".parse::<ProviderKind>().unwrap(),
            ProviderKind::GoogleAiStudio
        );
        assert_eq!(
            "openrouter".parse::<ProviderKind>().unwrap(),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            "openai-compat".parse::<ProviderKind>().unwrap(),
            ProviderKind::OpenAiCompat
        );
        assert!("nope".parse::<ProviderKind>().is_err());
    }
}
