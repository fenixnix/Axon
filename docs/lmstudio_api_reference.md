# LM Studio API 参考文档

> **更新时间**: 2026-03-17
> **测试模型**: qwen/qwen3.5-9b

---

## 基础信息

| 项目 | 值 |
|------|-----|
| **Base URL** | `http://localhost:1234/v1` |
| **协议** | OpenAI Compatible API |
| **认证** | 不需要或任意 API key（如 `not-needed`） |
| **测试模型** | qwen/qwen3.5-9b |

---

## 端点

### 1. Chat Completions

**URL**: `POST /chat/completions`

**请求头**:
```http
Content-Type: application/json
Authorization: Bearer not-needed
```

**请求体格式**:
```json
{
  "model": "qwen/qwen3.5-9b",
  "messages": [
    {"role": "system", "content": "你是一个 helpful assistant。"},
    {"role": "user", "content": "用户消息"}
  ],
  "temperature": 0.7,
  "stream": false,
  "tools": [...],
  "tool_choice": "auto"
}
```

---

## 工具调用 (Function Calling)

### 工具定义格式

**标准 OpenAI 格式** (推荐):
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

### 实际测试结果

**请求**:
```json
{
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "read_file",
        "description": "读取文件内容",
        "parameters": {
          "type": "object",
          "properties": {
            "path": {
              "type": "string",
              "description": "文件路径"
            }
          },
          "required": ["path"]
        }
      }
    }
  ]
}
```

**响应** (成功调用工具):
```json
{
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": null,
        "tool_calls": [
          {
            "type": "function",
            "id": "736917795",
            "function": {
              "name": "read_file",
              "arguments": "{\"path\":\"README.md\"}"
            }
          }
        ]
      },
      "finish_reason": "tool_calls"
    }
  ]
}
```

### 关键字段说明

| 字段 | 说明 |
|------|------|
| `finish_reason: "tool_calls"` | 表示模型请求调用工具 |
| `tool_calls[].function.name` | 工具名称 |
| `tool_calls[].function.arguments` | JSON 字符串格式的参数 |
| `tool_calls[].id` | 工具调用 ID，用于返回结果 |

---

## 工具执行流程

```
1. 发送: 用户消息 + tools 定义
         ↓
2. 响应: AI 返回 tool_calls (finish_reason: "tool_calls")
         ↓
3. 执行: 本地执行工具函数
         ↓
4. 添加消息:
   - role: "assistant", tool_calls: [...]
   - role: "tool", tool_call_id: "xxx", content: "执行结果"
         ↓
5. 再次发送: 获取 AI 最终回复
```

### 工具结果消息格式

```json
{
  "role": "tool",
  "tool_call_id": "736917795",
  "content": "文件内容..."
}
```

---

## 注意事项

1. **模型支持**: 并非所有模型都支持工具调用，测试模型 `qwen/qwen3.5-9b` 支持
2. **工具格式**: 必须使用 `"type": "function"` 包装
3. **参数解析**: `arguments` 是 JSON 字符串，需要解析
4. **中文提示**: 使用中文提示词效果更好

---

## 常见问题

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `tool_calls` 为空 | 模型不支持工具调用或提示不明确 | 确认模型支持 tool calls，使用明确提示 |
| 参数解析失败 | `arguments` 不是有效 JSON | 检查参数格式 |
| 400 Bad Request | tools 格式错误 | 检查 JSON Schema 格式 |

---

## Axon 集成配置

```yaml
llm:
  model: "qwen/qwen3.5-9b"  # 或你加载的模型名
  api_key: ""                # LM Studio 不需要
  base_url: "http://localhost:1234/v1"
  timeout_secs: 120
```
