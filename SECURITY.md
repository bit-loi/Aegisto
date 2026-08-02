# Security Policy

## Scope

Aegisto parses **untrusted binaries** (PE / ELF / Mach-O). Parsing is done
with memory-safe crates (goblin, iced-x86), but defense-in-depth is a goal:
treat any analyzed file as hostile.

## Reporting a vulnerability

Please **do not open a public issue** for security bugs. Report privately by
opening a GitHub issue with the `security` label or reaching out to the
maintainers directly. Include:

- the affected crate and version
- a minimal reproduction (binary or test case)
- impact description

## Notes for contributors

- Never `unwrap()` on data derived from a parsed binary.
- Keep `anyhow::Context`/`Result` errors on all analysis paths.
- Be careful when adding new parsing dependencies.
