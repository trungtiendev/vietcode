//! Pipeline — Verification gates.
//!
//! Sau mỗi step, output qua gate để verify:
//! - Gate::Build → `cargo build`
//! - Gate::Test  → `cargo test`
//! - Gate::Lint  → `cargo clippy`

use anyhow::Result;

/// Các loại gate.
#[derive(Debug, Clone)]
pub enum Gate {
    Build,
    Test,
    Lint,
    Custom(String),
}

/// Kết quả của một gate check.
#[derive(Debug)]
pub struct GateResult {
    pub passed: bool,
    pub output: String,
    pub duration_ms: u64,
}

/// Pipeline quản lý các gate.
pub struct Pipeline;

impl Pipeline {
    pub fn new() -> Self {
        Self
    }

    /// Chạy một gate — Phase 3 sẽ implement thực sự.
    pub async fn run_gate(&self, _gate: &Gate) -> Result<GateResult> {
        Ok(GateResult {
            passed: true,
            output: String::new(),
            duration_ms: 0,
        })
    }
}
