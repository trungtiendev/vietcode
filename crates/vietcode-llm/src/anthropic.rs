//! Anthropic provider — Claude models.
//!
//! Sử dụng Messages API tại https://api.anthropic.com/v1/messages.
//! Cần biến môi trường ANTHROPIC_API_KEY.

use crate::provider::{ChatRequest, ChatResponse, Provider};
use anyhow::{Context, Result};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    http_client: reqwest::Client,
}

impl AnthropicProvider {
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
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/v1/messages", self.base_url);

        let system = request.messages.iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());

        let messages: Vec<_> = request.messages.iter()
            .filter(|m| m.role != "system")
            .map(|m| serde_json::json!({
                "role": if m.role == "assistant" { "assistant" } else { "user" },
                "content": m.content,
            }))
            .collect();

        let mut body = serde_json::json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "messages": messages,
            "stream": false,
        });

        if let Some(ref sys) = system {
            body["system"] = serde_json::json!(sys);
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let resp = self.http_client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Anthropic API request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error {}: {}", status, text);
        }

        let json: serde_json::Value = resp.json().await?;

        let content = json["content"]
            .as_array()
            .and_then(|blocks| {
                blocks.iter()
                    .filter(|b| b["type"].as_str() == Some("text"))
                    .filter_map(|b| b["text"].as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .into()
            })
            .unwrap_or_default();

        let usage = &json["usage"];

        Ok(ChatResponse {
            content,
            tokens_input: usage["input_tokens"].as_u64().unwrap_or(0),
            tokens_output: usage["output_tokens"].as_u64().unwrap_or(0),
            model: request.model,
        })
    }
}
