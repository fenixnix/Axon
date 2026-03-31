//! Executor Module - Axon
//!
//! Coordinates LLM calls and tool execution.

use crate::atoms::AtomRegistry;
use crate::llm::{LlmClient, LlmError};
use crate::memory::{Memory, Message, ToolCall};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("LLM调用失败: {0}")]
    LlmError(#[from] LlmError),

    #[error("工具执行失败: {0}")]
    ToolError(String),

    #[error("记忆错误: {0}")]
    MemoryError(String),

    #[error("执行超时")]
    Timeout,

    #[error("用户中断")]
    Interrupted,
}

/// Execution result
#[derive(Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Executor - coordinates LLM and tool execution
pub struct Executor {
    llm: LlmClient,
    registry: AtomRegistry,
    memory: Arc<Mutex<Memory>>,
}

impl Executor {
    pub fn new(llm: LlmClient, registry: AtomRegistry, memory: Arc<Mutex<Memory>>) -> Self {
        Self {
            llm,
            registry,
            memory,
        }
    }

    /// Access the memory instance
    pub fn memory(&self) -> &Arc<Mutex<Memory>> {
        &self.memory
    }

    /// Execute a single interaction with multi-round tool calling support
    pub async fn execute_once(&self, user_input: &str) -> Result<String, ExecutorError> {
        const MAX_TOOL_ITERATIONS: usize = 10;

        // Add user message
        {
            let mut memory = self.memory.lock().await;
            memory.add_message(Message::user(user_input));

            // Save to file
            memory
                .append(&Message::user(user_input))
                .await
                .map_err(|e| ExecutorError::MemoryError(e.to_string()))?;
        }

        // Initial LLM call
        let tools = self.registry.to_tool_definitions();
        let context = {
            let memory = self.memory.lock().await;
            memory.get_context(20)
        };
        let mut response = self.llm.chat(&context, Some(&tools)).await?;

        // Save assistant response to memory
        {
            let mut memory = self.memory.lock().await;
            memory.add_message(response.clone());
            memory
                .append(&response)
                .await
                .map_err(|e| ExecutorError::MemoryError(e.to_string()))?;
        }

        // Multi-round tool call loop
        let mut iteration = 0;
        while let Some(ref tool_calls) = response.tool_calls {
            if tool_calls.is_empty() {
                break;
            }

            iteration += 1;
            if iteration > MAX_TOOL_ITERATIONS {
                return Err(ExecutorError::ToolError(format!(
                    "Exceeded maximum tool call iterations ({})",
                    MAX_TOOL_ITERATIONS
                )));
            }

            eprintln!("[DEBUG] Tool iteration {}/{}: {} tool calls", iteration, MAX_TOOL_ITERATIONS, tool_calls.len());
            for tc in tool_calls.iter() {
                eprintln!(
                    "[DEBUG] Tool call: {}({})",
                    tc.name(),
                    tc.function.arguments
                );
            }

            // Execute all tools in parallel
            let results = self.execute_tools(tool_calls.clone()).await;

            // Add tool results to memory
            for (i, result) in results.iter().enumerate() {
                let tool_call_id = tool_calls
                    .get(i)
                    .map(|tc| tc.id.clone())
                    .unwrap_or_default();
                let output_str = result.output.to_string();
                eprintln!("[DEBUG] Tool result: {}", &output_str[..output_str.len().min(200)]);
                let msg = Message::tool(tool_call_id, output_str);
                let mut memory = self.memory.lock().await;
                memory.add_message(msg.clone());
                memory
                    .append(&msg)
                    .await
                    .map_err(|e| ExecutorError::MemoryError(e.to_string()))?;
            }

            // Continue conversation with tool results
            let context = {
                let memory = self.memory.lock().await;
                memory.get_context(20)
            };

            response = self.llm.chat(&context, None).await?;

            // Save this assistant response to memory
            {
                let mut memory = self.memory.lock().await;
                memory.add_message(response.clone());
                memory
                    .append(&response)
                    .await
                    .map_err(|e| ExecutorError::MemoryError(e.to_string()))?;
            }
        }

        Ok(response.content.as_str().unwrap_or("").to_string())
    }

    /// Execute a single tool
    pub async fn execute_tool(&self, call: ToolCall) -> Result<ExecutionResult, ExecutorError> {
        let start = std::time::Instant::now();

        let atom = self
            .registry
            .get(call.name())
            .ok_or_else(|| ExecutorError::ToolError(format!("Unknown tool: {}", call.name())))?;

        let args = call
            .arguments()
            .map_err(|e| ExecutorError::ToolError(format!("Invalid arguments: {}", e)))?;

        let output = atom
            .execute(args)
            .await
            .map_err(|e: crate::atoms::AtomError| ExecutorError::ToolError(e.to_string()))?;

        let duration = start.elapsed().as_millis() as u64;

        Ok(ExecutionResult {
            success: true,
            output,
            error: None,
            duration_ms: duration,
        })
    }

    /// Execute multiple tools in parallel
    pub async fn execute_tools(&self, calls: Vec<ToolCall>) -> Vec<ExecutionResult> {
        let tasks: Vec<_> = calls
            .into_iter()
            .map(|call| {
                let registry = &self.registry;
                async move {
                    let start = std::time::Instant::now();

                    let atom = match registry.get(call.name()) {
                        Some(a) => a,
                        None => {
                            return ExecutionResult {
                                success: false,
                                output: serde_json::Value::Null,
                                error: Some(format!("Unknown tool: {}", call.name())),
                                duration_ms: 0,
                            };
                        }
                    };

                    let args = match call.arguments() {
                        Ok(a) => a,
                        Err(e) => {
                            return ExecutionResult {
                                success: false,
                                output: serde_json::Value::Null,
                                error: Some(format!("Invalid arguments: {}", e)),
                                duration_ms: 0,
                            };
                        }
                    };

                    let output: Result<serde_json::Value, crate::atoms::AtomError> =
                        atom.execute(args).await;
                    let duration = start.elapsed().as_millis() as u64;

                    match output {
                        Ok(v) => ExecutionResult {
                            success: true,
                            output: v,
                            error: None,
                            duration_ms: duration,
                        },
                        Err(e) => ExecutionResult {
                            success: false,
                            output: serde_json::Value::Null,
                            error: Some(e.to_string()),
                            duration_ms: duration,
                        },
                    }
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let config = crate::llm::LlmConfig {
            model: "test".to_string(),
            api_key: "test".to_string(),
            base_url: "http://localhost:1234/v1".to_string(),
            timeout_secs: 60,
        };

        let llm = LlmClient::new(config).unwrap();
        let registry = crate::atoms::create_default_registry();
        let memory = Arc::new(Mutex::new(Memory::new(std::path::PathBuf::from(
            "/tmp/test.jsonl",
        ))));

        let _executor = Executor::new(llm, registry, memory);

        // Just verify it was created - can't actually run without LLM
        assert!(true);
    }
}
