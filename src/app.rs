//! Application state and core logic: file browsing, command handling, the
//! static-analysis pipeline behind `:analyze`, and the main event loop.

use std::{
    fs,
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::Result;
use crossterm::{
    cursor::Show,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};

use crate::analysis::{disasm, parser, strings};
use crate::types::AnalysisResult;
use crate::ui::format::{format_modified, format_size};
use crate::ui::{input, render, splash};

/// Maximum number of instructions to disassemble during `:analyze`.
const MAX_INSTRUCTIONS: usize = 2000;

// ---------------------------------------------------------------------------
// File entry
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct FileEntry {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) is_dir: bool,
    pub(crate) size: u64,
    pub(crate) modified: SystemTime,
}

impl FileEntry {
    pub(crate) fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let metadata = fs::metadata(path).ok()?;
        Some(FileEntry {
            name,
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::now()),
        })
    }
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

pub(crate) struct App {
    // File explorer
    pub(crate) current_dir: PathBuf,
    pub(crate) entries: Vec<FileEntry>,
    pub(crate) list_state: ListState,
    pub(crate) selected_file: Option<FileEntry>,

    // Command input bar
    pub(crate) input_buffer: String,
    pub(crate) input_mode: bool,

    // Right panel / status
    pub(crate) info_lines: Vec<String>,
    pub(crate) status_message: String,
    pub(crate) show_menu: bool,
    pub(crate) quit: bool,

    // Last analysis result (used by `:export`)
    pub(crate) report: Option<AnalysisResult>,
}

