//! `vietcode ask` — Gửi task cho AI agent.

use anyhow::{Context, Result};
use std::sync::Arc;
use vietcode_core::Orchestrator;

pub fn run(task: &str) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { run_async(task).await })
}

async fn run_async(task: &str) -> Result<()> {
    // ── Config ─────────────────────────────────────────
    let cortex_db = ".vietcode/index.db";
    let ollama_url = std::env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let model = std::env::var("VIETCODE_MODEL")
        .unwrap_or_else(|_| "codellama:7b".to_string());

    println!("VietCode Ask");
    println!("  Task: {}", task);
    println!("  Model: {}", model);
    println!("  Cortex DB: {}", cortex_db);
    println!();

    // ── Provider ───────────────────────────────────────
    let provider = vietcode_llm::ollama::OllamaProvider::new(&ollama_url);
    let provider: Arc<dyn vietcode_llm::provider::Provider> = Arc::new(provider);

    // ── Orchestrator ───────────────────────────────────
    let orch = Orchestrator::new(cortex_db);

    println!("Đang phân tích task...");
    let result = orch.run(task, provider, &model).await
        .context("Orchestrator failed")?;

    // ── Output ─────────────────────────────────────────
    println!("\n=== KẾT QUẢ ===");
    println!("Tokens: {} (model: {})", result.output.tokens_used, result.output.model);

    if let Some(ref file) = result.written_file {
        println!("Đã ghi code vào: {}", file);
    }

    println!("\nContext sử dụng:");
    println!("  Symbols liên quan: {}", result.context_used.relevant_symbols.len());
    for sym in &result.context_used.relevant_symbols {
        println!("    {}", sym);
    }

    println!("\n=== CODE ===");
    println!("{}", result.output.content);

    Ok(())
}
