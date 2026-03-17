//! LLM Module - Neural Transmitter
//! 
//! Handles LLM API communication.

use crate::memory::Message;
use crate::atoms::ToolDefinition;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("API请求失败: {0}")]
    RequestFailed(String),
    
    #[error("API响应解析失败: {0}")]
    ParseError(String),
    
    #[error("API密钥未配置")]
    MissingApiKey,
    
    #[error("模型不支持: {0}")]
    UnsupportedModel(String),
    
    #[error("速率限制")]
    RateLimited,
}

/// LLM Client configuration
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    pub timeout_secs: u64,
}

impl LlmConfig {
    /// Check if API key is required (false for LM Studio)
    pub fn requires_api_key(&self) -> bool {
        !self.api_key.is_empty() && self.api_key != "sk-dummy"
    }
}

/// Chat request payload
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinition>>,
    stream: bool,
}

/// Chat response choice
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// LLM Client
pub struct LlmClient {
    client: reqwest::Client,
    config: LlmConfig,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;
        
        Ok(Self { client, config })
    }
    
    /// Send a chat request (non-streaming)
    pub async fn chat(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>,
    ) -> Result<Message, LlmError> {
        let url = format!("{}/chat/completions", self.config.base_url);
        
        let mut request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            tools: None,
            stream: false,
        };
        
        if let Some(t) = tools {
            request.tools = Some(t.to_vec());
        }
        
        // Debug: print request body
        #[cfg(debug_assertions)]
        {
            if let Ok(json) = serde_json::to_string_pretty(&request) {
                eprintln!("[DEBUG] LLM Request:\n{}", json);
            }
        }
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;
        
        if response.status() == 429 {
            return Err(LlmError::RateLimited);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::RequestFailed(format!("{}: {}", status, body)));
        }
        
        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;
        
        chat_response.choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .ok_or_else(|| LlmError::ParseError("No choices in response".to_string()))
    }
    
    /// Test connection to LLM API
    pub async fn test_connection(&self) -> Result<(), LlmError> {
        let test_message = Message::user("ping");
        self.chat(&[test_message], None).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_llm_config() {
        let config = LlmConfig {
            model: "gpt-4".to_string(),
            api_key: "test".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            timeout_secs: 60,
        };
        
        assert_eq!(config.model, "gpt-4");
    }
    
    #[test]
    fn test_requires_api_key() {
        let config_with_key = LlmConfig {
            model: "gpt-4".to_string(),
            api_key: "sk-123".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            timeout_secs: 60,
        };
        assert!(config_with_key.requires_api_key());
        
        let config_no_key = LlmConfig {
            model: "local-model".to_string(),
            api_key: "".to_string(),
            base_url: "http://localhost:1234/v1".to_string(),
            timeout_secs: 60,
        };
        assert!(!config_no_key.requires_api_key());
    }
}
