# Memory: desktop-rs

## Decisions
- Pure Rust reimplementation of GitHub Desktop (`desktop/desktop`).
- Eliminated 300MB+ Electron overhead in favor of a lightning-fast native binary (<15MB release) with sub-millisecond execution times.
- Split into modular crates under a clean Cargo workspace:
  - `desktop-core`: Pure domain models, diff hunk parsing, and commit co-author formatting.
  - `desktop-git`: High-performance porcelain v2 git subprocess engine.
  - `desktop-github`: GitHub REST API client with automatic token resolution (`gh` CLI / env).
  - `desktop-tui`: Interactive ratatui + crossterm terminal user interface replicating GitHub Desktop's 2-pane visual workflow.
  - `desktop-cli`: Command-line interface with dual binaries (`desktop` and `github-desktop`).

## Active Context
- All crates compiling cleanly, zero clippy warnings, full integration tests passing.
