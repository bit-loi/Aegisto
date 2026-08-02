//! Ollama preset (including the NVIDIA GPU build — same local endpoint).
//!
//! Ollama exposes an OpenAI-compatible API at `{OLLAMA_HOST}/v1`. The NVIDIA
//! build of Ollama runs the exact same local endpoint, so it needs no special
//! handling here — just pick an NVIDIA-flavored model tag via `OLLAMA_MODEL`.
//!
//! Env:
//! - `OLLAMA_HOST` — default `http://localhost:11434`
//! - `OLLAMA_API_KEY` — optional (local Ollama needs none)
//! - `OLLAMA_MODEL` — default `qwen2.5:7b`

use anyhow::Result;

use super::openai_compat::OpenAiCompatProvider;

/// Build an Ollama provider from the environment.
pub fn from_env() -> Result<OpenAiCompatProvider> {
    Ok(OpenAiCompatProvider::from_parts(
        "ollama",
        "OLLAMA_HOST",
        "OLLAMA_API_KEY",
        "OLLAMA_MODEL",
        "http://localhost:11434/v1",
        "qwen2.5:7b",
    ))
}
