//! Planner — Phân rã task thành implementation plan.
//!
//! Nhận yêu cầu tự nhiên → sinh danh sách bước cụ thể.

use anyhow::Result;

/// Một bước trong plan.
#[derive(Debug, Clone)]
pub struct PlanStep {
    pub order: usize,
    pub description: String,
    pub file_path: Option<String>,
    pub depends_on: Vec<usize>,
}

/// Implementation plan hoàn chỉnh.
#[derive(Debug)]
pub struct Plan {
    pub steps: Vec<PlanStep>,
    pub summary: String,
}

pub struct Planner;

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner {
    pub fn new() -> Self {
        Self
    }

    /// Sinh plan từ yêu cầu — Phase 3 sẽ gọi LLM.
    pub async fn plan(&self, _task: &str) -> Result<Plan> {
        Ok(Plan {
            steps: vec![],
            summary: "Planner placeholder".into(),
        })
    }
}
