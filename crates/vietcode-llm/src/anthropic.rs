//! Anthropic provider — Claude models.

use anyhow::Result;
use crate::provider::{ChatRequest, ChatResponse, Provider};

/// Placeholder — Phase 3 sẽ implement đầy đủ.
pub struct AnthropicProvider;

impl AnthropicProvider {
    pub fn new(_api_key: &str) -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        Ok(ChatResponse {
            content: "Anthropic placeholder".into(),
            tokens_input: 0,
            tokens_output: 0,
            model: "claude-sonnet".into(),
        })
    }
}
