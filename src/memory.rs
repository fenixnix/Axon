//! Memory Module - Neural Trace
//!
//! Manages conversation history with JSONL persistence.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("文件操作失败: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON解析失败: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("记忆文件格式错误")]
    InvalidFormat,
}

/// Content part for multi-modal messages
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

/// Image URL for vision models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageUrl {
    pub url: String,
}

/// A message in the conversation
/// For vision models, content can be a string or an array of content parts
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    /// Content can be:
    /// - String: for text-only messages
    /// - Array of ContentPart: for multi-modal messages (vision)
    #[serde(with = "serde_content")]
    pub content: Content,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Content type that can be serialized as either string or array
#[derive(Debug, Clone)]
pub enum Content {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl Content {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Content::Text(s) => Some(s),
            Content::Parts(_) => None,
        }
    }
}

mod serde_content {
    use super::{Content, ContentPart};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(content: &Content, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match content {
            Content::Text(text) => text.serialize(serializer),
            Content::Parts(parts) => parts.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Content, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if let Some(text) = value.as_str() {
            Ok(Content::Text(text.to_string()))
        } else if let Some(_array) = value.as_array() {
            let parts: Vec<ContentPart> =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(Content::Parts(parts))
        } else {
            Err(serde::de::Error::custom("content must be string or array"))
        }
    }
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: Content::Text(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a user message with image for vision models
    pub fn user_with_image(text: impl Into<String>, image_base64: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: Content::Parts(vec![
                ContentPart::Text { text: text.into() },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: format!("data:image/jpeg;base64, {}", image_base64.into()),
                    },
                },
            ]),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Content::Text(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant_with_tools(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Content::Text(String::new()),
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: Content::Text(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a tool result message
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: Content::Text(content.into()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Function call details within a tool call
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON string, e.g., "{\"path\": \"file.txt\"}"
}

/// A tool call returned by LLM (OpenAI format)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

impl ToolCall {
    pub fn new(name: impl Into<String>, arguments: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: name.into(),
                arguments: arguments.to_string(),
            },
        }
    }

    /// Get the function name
    pub fn name(&self) -> &str {
        &self.function.name
    }

    /// Get parsed arguments as JSON Value
    pub fn arguments(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.function.arguments)
    }
}

/// Memory manager - handles conversation history
#[derive(Debug)]
pub struct Memory {
    path: PathBuf,
    messages: Vec<Message>,
}

impl Memory {
    /// Create a new memory manager
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            messages: Vec::new(),
        }
    }

    /// Load history from JSONL file
    pub async fn load(&mut self) -> Result<(), MemoryError> {
        if !self.path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.path).await?;

        self.messages.clear();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let msg: Message = serde_json::from_str(line)?;
            self.messages.push(msg);
        }

        Ok(())
    }

    /// Append a message to history
    pub async fn append(&self, msg: &Message) -> Result<(), MemoryError> {
        let line = serde_json::to_string(msg)?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;

        use tokio::io::AsyncWriteExt;
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;

        Ok(())
    }

    /// Clear all memory
    pub async fn clear(&self) -> Result<(), MemoryError> {
        tokio::fs::write(&self.path, "").await?;
        Ok(())
    }

    /// Export memory to a file
    pub async fn export(&self, path: &Path) -> Result<(), MemoryError> {
        let content = serde_json::to_string_pretty(&self.messages)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Import memory from a file
    pub async fn import(&mut self, path: &Path) -> Result<(), MemoryError> {
        let content = tokio::fs::read_to_string(path).await?;
        self.messages = serde_json::from_str(&content)?;
        Ok(())
    }

    /// Get all messages
    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    /// Get messages for context (with simple truncation)
    pub fn get_context(&self, max_messages: usize) -> Vec<Message> {
        if self.messages.len() <= max_messages {
            return self.messages.clone();
        }

        // Keep system prompt if present, then take last max_messages
        let system_msgs: Vec<_> = self
            .messages
            .iter()
            .filter(|m| m.role == "system")
            .cloned()
            .collect();

        let other_msgs: Vec<_> = self
            .messages
            .iter()
            .filter(|m| m.role != "system")
            .rev()
            .take(max_messages)
            .cloned()
            .collect();

        let mut result = system_msgs;
        result.extend(other_msgs.into_iter().rev());
        result
    }

    /// Add a message to memory (in-memory)
    pub fn add_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    /// Get message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_memory_append() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("memory.jsonl");

        let memory = Memory::new(path);
        let msg = Message::user("Hello");
        memory.append(&msg).await.unwrap();

        let content = tokio::fs::read_to_string(&memory.path).await.unwrap();
        assert!(content.contains("Hello"));
    }

    #[tokio::test]
    async fn test_memory_load() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("memory.jsonl");

        // Write test data
        let data = r#"{"role":"user","content":"Hi"}
{"role":"assistant","content":"Hello"}
"#;
        tokio::fs::write(&path, data).await.unwrap();

        let mut memory = Memory::new(path);
        memory.load().await.unwrap();

        assert_eq!(memory.len(), 2);
        assert_eq!(memory.get_messages()[0].content.as_str(), Some("Hi"));
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::user("test");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content.as_str(), Some("test"));
    }
}
