# Axon：像神经冲动一样思考与执行

---

## 当命令行遇上神经网络

想象一下，你的终端里住着一个不知疲倦的数字神经元——它接收你的想法，瞬间转化为行动，然后悄然退场，不留下任何负担。

这就是 **Axon**。

---

## 这可不是又一个 AI CLI

市面上的 AI 助手不少，但大多臃肿不堪——后台跑着守护进程，占着内存，等着被你召唤。

Axon 不一样。

它更像**一发神经脉冲**——你下令，它执行，然后消失。没有后台状态，没有额外负担。整个程序就一个约 5MB 的可执行文件，拷进 U 盘就能带走。

```bash
axon exec "帮我找找这个目录下最大的文件"
```

0.23 秒，结果就摆在眼前。

---

## 从生物神经元汲取灵感

Axon 的架构直接借鉴了神经元的工作方式：

- **Dendrite（树突）** → 解析你的输入指令
- **Soma（胞体）** → 异步处理核心逻辑
- **Axon（轴突）** → 路由并执行任务
- **Atoms（原子）** → 具体的工具能力

每个工具都是一个"原子"，你可以像搭乐高一样自由组合。文件读写、Shell 命令、代码搜索、文件匹配——全部模块化，全部可编程。

---

## LLM？随便你选

Axon 不绑定任何模型供应商。

- 用 OpenAI GPT-4o Mini？配置好 API Key 就能跑
- 想跑本地模型？LM Studio 一行命令
- 以后想换别的？改个配置文件的事

简单说：**你想用什么模型，就用什么模型**。

---

## 用完即走，不拖泥带水

Axon 采用的是"Magazine Mode"（弹匣模式）——听名字就知道什么意思。

传统 AI 助手像一把时刻上膛的枪，随时待命。Axon 更像子弹——需要时装填，击发，结束。

这意味着：
- 没有后台进程占资源
- 没有状态污染
- 每次调用都是干净的开始

如果你想要一个随叫随到、用完即走的工具，Axon 正合适。

---

## 技能系统：直接兼容 Claude Code

Axon 原生支持 Claude Code 的技能格式。

把一个技能文件夹丢进 `skills/` 目录，它就能用：

```
skills/
├── code-review/
│   ├── skill.json
│   └── SKILL.md
└── git-workflow.md
```

然后：
```bash
axon exec "@code-review review src/main.rs"
```

别人写好的工作流，拿来直接用。这才是真正的"站在巨人的肩膀上"。

---

## 快速上手

一行安装：

```bash
curl -fsSL https://raw.githubusercontent.com/fenixnix/Axon/main/install.sh | bash
```

或者自己编译：

```bash
git clone https://github.com/fenixnix/Axon.git
cd Axon
cargo build --release
```

配置好你的 OpenAI API Key（或者直接用本地 LM Studio），然后：

```bash
axon exec "Hello world"
```

就这么简单。

---

## 谁会用得上？

- **运维工程师**：查日志、跑脚本，快准狠
- **开发者**：代码搜索、批量文件操作，效率翻倍
- **效率控**：任何需要 AI 搭把手的终端场景
- **模型玩家**：想用自己的 LLM 跑自动化任务

---

## 未来会怎样？

Axon 还在进化：

- WASM 支持 → 在浏览器里跑
- 插件系统 → 动态加载自定义原子
- GUI 界面 → 可选的桌面应用
- 云部署 → Serverless 函数支持

但核心理念不会变：**轻量、瞬时、高效执行**。

---

## 项目地址

🔗 **GitHub**: https://github.com/fenixnix/Axon

---

**Axon** ——

*记忆安全，神经速度。*
