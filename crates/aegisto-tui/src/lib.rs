//! Aegisto TUI application: app state & event loop, ratatui rendering, key
//! handling, splash screen and display helpers.

pub mod app;
pub mod format;
pub mod input;
pub mod render;
pub mod splash;

pub use app::run;
