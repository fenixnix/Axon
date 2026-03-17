# AGENTS.md - Axon Development Guide

> **Project**: Axon - Biological-inspired CLI Agent (Rust)
> **Status**: Greenfield Project (Specification Phase)
> **Last Updated**: 2026-03-17

---

## 1. Project Overview

Axon is a high-performance, memory-safe CLI agent written in **Rust**. It serves as a biological-inspired transmission channel between human intent and system execution.

**Technology Stack**: Rust (Edition 2024+), tokio, reqwest, serde, crossterm, anyhow, thiserror

**Target Structure**:
```
axon/
├── Cargo.toml           # Dependencies & Metadata
├── src/
│   ├── main.rs          # Entry Point
│   ├── config.rs        # Genome: Config Structs
│   ├── memory.rs        # Trace: JSONL Handling
│   ├── atoms.rs         # Effectors: Trait & Macros
│   ├── llm.rs           # LLM Client Interface
│   └── executor.rs      # Axon: Tool Routing
├── config.yaml          # Configuration File
└── memory.jsonl         # Persistent Storage
```

---

## 2. Build & Test Commands

```bash
# Development build
cargo build

# Release build (optimized, ~5-10MB binary)
cargo build --release

# Check without building
cargo check

# Format code
cargo fmt
cargo fmt -- --check  # Check only

# Run all tests
cargo test

# Run single test by name
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run clippy linter
cargo clippy

# Lint with warnings as errors
cargo clippy -- -D warnings

# Generate documentation
cargo doc
cargo doc --open
```

### Running the Application

```bash
export OPENAI_API_KEY="sk-..."

# Interactive mode
cargo run -- --verbose

# Single command
cargo run -- exec "list files"

# With custom config
cargo run -- -c /path/to/config.yaml run
```

---

## 3. Code Style Guidelines

### 3.1 General Principles

- **Safety First**: Leverage Rust's ownership model to prevent buffer overflows and leaks
- **Memory Safety**: Never use `unsafe` unless absolutely necessary
- **Type Safety**: Prefer strong types over raw strings/integers
- **Async-First**: Use async/await for I/O operations; prefer tokio

### 3.2 Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Modules | snake_case | `memory.rs`, `executor.rs` |
| Structs | PascalCase | `AtomRegistry` |
| Enums | PascalCase | `ToolCall` |
| Functions | snake_case | `execute_tool_call()` |
| Variables | snake_case | `let config = ...` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_RETRIES: u32 = 3` |
| Traits | PascalCase | `trait Atom` |

### 3.3 Imports Order

```rust
// Standard library first
use std::collections::HashMap;
use std::path::PathBuf;

// External crates (alphabetically)
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;

// Local modules - absolute paths
use crate::config::Config;
use crate::atoms::{Atom, AtomRegistry};
```

### 3.4 Formatting Rules

- **Line Length**: Maximum 100 characters
- **Indentation**: 4 spaces (no tabs)
- **Trailing Commas**: Always use in multi-line constructs

### 3.5 Type Annotations & Error Handling

- Explicit return types for public functions
- Prefer type inference for local variables
- Use `Result<T, E>` for fallible operations
- Use `Option<T>` for nullable values

**Error Handling**:
- Use `anyhow` for application-level errors (context-rich)
- Use `thiserror` for library-specific error types
- Never use `unwrap()` in production code
- Use `?` operator for error propagation

```rust
// Application code
use anyhow::{Context, Result};

async fn process_request(req: Request) -> Result<Response> {
    let config = load_config()
        .context("Failed to load configuration")?;
    Ok(Response::new(config))
}

