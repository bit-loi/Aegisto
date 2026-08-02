# Contributing to Aegisto

Thanks for helping out! Here is how to work on the codebase.

## Workspace layout

This repository is a Cargo workspace:

- `bin/aegisto` — the TUI binary entry point
- `crates/aegisto-core` — shared types + static binary analysis
- `crates/aegisto-tui` — the ratatui application (state, event loop, rendering)
- `crates/aegisto-agent` — AI agents + LLM providers (scaffolding)

## Build & test

```bash
cargo build --workspace     # compile everything
cargo test --workspace      # run all unit tests
cargo clippy --workspace    # lint
cargo fmt --all             # format
cargo run -p aegisto        # run the TUI
```

## Style

- Follow `rustfmt` (see `rustfmt.toml`) and keep clippy clean.
- Keep messages in the TUI free of emoji (Grok-CLI style).
- Add tests next to the code they cover.

## Pull requests

1. Keep changes focused and rebase on latest `main`.
2. Make sure `cargo test --workspace` passes.
3. Mention the crate(s) you touched in the PR description.
