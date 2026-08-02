//! TUI rendering — Grok-CLI inspired: clean typography, no emoji,
//! monochrome grays with a single blue accent.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::App;
use crate::format::{format_size, truncate};

// Grok-style palette: pure grays + one blue accent.
const ACCENT: Color = Color::Rgb(100, 149, 237); // cornflower blue
const TEXT: Color = Color::Rgb(220, 220, 220);
const GRAY: Color = Color::Rgb(150, 150, 150);
const DIM: Color = Color::Rgb(100, 100, 100);
const FAINT: Color = Color::Rgb(60, 60, 60);
const OK: Color = Color::Rgb(90, 200, 120);
const WARN: Color = Color::Rgb(220, 180, 70);
const ERR: Color = Color::Rgb(230, 90, 90);

pub(crate) fn draw(f: &mut Frame<'_>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(10),   // main area (file browser | detail/log)
            Constraint::Length(2), // input bar
            Constraint::Length(1), // footer hotkeys
        ])
        .split(f.area());

    // === 1. HEADER (borderless: brand left, current dir right) ===
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[0]);

    let brand = Paragraph::new(Line::from(vec![
        Span::styled(
            "AEGISTO",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  ·  v0.1.0-alpha  ·  binary AI agent",
            Style::default().fg(DIM),
        ),
    ]));
    f.render_widget(brand, header_chunks[0]);

    let path = Paragraph::new(truncate(&app.current_dir.display().to_string(), 44))
        .style(Style::default().fg(DIM))
        .alignment(Alignment::Right);
    f.render_widget(path, header_chunks[1]);

    // === 2. MAIN: LEFT (file browser) + RIGHT (detail / log) ===
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    // LEFT — file browser. No emoji icons: folders get a trailing '/'.
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|entry| {
            let name = truncate(&entry.name, 30);
            let style = if entry.is_dir {
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
            } else {
                match entry.path.extension().and_then(|e| e.to_str()) {
                    Some("exe") | Some("elf") | Some("bin") | Some("dll") => {
                        Style::default().fg(OK).add_modifier(Modifier::BOLD)
                    }
                    Some("rs") | Some("toml") | Some("md") | Some("json") => {
                        Style::default().fg(WARN)
                    }
                    _ => Style::default().fg(TEXT),
                }
            };
            let display_name = if entry.is_dir {
                format!("{name}/")
            } else {
                name
            };
            let size = if entry.is_dir {
                String::new()
            } else {
                format!("  {}", format_size(entry.size))
            };
            ListItem::new(Line::from(vec![
                Span::styled(display_name, style),
                Span::styled(size, Style::default().fg(DIM)),
            ]))
        })
        .collect();

    let file_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" file browser ")
                .border_style(Style::default().fg(FAINT)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(45, 45, 45))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");
    f.render_stateful_widget(file_list, main_chunks[0], &mut app.list_state);

    // RIGHT — detail / log (or command menu when open)
    let (right_lines, right_title) = if app.show_menu {
        (App::menu_lines().join("\n"), " command menu ")
    } else {
        (app.info_lines.join("\n"), " detail / log ")
    };
    let info_panel = Paragraph::new(right_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(right_title)
                .border_style(Style::default().fg(FAINT)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(info_panel, main_chunks[1]);

    // === 3. INPUT BAR (top border only, Grok-style) ===
    let input_style = if app.input_mode {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(GRAY)
    };
    let input_content = if app.input_mode {
        if app.input_buffer.is_empty() {
            "type a command…  (Esc cancels)".to_string()
        } else {
            app.input_buffer.clone()
        }
    } else {
        "':' for commands · '?' for menu · 'q' quit".to_string()
    };
    let input_bar = Paragraph::new(Line::from(vec![
        Span::styled("⟩ ", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        Span::styled(input_content, input_style),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(FAINT)),
    );
    f.render_widget(input_bar, chunks[2]);

    // === 4. FOOTER (hotkeys left · status right) ===
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            "↑/↓",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" navigate   ", Style::default().fg(DIM)),
        Span::styled(
            "Enter",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" open   ", Style::default().fg(DIM)),
        Span::styled(":", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        Span::styled(" command   ", Style::default().fg(DIM)),
        Span::styled(
            "Ctrl+C",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit", Style::default().fg(DIM)),
    ]));
    f.render_widget(footer, chunks[3]);

    // Status overlay on the right of the footer, colored by severity.
    // Only on wide terminals so it never covers the hotkey hints.
    if chunks[3].width >= 90 {
        let status_color = if app.status_message.starts_with("[OK]") {
            OK
        } else if app.status_message.starts_with("[WARN]") {
            WARN
        } else if app.status_message.starts_with("[ERR]") {
            ERR
        } else {
            DIM
        };
        let status = Paragraph::new(truncate(&app.status_message, 48))
            .style(Style::default().fg(status_color))
            .alignment(Alignment::Right);
        let status_area = Rect::new(
            chunks[3].x + chunks[3].width.saturating_sub(50),
            chunks[3].y,
            50,
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
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

    fn render_app(app: &mut App) -> Buffer {
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn buffer_contains(buf: &Buffer, needle: &str) -> bool {
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        text.contains(needle)
    }

    #[test]
    fn render_shows_file_details_and_header() {
        let mut app = App::new();
        assert!(!app.entries.is_empty(), "file list should not be empty");
        let buf = render_app(&mut app);
        // Header brand.
        assert!(buffer_contains(&buf, "AEGISTO"));
        // Right panel shows the highlighted file's details.
        assert!(buffer_contains(&buf, "Size:"));
        assert!(buffer_contains(&buf, "Modified:"));
        // Input bar prefix renders.
        assert!(buffer_contains(&buf, "⟩"));
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
