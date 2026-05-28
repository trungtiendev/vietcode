//! DeepSeek provider — DeepSeek models.

use anyhow::Result;
use crate::provider::{ChatRequest, ChatResponse, Provider};

/// Placeholder — Phase 3 sẽ implement đầy đủ.
pub struct DeepSeekProvider;

impl DeepSeekProvider {
    pub fn new(_api_key: &str) -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Provider for DeepSeekProvider {
    fn name(&self) -> &str {
        "deepseek"
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        Ok(ChatResponse {
            content: "DeepSeek placeholder".into(),
            tokens_input: 0,
            tokens_output: 0,
            model: "deepseek-chat".into(),
        })
    }
}
