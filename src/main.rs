//! Aegisto TUI — entry point.
//!
//! Layout:
//!   - Left panel:  file/folder listing of the current directory
//!   - Right panel: details of the highlighted file + analysis log
//!   - Bottom bar:  command input (like Claude Code's `:analyze`)
//!
//! The `:analyze` command runs the *real* static pipeline
//! (goblin parsing + iced-x86 disassembly + string extraction) on the
//! highlighted binary — no fake logs.
//!
//! Module map:
//!   - app     application state & logic (browsing, commands, analysis)
//!   - ui      rendering of the three panels
//!   - input   key handling (navigation + command input)
//!   - format  small display helpers

mod app;
mod disasm;
mod format;
mod input;
mod parser;
mod splash;
mod strings;
mod types;
mod ui;

use std::io;

use anyhow::Result;
use crossterm::{
    cursor::Show,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;

/// Restores the terminal when dropped, so every exit path (including errors
/// and panics) leaves the user's shell usable. Same pattern as splash.rs.
struct ScreenGuard;

impl ScreenGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        if let Err(e) = execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(e.into());
        }
        Ok(Self)
    }
}

impl Drop for ScreenGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            Show,
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = disable_raw_mode();
    }
}

fn main() -> Result<()> {
    // Animated splash (4s, press q to skip; auto-skipped when output is piped).
    match splash::run()? {
        splash::SplashOutcome::Aborted => std::process::exit(130),
        splash::SplashOutcome::Skipped | splash::SplashOutcome::Completed => {}
    }

    let guard = ScreenGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    while !app.quit {
        // Show/hide the cursor and position it inside the input bar.
        if app.input_mode {
            terminal.show_cursor()?;
            let x = 4 + app.input_buffer.chars().count() as u16; // margin + border + "> " prefix
            let y = terminal.size()?.height.saturating_sub(3);
            terminal.set_cursor_position((x, y))?;
        } else {
            terminal.hide_cursor()?;
        }

        terminal.draw(|f| ui::render(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if app.input_mode {
                    input::handle_input_key(&mut app, key);
                } else {
                    input::handle_nav_key(&mut app, key);
                }
            }
        }
    }

    drop(guard); // restore the terminal before printing the farewell
    println!("✅ Aegisto exited cleanly.");
    Ok(())
}
