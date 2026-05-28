# JEAN: Software Genesis Engine

> Kiến trúc tổng thể cho đội quân Agent code — vượt xa Claude Code, Cursor, Copilot.

---

## Mục Lục

1. [Kiến Trúc Cơ Bản](#1-kiến-trúc-cơ-bản)
2. [Codebase Cortex](#2-codebase-cortex)
3. [Debate Chamber](#3-debate-chamber)
4. [Local-First Intelligence](#4-local-first-intelligence)
5. [Simulation Engine](#5-simulation-engine)
6. [Cross-Project Memory](#6-cross-project-memory)
7. [Self-Evolving Architecture](#7-self-evolving-architecture)
8. [Multi-Temporal Reasoning](#8-multi-temporal-reasoning)
9. [Intent-Driven Development](#9-intent-driven-development)
10. [Virtual Engineering Organization](#10-virtual-engineering-organization)
11. [Antifragile Development](#11-antifragile-development)
12. [Zero-Token Operations](#12-zero-token-operations)
13. [Adversarial Co-Evolution](#13-adversarial-co-evolution)
14. [Spec-As-Truth](#14-spec-as-truth)
15. [Lộ Trình Phát Triển](#15-lộ-trình-phát-triển)
16. [So Sánh Với Công Cụ Hiện Tại](#16-so-sánh-với-công-cụ-hiện-tại)

---

## 1. Kiến Trúc Cơ Bản

### Orchestrator + Specialists

```
┌──────────────────────────────────────────────────────────────┐
│                      ORCHESTRATOR                             │
│  (Điều phối, không code. Giống PM + Architect + DevOps)       │
└──────────┬──────────┬──────────┬──────────┬──────────────────┘
           │          │          │          │
     ┌─────▼──┐  ┌───▼───┐  ┌──▼────┐  ┌──▼─────┐
     │PLANNER │  │CODER  │  │REVIEW │  │TESTER  │
     │Phân rã │  │Theo   │  │Adver- │  │Sinh +  │
     │task    │  │domain │  │sarial │  │chạy test│
     └────────┘  └───────┘  └───────┘  └────────┘
```

### Vai trò từng agent

| Vai trò | Trách nhiệm |
|---|---|
| **Orchestrator** | Nhận task từ user → phân rã → phân công → tổng hợp kết quả |
| **Planner** | Biến yêu cầu thành implementation plan có bước cụ thể, dependency rõ ràng |
| **Coder (specialized)** | Frontend, Backend, Database, Infra — mỗi con thuần thục một stack riêng |
| **Reviewer / Critic** | Đọc output của Coder, tìm bug, đánh giá style, bảo mật, performance |
| **Tester** | Tự sinh test case từ plan, chạy, báo cáo. Không đợi dev viết xong mới test |
| **Context Curator** | Duy trì knowledge graph của codebase, index file, dependency map |

### Pipeline với Verification Gates

```
User request
    └─ Orchestrator phân rã thành Plan
        └─ Mỗi step: Coder implement → Reviewer duyệt → Tester verify
            └─ Fail gate → quay lại Coder với feedback cụ thể
        └─ Tất cả step pass → Orchestrator merge, chạy integration test
```

### Multi-Model Router

Mỗi task được route đến model phù hợp nhất dựa trên:
- **Độ phức tạp** (token count + số file liên quan)
- **Domain** (frontend → model giỏi CSS/React, backend → model giỏi Rust/DB)
- **Ngân sách** (task đơn giản → model rẻ, task khó → model mạnh)

---

## 2. Codebase Cortex

Hệ thống **3 tầng representation**, tự build khi nhập codebase, incremental update, query được theo nhiều chiều.

### Tầng 1: Symbol Index

```
AST parser → function, class, type, import, export, interface, route, schema
Store: SQLite + FTS5 full-text search
Query: "func nào nhận *sql.DB làm param?"
       "route DELETE /api/users/:id ở file nào?"
```

### Tầng 2: Relation Graph

```
Call graph, import graph, type dependency, data flow
Store: adjacency list trong SQLite
Query: "nếu sửa hàm A, những hàm nào bị ảnh hưởng?"
       "middleware nào wrap route này?"
```

### Tầng 3: Semantic Understanding

```
LLM embedding vector của từng function/module
+ summary tự sinh (mô tả 1-2 câu)
+ tag tự động: "auth", "payment", "db-migration"
Store: sqlite-vec / LanceDB
Query: "tìm code xử lý thanh toán qua Stripe"
```

### Incremental Update

```
File changed (git hook / file watcher)
    ├─ Tree-sitter re-parse file đã đổi
    ├─ Cập nhật symbol của file đó (Tầng 1)
    ├─ Cập nhật edges liên quan trong graph (Tầng 2)
    └─ Nếu thay đổi > threshold → re-embed (Tầng 3)
```

### Tiết Kiệm Token

| Scenario | Không có Cortex | Với Cortex |
|---|---|---|
| "Sửa hàm authenticate" | Đọc 5-10 file, ~15K token | Query → đọc đúng 2 file, ~3K token |
| Session mới, cùng project | Đọc lại tất cả, ~20K token | Query vài lần, ~1K token |
| Code review | Đọc PR diff + context, ~30K token | Impacted callers + existing patterns |

---

## 3. Debate Chamber

Lớp phản biện **trước khi hành động** — Planner sinh nhiều phương án, các agent tranh luận, chọn phương án tối ưu.

### Flow

```
User Request
    │
    ▼
Planner sinh N phương án (Plan A, B, C...)
    │
    ▼
┌─── DEBATE CHAMBER ───────────────────────────┐
│  Agent A: "Dùng PostgreSQL vì ACID"          │
│  Agent B: "Dùng SQLite vì đơn giản"          │
│  Agent C: "SQLite + WAL mode, vừa ACID vừa nhẹ"│
│                                               │
│  → Round 1: Mỗi agent phản biện agent khác    │
│  → Round 2: Agent phản hồi, củng cố lập luận  │
│  → Round 3: Tổng hợp, chấm điểm từng phương án │
│                                               │
│  → Output: Plan C thắng (điểm 8.7/10)         │
└───────────────────────────────────────────────┘
```

### Các Pattern Debate

| Pattern | Agent | Token cost | Phù hợp |
|---|---|---|---|
| **Adversarial Pair** | 2 | Thấp nhất | MVP |
| **Round-Robin** | N | O(N² × rounds) | Full system |
| **Judge-Jury** | 2-3 + 1 Judge | Trung bình | Quyết định phức tạp |
| **Tree-of-Thought** | Nhiều | Cao nhất | Không gian giải pháp rộng |

### Tiêu Chí Chấm Điểm

| Tiêu chí | Trọng số | Mô tả |
|---|---|---|
| Correctness | 30% | Thỏa mãn requirement, không bug |
| Simplicity | 20% | Độ phức tạp triển khai |
| Performance | 15% | Time/space complexity |
| Maintainability | 15% | Người đọc hiểu được không |
| Security | 10% | Bề mặt tấn công |
| Extensibility | 10% | Dễ mở rộng khi requirement thay đổi |

---

## 4. Local-First Intelligence

### Router Thông Minh

| Loại task | Model | Lý do |
|---|---|---|
| Boilerplate, CRUD, CSS | Local 7B | Pattern rõ ràng |
| Refactor đơn giản | Local 7B | Tree-sitter làm phần cứng |
| Viết test unit | Local 7B | Input → output xác định |
| Viết documentation | Local 3B | Tóm tắt code |
| Review code (anti-pattern) | Local 7B × 3 ensemble | Pattern recognition |
| Format, lint fix | Không cần model | Tool deterministic |
| Kiến trúc, design pattern | Cloud model | Cần reasoning sâu |
| Auth, crypto, security | Cloud model | Không được sai |
| Debug bug phức tạp | Cloud model | Cần suy luận đa hypothesis |

### Self-Consistency Voting

```
Cùng 1 subtask → Local 7B × 3 (temperature khác nhau)
    ├─ 3/3 giống → confidence cao
    ├─ 2/3 giống → dùng đa số
    └─ Cả 3 khác → escalate lên cloud review
```

### Decompose Cực Nhỏ

- Mỗi subtask fit trong 2K token prompt, output <500 token
- Có input/output contract rõ ràng (Planner viết)
- Có ít nhất 1 example trong prompt
- Feedback loop chặt: lỗi compiler → sửa → rẻ đến mức loop 10 lần

### Cloud Reviewer + Local Coder

```
Planner (cloud, 1 lần) → Coder (local 7B, parallel) → Reviewer (cloud, rẻ hơn generate 3-5x)
→ Tester (local) → Gate check → merge
```

**Tỉ lệ**: 25% cloud token, 75% local → tiết kiệm ~70% so với all-cloud.

---

## 5. Simulation Engine

"Chạy code trước khi viết code" — sandbox mô phỏng logic dưới dạng abstract model.

```
User: "Thêm rate limiter cho API login"
    │
    ▼
Planner sinh abstract model:
    {
      input: [requests/s, IP, time_window],
      state: { counter: Map<IP, int>, last_reset: timestamp },
      output: { allow | block },
      invariants: [
        "không IP nào vượt quá limit trong window",
        "counter reset sau window"
      ]
    }
    │
    ▼
Simulation Engine chạy:
    ├─ Property-based test: 10000 IP, random traffic
    ├─ Tìm edge case: bypass rate limit ở biên window
    ├─ Tự sinh constraint: "window phải sliding, không fixed"
    └─ Output: refined spec → Coder implement 0 bug từ lần đầu
```

**Công nghệ**: TLA+/Alloy model checking + LLM dịch requirement → formal spec → verify → sinh code.

---

## 6. Cross-Project Memory

Học từ tất cả project đã làm, nhận ra pattern xuyên suốt.

```
JEAN làm Project A (SaaS): học pattern JWT auth, multi-tenant DB
    │
    ▼ Lưu vào Global Knowledge Base
    │
JEAN gặp Project B (E-commerce):
    ├─ Nhận ra: "cần auth giống Project A"
    ├─ Áp dụng pattern đã proven, đã battle-tested
    └─ KHÔNG code lại từ đầu
```

**Cơ chế**:
- Lưu abstract pattern (intent + constraint + architecture decision + performance data)
- Embedding vào vector space
- Query similarity khi gặp task mới
- Opt-out privacy: local-only mode

---

## 7. Self-Evolving Architecture

Theo dõi, đo lường, tự đề xuất cải tiến kiến trúc.

```
Monitoring:
  - Endpoint /api/search: p99 latency 2.3s, đang tăng
  - 80% truy vấn full-text scan
  - DB CPU: 78%

→ Tự sinh proposal:
  "Thêm GIN index → latency giảm 90% (5 phút)
   Thêm Redis cache → DB load giảm 70% (2h)
   Expected: 2.3s → 50ms, CPU 78% → 15%"

→ Debate Chamber → chọn → PR tự động
```

### Architectural Fitness Landscape

```
Hiện tại: Monolith PostgreSQL
    ├─ Mutant A: + Redis cache (latency ↓, cost ↑)
    ├─ Mutant B: + Read Replicas (throughput ↑, complexity ↑)
    ├─ Mutant C: CQRS + Event Sourcing (scale ↑↑, complexity ↑↑↑)
    └─ Mutant D: Microservices (team scale ↑, latency ↑)

Mỗi mutant: cost estimate, performance projection, risk, migration path
Khi traffic tăng 10x → tự đề xuất chuyển đổi
```

---

## 8. Multi-Temporal Reasoning

Suy nghĩ trên nhiều trục thời gian — mỗi quyết định code được gắn time-horizon tag.

| Time horizon | Ví dụ | Xử lý bởi |
|---|---|---|
| **NOW** (ms) | Race condition, memory allocation | Static Analyzer + Compiler |
| **SOON** (hours) | Test coverage, deploy downtime | Tester + Reviewer |
| **LATER** (months) | Technical debt, maintainability | Maintainability Analyzer |
| **MUCH LATER** (years) | Scalability, architecture pivot | Architecture Evolution Engine |

---

## 9. Intent-Driven Development

Từ "viết gì" thành "muốn gì".

### Level 1 - Command (hiện tại)
```
User: "Tạo file auth.ts với hàm authenticate dùng JWT"
Agent: "Đây là code."
```

### Level 2 - Intent (JEAN)
```
User: "Tôi muốn người dùng đăng nhập an toàn"
JEAN:
  ├─ Debate: OAuth2? JWT? Session? Passkey?
  ├─ Chọn Passkey + JWT fallback
  ├─ Sinh: DB schema, API, middleware, test, doc, migration
  └─ "Xong. Tôi chọn Passkey vì [lý do]."
```

### Level 3 - Vision (JEAN tương lai)
```
User: "Xây app giống Grab, nhưng cho chợ hải sản"
JEAN:
  ├─ Phân tích Grab → adapt sang domain
  ├─ Sinh spec → debate → architect → code → deploy
  └─ "App đã sẵn sàng. Dashboard đây."
```

---

## 10. Virtual Engineering Organization

Mô phỏng cả một tổ chức kỹ sư — phân cấp, accountability, learning loop.

```
┌──────────────────────────────────────────────────┐
│  LEADERSHIP: CTO, Architect, PM Agent            │
│  EXECUTION: Senior Dev, Junior Dev, DevOps, Sec  │
│  QUALITY: QA, Reviewer, Performance Agent        │
│  CULTURE: Daily standup, Retro, Knowledge share  │
└──────────────────────────────────────────────────┘
```

---

## 11. Antifragile Development

Càng dùng càng giỏi — không phải càng dùng càng thấy điểm yếu.

| Giai đoạn | Học gì |
|---|---|
| **Short-term** | Naming convention, thói quen của user |
| **Medium-term** | Style guide team, domain knowledge, landmine codebase |
| **Long-term** | Pattern proven, pattern gây bug, điểm mạnh/yếu team |
| **Global** (opt-in) | Pattern phổ biến nhất, bug thường gặp, migration path hiệu quả |

---

## 12. Zero-Token Operations

Nhiều thứ không cần LLM. Mục tiêu: 40-50% operation deterministic.

| Operation | Công cụ |
|---|---|
| Đổi tên hàm | Tree-sitter rename |
| Tìm biến global | Cortex query |
| Format code | Formatter (rustfmt, prettier) |
| Sinh boilerplate CRUD | Template engine từ schema |
| Kiểm tra PR phá test | Chạy test suite |
| Thêm log vào function public | AST transformation |

---

## 13. Adversarial Co-Evolution

Mô hình GAN áp dụng vào software engineering.

```
BLUE TEAM (Coder)              RED TEAM (Attacker)
┌──────────────────┐          ┌──────────────────┐
│ Viết code đúng,   │    ⚔️    │ Tìm cách phá code │
│ an toàn, hiệu quả │          │ của Blue Team     │
└──────────────────┘          └──────────────────┘

Mỗi lần Red Team thắng → cả hai agent cùng học:
  - Blue Team học cách phòng thủ
  - Red Team học kỹ thuật tấn công mới
  
Generation 1: Red dễ dàng phá
Generation 10: Red hầu như không tìm ra lỗi → code battle-tested
```

Red Team thử: refactor để phá, thay đổi dependency, mô phỏng traffic bất thường.

---

## 14. Spec-As-Truth

Specification là nguồn sự thật duy nhất — executable contract, không phải document tĩnh.

```
User Intent
    │
    ▼
FORMAL SPEC ← NGUỒN SỰ THẬT DUY NHẤT
    │
    ├─ Coder implement từ spec
    ├─ Tester sinh property-based test từ spec
    └─ Reviewer audit code dựa trên spec
```

Spec định nghĩa: input/output types, invariants, state transitions.

---

## 15. Lộ Trình Phát Triển

### Phase 1: Foundation (3 tháng)
- Cortex Tầng 1 + 2 (symbol index + relation graph) cho Rust
- Orchestrator + Planner + 1 Coder (Rust specialist)
- Router local/cloud cơ bản
- Zero-token operations

### Phase 2: Intelligence (3 tháng)
- Debate Chamber (Adversarial Pair)
- Spec-driven development pipeline
- Self-consistency voting cho local model
- Reviewer adversarial

### Phase 3: Evolution (6 tháng)
- Cross-project memory
- Simulation Engine (model checking)
- Self-evolving architecture
- Multi-temporal reasoning

### Phase 4: Genesis (6 tháng)
- Virtual Engineering Organization đầy đủ
- Antifragile learning loop
- Intent-driven development Level 2-3
- Adversarial co-evolution (GAN for code)
- Global collective intelligence (opt-in)

---

## 16. So Sánh Với Công Cụ Hiện Tại

| Tính năng | Claude Code | Cursor | Copilot | JEAN |
|---|---|---|---|---|
| Hiểu codebase semantic | ❌ | ❌ | ❌ | ✅ Cortex |
| Cross-session memory | ❌ | ❌ | ❌ | ✅ |
| Multi-agent debate trước khi code | ❌ | ❌ | ❌ | ✅ Debate Chamber |
| Verify thiết kế trước khi code | ❌ | ❌ | ❌ | ✅ Simulation Engine |
| Học xuyên project | ❌ | ❌ | ❌ | ✅ Cross-project |
| Tự đề xuất cải tiến kiến trúc | ❌ | ❌ | ❌ | ✅ Self-evolving |
| Local model tối ưu | ❌ | ❌ | ❌ | ✅ Local-first |
| Red Team adversarial | ❌ | ❌ | ❌ | ✅ |
| Zero-token operations | ❌ | ❌ | ❌ | ✅ 40-50% tasks |
| Intent-driven (Level 2-3) | ❌ | ❌ | ❌ | ✅ |
| Multi-temporal reasoning | ❌ | ❌ | ❌ | ✅ |

---

> **JEAN không phải "code agent tốt hơn". Là Software Genesis Engine.**
>
> Bắt đầu từ intent, tự tranh luận chọn hướng đi, tự hiểu codebase đến từng symbol, tự mô phỏng ngăn bug từ trong trứng nước, tự tiến hóa kiến trúc, và càng dùng càng giỏi.
