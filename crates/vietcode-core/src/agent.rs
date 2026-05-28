//! Agent trait và concrete implementations.
//! Coder agent: sinh code từ LLM provider + Cortex context.

use anyhow::{Context, Result};
use std::sync::Arc;

/// Kết quả từ một agent.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub content: String,
    pub tokens_used: u64,
    pub model: String,
}

/// Context từ codebase (do Cortex cung cấp).
#[derive(Debug, Clone)]
pub struct CodebaseContext {
    pub relevant_symbols: Vec<String>,
    pub code_patterns: Vec<String>,
    pub target_file: Option<String>,
}

// ── Coder Agent ───────────────────────────────────────────────

pub struct Coder {
    provider: Arc<dyn vietcode_llm::provider::Provider>,
    model: String,
}

impl Coder {
    pub fn new(provider: Arc<dyn vietcode_llm::provider::Provider>, model: &str) -> Self {
        Self { provider, model: model.to_string() }
    }

    pub async fn generate_code(&self, task: &str, context: &CodebaseContext) -> Result<AgentOutput> {
        let prompt = self.build_prompt(task, context);
        self.call_llm(&prompt).await
    }

    fn build_prompt(&self, task: &str, context: &CodebaseContext) -> String {
        let mut p = String::new();
        p.push_str("Bạn là lập trình viên Rust. Chỉ trả về code, không giải thích.\n\n");

        if !context.code_patterns.is_empty() {
            p.push_str("--- PATTERNS CODEBASE ---\n");
            for pat in &context.code_patterns { p.push_str(pat); p.push('\n'); }
            p.push('\n');
        }
        if !context.relevant_symbols.is_empty() {
            p.push_str("--- SYMBOLS LIÊN QUAN ---\n");
            for s in &context.relevant_symbols { p.push_str(&format!("  {}\n", s)); }
            p.push('\n');
        }
        if let Some(ref f) = context.target_file {
            p.push_str(&format!("--- GHI VÀO: {} ---\n\n", f));
        }
        p.push_str("--- TASK ---\n");
        p.push_str(task);
        p.push('\n');
        p
    }

    async fn call_llm(&self, prompt: &str) -> Result<AgentOutput> {
        let req = vietcode_llm::provider::ChatRequest {
            model: self.model.clone(),
            messages: vec![vietcode_llm::provider::ChatMessage {
                role: "user".into(), content: prompt.to_string(),
            }],
            temperature: Some(0.3),
            max_tokens: Some(2048),
        };
        let resp = self.provider.chat(req).await.context("LLM failed")?;
        Ok(AgentOutput {
            content: extract_code(&resp.content),
            tokens_used: resp.tokens_input + resp.tokens_output,
            model: resp.model,
        })
    }
}

fn extract_code(response: &str) -> String {
    let r = response.trim();
    if let Some(start) = r.find("```") {
        let rest = &r[start + 3..];
        let code_start = rest.find('\n').map(|i| i + 1).unwrap_or(0);
        if let Some(end) = rest.rfind("```") {
            return rest[code_start..end].trim().to_string();
        }
        return rest[code_start..].trim().to_string();
    }
    r.to_string()
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use vietcode_llm::provider::*;

    struct DummyProvider;
    #[async_trait::async_trait]
    impl Provider for DummyProvider {
        fn name(&self) -> &str { "dummy" }
        async fn chat(&self, _: ChatRequest) -> Result<ChatResponse> {
            Ok(ChatResponse { content: "fn test() {}".into(), tokens_input: 1, tokens_output: 1, model: "d".into() })
        }
    }

    #[test]
    fn extract_code_strips_fence() {
        assert_eq!(extract_code("```rust\nfn hi() {}\n```"), "fn hi() {}");
    }

    #[test]
    fn extract_code_no_fence() {
        assert_eq!(extract_code("fn hi() {}"), "fn hi() {}");
    }

    #[test]
    fn prompt_includes_context() {
        let c = Coder::new(Arc::new(DummyProvider), "m");
        let ctx = CodebaseContext {
            relevant_symbols: vec!["User".into()],
            code_patterns: vec!["fn f() {}".into()],
            target_file: Some("x.rs".into()),
        };
        let p = c.build_prompt("test", &ctx);
        assert!(p.contains("User") && p.contains("PATTERNS") && p.contains("x.rs"));
    }
}
