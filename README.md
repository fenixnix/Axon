# Axon 🧠⚡

<p align="center">
  <img src="assets/logo.webp" alt="Axon Logo" width="400">
</p>

<p align="center">
  <a href="https://github.com/fenixnix/Axon/releases"><img src="https://img.shields.io/github/v/release/fenixnix/Axon?style=flat-square&color=blue" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <br>
  <a href="https://github.com/fenixnix/Axon/stargazers"><img src="https://img.shields.io/github/stars/fenixnix/Axon?style=flat-square&color=yellow" alt="Stars"></a>
  <a href="https://github.com/fenixnix/Axon/network/members"><img src="https://img.shields.io/github/forks/fenixnix/Axon?style=flat-square&color=green" alt="Forks"></a>
  <a href="https://github.com/fenixnix/Axon/issues"><img src="https://img.shields.io/github/issues/fenixnix/Axon?style=flat-square&color=red" alt="Issues"></a>
  <a href="https://crates.io/crates/axon"><img src="https://img.shields.io/crates/v/axon?style=flat-square&color=orange" alt="Crates.io"></a>
</p>

<p align="center">
  <b>English</b> | <a href="README.zh.md">中文</a>
</p>

<p align="center">
  <strong>An AI Agent CLI that thinks, decides, and executes.</strong>
</p>

<p align="center">
  Axon is an autonomous AI agent that understands natural language, plans tasks, and executes tools.<br>
  Every "nerve impulse" (command) is processed with maximum speed.
</p>

---

## 📋 Table of Contents

