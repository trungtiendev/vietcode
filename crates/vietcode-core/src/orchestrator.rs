//! Orchestrator — Điều phối pipeline.
//!
//! Nhận task → query Cortex lấy context → gọi Coder → ghi file → verify gate.

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use vietcode_cortex::index::SymbolIndex;
use vietcode_llm::provider::Provider;

use crate::agent::{AgentOutput, CodebaseContext, Coder};
use crate::pipeline::{Gate, GateResult, Pipeline};

/// Orchestrator điều phối toàn bộ flow xử lý task.
pub struct Orchestrator {
    cortex_db: String,
}

impl Orchestrator {
    pub fn new(cortex_db: &str) -> Self {
        Self {
            cortex_db: cortex_db.to_string(),
        }
    }

    /// Chạy task với LLM provider.
    ///
    /// Flow:
    /// 1. Parse task để lấy intent cơ bản
    /// 2. Query Cortex lấy symbols + patterns liên quan
    /// 3. Gọi Coder sinh code
    /// 4. Ghi code vào file (nếu task chỉ định)
    /// 5. Chạy cargo build để verify
    pub async fn run(
        &self,
        task: &str,
        provider: Arc<dyn Provider>,
        model: &str,
    ) -> Result<TaskResult> {
        // Parse intent: tìm từ khóa "file X", "hàm Y", "trong file Z"
        let intent = parse_intent(task);

        // Query Cortex
        let context = self.gather_context(&intent)?;

        // Gọi Coder
        let coder = Coder::new(provider, model);
        let output = coder.generate_code(task, &context).await?;

        // Ghi file nếu có target
        let mut written_file = None;
        if let Some(ref file_path) = intent.target_file {
            let path = Path::new(file_path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, &output.content)
                .context("Không thể ghi file")?;
            written_file = Some(file_path.clone());
        }

        // Chạy pipeline gates nếu có file được ghi
        let gate_results = if written_file.is_some() {
            let pipeline = Pipeline::new();
            let gates = vec![Gate::Build, Gate::Test];
            pipeline.run_gates(&gates).await.unwrap_or_default()
        } else {
            vec![]
        };

        Ok(TaskResult {
            output,
            context_used: context,
            written_file,
            intent,
            gate_results,
        })
    }

    /// Query Cortex để lấy context cho task.
    fn gather_context(&self, intent: &TaskIntent) -> Result<CodebaseContext> {
        let db_path = Path::new(&self.cortex_db);
        let index = SymbolIndex::open(db_path)
            .context("Không mở được Cortex database. Chạy `vietcode index` trước.")?;

        // Tìm symbols khớp với query
        let query = &intent.search_query;
        let symbols = index.search(query).unwrap_or_default();

        let relevant_symbols: Vec<String> = symbols.iter()
            .map(|s| format!("{} {} ({}:{})", s.kind.as_str(), s.name, s.file_path, s.line_start))
            .collect();

        // Lấy code patterns từ file đích (nếu có)
        let mut code_patterns = Vec::new();
        if let Some(ref target) = intent.target_file {
            let target_path = Path::new(target);
            if target_path.exists()
                && let Ok(source) = std::fs::read_to_string(target_path) {
                    code_patterns.push(source);
                }
        }

        // Nếu không có file đích, lấy pattern từ file gần nhất trong kết quả
        if code_patterns.is_empty()
            && let Some(sym) = symbols.first() {
                let sym_path = Path::new(&sym.file_path);
                if sym_path.exists()
                    && let Ok(source) = std::fs::read_to_string(sym_path) {
                        // Chỉ lấy 50 dòng đầu làm pattern
                        let lines: Vec<&str> = source.lines().take(50).collect();
                        code_patterns.push(lines.join("\n"));
                    }
            }

        Ok(CodebaseContext {
            relevant_symbols,
            code_patterns,
            target_file: intent.target_file.clone(),
        })
    }
}

/// Kết quả của một task.
#[derive(Debug)]
pub struct TaskResult {
    pub output: AgentOutput,
    pub context_used: CodebaseContext,
    pub written_file: Option<String>,
    pub intent: TaskIntent,
    pub gate_results: Vec<GateResult>,
}

/// Intent parsed từ natural language task.
#[derive(Debug, Clone)]
pub struct TaskIntent {
    pub search_query: String,
    pub target_file: Option<String>,
    pub function_name: Option<String>,
}

/// Parse intent đơn giản từ task tiếng Việt.
fn parse_intent(task: &str) -> TaskIntent {
    let task_lower = task.to_lowercase();

    // Tìm file đích: "trong file X.rs", "file X.rs", "vào file X"
    let target_file = find_file_in_task(&task_lower);

    // Tìm tên hàm: "hàm X", "function X", "viết hàm X"
    let function_name = find_function_in_task(&task_lower);

    // Query cho Cortex: dùng các từ khóa chính
    let search_query = task
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .take(5)
        .collect::<Vec<_>>()
        .join(" ");

    TaskIntent {
        search_query,
        target_file,
        function_name,
    }
}

fn find_file_in_task(task: &str) -> Option<String> {
    // Pattern: "file X.rs", "trong file X", "vào X.rs"
    for prefix in &["file ", "trong file ", "vào "] {
        if let Some(pos) = task.find(prefix) {
            let rest = &task[pos + prefix.len()..];
            let file = rest.split_whitespace().next()?;
            let file = file.trim_matches(|c: char| c == '"' || c == '\'');
            if file.contains('.') || file.ends_with(".rs") {
                return Some(file.to_string());
            }
            return Some(format!("{}.rs", file));
        }
    }
    // Nếu task chứa path kết thúc .rs
    for word in task.split_whitespace() {
        if word.ends_with(".rs") {
            return Some(word.to_string());
        }
    }
    None
}

fn find_function_in_task(task: &str) -> Option<String> {
    for prefix in &["hàm ", "function ", "fn ", "viết hàm ", "tạo hàm "] {
        if let Some(pos) = task.to_lowercase().find(prefix) {
            let rest = &task[pos + prefix.len()..];
            let name = rest.split_whitespace().next()?;
            let name = name.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
            return Some(name.to_string());
        }
    }
    None
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_intent_with_file_and_function() {
        let intent = parse_intent("viết hàm validate_email trong file src/validator.rs");
        assert_eq!(intent.function_name.as_deref(), Some("validate_email"));
        assert!(intent.target_file.as_deref().unwrap().contains("validator.rs"));
    }

    #[test]
    fn parse_intent_simple() {
        let intent = parse_intent("thêm nút login vào navbar");
        assert!(!intent.search_query.is_empty());
    }
}
