# Axon 模块详细设计

> **版本**: 2.0.0  
> **状态**: 积极开发中  
> **最后更新**: 2026-03-17

---

## 1. 模块总览

Axon 项目采用生物启发式架构，包含以下核心模块：

| 模块 | 生物对应 | 职责 | 复杂度 |
|------|---------|------|--------|
| `config` | 基因组 | 配置解析与管理 | 低 |
| `memory` | 神经痕迹 | 记忆持久化 (JSONL) | 中 |
| `atoms` | 突触/效应器 | 工具能力抽象 | 中 |
| `llm` | 神经递质 | LLM API 调用 | 中 |
| `executor` | 轴突 | 任务路由与执行 | 高 |
| `cli` | 树突 | 命令行交互 | 低 |

---

## 2. 配置模块 (`config`)

### 2.1 职责

- 加载并解析 `config.yaml`
- 管理运行时配置
- 提供配置验证

### 2.2 核心结构

```rust
// 配置结构体
pub struct Config {
    pub core: CoreConfig,
    pub llm: LlmConfig,
    pub system: SystemConfig,
    pub atoms: AtomsConfig,
}

pub struct CoreConfig {
    pub name: String,
    pub version: String,
}

pub struct LlmConfig {
    pub model: String,
    pub api_key: String,          // 或从环境变量加载
    pub base_url: String,
    pub timeout_secs: u64,        // 超时配置
}

pub struct SystemConfig {
    pub persona: String,
}

pub struct AtomsConfig {
    pub active: Vec<String>,      // 启用的原子列表
}
```

### 2.3 公开接口

```rust
impl Config {
    pub fn load(path: &Path) -> Result<Self>;    // 加载配置文件
    pub fn validate(&self) -> Result<()>;        // 验证配置合法性
    pub fn get_atom_enabled(&self, name: &str) -> bool; // 检查原子是否启用
}
```

### 2.4 错误类型

```rust
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置文件不存在: {0}")]
    FileNotFound(String),
    
    #[error("配置解析失败: {0}")]
    ParseError(String),
    
    #[error("配置验证失败: {0}")]
    ValidationError(String),
    
    #[error("缺少必需配置: {0}")]
    MissingField(String),
}
```

---

## 3. 记忆模块 (`memory`)

### 3.1 职责

- 管理会话历史 (JSONL 格式)
- 提供上下文加载
- 支持记忆导出/导入

### 3.2 核心结构

```rust
// 消息结构 (用于 LLM 上下文)
#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: String,        // "system", "user", "assistant"
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

// LLM 返回的工具调用
#[derive(Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,    // JSON 对象
}

// 记忆管理器
pub struct Memory {
    path: PathBuf,
    messages: Vec<Message>,
}
```

### 3.3 公开接口

```rust
impl Memory {
    pub fn new(path: PathBuf) -> Self;
    
    pub async fn load(&mut self) -> Result<()>;       // 加载历史消息
    pub async fn append(&self, msg: &Message) -> Result<()>;
    pub async fn clear(&self) -> Result<()>;           // 清除记忆
    pub async fn export(&self, path: &Path) -> Result<()>;
    pub async fn import(&mut self, path: &Path) -> Result<()>;
    
    pub fn get_messages(&self) -> &[Message];          // 获取消息列表
    pub fn get_context(&self, max_tokens: usize) -> Vec<Message>; // 截断后的上下文
}
```

### 3.4 实现要点

- **JSONL 格式**: 每行一个 JSON 对象，便于追加
- **上下文截断**: 超过 token 限制时，从最早消息开始丢弃
- **异步写入**: 使用 `tokio::fs` 异步追加

### 3.5 错误类型

```rust
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("文件操作失败: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON 解析失败: {0}")]
    ParseError(#[from] serde_json::Error),
    
    #[error("记忆文件格式错误")]
    InvalidFormat,
}
```

---

## 4. 原子模块 (`atoms`)

### 4.1 职责

- 定义工具能力的抽象接口
- 管理和注册原子
- 内置原子实现

### 4.2 核心结构

```rust
/// 原子 trait - 所有工具能力的基础接口
#[async_trait]
pub trait Atom: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    
    async fn execute(&self, args: Value) -> Result<Value>;
}

// 原子注册表
pub struct AtomRegistry {
    atoms: HashMap<String, Box<dyn Atom>>,
}

impl AtomRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, atom: Box<dyn Atom>);
    pub fn get(&self, name: &str) -> Option<&Box<dyn Atom>>;
    pub fn list(&self) -> Vec<(&str, &str)>;  // (name, description)
}
```

### 4.3 宏定义

```rust
#[macro_export]
macro_rules! define_atom {
    ($name:ident, $desc:expr, $func:expr) => {
        // 宏实现见设计文档
    };
}
```

### 4.4 内置原子

| 原子名称 | 功能 | 参数 |
|---------|------|------|
| `shell_exec` | 执行 Shell 命令 | `{ "command": "ls -la" }` |
| `file_read` | 读取文件内容 | `{ "path": "/path/to/file" }` |
| `file_write` | 写入文件 | `{ "path": "...", "content": "..." }` |

