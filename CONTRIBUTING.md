# Contributing to Axon

Thank you for your interest in contributing to Axon! This document provides guidelines and instructions for contributing.

## 🚀 Getting Started

### Prerequisites

- Rust 1.75+ with Edition 2024 support
- Git
- A code editor (VS Code, IntelliJ Rust, etc.)

### Setting Up Development Environment

```bash
# Fork and clone the repository
git clone https://github.com/your-username/axon.git
cd axon

# Build the project
cargo build

# Run tests
cargo test
```

## 📋 Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

- Follow the existing code style
- Write tests for new functionality
- Update documentation as needed

### 3. Run Quality Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Check without building
cargo check
```

### 4. Commit Changes

We follow conventional commits:

```
feat: add new atom for file search
fix: resolve memory leak in executor
docs: update README with new examples
refactor: simplify error handling in llm module
test: add tests for skill loader
```

### 5. Submit Pull Request

- Provide a clear description of changes
- Reference any related issues
- Ensure all CI checks pass

## 🎨 Code Style

### Rust Conventions

- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Use `SCREAMING_SNAKE_CASE` for constants
- Maximum line length: 100 characters
- 4 spaces for indentation

### Error Handling

```rust
// Application code - use anyhow
use anyhow::{Context, Result};

async fn process() -> Result<()> {
    let data = load_data()
        .context("Failed to load data")?;
    Ok(())
}

// Library code - use thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### Async Patterns

```rust
// Prefer async/await
pub async fn execute(&self) -> Result<Value> {
    let result = self.process().await?;
    Ok(result)
}

// Use tokio::spawn for parallelism
let handles: Vec<_> = tasks
    .into_iter()
    .map(|t| tokio::spawn(process(t)))
    .collect();
```

## 🧪 Testing

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_atom_execution() {
        let atom = ShellExec;
        let args = json!({"command": "echo hello"});
        let result = atom.execute(args).await.unwrap();
        assert!(result["stdout"].as_str().unwrap().contains("hello"));
    }
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test integration
```

## 📚 Documentation

- Update relevant docs in `docs/` directory
- Add rustdoc comments for public APIs
- Include examples in documentation

## 🔒 Security

- Never commit API keys or secrets
- Validate all user inputs
- Use parameterized commands (avoid shell injection)
- Report security issues privately

## 💬 Communication

- Open an issue for bug reports or feature requests
- Join discussions in existing issues
- Be respectful and constructive

## 🏷️ Issue Labels

| Label | Description |
|-------|-------------|
| `bug` | Something isn't working |
| `enhancement` | New feature or request |
| `documentation` | Documentation improvements |
| `good first issue` | Good for newcomers |
| `help wanted` | Extra attention needed |

## 🙏 Thank You!

Every contribution, whether it's:
- Reporting a bug
- Suggesting a feature
- Writing code
- Improving documentation
- Sharing the project

...is greatly appreciated!

---

**Happy coding!** 🦀⚡