// Library code
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AtomError {
    #[error("Unknown atom: {0}")]
    UnknownAtom(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(#[from] std::io::Error),
}
```

### 3.6 Async/Await Patterns

- Use tokio for async runtime
- Prefer `async fn` over manual futures
- Use `tokio::spawn` for parallel execution
- Implement timeouts for external calls

```rust
pub async fn execute_all(&self, calls: Vec<ToolCall>) -> Vec<Result<Value>> {
    futures::future::join_all(
        calls.into_iter().map(|call| self.execute_tool_call(call))
    ).await
}
```

---

## 4. Security Guidelines

### 4.1 Command Execution

- **NEVER** pass unsanitized user input to shell commands
- Use `Command::new()` with explicit `.arg()` calls instead of shell strings
- Validate and sanitize all file paths

```rust
// UNSAFE - DO NOT USE
// Command::new("sh").arg("-c").arg(user_input)

// SAFE - Use explicit arguments
Command::new("ls")
    .arg("-la")
    .arg(target_dir)
    .output()
    .await?
```

### 4.2 Secret Management

- Never hardcode API keys or secrets
- Use environment variables: `std::env::var("OPENAI_API_KEY")`
- Never log sensitive information

---

## 5. Development Workflow

1. **Feature Implementation**: Create branch → Implement → Test → PR
2. **Testing**: Run `cargo test` before every commit
3. **Linting**: Run `cargo clippy -- -D warnings` before PR
4. **Documentation**: Update docs for public APIs

### Pre-commit Checklist

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

---

## 6. Claude Code Skill Integration

Axon supports loading Claude Code format Skills from the `skills/` directory:

```
skills/
├── skill.json           # Skill metadata
├── SKILL.md            # Skill definition
└── (optional resources)
```

**Skill Manifest Format**:
```json
{
  "name": "code-review",
  "description": "Code review assistant",
  "version": "1.0.0",
  "allowed-tools": ["read", "write", "bash", "glob", "grep"]
}
```

---

## 7. Important Notes

- **Greenfield project** - implementation starts from scratch
- Follow specification in `docs/design.md` for architecture
- All new code must pass `cargo clippy` and `cargo fmt`
- Use `anyhow` for application errors, `thiserror` for library errors
- Prioritize memory safety and type safety

---

## 8. LM Studio 本地模型配置

### 8.1 配置示例

Axon 支持使用 LM Studio 运行本地 LLM。首先在 LM Studio 中启动一个模型，然后配置 Axon：

```yaml
# config-lmstudio.yaml
core:
  name: "Axon"
  version: "2.0.0"

llm:
  model: "local-model"  # LM Studio 中显示的模型名称
  api_key: ""           # 本地运行不需要 API Key
  base_url: "http://localhost:1234/v1"
  timeout_secs: 120

system:
  persona: |
    You are Axon, a high-speed neural conduit.
    Execute tools efficiently. Report errors clearly.

atoms:
  active:
    - shell_exec
    - file_read
```

### 8.2 启动 LM Studio

1. 打开 LM Studio
2. 选择并加载一个模型 (如 llama, qwen)
3. 点击 "Start Server" 启动 API 服务
4. 默认地址: `http://localhost:1234`

### 8.3 运行 Axon

```bash
# 使用 LM Studio 配置
cargo run -- -c config-lmstudio.yaml exec "你好"

# 或设置环境变量
export OPENAI_API_KEY="sk-dummy"  # 某些客户端可能需要
cargo run -- exec "你好"
```

---

## 9. 中文文档 (Chinese Docs)

| 文件 | 说明 |
|------|------|
| `docs/design.md` | 完整技术规范 (英文) |
| `docs/design_zh.md` | 技术规范 (中文) |
| `docs/spec_modules.md` | 模块详细设计 |
| `docs/spec_tasks.md` | 任务清单 |
| `docs/spec_acceptance.md` | 验收清单 |
| `AGENTS.md` | 开发指南 (本文件) |

---

## 10. File Locations

| File | Purpose |
|------|---------|
| `docs/design.md` | Full technical specification |
| `docs/design_zh.md` | Chinese specification |
| `docs/spec_modules.md` | Module design |
| `docs/spec_tasks.md` | Task list |
| `docs/spec_acceptance.md` | Acceptance criteria |
| `AGENTS.md` | This file - agent guidelines |

> *"Memory safety meets neural speed."*
