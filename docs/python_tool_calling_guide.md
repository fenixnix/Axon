# Python 工具调用开发指南

基于 LM Studio API 测试经验总结

---

## 1. 工具调用流程概述

```
用户输入 → LLM 判断 → 返回 tool_calls → 执行工具 → 返回结果 → LLM 生成回复
```

## 2. 关键发现

### 2.1 工具定义格式（重要！）

**✅ 正确的 OpenAI 格式：**
```json
{
  "type": "function",
  "function": {
    "name": "shell_exec",
    "description": "执行 shell 命令",
    "parameters": {
      "type": "object",
      "properties": {
        "command": {
          "type": "string",
          "description": "要执行的命令"
        }
      },
      "required": ["command"]
    }
  }
}
```

**❌ 错误的简化格式：**
```json
{
  "name": "shell_exec",
  "description": "执行 shell 命令",
  "parameters": {...}
}
```

**错误信息：**
```
400 Bad Request: Invalid literal value, expected "function"
```

### 2.2 响应格式

**LLM 返回的 tool_calls 格式：**
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": null,
      "tool_calls": [{
        "id": "280535024",
        "type": "function",
        "function": {
          "name": "read_file",
          "arguments": "{\"path\": \"README.md\"}"
        }
      }]
    },
    "finish_reason": "tool_calls"
  }]
}
```

**关键字段：**
- `tool_calls[].function.name` - 工具名称
- `tool_calls[].function.arguments` - **JSON 字符串**，需要解析
- `tool_calls[].id` - 工具调用 ID，后续需要返回

### 2.3 完整执行流程

```python
# 1. 第一轮请求（带 tools）
messages = [
    {"role": "system", "content": "你是一个助手"},
    {"role": "user", "content": "列出当前目录文件"}
]

response = client.chat_completion(messages, tools=tools)

# 2. 检查是否有工具调用
if response["choices"][0]["message"].get("tool_calls"):
    tool_calls = response["choices"][0]["message"]["tool_calls"]
    
    # 3. 添加 assistant 消息（包含 tool_calls）到历史
    messages.append({
        "role": "assistant",
        "content": None,
        "tool_calls": tool_calls
    })
    
    # 4. 执行工具并添加结果
    for tc in tool_calls:
        func = tc["function"]
        name = func["name"]
        args = json.loads(func["arguments"])  # 解析 JSON 字符串
        
        # 执行工具
        result = execute_tool(name, args)
        
        # 添加 tool 消息
        messages.append({
            "role": "tool",
            "tool_call_id": tc["id"],
            "content": str(result)
        })
    
    # 5. 第二轮请求获取最终回复
    final_response = client.chat_completion(messages, tools=tools)
```

## 3. 常见陷阱

### 3.1 arguments 是字符串不是对象
```python
# ❌ 错误
args = tool_call["function"]["arguments"]["param"]

# ✅ 正确
args = json.loads(tool_call["function"]["arguments"])
param = args["param"]
```

### 3.2 必须添加 assistant 消息
在执行工具前，必须将包含 `tool_calls` 的 assistant 消息添加到历史：
```python
messages.append({
    "role": "assistant",
    "content": None,  # 必须有 content 字段，即使为 null
    "tool_calls": tool_calls
})
```

### 3.3 tool 消息必须包含 tool_call_id
```python
messages.append({
    "role": "tool",
    "tool_call_id": tc["id"],  # 必须与 tool_calls 中的 id 匹配
    "content": result
})
```

## 4. 模型兼容性

### 4.1 测试过的模型

| 模型 | 工具调用支持 | 备注 |
| :--- | :---: | :--- |
| qwen/qwen3.5-9b | ✅ | 表现良好 |
| qwen/qwen3-coder-30b | ✅ | 适合代码任务 |
| qwen/qwen2.5-vl-7b | ⚠️ | 视觉模型，工具支持一般 |

### 4.2 工具调用触发率

不是所有请求都会触发工具调用，取决于：
1. 用户请求的明确性（"使用 shell_exec 列出文件" vs "列出文件"）
2. 系统提示词的引导
3. 模型的工具调用能力

## 5. 调试技巧

### 5.1 打印完整请求体
```python
print(json.dumps(payload, indent=2, ensure_ascii=False))
```

### 5.2 检查响应中的 finish_reason
- `"stop"` - 正常完成
- `"tool_calls"` - 触发了工具调用
- `"length"` - 达到长度限制

### 5.3 验证工具定义格式
```python
# 发送测试请求验证格式
response = client.chat_completion(
    messages=[{"role": "user", "content": "测试"}],
    tools=tools
)
if response.status_code == 200:
    print("工具定义格式正确")
else:
    print(f"格式错误: {response.text}")
```

## 6. 最佳实践

### 6.1 参数设计
- 使用清晰的参数名
- 提供详细的 description
- 明确标记 required 字段

### 6.2 错误处理
```python
try:
    args = json.loads(func["arguments"])
except json.JSONDecodeError:
    return {"error": "Invalid arguments format"}

try:
    result = execute_tool(args)
except Exception as e:
    return {"error": str(e)}
```

### 6.3 超时控制
```python
response = requests.post(
    url,
    json=payload,
    timeout=60  # 设置合理的超时时间
)
```

## 7. 参考代码

完整测试脚本见：`scripts/test_lmstudio_api.py`

核心类：
```python
class LMStudioClient:
    def chat_completion(self, messages, tools=None, ...)
    def list_models(self)
```

## 8. 故障排查

| 问题 | 可能原因 | 解决方案 |
| :--- | :--- | :--- |
| 400 Bad Request | 工具格式错误 | 检查 `type` 和 `function` 包装 |
| tool_calls 为空 | 模型不支持/提示不明确 | 明确请求使用工具 |
| 参数解析失败 | arguments 格式错误 | 确保是 JSON 字符串 |
| 工具执行无响应 | 网络/模型问题 | 检查 LM Studio 状态和日志 |

---

> **经验总结**: 工具调用的关键是严格遵循 OpenAI API 格式，特别是 `type: "function"` 包装层和 `arguments` 字符串格式。
