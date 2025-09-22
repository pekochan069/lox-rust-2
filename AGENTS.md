# Repository Guidelines

## Project Structure & Module Organization
- Source lives in `src/`. Key modules: `lexer.rs`, `parser.rs`, `compile.rs`, `vm.rs`, `value.rs`, `token.rs`, `debug.rs`, `args.rs`, `logger.rs`. Proc‑macro helpers are in `src/lib.rs` (crate name `lox_rust_2`).
- Binary entrypoint is `src/main.rs`. Sample program: `test.lox`. Build artifacts go to `target/`.
- Add integration tests under `tests/` and unit tests alongside modules using Rust’s `#[cfg(test)]`.

## Build, Test, and Development Commands
- Build: `cargo build` — compiles the binary and proc‑macro library.
- Run (REPL): `cargo run` — starts an interactive REPL (`exit` to quit).
- Run file: `cargo run -- test.lox` or `cargo run -- path/to/file.lox`.
- Trace execution: `cargo run --features trace_execution -- test.lox` — prints disassembly and stack during execution.
- Format: `cargo fmt`.
- Lint: `cargo clippy -- -D warnings`.
- Test (when added): `cargo test`.

## Coding Style & Naming Conventions
- Rust edition: 2024. Use 4‑space indentation and `rustfmt` defaults.
- Naming: modules/files `snake_case` (e.g., `chunk_store.rs`), functions/vars `snake_case`, types/enums `CamelCase`, constants `SCREAMING_SNAKE_CASE`.
- Prefer small, focused modules; keep `vm.rs` execution logic pure and side‑effect boundaries explicit (I/O via `print` op and logger only).

## Testing Guidelines
- Place integration tests in `tests/` (e.g., `tests/vm_prints.rs`) and unit tests in the same file under `#[cfg(test)]` blocks.
- Favor black‑box tests that execute Lox snippets end‑to‑end via the VM.
- Run `cargo test`; keep coverage high for `lexer`, `parser`, and `vm` branches.

## Commit & Pull Request Guidelines
- Commits: imperative mood, concise subject (<50 chars), detailed body when needed; reference issues (`Fixes #123`).
- PRs: include summary, motivation, notable design choices, test coverage notes, and CLI/feature flag changes (`--log-level`, `trace_execution`). Add screenshots or sample Lox code/output when helpful.

## Security & Configuration Tips
- Logging is controlled via CLI: `--log-level trace|debug|info|warn|error` (e.g., `cargo run -- --log-level trace -- test.lox`).
- Avoid panics in runtime paths; return `InterpretResult` for errors. Validate stack accesses and indices defensively.
