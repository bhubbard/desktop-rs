# Contributing to desktop-rs 🦀

We love your input! We want to make contributing to `desktop-rs` as easy and transparent as possible.

## Development Workflow

1. Fork the repo and create your branch from `main`.
2. Ensure you have Rust (edition 2021) installed:
   ```bash
   rustup update stable
   ```
3. Run tests across all workspace crates:
   ```bash
   make test
   # or
   cargo test --workspace --all-targets
   ```
4. Check code formatting:
   ```bash
   make fmt-check
   # or
   cargo fmt --all -- --check
   ```
5. Check Clippy lints:
   ```bash
   make clippy
   # or
   cargo clippy --workspace --all-targets -- -D warnings
   ```
6. Open a Pull Request with a clear description of the changes.

## License

By contributing, you agree that your contributions will be dual-licensed under both the **MIT License** and the **Apache License (Version 2.0)**.
