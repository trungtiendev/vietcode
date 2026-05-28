//! Pipeline — Verification gates.
//!
//! Sau mỗi step, output qua gate để verify:
//! - Gate::Build → `cargo build`
//! - Gate::Test  → `cargo test`
//! - Gate::Lint  → `cargo clippy`

use anyhow::Result;
use std::time::Instant;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub enum Gate {
    Build,
    Test,
    Lint,
    Custom(String),
}

impl Gate {
    fn command(&self) -> (&str, Vec<&str>) {
        match self {
            Gate::Build => ("cargo", vec!["build"]),
            Gate::Test => ("cargo", vec!["test"]),
            Gate::Lint => ("cargo", vec!["clippy"]),
            Gate::Custom(cmd) => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.is_empty() {
                    ("", vec![])
                } else {
                    (parts[0], parts[1..].to_vec())
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct GateResult {
    pub gate: String,
    pub passed: bool,
    pub output: String,
    pub duration_ms: u64,
}

pub struct Pipeline;

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    pub fn new() -> Self {
        Self
    }

    pub async fn run_gate(&self, gate: &Gate) -> Result<GateResult> {
        let (cmd, args) = gate.command();
        let gate_name = format!("{:?}", gate);

        let start = Instant::now();
        let output = Command::new(cmd)
            .args(&args)
            .output()
            .await?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = if stderr.is_empty() {
            stdout
        } else if stdout.is_empty() {
            stderr
        } else {
            format!("{}\n{}", stdout, stderr)
        };

        Ok(GateResult {
            gate: gate_name,
            passed: output.status.success(),
            output: combined,
            duration_ms,
        })
    }

    pub async fn run_gates(&self, gates: &[Gate]) -> Result<Vec<GateResult>> {
        let mut results = Vec::new();
        for gate in gates {
            let result = self.run_gate(gate).await?;
            let passed = result.passed;
            results.push(result);
            if !passed {
                break;
            }
        }
        Ok(results)
    }
}
