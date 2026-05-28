# VietCode

> Công cụ dòng lệnh Rust — index codebase, trace quan hệ function, và sinh code với LLM.

[![CI](https://github.com/trungtiendev/vietcode/actions/workflows/ci.yml/badge.svg)](https://github.com/trungtiendev/vietcode/actions)
![Rust](https://img.shields.io/badge/rust-1.95%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Tính năng

- **Index codebase** — parse toàn bộ file `.rs` bằng tree-sitter, lưu vào SQLite FTS5
- **Tìm kiếm symbol** — full-text search theo tên, doc comment, lọc theo kind/file
- **Trace quan hệ** — callers, callees, impact analysis, shortest path giữa 2 function
- **Sinh code với LLM** — gọi Ollama (local) hoặc DeepSeek (cloud), tự verify bằng `cargo build` + `cargo test`

## Cài đặt

Yêu cầu **Rust ≥ 1.85** (edition 2024).

```powershell
git clone https://github.com/trungtiendev/vietcode.git
cd vietcode
cargo build --release
```

**Windows**: cần MSYS2 GCC (cấu hình trong `.cargo/config.toml`). Nếu không có, cài [MSYS2](https://www.msys2.org/) rồi chạy:
```powershell
pacman -S mingw-w64-ucrt-x86_64-gcc
```

**Linux/macOS**: không cần thêm gì, cargo build trực tiếp.

## Sử dụng

### 1. Index codebase

```powershell
cargo run -- index .
```

Output:
```
Đang index codebase từ: .
Database: .vietcode/index.db

Hoàn thành:
  Files processed: 142
  Symbols found:  1847
  Thời gian:      320ms
```

### 2. Tìm kiếm symbol

```powershell
# Full-text search
cargo run -- query "authenticate"

# Filter theo kind + file
cargo run -- query --kind fn --file src/auth.rs "login"

# Output JSON
cargo run -- query --format json "User"
```

### 3. Trace quan hệ

```powershell
# Ai gọi hàm này?
cargo run -- callers authenticate

# Hàm này gọi những ai?
cargo run -- callees main

# Impact analysis (cả callers + callees)
cargo run -- impact login
```

### 4. Sinh code với LLM

**Ollama** (local, mặc định):
```powershell
cargo run -- ask "viết hàm validate_email trong file src/validator.rs"
```

**DeepSeek** (cloud):
```powershell
$env:VIETCODE_PROVIDER = "deepseek"
$env:DEEPSEEK_API_KEY = "sk-..."
$env:VIETCODE_MODEL = "deepseek-v4-pro"
cargo run -- ask "viết hàm authenticate trong file src/auth.rs"
```

**Anthropic** (cloud):
```powershell
$env:VIETCODE_PROVIDER = "anthropic"
$env:ANTHROPIC_API_KEY = "sk-ant-..."
cargo run -- ask "viết hàm authenticate trong file src/auth.rs"
```

**OpenAI** (cloud):
```powershell
$env:VIETCODE_PROVIDER = "openai"
$env:OPENAI_API_KEY = "sk-..."
cargo run -- ask "viết hàm authenticate trong file src/auth.rs"
```

Sau khi sinh code, pipeline tự động chạy `cargo build` + `cargo test` để verify.

## Kiến trúc

```
vietcode/
├── crates/
│   ├── vietcode-cortex/   # Bộ não hiểu codebase (parser, index, graph)
│   ├── vietcode-core/     # Agent pipeline orchestrator
│   ├── vietcode-llm/      # LLM provider (Ollama, DeepSeek)
│   └── vietcode-cli/      # CLI (clap)
├── JEAN_ARCHITECTURE.md   # Kiến trúc tổng thể
└── ROADMAP.md             # Lộ trình phát triển
```

3 tầng của Cortex:
- **Tầng 1** — Symbol Index: tree-sitter parse → SQLite FTS5
- **Tầng 2** — Relation Graph: call graph, dependency graph, BFS shortest path
- **Tầng 3** — Semantic Understanding (tương lai: embedding + vector search)

## Trạng thái

| Thành phần | Trạng thái |
|---|---|
| Symbol index (Tầng 1) | ✓ Hoạt động |
| Relation graph (Tầng 2) | ✓ Hoạt động |
| Ollama provider | ✓ Hoạt động |
| DeepSeek provider | ✓ Hoạt động |
| Anthropic provider | ✓ Hoạt động |
| OpenAI provider | ✓ Hoạt động |
| Pipeline gates (build/test) | ✓ Hoạt động |
| CI/CD (GitHub Actions) | ✓ Ubuntu + Windows |
| Planner, Router, FileWatcher | Stub |
| Tầng 3 (Semantic) | Chưa bắt đầu |

Xem chi tiết tại [ROADMAP.md](ROADMAP.md) và [JEAN_ARCHITECTURE.md](JEAN_ARCHITECTURE.md).

## License

MIT
