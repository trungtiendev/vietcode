# VietCode — Lộ Trình Phát Triển

> Dựa trên kiến trúc JEAN (`JEAN_ARCHITECTURE.md`).
> Bắt đầu từ MVP nhỏ nhất có thể chạy được, tiến dần đến Software Genesis Engine.

---

## Nguyên Tắc Xây Dựng

1. **Ship sớm, ship thường xuyên** — mỗi phase kết thúc bằng thứ có thể dùng được
2. **Rust-first** — core engine viết bằng Rust (performance + safety cho AST processing)
3. **Tự ăn thức ăn của chính mình** — dùng VietCode để phát triển VietCode
4. **Local-first** — chạy được hoàn toàn offline với model local trước khi tích hợp cloud

---

## Kiến Trúc Codebase

```
vietcode/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── vietcode-cortex/        # Bộ não hiểu codebase
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parser.rs       # tree-sitter wrapper (đa ngôn ngữ)
│   │   │   ├── index.rs        # SQLite symbol index (Tầng 1)
│   │   │   ├── graph.rs        # Relation graph (Tầng 2)
│   │   │   ├── embed.rs        # Semantic embedding (Tầng 3 - tương lai)
│   │   │   └── watcher.rs      # File watcher + incremental update
│   │   └── tests/
│   │
│   ├── vietcode-core/          # Engine chính
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── orchestrator.rs # Điều phối pipeline
│   │   │   ├── planner.rs      # Phân rã task → plan
│   │   │   ├── pipeline.rs     # Pipeline trait + verification gates
│   │   │   ├── agent.rs        # Agent trait + các role (Coder, Reviewer, Tester)
│   │   │   └── router.rs       # Model router (local/cloud)
│   │   └── tests/
│   │
│   ├── vietcode-llm/           # Abstraction layer cho LLM providers
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── provider.rs     # Provider trait
│   │   │   ├── ollama.rs       # Local models via Ollama
│   │   │   ├── anthropic.rs    # Claude
│   │   │   ├── openai.rs       # GPT-4
│   │   │   └── deepseek.rs     # DeepSeek
│   │   └── tests/
│   │
│   └── vietcode-cli/           # Giao diện dòng lệnh
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/
│       │   │   ├── mod.rs
│       │   │   ├── index.rs    # `vietcode index` — build Cortex
│       │   │   ├── query.rs    # `vietcode query` — tra cứu symbol
│       │   │   ├── ask.rs      # `vietcode ask`   — gửi task cho agent
│       │   │   └── watch.rs    # `vietcode watch` — file watcher
│       │   └── config.rs
│       └── tests/
```

---

## Phase 1: Foundation — "Hiểu code trước khi code"

**Mục tiêu**: Cortex Tầng 1 hoạt động — parse codebase, index symbols, query được.

**Thời gian**: 4 tuần (nếu làm full-time) / 8 tuần (part-time)

### Week 1-2: Project Setup + Tree-sitter Parser

| # | Task | Output |
|---|---|---|
| 1.1 | `cargo init` workspace với 4 crates | Cargo.toml, cây thư mục |
| 1.2 | `vietcode-cortex::parser` — wrap tree-sitter-rust | Parse 1 file .rs → AST nodes |
| 1.3 | Extract symbols: `fn`, `struct`, `enum`, `trait`, `impl`, `mod`, `use`, `type`, `const`, `static` | `Vec<Symbol>` từ AST |
| 1.4 | Test: parse chính codebase của VietCode | Tự parse chính mình |
| 1.5 | Hỗ trợ thêm TypeScript/JavaScript (tree-sitter-typescript) | Parse file .ts/.tsx |

### Week 3-4: Symbol Index + Query

| # | Task | Output |
|---|---|---|
| 1.6 | `vietcode-cortex::index` — SQLite schema + FTS5 | Bảng `symbols`, full-text search |
| 1.7 | Build command: quét toàn bộ thư mục → parse → insert SQLite | `vietcode index ./src` |
| 1.8 | Query: `vietcode query "authenticate"` → list symbols | Kết quả dạng bảng |
| 1.9 | Query: `vietcode query --kind fn --file src/auth.rs` | Filter theo kind/file |
| 1.10 | Incremental update: file thay đổi → re-parse → update SQLite | `vietcode watch` |

### Week 4: CLI Polish + Docs

| # | Task | Output |
|---|---|---|
| 1.11 | CLI đẹp: `clap` + màu sắc + output format (table/json) | Trải nghiệm dùng được |
| 1.12 | Viết README, hướng dẫn cài đặt | Người khác clone về chạy được |
| 1.13 | CI/CD: GitHub Actions build + test | Build pass trên push |

