//! Ollama provider — Local models qua Ollama API.

use crate::provider::{ChatRequest, ChatResponse, Provider};
use anyhow::{Context, Result};

pub struct OllamaProvider {
    base_url: String,
    http_client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.base_url);

        let body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            }).collect::<Vec<_>>(),
            "stream": false,
            "options": {
                "temperature": request.temperature.unwrap_or(0.7),
                "num_predict": request.max_tokens.unwrap_or(4096),
            }
        });

        let resp = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Ollama API request failed")?;

        let json: serde_json::Value = resp.json().await?;

        let content = json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(ChatResponse {
            content,
            tokens_input: json.get("prompt_eval_count").and_then(|v| v.as_u64()).unwrap_or(0),
            tokens_output: json.get("eval_count").and_then(|v| v.as_u64()).unwrap_or(0),
            model: request.model,
        })
    }
}
