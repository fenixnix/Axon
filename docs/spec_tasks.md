# Axon 任务清单

> **版本**: 2.0.0  
> **更新时间**: 2026-03-17  
> **实现顺序**: 核心优先

---

## 任务总览

| 阶段 | 名称 | 包含模块 | 优先级 |
|------|------|----------|--------|
| Phase 1 | 基础设施 | config, memory, atoms(基础) | P0 |
| Phase 2 | LLM 集成 | llm | P0 |
| Phase 3 | 执行器 | executor | P0 |
| Phase 4 | CLI | cli | P1 |
| Phase 5 | 完善 | 高级 atoms, 技能加载 | P2 |

---

## Phase 1: 基础设施 (P0)

### 1.1 项目初始化

- [ ] **T001**: 创建 `Cargo.toml`，配置项目元数据和依赖
- [ ] **T002**: 创建基础目录结构 `src/{config,memory,atoms,llm,executor,cli}.rs`
- [ ] **T003**: 配置 `rustfmt.toml` 和 `.gitignore`

### 1.2 配置模块 (config)

- [ ] **T011**: 定义 `Config`, `CoreConfig`, `LlmConfig`, `SystemConfig`, `AtomsConfig` 结构体
- [ ] **T012**: 实现 `Config::load(path)` - 加载 YAML 配置文件
- [ ] **T013**: 实现 `Config::validate()` - 验证配置合法性
- [ ] **T014**: 实现 `Config::get_atom_enabled(name)` - 检查原子是否启用
- [ ] **T015**: 实现 `ConfigError` 错误类型
- [ ] **T016**: 创建示例配置文件 `config.yaml`
- [ ] **T017**: 创建 LM Studio 配置文件 `config-lmstudio.yaml`

### 1.3 记忆模块 (memory)

- [ ] **T021**: 定义 `Message`, `ToolCall` 结构体
- [ ] **T022**: 实现 `Memory::new(path)` - 初始化记忆管理器
- [ ] **T023**: 实现 `Memory::load()` - 异步加载 JSONL 历史
- [ ] **T024**: 实现 `Memory::append(msg)` - 追加单条消息
- [ ] **T025**: 实现 `Memory::clear()` - 清除所有记忆
- [ ] **T026**: 实现 `Memory::get_context(max_tokens)` - 截断获取上下文
- [ ] **T027**: 实现 `Memory::export()` / `Memory::import()` - 导出导入功能
- [ ] **T028**: 实现 `MemoryError` 错误类型

### 1.4 基础原子 (atoms)

- [ ] **T031**: 定义 `Atom` trait 接口
- [ ] **T032**: 实现 `AtomRegistry` 注册表
- [ ] **T033**: 实现 `define_atom!` 宏
- [ ] **T034**: 实现内置原子 `ShellExec` - 执行 Shell 命令
- [ ] **T035**: 实现内置原子 `FileRead` - 读取文件
- [ ] **T036**: 实现内置原子 `FileWrite` - 写入文件
- [ ] **T037**: 实现 `AtomError` 错误类型
- [ ] **T038**: 编写 atoms 模块单元测试 (至少覆盖基础功能)

**Phase 1 里程碑**: 
- ✅ 可通过配置文件加载设置
- ✅ 记忆可持久化到 JSONL
- ✅ 可注册和调用基础工具

---

## Phase 2: LLM 集成 (P0)

### 2.1 LLM 客户端

- [ ] **T041**: 定义 `LlmClient` 结构体
- [ ] **T042**: 实现 `LlmClient::new()` - 初始化 HTTP 客户端
- [ ] **T043**: 实现 `LlmClient::chat()` - 发送聊天请求
- [ ] **T044**: 实现 `LlmClient::chat_streaming()` - 流式响应
- [ ] **T045**: 定义工具调用格式 (`ToolDefinition`)
- [ ] **T046**: 实现 `LlmError` 错误类型

### 2.2 LM Studio 支持 (新增)

- [ ] **T047**: 配置示例 - LM Studio 默认配置
- [ ] **T048**: 实现 `LlmConfig` 支持自定义 base_url
- [ ] **T049**: 实现 API 密钥可选 (LM Studio 不需要)
- [ ] **T050**: 添加连接测试命令

### 2.3 工具描述生成

- [ ] **T051**: 实现从 `AtomRegistry` 生成 OpenAI 格式的工具描述
- [ ] **T052**: 实现 JSON Schema 参数生成

**Phase 2 里程碑**:
- ✅ 可调用 LLM API
- ✅ 可获取 LLM 响应
- ✅ 可传递工具列表给 LLM
- ✅ 支持 LM Studio 本地模型

---

## Phase 3: 执行器 (P0)

### 3.1 执行器核心

- [ ] **T061**: 定义 `Executor` 结构体
- [ ] **T062**: 实现 `Executor::new()` - 初始化执行器
- [ ] **T063**: 实现 `Executor::execute_tool()` - 单个工具调用执行
- [ ] **T064**: 实现 `Executor::execute_tools()` - 并行执行多个工具
- [ ] **T065**: 实现超时机制 (`tokio::time::timeout`)
- [ ] **T066**: 实现错误处理和日志记录

### 3.2 主循环

- [ ] **T071**: 实现 `Executor::execute_once()` - 单次交互流程
- [ ] **T072**: 实现 `Executor::run_interactive()` - 交互式主循环
- [ ] **T073**: 实现 Ctrl+C 中断处理
- [ ] **T074**: 实现结果聚合返回 LLM