### 4.5 错误类型

```rust
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
}
```

---

## 5. LLM 模块 (`llm`)

### 5.1 职责

- 封装 LLM API 调用
- 处理请求/响应序列化
- 管理对话上下文

### 5.2 核心结构

```rust
pub struct LlmClient {
    client: reqwest::Client,
    config: LlmConfig,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    tools: Option<Value>,      // 工具定义
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    // ...
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}
```

### 5.3 公开接口

```rust
impl LlmClient {
    pub fn new(config: LlmConfig) -> Result<Self>;
    
    pub async fn chat(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>,
    ) -> Result<ChatResponse>;
    
    pub async fn chat_streaming(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>,
    ) -> impl Stream<Item = Result<String>>;  // 流式响应
}
```

### 5.4 工具定义格式

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,  // JSON Schema
}
```

### 5.5 错误类型

```rust
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("API 请求失败: {0}")]
    RequestFailed(String),
    
    #[error("API 响应解析失败: {0}")]
    ParseError(String),
    
    #[error("API 密钥未配置")]
    MissingApiKey,
    
    #[error("模型不支持: {0}")]
    UnsupportedModel(String),
    
    #[error("速率限制")]
    RateLimited,
}
```

---

## 6. 执行器模块 (`executor`)

### 6.1 职责

- 协调 LLM 调用与工具执行
- 实现主循环逻辑
- 聚合结果返回给 LLM

### 6.2 核心结构

```rust
pub struct Executor {
    llm: LlmClient,
    registry: AtomRegistry,
    memory: Arc<Mutex<Memory>>,
}

pub struct ExecutionResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}
```

### 6.3 公开接口

```rust
impl Executor {
    pub fn new(
        llm: LlmClient,
        registry: AtomRegistry,
        memory: Arc<Mutex<Memory>>,
    ) -> Self;
    
    /// 单次执行流程
    pub async fn execute_once(&self, user_input: String) -> Result<String>;
    
    /// 交互式主循环
    pub async fn run_interactive(&self, system_prompt: Option<String>) -> Result<()>;
    
    /// 执行工具调用
    pub async fn execute_tool(&self, call: &ToolCall) -> Result<ExecutionResult>;
    
    /// 并行执行多个工具调用
    pub async fn execute_tools(&self, calls: Vec<ToolCall>) -> Vec<ExecutionResult>;
}
```

### 6.4 执行流程

```
用户输入 → Memory 加载上下文 → LLM.chat() → 
    ├── 内容响应 → 输出到终端
    └── 工具调用 → Executor.execute_tools() → 
        → 原子执行 → 结果聚合 → LLM.chat() (继续对话)
```

### 6.5 错误类型

```rust
#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("LLM 调用失败: {0}")]
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
```

---

## 7. CLI 模块 (`cli`)

### 7.1 职责

- 解析命令行参数
- 提供交互界面
- 子命令实现

### 7.2 命令结构

使用 `clap` 或 `crossterm` 实现：

```rust
// CLI 参数定义
pub struct Cli {
    pub config: PathBuf,      // -c, --config
    pub memory: PathBuf,      // -m, --memory
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub no_memory: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub command: Command,
}

pub enum Command {
    Run {
        system: Option<String>,
    },
    Exec {
        command: String,
        stream: bool,
    },
    Chat {
        continue_: bool,
        clear: bool,
    },
    Atom {
        subcommand: AtomSubcommand,
    },
    Memory {
        subcommand: MemorySubcommand,
    },
}
```

---

## 8. 模块依赖关系

```
┌─────────────────────────────────────────────────────┐
│                      cli                             │
│         (命令行参数解析，交互界面)                     │
└─────────────────────┬───────────────────────────────┘
                      │
          ┌───────────┴───────────┐
          │                       │
          ▼                       ▼
    ┌──────────┐           ┌──────────┐
    │  config  │           │  memory  │
    └──────────┘           └──────────┘
          │                       │
          └───────────┬───────────┘
                      │
          ┌───────────┴───────────┐
          │                       │
          ▼                       ▼
    ┌──────────┐           ┌──────────┐
    │    llm   │◄─────────►│ executor │
    └──────────┘           └─────┬────┘
                                 │
                                 ▼
                           ┌──────────┐
                           │  atoms   │
                           └──────────┘
```

---

## 9. 关键设计决策

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 异步运行时 | tokio | 与 reqwest 兼容，社区成熟 |
| 配置格式 | YAML | 易于人类读写 |
| 记忆格式 | JSONL | 追加高效，便于调试 |
| 错误处理 | anyhow + thiserror | 前者用于应用，后者用于库 |
| 原子注册 | 运行时注册 | 支持动态加载插件 |

---

## 10. 下一步

- [ ] 确认模块设计
- [ ] 开始任务分解
- [ ] 制定验收标准
