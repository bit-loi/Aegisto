//! TUI rendering: header banner, file browser, detail/log panel and the
//! command input bar.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::format::{format_size, truncate};

pub(crate) fn render(f: &mut Frame<'_>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main area (split left / right)
            Constraint::Length(3), // Input bar
        ])
        .split(f.area());

    // === HEADER ===
    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            "█████  ███████ ███████ ██ ███████ ████████  ██████",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "██   ██ ██      ██      ██ ██         ██    ██    ██",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "v0.1.0-alpha  ───  Standalone Binary AI Agent (TUI)",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(header, chunks[0]);

    // === MAIN: LEFT (file list) + RIGHT (detail / log) ===
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    // LEFT — file explorer
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|entry| {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let name = truncate(&entry.name, 28);
            let size = if entry.is_dir {
                String::new()
            } else {
                format!("  {}", format_size(entry.size))
            };
            let style = if entry.is_dir {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                // Highlight executables / common source files.
                match entry.path.extension().and_then(|e| e.to_str()) {
                    Some("exe") | Some("elf") | Some("bin") | Some("dll") => Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                    Some("rs") | Some("toml") | Some("md") | Some("json") => {
                        Style::default().fg(Color::Yellow)
                    }
                    _ => Style::default().fg(Color::White),
                }
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{icon} "), Style::default().fg(Color::DarkGray)),
                Span::styled(name, style),
                Span::styled(size, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let dir_title = truncate(&app.current_dir.display().to_string(), 42);
    let file_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" 📂 {dir_title} "))
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(file_list, main_chunks[0], &mut app.list_state);

    // RIGHT — detail / log (or command menu when open)
    let (right_lines, right_title, border_color) = if app.show_menu {
        (
            App::menu_lines().join("\n"),
            " 📋 Command Menu ",
            Color::Magenta,
        )
    } else {
        (app.info_lines.join("\n"), " 📋 Detail / Log ", Color::White)
    };
    let info_panel = Paragraph::new(right_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(right_title)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(info_panel, main_chunks[1]);

    // === INPUT BAR ===
    let input_style = if app.input_mode {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let input_content = if app.input_mode {
        if app.input_buffer.is_empty() {
            "Type a command… (Esc cancels)".to_string()
        } else {
            app.input_buffer.clone()
        }
    } else {
        "Press ':' for commands  ·  '?' for help menu".to_string()
    };
    let input_bar = Paragraph::new(Line::from(vec![
        Span::styled(
            if app.input_mode { "> " } else { ": " },
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(input_content, input_style),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" ⚡ Command ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(input_bar, chunks[2]);

    // Status overlay on the right side of the input bar. Skipped while the
    // user is typing (so it never covers the command text) and on tiny terminals.
    if !app.input_mode && chunks[2].height >= 2 {
        let status =
            Paragraph::new(app.status_message.clone()).style(Style::default().fg(Color::Green));
        let status_area = Rect::new(
            chunks[2].x + chunks[2].width.saturating_sub(46),
            chunks[2].y + 1,
            45,
            1,
        );
        f.render_widget(status, status_area);
    }
}

// ---------------------------------------------------------------------------
// Tests (headless rendering via TestBackend)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    fn render_app(app: &mut App) -> Buffer {
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn buffer_contains(buf: &Buffer, needle: &str) -> bool {
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        text.contains(needle)
    }

    #[test]
    fn render_shows_file_details_and_command_bar() {
        let mut app = App::new();
        assert!(!app.entries.is_empty(), "file list should not be empty");
        let buf = render_app(&mut app);
        // Right panel shows the highlighted file's details.
        assert!(buffer_contains(&buf, "Size:"));
        assert!(buffer_contains(&buf, "Modified:"));
        // Bottom bar renders.
        assert!(buffer_contains(&buf, "Command"));
    }

    #[test]
    fn render_shows_command_menu_when_open() {
        let mut app = App::new();
        app.show_menu = true;
        let buf = render_app(&mut app);
        assert!(buffer_contains(&buf, "AEGISTO COMMAND MENU"));
        assert!(buffer_contains(&buf, ":analyze"));
        assert!(buffer_contains(&buf, ":export"));
    }
}
