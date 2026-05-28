//! VietCode LLM — Abstraction layer cho các LLM provider.
//!
//! Hỗ trợ:
//! - Ollama (local models)
//! - Anthropic (Claude)
//! - OpenAI (GPT-4)
//! - DeepSeek

pub mod provider;
pub mod ollama;
pub mod anthropic;
pub mod openai;
pub mod deepseek;
