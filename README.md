# Aegisto

Aegisto is an autonomous reverse engineering framework powered by AI agents. It performs static malware analysis by disassembling binaries and routing the output through a multi-agent cognitive pipeline to generate structured, human readable reports without requiring manual assembly reading. Ships as a single standalone executable, no runtime dependencies to install.

**One liner:** Feed it a binary, get back a structured report explaining what it does and why it is suspicious.

## Problem Statement

| Current Gap | Why It Matters |
| --- | --- |
| Malware is increasingly packed and obfuscated | Manual reverse engineering requires weeks per sample |
| Shortage of skilled reverse engineers | Supply of analysts cannot match the volume of new malware |
| Static analysis is repetitive and pattern heavy | AI is well suited for pattern recognition in assembly and pseudocode |
| Existing tools are black boxes | VirusTotal tells you what, but not why |
| Cloud analysis risks data leakage | Sensitive binaries should not be uploaded to third party APIs |

**Gap addressed by Aegisto:** No existing open source tool combines native, memory safe disassembly with a local cognitive agent pipeline that explains binary behavior through structured reasoning, all shipped as one executable.

## Architecture

### Pipeline

```
Input Binary (EXE / APK / Firmware)

[Static Extraction Layer]
Binary parsing via goblin (PE / ELF / Mach-O)
Disassembly via iced-x86
String extraction
Import / API table extraction
Entropy and section analysis

[Agent Orchestration Layer]
Explorer Agent: scan all functions, flag suspicious patterns
Systemizer Agent: structure call graph and data flow
Verifier Agent: validate hypotheses against evidence

[Output Layer]
Structured markdown report
IOC extraction
Behavior summary
Recommended defensive actions
```

### Agent Roles

| Agent | Function |
| --- | --- |
| Explorer | Identify suspicious functions, API calls, encryption routines, packer signatures |
| Systemizer | Build call graph, map data flow, structure execution path |
| Verifier | Check hypotheses against evidence, detect false positives, validate vulnerability claims |

## Tech Stack

| Layer | Tool / Library | Reason |
| --- | --- | --- |
| Binary Processing | goblin | Pure Rust PE / ELF / Mach-O parser, no external dependency, memory safe against malformed binaries |
| Disassembly | iced-x86 | Fast, pure Rust x86 / x64 disassembler, no C bindings |
| Local LLM | Qwen2.5 7B via Ollama, called through reqwest | No API keys, no data leaves local machine |
| Agent Framework | Custom async orchestrator on tokio | Simple state machine to switch between Explorer, Systemizer, and Verifier modes |
| Feature Encoding | Handcrafted static features: opcode n-gram histogram, import hash, section entropy | Deterministic, fast to compute in Rust, no pretrained model or external corpus needed |
| Vector Store | Qdrant, embedded mode | Local similarity search for malware family clustering, no external service to run |
| Report UI | Tauri, or a static HTML report opened in browser | Native desktop shell or zero server dependency |
| CLI | clap | Standard Rust argument parsing |
| Serialization | serde / serde_json | Structured output, IOC and report generation |
| Language | Rust (2021 edition) | Memory safety when parsing adversarial input, single binary distribution, no interpreter to ship |

## License

This project is open source and available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
