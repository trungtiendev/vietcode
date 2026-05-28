# VietCode — Agent Guide

## Workspace

4 crates under `crates/`:
- **vietcode-cortex** — tree-sitter + SQLite codebase index (parser, symbol index, relation graph, file watcher)
- **vietcode-core** — agent pipeline orchestrator (planner, router, pipeline are stubs)
- **vietcode-llm** — LLM provider abstraction (only `OllamaProvider` is real; `Anthropic`, `OpenAI`, `DeepSeek` are stubs)
- **vietcode-cli** — CLI binary via clap

## Build & Test

```powershell
# Build entire workspace
cargo build

# Run all tests (13 unit tests across 2 crates, all inline `#[cfg(test)]`)
cargo test

# Run a single crate's tests
cargo test -p vietcode-cortex
cargo test -p vietcode-core
```

No integration tests, no test fixtures, no snapshot tests (insta declared but unused). Tests need no external services.

## Toolchain

- Rust edition 2024 — requires Rust ≥ 1.85 (currently 1.95.0)
- Windows: uses GNU toolchain via MSYS2 GCC (`C:\msys64\ucrt64\bin\gcc.exe`) configured in `.cargo/config.toml`
- No `rustfmt.toml`, no `clippy.toml`, no `rust-toolchain.toml`

## CLI Usage

```powershell
# Build symbol index from current directory
cargo run -- index .

# Search symbols by name / doc comment
cargo run -- query "login"

# Filter by kind or file
cargo run -- query --kind fn --file src/auth.rs

# Trace callers / callees / impact
cargo run -- callers authenticate
cargo run -- callees main
cargo run -- impact login
```

Default database: `.vietcode/index.db`

## Key Conventions

- **Vietnamese** for comments, commit messages, and user-facing CLI
- All tests are inline `#[cfg(test)] mod tests { ... }` in source files
- `anyhow` for error propagation, `thiserror` for library errors
- `tokio` async runtime throughout
- `tree-sitter-rust` only — TypeScript/JavaScript language support is commented out
- `serde` for serialization, `rusqlite` with bundled SQLite + FTS5

## State (Phase 1)

- Symbol index (Tầng 1): works — parse → SQLite + FTS5 full-text search
- Relation graph (Tầng 2): works — call/import/type edges with BFS shortest-path
- Core pipeline: **stubs** — `Planner`, `Router`, `Pipeline`, `FileWatcher` all return no-op or dummy values
- LLM providers: only Ollama works; Anthropic/OpenAI/DeepSeek return placeholder text
- No CI/CD, no git repo, no `.gitignore` at root

## Gotchas

- `%SystemDrive%/` directory and `vs_buildtools.exe` in repo root are accidental artifacts
- `Cargo.lock` should be committed (binary crate workspace)
- Building the graph requires the symbol index to exist first (`build_index` in `index.rs` parses source and then builds graph)
- The `shortest_path` BFS uses `Vec::insert(0, …)` (O(n) enqueue) — not `VecDeque`