impl App {
    pub(crate) fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut app = App {
            current_dir: current_dir.clone(),
            entries: Vec::new(),
            list_state: ListState::default(),
            selected_file: None,
            input_buffer: String::new(),
            input_mode: false,
            info_lines: Vec::new(),
            status_message: String::new(),
            show_menu: false,
            quit: false,
            report: None,
        };
        app.load_directory(current_dir);
        app
    }

    // --- Directory listing ------------------------------------------------

    fn load_directory(&mut self, path: PathBuf) {
        self.current_dir = path;
        let mut entries: Vec<FileEntry> = Vec::new();

        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            for entry in read_dir.flatten() {
                if let Some(file_entry) = FileEntry::from_path(&entry.path()) {
                    // Skip dotfiles so the listing stays tidy.
                    if !file_entry.name.starts_with('.') {
                        entries.push(file_entry);
                    }
                }
            }
        }

        // Folders first, then files; both sorted alphabetically.
        entries.sort_by(|a, b| {
            if a.is_dir && !b.is_dir {
                std::cmp::Ordering::Less
            } else if !a.is_dir && b.is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        self.entries = entries;
        if self.entries.is_empty() {
            self.list_state.select(None);
            self.selected_file = None;
        } else {
            self.list_state.select(Some(0));
            self.selected_file = self.entries.first().cloned();
        }
        self.update_info_panel();
    }

    // --- Right panel ------------------------------------------------------

    pub(crate) fn update_info_panel(&mut self) {
        let Some(file) = &self.selected_file else {
            self.info_lines = vec!["no file selected.".to_string()];
            return;
        };

        let mut lines = vec![
            file.name.clone(),
            format!("Path: {}", file.path.display()),
            format!("Size: {} ({} bytes)", format_size(file.size), file.size),
            format!("Modified: {}", format_modified(file.modified)),
            format!("Type: {}", if file.is_dir { "folder" } else { "file" }),
        ];
        if !file.is_dir {
            if let Some(ext) = file.path.extension() {
                lines.push(format!("Extension: {}", ext.to_string_lossy()));
            }
        }
        lines.push("─".repeat(34));
        if file.is_dir {
            lines.push("Enter: open this folder".to_string());
            lines.push(":up → parent folder".to_string());
        } else {
            lines.push(":analyze → run static analysis".to_string());
            lines.push(":export → save report.json".to_string());
        }
        lines.push("? → command menu".to_string());
        self.info_lines = lines;
    }

    fn add_log(&mut self, msg: String) {
        self.info_lines.push(msg);
        if self.info_lines.len() > 80 {
            let excess = self.info_lines.len() - 80;
            self.info_lines.drain(0..excess);
        }
    }

    // --- Navigation -------------------------------------------------------

    pub(crate) fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.load_directory(path.clone());
            self.status_message = format!("[INFO] opened: {}", path.display());
        } else if path.is_file() {
            // Jump to the file's parent folder and highlight the file itself.
            if let Some(parent) = path.parent() {
                self.load_directory(parent.to_path_buf());
            }
            if let Some(idx) = self.entries.iter().position(|e| e.path == path) {
                self.list_state.select(Some(idx));
                self.selected_file = self.entries.get(idx).cloned();
                self.update_info_panel();
            }
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            self.status_message = format!("[INFO] selected: {}", name);
        } else {
            self.status_message = format!("[ERR] not found: {}", path.display());
        }
    }

    fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.navigate_to(parent.to_path_buf());
        }
    }

    // --- Command menu (C/C++-style, so nobody gets lost) -------------------

    pub(crate) fn toggle_menu(&mut self) {
        self.show_menu = !self.show_menu;
    }

    pub(crate) fn menu_lines() -> Vec<String> {
        vec![
            "AEGISTO COMMAND MENU".to_string(),
            "────────────────────────────────".to_string(),
            "  :analyze      analyze selected file".to_string(),
            "  :cd <path>    change directory".to_string(),
            "  :up           go to parent folder".to_string(),
            "  :home         go to project root".to_string(),
            "  :refresh      reload current folder".to_string(),
            "  :export       save last report to report.json".to_string(),
            "  :clear        clear the log panel".to_string(),
            "  :help / :menu show this menu (or press ?)".to_string(),
            "  :exit / :q    quit aegisto".to_string(),
            "────────────────────────────────".to_string(),
            "  ↑/↓ navigate · Enter open/select".to_string(),
            "  press any key to close this menu".to_string(),
        ]
    }

    // --- Commands ----------------------------------------------------------

    pub(crate) fn execute_command(&mut self, cmd: &str) {
        if cmd == ":analyze" {
            self.analyze_selected();
        } else if let Some(path_str) = cmd.strip_prefix(":cd ") {
            let path_str = path_str.trim();
            if path_str.is_empty() {
                self.status_message = "[WARN] usage: :cd <path>".to_string();
                return;
            }
            let new_path = PathBuf::from(path_str);
            if new_path.is_absolute() {
                self.navigate_to(new_path);
            } else {
                self.navigate_to(self.current_dir.join(new_path));
            }
        } else if cmd == ":cd" {
            self.status_message = "[WARN] usage: :cd <path>".to_string();
        } else if cmd == ":up" {
            self.go_up();
        } else if cmd == ":home" {
            if let Ok(home) = std::env::current_dir() {
                self.navigate_to(home);
            }
        } else if cmd == ":refresh" {
            self.load_directory(self.current_dir.clone());
            self.status_message = "[OK] reloaded".to_string();
        } else if cmd == ":export" {
            self.export_report();
        } else if cmd == ":clear" {
            self.info_lines.clear();
            self.add_log("[OK] log cleared".to_string());
            self.status_message = "[OK] log cleared".to_string();
        } else if cmd == ":help" || cmd == ":menu" || cmd == ":?" {
            self.toggle_menu();
        } else if cmd == ":exit" || cmd == ":q" || cmd == ":quit" {
            self.quit = true;
        } else if !cmd.is_empty() {
            self.status_message = format!("[ERR] unknown command: '{cmd}' — type ':help'");
        }
    }

    // --- Analysis (REAL static pipeline: parser + disasm + strings) --------

    fn analyze_selected(&mut self) {
        self.show_menu = false;

        let Some(file) = self.selected_file.clone() else {
            self.status_message = "[WARN] no file selected — use ↑/↓ first".to_string();
            return;
        };
        if file.is_dir {
            self.status_message = "[WARN] cannot analyze a folder".to_string();
            return;
        }

        self.status_message = format!("[INFO] analyzing {} ...", file.name);
        self.add_log(format!("=== ANALYSIS: {} ===", file.name));

        match parser::parse_file(&file.path) {
            Ok(parsed) => {
                self.add_log(format!("Format: {}", parsed.format));
                self.add_log(format!("Entry point: {:#x}", parsed.entry_point));

                self.add_log(format!("Sections ({}):", parsed.sections.len()));
                for s in parsed.sections.iter().take(12) {
                    self.add_log(format!(
                        "    {:<16} size={:<#10x} vaddr={:#010x} [{}]",
                        s.name, s.size, s.virtual_address, s.flags
                    ));
                }
                if parsed.sections.len() > 12 {
                    self.add_log(format!("    … +{} more", parsed.sections.len() - 12));
                }

                self.add_log(format!("Imports ({}):", parsed.imports.len()));
                for imp in parsed.imports.iter().take(15) {
                    self.add_log(format!("    {} ({})", imp.name, imp.library));
                }
                if parsed.imports.len() > 15 {
                    self.add_log(format!("    … +{} more", parsed.imports.len() - 15));
                }

                let raw_bytes = fs::read(&file.path).unwrap_or_default();
                let extracted = strings::extract_strings(&raw_bytes, 6);
                self.add_log(format!("Strings extracted: {}", extracted.len()));

                let instructions = match &parsed.executable_section {
                    Some((sec_name, base_va, code_bytes)) => {
                        match disasm::disassemble(code_bytes, *base_va, MAX_INSTRUCTIONS) {
                            Ok(insts) => {
                                self.add_log(format!(
                                    "Disassembled {} instructions from {} @ {:#x}",
                                    insts.len(),
                                    sec_name,
                                    base_va
                                ));
                                for inst in insts.iter().take(10) {
                                    self.add_log(format!(
                                        "    {:#010x}:  {} {}",
                                        inst.address, inst.mnemonic, inst.operands
                                    ));
                                }
                                if insts.len() > 10 {
                                    self.add_log(format!("    … +{} more", insts.len() - 10));
                                }
                                insts
                            }
                            Err(e) => {
                                self.add_log(format!("[WARN] disassembly error: {e}"));
                                Vec::new()
                            }
                        }
                    }
                    None => {
                        self.add_log("[WARN] no executable section found".to_string());
                        Vec::new()
                    }
                };

                self.report = Some(AnalysisResult {
                    file_path: file.path.display().to_string(),
                    format: parsed.format,
                    entry_point: parsed.entry_point,
                    sections: parsed.sections,
                    imports: parsed.imports,
                    instructions,
                    strings: extracted,
                });
                self.add_log("[OK] analysis complete — type ':export' for report.json".to_string());
                self.status_message = "[OK] analysis complete — type ':export' for report.json".to_string();
            }
            Err(e) => {
                self.add_log(format!("[ERR] analysis failed: {e}"));
                self.status_message = "[ERR] analysis failed — is this a valid PE/ELF binary?".to_string();
            }
        }
    }

    fn export_report(&mut self) {
        let Some(report) = &self.report else {
            self.status_message = "[WARN] no analysis yet — run ':analyze' first".to_string();
            return;
        };
        let path = self.current_dir.join("report.json");
        match serde_json::to_string_pretty(report) {
            Ok(json) => match fs::write(&path, json) {
                Ok(()) => {
                    self.add_log(format!("Report saved to {}", path.display()));
                    self.status_message = "[OK] report exported".to_string();
                }
                Err(e) => {
                    self.status_message = format!("[ERR] failed to write report: {e}");
                }
            },
            Err(e) => {
                self.status_message = format!("[ERR] serialization error: {e}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

/// Restores the terminal when dropped, so every exit path (including errors
/// and panics) leaves the user's shell usable. Same pattern as ui/splash.rs.
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

/// Run the TUI: splash, terminal setup and the main event loop.
pub(crate) fn run() -> Result<()> {
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
            // Borderless input bar: "⟩ " prefix starts at x=0, content on the
            // row above the footer.
            let x = 2 + app.input_buffer.chars().count() as u16;
            let y = terminal.size()?.height.saturating_sub(2);
            terminal.set_cursor_position((x, y))?;
        } else {
            terminal.hide_cursor()?;
        }

        terminal.draw(|f| render::draw(f, &mut app))?;

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
    println!("Aegisto exited cleanly.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_toggle_restores_info_panel() {
        let mut app = App::new();
        app.toggle_menu();
        assert!(app.show_menu);
        app.toggle_menu();
        assert!(!app.show_menu);
        assert!(!app.info_lines.is_empty());
    }

    #[test]
    fn command_clear_empties_log() {
        let mut app = App::new();
        app.execute_command(":clear");
        assert!(app.info_lines.iter().any(|l| l.contains("log cleared")));
    }

    #[test]
    fn unknown_command_reports_message() {
        let mut app = App::new();
        app.execute_command(":bogus");
        assert!(app.status_message.contains("unknown command"));
    }

    #[test]
    fn analyze_non_binary_fails_gracefully() {
        let mut app = App::new();
        let tmp = std::env::temp_dir().join("aegisto_not_a_binary.txt");
        std::fs::write(&tmp, "this is not a binary").unwrap();
        app.selected_file = Some(FileEntry {
            name: "aegisto_not_a_binary.txt".to_string(),
            path: tmp.clone(),
            is_dir: false,
            size: 11,
            modified: SystemTime::now(),
        });
        app.analyze_selected();
        assert!(
            app.status_message.contains("analysis failed"),
            "status was: {}",
            app.status_message
        );
        let _ = std::fs::remove_file(tmp);
    }

    #[test]
    fn analyze_real_pe_produces_report() {
        // Run the full static pipeline against the test binary itself.
        let mut app = App::new();
        let exe = std::env::current_exe().unwrap();
        app.selected_file = Some(FileEntry {
            name: exe.file_name().unwrap().to_string_lossy().to_string(),
            path: exe,
            is_dir: false,
            size: 0,
            modified: SystemTime::now(),
        });
        app.analyze_selected();
        assert!(
            app.report.is_some(),
            "expected a report; status was: {}",
            app.status_message
        );
        assert!(app.info_lines.iter().any(|l| l.contains("Format:")));
        assert!(app.info_lines.iter().any(|l| l.contains("Sections")));
    }
}
