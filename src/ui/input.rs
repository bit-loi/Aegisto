//! Key event handling: navigation mode (browsing files) and command input
//! mode (typing `:analyze` and friends in the bottom bar).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub(crate) fn handle_nav_key(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    match key.code {
        Char('q') => app.quit = true,
        Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit = true,
        Char('?') => app.toggle_menu(),
        _ => {
            // When the command menu is open, ANY key closes it first.
            if app.show_menu {
                app.show_menu = false;
                return;
            }
            match key.code {
                Char(':') => {
                    app.input_mode = true;
                    app.input_buffer.clear();
                }
                Down => move_selection(app, 1),
                Up => move_selection(app, -1),
                Enter => {
                    if let Some(file) = app.selected_file.clone() {
                        if file.is_dir {
                            app.navigate_to(file.path.clone());
                        } else {
                            app.status_message =
                                format!("[INFO] selected: {} — type ':analyze'", file.name);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

pub(crate) fn handle_input_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = false;
            app.input_buffer.clear();
        }
        KeyCode::Enter => {
            app.input_mode = false;
            let cmd = app.input_buffer.trim().to_string();
            app.input_buffer.clear();
            app.execute_command(&cmd);
        }
        KeyCode::Char(c) => {
            if c == 'c' && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.quit = true;
            } else if key.modifiers.contains(KeyModifiers::CONTROL) {
                // Ignore other control sequences.
            } else {
                app.input_buffer.push(c);
            }
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        _ => {}
    }
}

fn move_selection(app: &mut App, delta: isize) {
    if app.entries.is_empty() {
        return;
    }
    let idx = (app.list_state.selected().unwrap_or(0) as isize + delta)
        .clamp(0, app.entries.len() as isize - 1) as usize;
    app.list_state.select(Some(idx));
    app.selected_file = app.entries.get(idx).cloned();
    app.update_info_panel();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::FileEntry;
    use std::time::SystemTime;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn test_binary_entry() -> FileEntry {
        let exe = std::env::current_exe().unwrap();
        FileEntry {
            name: exe.file_name().unwrap().to_string_lossy().to_string(),
            path: exe,
            is_dir: false,
            size: 0,
            modified: SystemTime::now(),
        }
    }

    #[test]
    fn nav_colon_enters_input_mode() {
        let mut app = App::new();
        handle_nav_key(&mut app, key(KeyCode::Char(':')));
        assert!(app.input_mode);
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn nav_question_mark_toggles_menu() {
        let mut app = App::new();
        handle_nav_key(&mut app, key(KeyCode::Char('?')));
        assert!(app.show_menu);
        handle_nav_key(&mut app, key(KeyCode::Char('?')));
        assert!(!app.show_menu);
    }

    #[test]
    fn any_key_closes_open_menu() {
        let mut app = App::new();
        app.show_menu = true;
        handle_nav_key(&mut app, key(KeyCode::Down));
        assert!(!app.show_menu, "menu should close on any key");
    }

    #[test]
    fn nav_q_quits() {
        let mut app = App::new();
        handle_nav_key(&mut app, key(KeyCode::Char('q')));
        assert!(app.quit);
    }

    #[test]
    fn input_chars_accumulate_and_enter_runs_command() {
        let mut app = App::new();
        app.input_mode = true;
        handle_input_key(&mut app, key(KeyCode::Char(':')));
        handle_input_key(&mut app, key(KeyCode::Char('a')));
        handle_input_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.input_buffer, ":an");
        // :analyze on a real PE produces a report.
        app.selected_file = Some(test_binary_entry());
        handle_input_key(&mut app, key(KeyCode::Char('a')));
        handle_input_key(&mut app, key(KeyCode::Char('l')));
        handle_input_key(&mut app, key(KeyCode::Char('y')));
        handle_input_key(&mut app, key(KeyCode::Char('z')));
        handle_input_key(&mut app, key(KeyCode::Char('e')));
        handle_input_key(&mut app, key(KeyCode::Enter));
        assert!(!app.input_mode);
        assert!(app.report.is_some(), "status: {}", app.status_message);
    }

    #[test]
    fn input_exit_command_sets_quit() {
        let mut app = App::new();
        app.input_mode = true;
        app.input_buffer = ":exit".to_string();
        handle_input_key(&mut app, key(KeyCode::Enter));
        assert!(app.quit);
    }

    #[test]
    fn input_help_command_opens_menu() {
        let mut app = App::new();
        app.input_mode = true;
        app.input_buffer = ":help".to_string();
        handle_input_key(&mut app, key(KeyCode::Enter));
        assert!(app.show_menu);
    }

    #[test]
    fn esc_cancels_input_mode() {
        let mut app = App::new();
        app.input_mode = true;
        app.input_buffer = ":partial".to_string();
        handle_input_key(&mut app, key(KeyCode::Esc));
        assert!(!app.input_mode);
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn ctrl_c_quits_from_input_mode() {
        let mut app = App::new();
        app.input_mode = true;
        handle_input_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(app.quit);
    }
}