- [Features](#-features)
- [Demo](#-demo)
- [Quick Start](#-quick-start)
- [Installation](#-installation)
- [Usage](#-usage)
- [Architecture](#-architecture)
- [Skills System](#-skills-system)
- [Documentation](#-documentation)
- [Contributing](#-contributing)
- [License](#-license)

---

## ✨ Features

<table>
<tr>
<td width="50%">

### ⚡ Ultra-Lightweight
- 🪶 **Tiny Binary** — ~5MB single executable, no dependencies
- 🚀 **One-Line Run** — Works out of the box
- 🔋 **Zero State** — No daemon, no background process

### 🧠 AI Integration
- 🧠 **LLM Agnostic** — OpenAI, LM Studio, and more
- 💥 **Magazine Mode** — Single shot, fire and release
- 🔧 **Claude Code Compatible** — Load Claude Code Skills

</td>
<td width="50%">

### 🔧 Developer Experience
- 🧬 **Macro-Based Atoms** — Clean, declarative skill syntax
- 📜 **Async I/O** — Non-blocking execution
- 📋 **CLI-Friendly** — Clean argument parsing and help messages

### 🛠️ Built-in Tools
- 📝 File read/write operations
- 🔍 Code search and grep
- 🐚 Shell command execution
- 🌐 Web search capabilities

</td>
</tr>
</table>

---

## 🎬 Demo

```bash
$ axon exec "Find all Rust files and count lines of code"

🔍 Executing: glob + grep + shell
📁 Found 12 .rs files
📊 Total lines: 3,847
✅ Task completed in 0.23s
```

---

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rust-lang.org) 1.75+ (for building from source)
- API key for your preferred LLM provider

### One-Line Install

```bash
curl -fsSL https://raw.githubusercontent.com/fenixnix/Axon/main/install.sh | bash
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/fenixnix/Axon.git
cd Axon

# Build release binary
cargo build --release

# The binary will be at ./target/release/axon
```

---

## ⚙️ Configuration

Create a `config.yaml` file:

```yaml
core:
  name: "Axon"
  version: "0.1.0"

llm:
  model: "openai/gpt-4o-mini"
  api_key: "${OPENAI_API_KEY}"
  base_url: "https://api.openai.com/v1"

system:
  persona: |
    You are Axon, a high-speed neural conduit written in Rust.
    Execute tools efficiently. Report errors clearly.

atoms:
  active:
    - shell_exec
    - file_read
    - file_write
    - grep
    - glob
```

### LM Studio (Local LLM)

```yaml
llm:
  model: "local-model"
  api_key: ""
  base_url: "http://localhost:1234/v1"
```

---

## 💻 Usage

### Interactive Mode

```bash
# Start interactive session
export OPENAI_API_KEY="sk-..."
axon

# Or with custom config
axon -c config-lmstudio.yaml
```

### Single Command Execution

```bash
# Execute a single command
axon exec "List all files in current directory"

# With streaming output
axon exec "Analyze this codebase" --stream
```

### Atom Management

```bash
# List available atoms
axon atom list

# Show atom details
axon atom info shell_exec
```

### Memory Management

```bash
# Show conversation history
axon memory show

# Clear memory
axon memory clear

# Export memory
axon memory export backup.jsonl
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        User Input                            │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    CLI Handler (Dendrite)                    │
│              Input parsing • Context loading                 │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   Core Logic (Soma)                          │
│              Async runtime • State machine                   │
└─────────────────────────┬───────────────────────────────────┘
                          │
              ┌───────────┴───────────┐
              │                       │
              ▼                       ▼
┌─────────────────────┐   ┌─────────────────────┐
│   LLM Provider      │   │   Memory (JSONL)    │
│   OpenAI/LM Studio  │   │   Persistent Store  │
└──────────┬──────────┘   └─────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│                 Execution Layer (Axon)                       │
│              Task routing • Parallel execution               │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   Skill Atoms                                │
│   shell_exec • file_read • file_write • grep • glob         │
└─────────────────────────────────────────────────────────────┘
```

### Component Mapping

| Component | Biological Analog | Implementation | Purpose |
|-----------|-------------------|----------------|---------|
| **Stimulus** | External Stimulus | CLI with `crossterm` | User input handling |
| **Dendrite** | Dendrites | Input parser | Context loading |
| **Soma** | Cell Body | Async core | State management |
| **Axon** | Axon | Executor | Task routing |
| **Atoms** | Synapses | Trait-based skills | Tool execution |
| **Memory** | Neural Trace | JSONL append | Persistence |
| **Genome** | DNA | `serde` config | Configuration |

---

## 🧩 Skills System

Axon supports **Claude Code compatible Skills** stored in the `skills/` directory:

```
skills/
├── code-review/
│   ├── skill.json          # Skill metadata
│   └── SKILL.md            # Skill instructions
├── git-workflow.md         # Single-file skill
└── web-search/
    ├── skill.json
    └── SKILL.md
```

### Skill Manifest (`skill.json`)

```json
{
  "name": "code-review",
  "description": "Code review assistant",
  "version": "1.0.0",
  "author": "Your Name",
  "allowed-tools": ["read", "write", "bash", "glob", "grep"]
}
```

### Using Skills

```bash
# Automatic activation
axon exec "Review my code for bugs"

# Manual invocation
axon exec "@code-review review src/main.rs"
axon exec "@git-workflow commit my changes"
axon exec "@web-search latest Rust features"
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [docs/design.md](docs/design.md) | Full technical specification (English) |
| [docs/design_zh.md](docs/design_zh.md) | 技术规范 (中文) |
| [docs/spec_modules.md](docs/spec_modules.md) | Module design details |
| [docs/spec_tasks.md](docs/spec_tasks.md) | Task list & roadmap |
| [docs/spec_acceptance.md](docs/spec_acceptance.md) | Acceptance criteria |
| [AGENTS.md](AGENTS.md) | Development guide |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guidelines |

---

## 🛣️ Roadmap

### 🌐 Cross-Platform Versions

- [ ] **C# Version** — Standalone CLI + Game Engine Embeddable Library
  - Standalone: CLI tool similar to the Rust version
  - Embeddable: Library integration for Godot, Unity, etc.
  - Target platforms: Windows, macOS, Linux
  - Core features: Feature parity with Rust version

### 🔧 Technical Evolution

- [ ] **WASM Support** — Run in browsers and edge environments
- [ ] **Plugin System** — Dynamic loading of custom atoms
- [ ] **eBPF Integration** — Kernel-level system monitoring
- [ ] **GUI Interface** — Optional desktop application
- [ ] **Cloud Deployment** — Serverless function support

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

```bash
# Fork and clone
git clone https://github.com/your-username/Axon.git
cd Axon

# Create branch
git checkout -b feature/amazing-feature

# Make changes and commit
cargo fmt
cargo clippy -- -D warnings
cargo test
git commit -m "feat: add amazing feature"

# Push and create PR
git push origin feature/amazing-feature
```

---

## 💬 Community

- 💡 [Discussions](https://github.com/fenixnix/Axon/discussions) — Ask questions, share ideas
- 🐛 [Issues](https://github.com/fenixnix/Axon/issues) — Report bugs, request features
- 📖 [Wiki](https://github.com/fenixnix/Axon/wiki) — Community documentation

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

## 🙏 Acknowledgments

- CLI framework by [Clap](https://github.com/clap-rs/clap)

---

<p align="center">
  <sub>Built with ❤️ by the Axon Team</sub>
</p>

<p align="center">
  <i>"Memory safety meets neural speed."</i>
</p>
