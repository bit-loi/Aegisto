//! Aegisto TUI — entry point.
//!
//! Layout:
//!   - Left panel:  file/folder listing of the current directory
//!   - Right panel: details of the highlighted file + analysis log
//!   - Bottom bar:  command input (like Claude Code's `:analyze`)
//!
//! The `:analyze` command runs the *real* static pipeline (goblin parsing +
//! iced-x86 disassembly + string extraction) on the highlighted binary.
//!
//! Module map:
//!   - app       app state, logic & the main event loop
//!   - analysis  binary analysis (parser / disasm / strings)
//!   - ui        TUI rendering, key input, splash & formatting
//!   - types     shared data structures

mod analysis;
mod app;
mod types;
mod ui;
// mod agent; // TODO: aktifkan saat AI agent layer (Ollama / multi-agent) dikerjakan

use anyhow::Result;

fn main() -> Result<()> {
    app::run()
}
