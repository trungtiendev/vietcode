//! VietCode Core — Engine chính.
//!
//! Pipeline: Orchestrator → Coder (LLM) → Cortex context → Gate.

pub mod agent;
pub mod orchestrator;
pub mod pipeline;
pub mod planner;
pub mod router;

// Re-export for convenience
pub use agent::Coder;
pub use orchestrator::Orchestrator;
