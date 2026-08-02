```text
  /$$$$$$                      /$$             /$$
 /$$__  $$                    |__/            | $$
| $$  \ $$  /$$$$$$   /$$$$$$  /$$  /$$$$$$$ /$$$$$$    /$$$$$$
| $$$$$$$$ /$$__  $$ /$$__  $$| $$ /$$_____/|_  $$_/   /$$__  $$
| $$__  $$| $$$$$$$$| $$  \ $$| $$|  $$$$$$   | $$    | $$  \ $$
| $$  | $$| $$_____/| $$  | $$| $$ \____  $$  | $$ /$$| $$  | $$
| $$  | $$|  $$$$$$$|  $$$$$$$| $$ /$$$$$$$/  |  $$$$/|  $$$$$$/
|__/  |__/ \_______/ \____  $$|__/|_______/    \___/   \______/
                        /$$  \ $$
                       |  $$$$$$/
                        \______/
```

> The AEGISTO banner above renders in the Aegisto brand blue `#2171B5` in the terminal splash; GitHub markdown does not support colored text.

# Aegisto

**Aegisto** is an autonomous binary analysis framework powered by AI agents. It performs static analysis on compiled binaries by disassembling them and routing the output through a multi-agent cognitive pipeline to generate structured, human-readable reports without requiring manual assembly reading. Ships as a single standalone executable, no runtime dependencies to install.

**One liner:** Feed it a binary, get back a structured report explaining what it does.

**Scope note:** Aegisto analyzes any compiled binary, not just malware. Primary use cases are CTF crackmes, closed source software auditing, and general reverse engineering practice. Malware analysis is a possible advanced use case later, not a requirement to build or demo the tool.

## Interface (TUI)

Aegisto is a terminal UI (ratatui + crossterm) styled after tools like Claude Code / Grok CLI — clean typography, no emoji, monochrome grays with a blue accent.

```
AEGISTO  ·  v0.1.0-alpha  ·  binary AI agent                    ~/Aegisto
┌ file browser ───────────────┐ ┌ detail / log ─────────────────────┐
│ ▸ crates/                   │ │ aegisto.exe                      │
│   bin/                      │ │ Path: ./target/debug/aegisto.exe │
│   Cargo.toml                │ │ Size: 5.3 MB (5611520 bytes)     │
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

1. `cargo run` — after the splash you should see `Cargo.toml`, `crates/`, `target/`, etc.
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

[Static Extraction Layer]        aegisto-core
Binary parsing via goblin
Disassembly via iced-x86
String extraction
Import / API table extraction

[Agent Orchestration Layer]      aegisto-agent (scaffolding)
Explorer Agent: flag notable patterns
Systemizer Agent: structure call graph & data flow
Verifier Agent: validate hypotheses against evidence

[Output Layer]                   aegisto-tui
Structured report (report.json via :export)
```

### Project structure (Cargo workspace)

```
Aegisto/
├── .cargo/config.toml           # shared Cargo settings
├── bin/
│   └── aegisto/                 # binary crate: entry point (main.rs)
├── crates/
│   ├── aegisto-core/            # types + analysis (parser, disasm, strings)
│   ├── aegisto-tui/             # TUI app: state, event loop, render, input
│   └── aegisto-agent/           # AI agents + LLM providers
├── third_party/                 # vendored third-party code (empty)
├── clippy.toml
├── rust-toolchain.toml
├── rustfmt.toml
├── Cargo.toml                   # workspace manifest
├── CONTRIBUTING.md
└── SECURITY.md
```

## AI Agent Layer (`aegisto-agent`)

The agent crate is ready to use but not yet wired into the TUI. It provides:

- **Pluggable LLM providers** behind one trait (`LlmProvider`) — everything speaks the OpenAI-compatible `/chat/completions` API, so one client serves them all.
- **Agent roles** (`Agent` trait): Explorer, Systemizer, Verifier, each with a role-specific system prompt that reasons over a static-analysis snapshot.

### Supported providers

| Provider | `AEGISTO_PROVIDER` | Required env | Optional env | Default model |
| --- | --- | --- | --- | --- |
| Ollama (incl. NVIDIA build) | `ollama` | — | `OLLAMA_HOST`, `OLLAMA_API_KEY`, `OLLAMA_MODEL` | `qwen2.5:7b` |
| Groq | `groq` | `GROQ_API_KEY` | `GROQ_MODEL` | `llama-3.3-70b-versatile` |
| Google AI Studio | `google` | `GOOGLE_API_KEY` (or `GEMINI_API_KEY`) | `GOOGLE_MODEL` | `gemini-2.0-flash` |
| OpenRouter | `openrouter` | `OPENROUTER_API_KEY` | `OPENROUTER_MODEL` | `openrouter/auto` |
| Any OpenAI-compatible endpoint | `openai-compat` | `AEGISTO_BASE_URL`, `AEGISTO_MODEL` | `AEGISTO_API_KEY` | — |

Example — local Ollama (works with the NVIDIA GPU build too, same local endpoint):

```bash
# Linux: sudo curl -fsSL https://ollama.com/install.sh | sh
# Windows: download the installer, or use the NVIDIA build for local GPU
ollama pull qwen2.5:7b

AEGISTO_PROVIDER=ollama OLLAMA_MODEL=qwen2.5:7b cargo run
```

Example — Groq:

```bash
export AEGISTO_PROVIDER=groq
export GROQ_API_KEY=your_key
cargo run
```

The provider is loaded with `load_provider_from_env()` (returns `None` when `AEGISTO_PROVIDER` is unset). The next milestone is wiring the agents into `:analyze` so the Explorer → Systemizer → Verifier chain runs over each binary's static extraction.

## Tech Stack

| Layer | Tool / Library | Reason |
| --- | --- | --- |
| Binary Processing | goblin | Pure Rust PE / ELF / Mach-O parser, no external dependency, memory safe against malformed binaries |
| Disassembly | iced-x86 | Fast, pure Rust x86 / x64 disassembler, no C bindings |
| TUI | ratatui + crossterm | Terminal native, runs over SSH on headless VMs, no WebView or browser dependency |
| LLM Clients | reqwest (rustls) | Single OpenAI-compatible client for Ollama / Groq / Google / OpenRouter |
| Orchestration | tokio + async-trait | Async agent pipeline (Explorer / Systemizer / Verifier) |
| Serialization | serde / serde_json | Structured output and report generation |
| Language | Rust (2024 edition) | Memory safety when parsing untrusted input, single binary distribution |

## Roadmap

- **Wire agents into `:analyze`** — run the Explorer → Systemizer → Verifier chain over the static extraction and stream results into the log panel.
- **Feature encoding** — opcode n-gram histograms, import hashes, section entropy for binary family clustering (Qdrant, embedded mode).
- **Async analysis** — run the pipeline on a background thread so the UI never freezes on large binaries.
- **Markdown export** — generate a human-readable `report.md` alongside `report.json`.

## License

This project is open source and available under the MIT License.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).
