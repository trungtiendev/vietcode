//! Router — Phân task vào model phù hợp.
//!
//! Dựa trên complexity, domain, budget để chọn local hoặc cloud model.

use anyhow::Result;

/// Các loại model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelTier {
    Local(String),     // model name (vd: "codellama:7b")
    Cloud(String),     // model name (vd: "claude-sonnet-4-20250514")
}

/// Kết quả route.
#[derive(Debug)]
pub struct RouteDecision {
    pub tier: ModelTier,
    pub reason: String,
    pub estimated_tokens: u64,
}

pub struct Router;

impl Router {
    pub fn new() -> Self {
        Self
    }

    /// Route task đến model phù hợp — Phase 3 sẽ implement logic thực.
    pub fn route(&self, _task: &str) -> Result<RouteDecision> {
        Ok(RouteDecision {
            tier: ModelTier::Cloud("claude-sonnet".into()),
            reason: "Router placeholder — mặc định cloud".into(),
            estimated_tokens: 0,
        })
    }
}
