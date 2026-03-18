//! Atoms Module - Effectors
//!
//! Defines tool capabilities as traits and provides built-in implementations.

use async_trait::async_trait;
use serde_json::Value;
use std::boxed::Box;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AtomError {
    #[error("未知原子: {0}")]
    UnknownAtom(String),

    #[error("参数错误: {0}")]
    InvalidArgs(String),

    #[error("执行失败: {0}")]
    ExecutionFailed(String),

    #[error("原子未启用")]
    NotEnabled,

    #[error("LLM错误: {0}")]
    LlmError(String),
}

impl From<crate::llm::LlmError> for AtomError {
    fn from(err: crate::llm::LlmError) -> Self {
        AtomError::LlmError(err.to_string())
    }
}

/// The fundamental interface for all atoms (tools)
#[async_trait]
pub trait Atom: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;

    /// Execute the atom with JSON arguments
    async fn execute(&self, args: Value) -> Result<Value, AtomError>;
}

/// Atom registry - manages available atoms
pub struct AtomRegistry {
    atoms: HashMap<String, Box<dyn Atom>>,
}

impl AtomRegistry {
    pub fn new() -> Self {
        Self {
            atoms: HashMap::new(),
        }
    }

    pub fn register(&mut self, atom: Box<dyn Atom>) {
        self.atoms.insert(atom.name().to_string(), atom);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Atom> {
        self.atoms.get(name).map(|b| b.as_ref())
    }

    pub fn list(&self) -> Vec<(&str, &str)> {
        self.atoms
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_ref().description()))
            .collect()
    }

    /// Get all atoms as tool definitions for LLM
    pub fn to_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.atoms
            .iter()
            .map(|(name, atom)| {
                // Build parameters schema based on atom name
                let parameters = match name.as_str() {
                    "shell_exec" => serde_json::json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The shell command to execute"
                            }
                        },
                        "required": ["command"]
                    }),
                    "file_read" => serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The path to the file to read"
                            }
                        },
                        "required": ["path"]
                    }),
                    "file_write" => serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The path to the file to write"
                            },
                            "content": {
                                "type": "string",
                                "description": "The content to write to the file"
                            }
                        },
                        "required": ["path", "content"]
                    }),
                    "image_ocr" => serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The path to the image file (JPEG, PNG, WebP supported)"
                            },
                            "prompt": {
                                "type": "string",
                                "description": "Optional prompt for OCR (default: '请提取这张图片中的文字')"
                            }
                        },
                        "required": ["path"]
                    }),
                    _ => serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                };

                ToolDefinition::new(
                    name.clone(),
                    atom.as_ref().description().to_string(),
                    parameters,
                )
            })
            .collect()
    }
}

impl Default for AtomRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Function definition for OpenAI-compatible tool format
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// Tool definition for LLM (OpenAI format with type="function")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

impl ToolDefinition {
    /// Create a new tool definition with type="function"
    pub fn new(name: String, description: String, parameters: Value) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name,
                description,
                parameters,
            },
        }
    }
}

// =============================================================================
// Built-in Atoms
// =============================================================================

/// Shell Execution Atom
pub struct ShellExec;

#[async_trait]
impl Atom for ShellExec {
    fn name(&self) -> &'static str {
        "shell_exec"
    }

    fn description(&self) -> &'static str {
        "Executes a shell command and returns the output."
    }

    async fn execute(&self, args: Value) -> Result<Value, AtomError> {
        let command = args["command"]
            .as_str()
            .ok_or_else(|| AtomError::InvalidArgs("Missing 'command' parameter".to_string()))?;

        if command.trim().is_empty() {
            return Ok(serde_json::json!({
                "error": "Empty command"
            }));
        }

        // Use shell to execute command for better cross-platform support
        #[cfg(windows)]
        let output = tokio::process::Command::new("cmd")
            .args(["/c", command])
            .output()
            .await
            .map_err(|e| AtomError::ExecutionFailed(e.to_string()))?;

        #[cfg(not(windows))]
        let output = tokio::process::Command::new("sh")
            .args(&["-c", command])
            .output()
            .await
            .map_err(|e| AtomError::ExecutionFailed(e.to_string()))?;

        Ok(serde_json::json!({
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "code": output.status.code()
        }))
    }
}

/// File Read Atom
pub struct FileRead;