### Phase 1 Deliverable

```
$ vietcode index .
Indexing 142 files... done.
  Symbols: 1,847 functions, 312 structs, 89 enums, 45 traits
  Time: 0.3s

$ vietcode query "login"
┌──────────┬──────────────────────────────────┬─────────────────────────┐
│ Kind     │ Name                             │ Location                │
├──────────┼──────────────────────────────────┼─────────────────────────┤
│ fn       │ handle_login                     │ src/auth/handler.rs:42  │
│ fn       │ login_with_oauth                 │ src/auth/oauth.rs:18    │
│ struct   │ LoginRequest                     │ src/auth/models.rs:7    │
│ fn       │ validate_login_credentials       │ src/auth/validate.rs:31 │
└──────────┴──────────────────────────────────┴─────────────────────────┘
```

---

## Phase 2: Cortex Tầng 2 — Relation Graph

**Mục tiêu**: Biết "sửa A ảnh hưởng gì".

**Thời gian**: 4 tuần

| # | Task |
|---|---|
| 2.1 | `vietcode-cortex::graph` — adjacency list trong SQLite |
| 2.2 | Build call graph: function A gọi function B |
| 2.3 | Build import graph: file X import module Y |
| 2.4 | Build type dependency: struct A chứa field kiểu B |
| 2.5 | Query: `vietcode impact src/auth.rs:42` → list callers + callees |
| 2.6 | Query: `vietcode callers authenticate` → ai gọi hàm này |
| 2.7 | Query: `vietcode path A B` → đường đi ngắn nhất giữa 2 symbol |
| 2.8 | Test: query trên chính codebase VietCode |

---

## Phase 3: Agent Pipeline — "Code đầu tiên"

**Mục tiêu**: Orchestrator + Planner + 1 Coder hoạt động, code được 1 feature đơn giản.

**Thời gian**: 8 tuần

| # | Task |
|---|---|
| 3.1 | `vietcode-llm::provider` trait + Ollama integration |
| 3.2 | `vietcode-llm::anthropic` + `vietcode-llm::openai` |
| 3.3 | `vietcode-core::agent` trait: `plan()`, `code()`, `review()`, `test()` |
| 3.4 | `vietcode-core::planner` — nhận yêu cầu → sinh plan |
| 3.5 | `vietcode-core::orchestrator` — chạy plan qua pipeline |
| 3.6 | `vietcode-core::pipeline` — Coder → Reviewer → Tester → Gate |
| 3.7 | `vietcode-core::router` — phân task local/cloud |
| 3.8 | CLI: `vietcode ask "thêm nút login vào navbar"` |
| 3.9 | Integration test: VietCode tự code 1 feature cho chính nó |

---

## Phase 4: Debate + Advanced

**Mục tiêu**: Multi-agent debate, self-evolving, cross-project memory.

**Thời gian**: 12+ tuần

| # | Task |
|---|---|
| 4.1 | Debate Chamber: Adversarial Pair pattern |
| 4.2 | Self-consistency voting cho local model |
| 4.3 | Spec-driven pipeline (Spec → Code → Test từ cùng 1 spec) |
| 4.4 | Virtual Engineering Organization (CTO, Architect, PM agents) |
| 4.5 | Cross-project memory (embedding + vector search) |

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (edition 2024) |
| AST Parser | tree-sitter + tree-sitter-rust, tree-sitter-typescript |
| Database | SQLite (rusqlite) + FTS5 |
| Embedding (tương lai) | sqlite-vec hoặc LanceDB |
| CLI | clap + comfy-table / tabled |
| Error handling | thiserror + anyhow |
| Async runtime | tokio |
| LLM client | reqwest (HTTP calls đến Ollama/Anthropic/OpenAI) |
| Logging | tracing |
| Testing | cargo test + insta (snapshot testing) |

---

## Quy Tắc Phát Triển

1. **Không merge nếu test fail** — CI phải xanh
2. **Mỗi PR ≤ 400 dòng** — dễ review
3. **Viết test trước khi viết code** (khi khả thi)
4. **Dùng chính VietCode để phát triển VietCode** từ Phase 3 trở đi
5. **Commit message bằng tiếng Việt** — `feat(cortex): thêm FTS5 full-text search`

---

## Trạng Thái Hiện Tại

- [x] JEAN_ARCHITECTURE.md — tài liệu kiến trúc tổng thể
- [ ] Phase 1: Foundation
- [ ] Phase 2: Relation Graph
- [ ] Phase 3: Agent Pipeline
- [ ] Phase 4: Advanced
