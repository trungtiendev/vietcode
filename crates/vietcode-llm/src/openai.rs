//! OpenAI provider — GPT-4 models.

use anyhow::Result;
use crate::provider::{ChatRequest, ChatResponse, Provider};

/// Placeholder — Phase 3 sẽ implement đầy đủ.
pub struct OpenAIProvider;

impl OpenAIProvider {
    pub fn new(_api_key: &str) -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        Ok(ChatResponse {
            content: "OpenAI placeholder".into(),
            tokens_input: 0,
            tokens_output: 0,
            model: "gpt-4".into(),
        })
    }
}
