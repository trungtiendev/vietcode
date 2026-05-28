//! OpenAI provider — GPT-4 models.
//!
//! Sử dụng Chat Completions API tại https://api.openai.com/v1/chat/completions.
//! Cần biến môi trường OPENAI_API_KEY.

use crate::provider::{ChatRequest, ChatResponse, Provider};
use anyhow::{Context, Result};

const DEFAULT_BASE_URL: &str = "https://api.openai.com";

pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    http_client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_base_url(api_key: &str, base_url: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            }).collect::<Vec<_>>(),
            "temperature": request.temperature.unwrap_or(0.7),
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "stream": false,
        });

        let resp = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("OpenAI API request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error {}: {}", status, text);
        }

        let json: serde_json::Value = resp.json().await?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = &json["usage"];

        Ok(ChatResponse {
            content,
            tokens_input: usage["prompt_tokens"].as_u64().unwrap_or(0),
            tokens_output: usage["completion_tokens"].as_u64().unwrap_or(0),
            model: request.model,
        })
    }
}
