//! Provider trait — interface chung cho mọi LLM backend.

use anyhow::Result;

/// Cấu hình cho một request.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Kết quả từ LLM.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub model: String,
}

/// Trait chung cho tất cả LLM provider.
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
}