### 3.3 上下文管理

- [ ] **T081**: 实现记忆自动加载和保存
- [ ] **T082**: 实现上下文截断 (token 数量控制)
- [ ] **T083**: 实现对话历史管理

**Phase 3 里程碑**:
- ✅ 可接收用户输入
- ✅ 可调用 LLM 并处理响应
- ✅ 可执行工具并返回结果
- ✅ 完整的对话流程可运行

---

## Phase 4: CLI (P1)

### 4.1 命令行参数

- [ ] **T091**: 使用 clap 定义全局参数
- [ ] **T092**: 实现 `--config`, `--memory`, `--model`, `--api-key` 等选项
- [ ] **T093**: 实现 `--verbose`, `--quiet` 输出控制

### 4.2 子命令实现

- [ ] **T101**: 实现 `run` 子命令 - 交互模式
- [ ] **T102**: 实现 `exec` 子命令 - 单次执行
- [ ] **T103**: 实现 `exec --stream` - 流式输出
- [ ] **T104**: 实现 `chat` 子命令 - 对话模式
- [ ] **T105**: 实现 `chat --continue`, `chat --clear`
- [ ] **T106**: 实现 `atom list`, `atom info` 子命令
- [ ] **T107**: 实现 `memory show`, `memory clear`, `memory export`, `memory import` 子命令

### 4.3 用户体验

- [ ] **T111**: 实现彩色输出 (使用 `colored` crate)
- [ ] **T112**: 实现加载动画/进度提示
- [ ] **T113**: 实现优雅的错误提示

**Phase 4 里程碑**:
- ✅ 完整的命令行界面可用
- ✅ 所有子命令可执行
- ✅ 用户可正常交互使用

---

## Phase 5: 完善 (P2)

### 5.1 高级原子

- [ ] **T121**: 实现 `Glob` 原子 - 文件模式匹配
- [ ] **T122**: 实现 `Grep` 原子 - 文本搜索
- [ ] **T123**: 实现 `Edit` 原子 - 文件编辑
- [ ] **T124**: 实现 `MultiFile` 原子 - 批量文件操作

### 5.2 技能系统

- [ ] **T131**: 实现 `SkillLoader` - 加载 Claude Code 格式 Skill
- [ ] **T132**: 实现 skill 目录/文件解析
- [ ] **T133**: 实现 skill 激活机制 (@skill-name 语法)

### 5.3 高级功能

- [ ] **T141**: 实现多 Provider 支持 (OpenAI, Anthropic, 本地模型)
- [ ] **T142**: 实现配置热重载
- [ ] **T143**: 实现插件系统 (动态加载 .so/.dll)

### 5.4 优化

- [ ] **T151**: 性能优化 (连接池, 缓存)
- [ ] **T152**: 单元测试补充 (覆盖率 > 80%)
- [ ] **T153**: 集成测试
- [ ] **T154**: 文档完善

**Phase 5 里程碑**:
- ✅ 丰富的内置工具
- ✅ 支持 Claude Code Skill
- ✅ 生产级稳定性

---

## 任务关联图

```
Phase 1 (基础)
    │
    ├── T001-T003 ──────────────────────────┐
    ├── T011-T016 (config) ──► Phase 1 产出  │
    ├── T021-T028 (memory) ──► config        │
    └── T031-T038 (atoms) ──► memory         │
                                            │
Phase 2 (LLM) ◄─────────────────────────────┤
    │                                       │
    ├── T041-T046 (llm client)              │
    └── T051-T052 (工具描述) ──────────────┘
                                            │
Phase 3 (执行器) ◄──────────────────────────┤
    │                                       │
    ├── T061-T066 (executor core) ◄──┐     │
    └── T071-T083 (主循环) ◄─────────┼─────┘
                                     │      
Phase 4 (CLI) ◄──────────────────────┘      
    │                                      
    ├── T091-T093 (参数) ──► executor       
    └── T101-T113 (子命令) ───► 所有模块    

Phase 5 (完善) ◄────────────────────────────
    │
    ├── T121-T124 (高级 atoms)
    ├── T131-T133 (技能系统)
    └── T141-T154 (高级功能)
```

---

## 验收检查点

| 阶段 | 检查点 | 判定标准 |
|------|--------|----------|
| Phase 1 | 基础编译 | `cargo build` 成功 |
| Phase 1 | 单元测试 | `cargo test` 通过 |
| Phase 2 | API 测试 | 可成功调用 OpenAI API |
| Phase 3 | 端到端测试 | 可执行完整对话流程 |
| Phase 4 | CLI 测试 | 所有子命令可执行 |
| Phase 5 | 发布就绪 | `cargo clippy -- -D warnings` 无警告 |

---

## 优先级说明

- **P0**: 必须完成，否则核心功能不可用
- **P1**: 重要功能，提升可用性
- **P2**: 增强功能，提升体验

---

## 建议迭代周期

每个 Phase 建议 1-2 天完成，可根据实际情况调整：

| Phase | 预计时间 | 累计 |
|-------|----------|------|
| Phase 1 | 3-4 天 | 3-4 天 |
| Phase 2 | 2-3 天 | 5-7 天 |
| Phase 3 | 2-3 天 | 7-10 天 |
| Phase 4 | 2-3 天 | 10-13 天 |
| Phase 5 | 5-7 天 | 15-20 天 |

---

*任务清单将根据实际开发情况持续更新*