#[async_trait]
impl Atom for FileRead {
    fn name(&self) -> &'static str {
        "file_read"
    }

    fn description(&self) -> &'static str {
        "Reads the content of a file."
    }

    async fn execute(&self, args: Value) -> Result<Value, AtomError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| AtomError::InvalidArgs("Missing 'path' parameter".to_string()))?;

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| AtomError::ExecutionFailed(format!("Failed to read file: {}", e)))?;

        Ok(serde_json::json!({
            "content": content,
            "path": path
        }))
    }
}

/// File Write Atom
pub struct FileWrite;

#[async_trait]
impl Atom for FileWrite {
    fn name(&self) -> &'static str {
        "file_write"
    }

    fn description(&self) -> &'static str {
        "Creates or overwrites a file with content."
    }

    async fn execute(&self, args: Value) -> Result<Value, AtomError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| AtomError::InvalidArgs("Missing 'path' parameter".to_string()))?;

        let content = args["content"]
            .as_str()
            .ok_or_else(|| AtomError::InvalidArgs("Missing 'content' parameter".to_string()))?;

        tokio::fs::write(path, content)
            .await
            .map_err(|e| AtomError::ExecutionFailed(format!("Failed to write file: {}", e)))?;

        Ok(serde_json::json!({
            "success": true,
            "path": path,
            "bytes_written": content.len()
        }))
    }
}

/// Image OCR Atom - Extract text from images using vision LLM
pub struct ImageOCR;

impl ImageOCR {
    /// Convert image to base64 JPEG format
    fn image_to_base64(&self, path: &str) -> Result<String, AtomError> {
        use image::ImageFormat;

        // Read and decode image
        let img = image::open(path)
            .map_err(|e| AtomError::ExecutionFailed(format!("Failed to open image: {}", e)))?;

        // Convert to RGB8 and encode as JPEG
        let rgb_img = img.to_rgb8();
        let mut buffer = std::io::Cursor::new(Vec::new());
        rgb_img
            .write_to(&mut buffer, ImageFormat::Jpeg)
            .map_err(|e| AtomError::ExecutionFailed(format!("Failed to encode image: {}", e)))?;

        // Base64 encode
        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            buffer.into_inner(),
        ))
    }
}

#[async_trait]
impl Atom for ImageOCR {
    fn name(&self) -> &'static str {
        "image_ocr"
    }

    fn description(&self) -> &'static str {
        "Extract text from an image file using vision AI. Supports JPEG, PNG, WebP formats."
    }

    async fn execute(&self, args: Value) -> Result<Value, AtomError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| AtomError::InvalidArgs("Missing 'path' parameter".to_string()))?;

        let prompt = args["prompt"].as_str().unwrap_or("请提取这张图片中的文字");

        // Convert image to base64
        let base64_image = self.image_to_base64(path)?;

        // Create LLM client from environment/config
        // Note: In a full implementation, this should be injected from the executor
        let config = crate::llm::LlmConfig {
            model: std::env::var("VISION_MODEL")
                .unwrap_or_else(|_| "qwen/qwen3-vl-30b".to_string()),
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:1234/v1".to_string()),
            timeout_secs: 120,
        };

        let client = crate::llm::LlmClient::new(config)?;

        // Call vision model for OCR
        match client.ocr_image(&base64_image, prompt).await {
            Ok(text) => Ok(serde_json::json!({
                "path": path,
                "text": text,
                "success": true
            })),
            Err(e) => Ok(serde_json::json!({
                "path": path,
                "error": format!("{}", e),
                "success": false
            })),
        }
    }
}

/// Create a default registry with built-in atoms
pub fn create_default_registry() -> AtomRegistry {
    let mut registry = AtomRegistry::new();
    registry.register(Box::new(ShellExec));
    registry.register(Box::new(FileRead));
    registry.register(Box::new(FileWrite));
    registry.register(Box::new(ImageOCR));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let mut registry = AtomRegistry::new();
        registry.register(Box::new(ShellExec));

        let atom = registry.get("shell_exec").unwrap();
        assert_eq!(atom.name(), "shell_exec");
    }

    #[test]
    fn test_list_atoms() {
        let registry = create_default_registry();
        let atoms = registry.list();

        assert!(atoms.iter().any(|(n, _)| *n == "shell_exec"));
        assert!(atoms.iter().any(|(n, _)| *n == "file_read"));
        assert!(atoms.iter().any(|(n, _)| *n == "file_write"));
    }

    #[tokio::test]
    async fn test_shell_exec() {
        let atom = ShellExec;
        let result = atom
            .execute(serde_json::json!({
                "command": "echo hello"
            }))
            .await
            .unwrap();

        assert!(result["stdout"].as_str().unwrap().contains("hello"));
    }
}
