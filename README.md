# Aegisto

**Aegisto** is an autonomous binary analysis framework powered by AI agents. It performs static analysis on compiled binaries by disassembling them and routing the output through a multi-agent cognitive pipeline to generate structured, human-readable reports without requiring manual assembly reading. Ships as a single standalone executable, no runtime dependencies to install.

**One liner:** Feed it a binary, get back a structured report explaining what it does.

**Scope note:** Aegisto analyzes any compiled binary, not just malware. Primary use cases are CTF crackmes, closed source software auditing, and general reverse engineering practice. Malware analysis is a possible advanced use case later, not a requirement to build or demo the tool.

## Interface (TUI)

Aegisto is a terminal UI (ratatui + crossterm) styled after tools like Claude Code / Grok CLI — clean typography, no emoji, monochrome grays with a blue accent.

```
AEGISTO  ·  v0.1.0-alpha  ·  binary analysis cockpit        ~/Aegisto
┌ file browser ───────────────┐ ┌ detail / log ─────────────────────┐
│ ▸ src/                      │ │ aegisto.exe                      │
│   Cargo.toml                │ │ Path: ./target/debug/aegisto.exe │
│   README.md                 │ │ Size: 5.3 MB (5611520 bytes)     │
│   target/                   │ │ Modified: 2026-08-02 14:31:20    │
│   ...                       │ │ Type: file                       │
│                             │ │ :analyze → run static analysis   │
└─────────────────────────────┘ └───────────────────────────────────┘
⟩ ':'
↑/↓ navigate  Enter open  : command  Ctrl+C quit
```

- **Left panel** — file/folder browser of the current directory (folders get a trailing `/`).
- **Right panel** — details of the highlighted file plus the analysis log / command menu.
- **Bottom bar** — command input with a `⟩` prompt (Claude Code style).

### Keybindings

| Key | Action |
| --- | --- |
| `↑` / `↓` | Move selection in the file browser |
| `Enter` | Folder: open it · File: select it |
| `:` | Enter command mode |
| `?` | Toggle the command menu |
| `Esc` | Cancel command input |
| `q` / `Ctrl+C` | Quit |

### Commands

Press `?` (or type `:help`) to see the command menu inside the TUI.

| Command | Description |
| --- | --- |
| `:analyze` | Run static analysis on the selected file |
| `:cd <path>` | Change directory — works for folders **and** files (jumps to parent & highlights) |
| `:up` | Go to parent folder |
| `:home` | Go to the project root |
| `:refresh` | Reload the current folder |
| `:export` | Save the last analysis to `report.json` |
| `:clear` | Clear the log panel |
| `:help` / `:menu` | Show the command menu |
| `:exit` / `:q` | Quit Aegisto |

### What `:analyze` does

Runs the **real** static pipeline on the highlighted binary:

1. **Parse** — goblin parses the PE / ELF headers (format, entry point, sections, imports).
2. **Disassemble** — iced-x86 decodes the executable section into instructions.
3. **Extract strings** — printable ASCII strings are collected.

Results (sections, imports, entry point, sample disassembly) appear in the detail panel. `:export` writes the full structured result to `report.json`.

## Setup

### Prerequisites

- **Rust toolchain** (edition 2024, so Rust 1.85+) — install via [https://rustup.rs](https://rustup.rs)
- **A modern Unicode terminal**:
  - Windows: [Windows Terminal](https://aka.ms/terminal) (recommended) or VS Code terminal
  - macOS / Linux: any terminal emulator (iTerm2, GNOME Terminal, kitty, …)
- (Optional) A binary you want to analyze — the easiest test target is Aegisto itself: `target/debug/aegisto.exe`.

### Quick start

```bash
# 1. enter the project
cd Aegisto

# 2. run in dev mode (or: cargo build --release && ./target/release/aegisto)
cargo run
```

That's it — no external services, no API keys. The app starts with an animated splash (press `q` to skip) and lands you directly in the file browser for the current directory.

**Try it in 3 steps:**

1. `cargo run` — after the splash you should see `Cargo.toml`, `src/`, `target/`, etc.
2. Press `:` then type `cd target/debug` and press `Enter`.
3. Press `Enter` on `aegisto.exe`, then `:` → `analyze` → `Enter`. The right panel fills with real analysis output; `:export` saves `report.json`.

### Notes

- The splash auto-skips when stdout is piped/redirected.
- Unicode glyphs (`⟩`, `↑/↓`, `─`) render best in Windows Terminal / modern terminals.
- `q` quits from navigation mode; `:exit` or `Ctrl+C` works from anywhere.

## Architecture

### Pipeline

```
Input Binary (EXE / ELF / Mach-O)

[Static Extraction Layer]
Binary parsing via goblin (PE / ELF / Mach-O)
Disassembly via iced-x86
String extraction
Import / API table extraction
Entropy and section analysis

[Agent Orchestration Layer]   (roadmap — see below)
Explorer Agent: scan all functions, flag notable patterns
Systemizer Agent: structure call graph and data flow
Verifier Agent: validate hypotheses against evidence

[Output Layer]
Structured markdown report
Notable pattern extraction
Behavior summary
Suggested areas for further review
```

### Project structure

```
src/
├── main.rs               # Entry point: module declarations + app::run()
├── app.rs                # App state, commands, analysis pipeline, event loop
├── types.rs              # Shared data structures (serde)
│
├── analysis/             # Binary analysis layer
│   ├── mod.rs
│   ├── parser.rs         # goblin PE / ELF parser
│   ├── disasm.rs         # iced-x86 disassembly
│   └── strings.rs        # string extraction
│
└── ui/                   # TUI layer (ratatui)
    ├── mod.rs
    ├── render.rs         # layout & styling (Grok-style, no emoji)
    ├── input.rs          # key handling
    ├── format.rs         # display helpers (sizes, dates, truncation)
    └── splash.rs         # startup splash animation
```

## Tech Stack

| Layer | Tool / Library | Reason |
| --- | --- | --- |
| Binary Processing | goblin | Pure Rust PE / ELF / Mach-O parser, no external dependency, memory safe against malformed binaries |
| Disassembly | iced-x86 | Fast, pure Rust x86 / x64 disassembler, no C bindings |
| TUI | ratatui + crossterm | Terminal native, runs over SSH on headless VMs, no WebView or browser dependency |
| Serialization | serde / serde_json | Structured output and report generation |
| Language | Rust (2024 edition) | Memory safety when parsing untrusted input, single binary distribution |

## Roadmap

- **Agent layer** (`src/agent/`): Explorer / Systemizer / Verifier agents orchestrated on tokio, backed by a local LLM (e.g. Qwen2.5 7B via Ollama) — no API keys, no data leaves the machine.
- **Feature encoding**: opcode n-gram histograms, import hashes, section entropy for binary family clustering (Qdrant, embedded mode).
- **Async analysis**: run `:analyze` on a background thread so the UI never freezes on large binaries.

## License

This project is open source and available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
