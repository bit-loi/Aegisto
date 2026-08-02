# Aegisto

## Overview

**Aegisto** is an autonomous binary analysis framework powered by AI agents. It performs static analysis on compiled binaries by disassembling them and routing the output through a multi-agent cognitive pipeline to generate structured, human readable reports without requiring manual assembly reading. Ships as a single standalone executable, no runtime dependencies to install.

**One liner:** Feed it a binary, get back a structured report explaining what it does.

**Scope note:** Aegisto analyzes any compiled binary, not just malware. Primary use cases are CTF crackmes, closed source software auditing, and general reverse engineering practice. Malware analysis is a possible advanced use case later, not a requirement to build or demo the tool.

## Problem Statement

| Current Gap | Why It Matters |
| --- | --- |
| Compiled binaries are opaque without source code | Manual reverse engineering requires hours to weeks per binary |
| Reverse engineering has a steep learning curve | Beginners struggle to read raw assembly and pseudocode |
| Static analysis is repetitive and pattern heavy | AI is well suited for pattern recognition in assembly and pseudocode |
| Existing tools like IDA Pro are expensive or require setup | A lightweight standalone tool lowers the barrier to entry |
| Cloud based analysis risks leaking proprietary or sensitive binaries | Sensitive binaries should not be uploaded to third party APIs |

**Gap addressed by Aegisto:** No existing open source tool combines native, memory safe disassembly with a local cognitive agent pipeline that explains binary behavior through structured reasoning, all shipped as one executable, without requiring IDA Pro or Ghidra.

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

[Agent Orchestration Layer]
Explorer Agent: scan all functions, flag notable patterns
Systemizer Agent: structure call graph and data flow
Verifier Agent: validate hypotheses against evidence

[Output Layer]
Structured markdown report
Notable pattern extraction
Behavior summary
Suggested areas for further review
```

### Agent Roles

| Agent | Function |
| --- | --- |
| Explorer | Identify notable functions, API calls, encryption routines, packer signatures |
| Systemizer | Build call graph, map data flow, structure execution path |
| Verifier | Check hypotheses against evidence, detect false positives, validate claims |

## Tech Stack

| Layer | Tool / Library | Reason |
| --- | --- | --- |
| Binary Processing | goblin | Pure Rust PE / ELF / Mach-O parser, no external dependency, memory safe against malformed binaries |
| Disassembly | iced-x86 | Fast, pure Rust x86 / x64 disassembler, no C bindings |
| Local LLM | Qwen2.5 7B via Ollama, called through reqwest | No API keys, no data leaves local machine |
| Agent Framework | Custom async orchestrator on tokio | Simple state machine to switch between Explorer, Systemizer, and Verifier modes |
| Feature Encoding | Handcrafted static features: opcode n-gram histogram, import hash, section entropy | Deterministic, fast to compute in Rust, no pretrained model or external corpus needed |
| Vector Store | Qdrant, embedded mode | Local similarity search for binary family clustering, no external service to run |
| Report UI | ratatui + crossterm | Terminal native, runs over SSH on headless VMs, no WebView or browser dependency |
| CLI | clap | Standard Rust argument parsing |
| Serialization | serde / serde_json | Structured output and report generation |
| Language | Rust (2021 edition) | Memory safety when parsing untrusted input, single binary distribution, no interpreter to ship |

## Setup Guide

### Initial Environment Setup

Root folder is already created. Since the folder exists, use `cargo init` instead of `cargo new`:

```bash
cd Aegisto
cargo init --name aegisto
```

Add starting dependencies to `Cargo.toml`:

```toml
[dependencies]
goblin = "0.8"
iced-x86 = "1.21"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
ratatui = "0.28"
crossterm = "0.28"
```

## License

This project is open source and available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
