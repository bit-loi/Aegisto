//! Aegisto — entry point.
//!
//! The whole application (app state, event loop, rendering, input, splash)
//! lives in the `aegisto-tui` crate; static analysis lives in `aegisto-core`;
//! the AI agent layer (LLM providers) lives in `aegisto-agent`.

use anyhow::Result;

fn main() -> Result<()> {
    aegisto_tui::run()
}
