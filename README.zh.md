# Axon 🧠⚡

<p align="center">
  <img src="assets/logo.webp" alt="Axon Logo" width="400">
</p>

<p align="center">
  <a href="https://github.com/fenixnix/Axon/releases"><img src="https://img.shields.io/github/v/release/fenixnix/Axon?style=flat-square&color=blue" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-2024%20edition-orange.svg?style=flat-square&logo=rust" alt="Rust"></a>
  <a href="https://tokio.rs"><img src="https://img.shields.io/badge/async-tokio-green.svg?style=flat-square" alt="Tokio"></a>
  <br>
  <a href="https://github.com/fenixnix/Axon/stargazers"><img src="https://img.shields.io/github/stars/fenixnix/Axon?style=flat-square&color=yellow" alt="Stars"></a>
  <a href="https://github.com/fenixnix/Axon/network/members"><img src="https://img.shields.io/github/forks/fenixnix/Axon?style=flat-square&color=green" alt="Forks"></a>
  <a href="https://github.com/fenixnix/Axon/issues"><img src="https://img.shields.io/github/issues/fenixnix/Axon?style=flat-square&color=red" alt="Issues"></a>
  <a href="https://crates.io/crates/axon"><img src="https://img.shields.io/crates/v/axon?style=flat-square&color=orange" alt="Crates.io"></a>
</p>

<p align="center">
  <a href="README.md">English</a> | <b>中文</b>
</p>

<p align="center">
  <strong>使用 Rust 编写的高性能、内存安全的 CLI 代理。</strong>
</p>

<p align="center">
  Axon 作为人类意图与系统执行之间的生物启发式传输通道。<br>
  借助 Rust 的零成本抽象和严格类型安全，每一次"神经冲动"（命令）都以最快速度传输，且运行时错误风险最小。
</p>

---

## 📋 目录

- [特性](#-特性)
- [演示](#-演示)
- [快速开始](#-快速开始)
- [安装](#-安装)
- [使用](#-使用)
- [架构](#-架构)
- [技能系统](#-技能系统)
- [文档](#-文档)
- [贡献](#-贡献)
- [许可证](#-许可证)

---

## ✨ 特性

<table>
<tr>
<td width="50%">

### 🚀 性能
- ⚡ **零延迟启动** — 编译后的二进制文件即时启动
- 🛡️ **内存安全** — Rust 所有权模型防止溢出
- 📦 **单二进制文件** — 静态部署，无依赖

### 🔧 开发者体验
- 🧬 **宏驱动的原子** — 简洁、声明式的技能语法
- 📜 **异步 I/O** — 基于 `tokio` 的非阻塞执行
- 🔍 **类型安全** — 编译时错误预防

</td>
<td width="50%">

### 🧠 AI 集成
- 🧠 **LLM 无关** — 支持 OpenAI、LM Studio 等
- 🔧 **Claude Code 兼容** — 加载 Claude Code Skill
- 💬 **交互式聊天** — 自然语言命令界面

### 🛠️ 内置工具
- 📝 文件读写操作
- 🔍 代码搜索和 grep
- 🐚 Shell 命令执行
- 🌐 网页搜索能力

</td>
</tr>
</table>

---

## 🎬 演示

```bash
$ axon exec "查找所有 Rust 文件并统计代码行数"

🔍 执行中: glob + grep + shell
📁 找到 12 个 .rs 文件
📊 总行数: 3,847
✅ 任务完成，耗时 0.23s
```

---

## 🚀 快速开始

### 环境要求

- [Rust](https://rust-lang.org) 1.75+ (支持 2024 Edition)
- 你选择的 LLM 提供商的 API 密钥

### 一行命令安装

```bash
curl -fsSL https://raw.githubusercontent.com/fenixnix/Axon/main/install.sh | bash
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/fenixnix/Axon.git
cd Axon

# 构建发布版本
cargo build --release

# 二进制文件位于 ./target/release/axon
```

---

## ⚙️ 配置

创建 `config.yaml` 文件：

```yaml
core:
  name: "Axon"
  version: "2.0.0"

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

### LM Studio (本地 LLM)

```yaml
llm:
  model: "local-model"
  api_key: ""
  base_url: "http://localhost:1234/v1"
```

---

## 💻 使用

### 交互模式

```bash
# 启动交互会话
export OPENAI_API_KEY="sk-..."
axon

# 或使用自定义配置
axon -c config-lmstudio.yaml
```

### 单命令执行

```bash
# 执行单个命令
axon exec "列出当前目录的所有文件"

# 流式输出
axon exec "分析这个代码库" --stream
```

### 原子管理

```bash
# 列出可用原子
axon atom list

# 显示原子详情
axon atom info shell_exec
```

### 记忆管理

```bash
# 显示对话历史
axon memory show

# 清除记忆
axon memory clear

# 导出记忆
axon memory export backup.jsonl
```

---

## 🏗️ 架构

```
┌─────────────────────────────────────────────────────────────┐
│                        用户输入                              │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    CLI 处理器 (树突)                         │
│              输入解析 • 上下文加载                           │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   核心逻辑 (胞体)                            │
│              异步运行时 • 状态机                             │
└─────────────────────────┬───────────────────────────────────┘
                          │
              ┌───────────┴───────────┐
              │                       │
              ▼                       ▼
┌─────────────────────┐   ┌─────────────────────┐
│   LLM 提供商        │   │   记忆 (JSONL)      │
│   OpenAI/LM Studio  │   │   持久化存储        │
└──────────┬──────────┘   └─────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│                 执行层 (轴突)                                │
│              任务路由 • 并行执行                             │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   技能原子                                   │
│   shell_exec • file_read • file_write • grep • glob         │
└─────────────────────────────────────────────────────────────┘
```

### 组件映射

| 组件 | 生物对应 | Rust 实现 | 用途 |
|------|----------|-----------|------|
| **刺激** | 外部刺激 | 基于 `crossterm` 的 CLI | 用户输入处理 |
| **树突** | 树突 | 输入解析器 | 上下文加载 |
| **胞体** | 细胞体 | 异步核心 | 状态管理 |
| **轴突** | 轴突 | 执行器 | 任务路由 |
| **原子** | 突触 | 基于 trait 的技能 | 工具执行 |
| **记忆** | 神经痕迹 | JSONL 追加 | 持久化 |
| **基因组** | DNA | `serde` 配置 | 配置管理 |

---

## 🧩 技能系统

Axon 支持 **Claude Code 兼容的 Skill**，存储在 `skills/` 目录：

```
skills/
├── code-review/
│   ├── skill.json          # Skill 元数据
│   └── SKILL.md            # Skill 说明
├── git-workflow.md         # 单文件 skill
└── web-search/
    ├── skill.json
    └── SKILL.md
```

### Skill 清单 (`skill.json`)

```json
{
  "name": "code-review",
  "description": "代码审查助手",
  "version": "1.0.0",
  "author": "你的名字",
  "allowed-tools": ["read", "write", "bash", "glob", "grep"]
}
```

### 使用 Skill

```bash
# 自动激活
axon exec "审查我的代码是否有 bug"

# 手动调用
axon exec "@code-review 审查 src/main.rs"
axon exec "@git-workflow 提交代码"
axon exec "@web-search 最新的 Rust 特性"
```

---

## 📚 文档

| 文档 | 说明 |
|------|------|
| [docs/design.md](docs/design.md) | 完整技术规范 (英文) |
| [docs/design_zh.md](docs/design_zh.md) | 技术规范 (中文) |
| [docs/spec_modules.md](docs/spec_modules.md) | 模块设计详情 |
| [docs/spec_tasks.md