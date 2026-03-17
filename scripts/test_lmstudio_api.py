#!/usr/bin/env python3
"""
LM Studio API 测试脚本
用于验证 API 调用格式和工具调用功能
"""

import json
import requests
from typing import List, Dict, Any, Optional

# LM Studio 默认配置
DEFAULT_BASE_URL = "http://localhost:1234/v1"
DEFAULT_MODEL = "qwen2.5-7b-instruct"

class LMStudioClient:
    """LM Studio API 客户端"""
    
    def __init__(self, base_url: str = DEFAULT_BASE_URL, api_key: str = "not-needed"):
        self.base_url = base_url.rstrip('/')
        self.headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}"
        }
    
    def chat_completion(
        self,
        messages: List[Dict[str, str]],
        model: str = DEFAULT_MODEL,
        tools: Optional[List[Dict]] = None,
        tool_choice: str = "auto",
        temperature: float = 0.7,
        stream: bool = False
    ) -> Dict:
        """发送聊天完成请求"""
        
        payload = {
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "stream": stream
        }
        
        # 只有在有工具时才添加 tools 参数
        if tools:
            payload["tools"] = tools
            payload["tool_choice"] = tool_choice
        
        print("=" * 60)
        print("请求 URL:", f"{self.base_url}/chat/completions")
        print("请求体:")
        print(json.dumps(payload, indent=2, ensure_ascii=False))
        print("=" * 60)
        
        try:
            response = requests.post(
                f"{self.base_url}/chat/completions",
                headers=self.headers,
                json=payload,
                timeout=60
            )
            
            print(f"\n状态码: {response.status_code}")
            
            if response.status_code == 200:
                result = response.json()
                print("\n响应:")
                print(json.dumps(result, indent=2, ensure_ascii=False))
                return result
            else:
                print(f"\n错误响应:")
                print(response.text)
                return {"error": response.text}
                
        except Exception as e:
            print(f"\n请求异常: {e}")
            return {"error": str(e)}
    
    def list_models(self) -> List[str]:
        """获取可用模型列表"""
        try:
            response = requests.get(
                f"{self.base_url}/models",
                headers=self.headers,
                timeout=10
            )
            if response.status_code == 200:
                data = response.json()
                models = [m.get("id", "unknown") for m in data.get("data", [])]
                return models
            return []
        except Exception as e:
            print(f"获取模型列表失败: {e}")
            return []


def test_basic_chat():
    """测试基础聊天功能"""
    print("\n" + "=" * 60)
    print("测试 1: 基础聊天（无工具）")
    print("=" * 60)
    
    client = LMStudioClient()
    
    messages = [
        {"role": "system", "content": "你是一个 helpful assistant。"},
        {"role": "user", "content": "你好，请介绍一下自己"}
    ]
    
    result = client.chat_completion(messages)
    
    if "choices" in result:
        content = result["choices"][0].get("message", {}).get("content", "")
        print(f"\nAI 回复: {content}")
    return result


def test_with_tools():
    """测试带工具的聊天功能"""
    print("\n" + "=" * 60)
    print("测试 2: 带工具的聊天")
    print("=" * 60)
    
    client = LMStudioClient()
    
    # 定义工具 - OpenAI 格式
    tools = [
        {
            "type": "function",
            "function": {
                "name": "get_current_time",
                "description": "获取当前时间",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        },
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
    
    messages = [
        {"role": "system", "content": "你是一个 helpful assistant，可以使用工具帮助用户。"},
        {"role": "user", "content": "请读取 README.md 文件的内容"}
    ]
    
    result = client.chat_completion(
        messages=messages,
        tools=tools,
        tool_choice="auto"
    )
    
    # 检查是否有工具调用
    if "choices" in result:
        message = result["choices"][0].get("message", {})
        tool_calls = message.get("tool_calls", [])
        
        if tool_calls:
            print(f"\n检测到 {len(tool_calls)} 个工具调用:")
            for tc in tool_calls:
                print(f"  - 工具: {tc.get('function', {}).get('name')}")
                print(f"    参数: {tc.get('function', {}).get('arguments')}")
        else:
            content = message.get("content", "")
            print(f"\nAI 直接回复: {content}")
    
    return result


def test_tool_execution_flow():
    """测试完整的工具执行流程"""
    print("\n" + "=" * 60)
    print("测试 3: 完整工具执行流程")
    print("=" * 60)
    
    client = LMStudioClient()
    
    tools = [
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
    ]
    
    # 第一轮：用户请求
    messages = [
        {"role": "system", "content": "你是一个 helpful assistant，可以使用工具帮助用户。"},
        {"role": "user", "content": "请列出当前目录的文件"}
    ]
    
    print("\n>>> 第一轮请求")
    result1 = client.chat_completion(messages, tools=tools)
    
    if "choices" not in result1:
        print("请求失败")
        return
    
    message1 = result1["choices"][0]["message"]
    
    # 检查是否有工具调用
    if "tool_calls" in message1:
        print("\nAI 请求调用工具:")
        for tc in message1["tool_calls"]:
            func = tc.get("function", {})
            print(f"  工具: {func.get('name')}")
            print(f"  参数: {func.get('arguments')}")
            
            # 模拟执行工具
            if func.get("name") == "shell_exec":
                import subprocess
                try:
                    args = json.loads(func.get("arguments", "{}"))
                    cmd = args.get("command", "")
                    print(f"\n  [执行工具] {cmd}")
                    # 实际执行（可选）
                    # result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
                    # tool_result = result.stdout
                    tool_result = "file1.txt\nfile2.txt\nREADME.md"
                except Exception as e:
                    tool_result = f"错误: {e}"
                
                # 添加工具调用和结果到消息历史
                messages.append({
                    "role": "assistant",
                    "content": None,
                    "tool_calls": [tc]
                })
                messages.append({
                    "role": "tool",
                    "tool_call_id": tc.get("id"),
                    "content": tool_result
                })
        
        # 第二轮：发送工具结果给 AI
        print("\n>>> 第二轮请求（带工具结果）")
        result2 = client.chat_completion(messages, tools=tools)
        
        if "choices" in result2:
            content = result2["choices"][0].get("message", {}).get("content", "")
            print(f"\nAI 最终回复: {content}")
    else:
        content = message1.get("content", "")
        print(f"\nAI 直接回复: {content}")


def test_different_tool_formats():
    """测试不同工具定义格式"""
    print("\n" + "=" * 60)
    print("测试 4: 不同工具定义格式对比")
    print("=" * 60)
    
    client = LMStudioClient()
    
    # 格式 1: 标准 OpenAI 格式
    tools_format1 = [
        {
            "type": "function",
            "function": {
                "name": "test_func",
                "description": "测试函数",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "arg1": {"type": "string"}
                    },
                    "required": ["arg1"]
                }
            }
        }
    ]
    
    # 格式 2: 简化的 tools 格式（某些模型可能支持）
    tools_format2 = [
        {
            "name": "test_func",
            "description": "测试函数",
            "parameters": {
                "type": "object",
                "properties": {
                    "arg1": {"type": "string"}
                },
                "required": ["arg1"]
            }
        }
    ]
    
    messages = [
        {"role": "user", "content": "测试工具调用"}
    ]
    
    print("\n>>> 测试格式 1: 标准 OpenAI 格式 (type: function)")
    result1 = client.chat_completion(messages, tools=tools_format1)
    
    print("\n>>> 测试格式 2: 简化格式（无 type 字段）")
    result2 = client.chat_completion(messages, tools=tools_format2)


def save_api_reference():
    """保存 API 参考文档"""
    doc = """# LM Studio API 参考文档

## 基础信息

- **Base URL**: `http://localhost:1234/v1`
- **协议**: OpenAI Compatible API
- **认证**: 通常不需要或任意 API key

## 端点

### 1. Chat Completions

**URL**: `POST /chat/completions`

**请求体格式**:
```json
{
  "model": "model-name",
  "messages": [
    {"role": "system", "content": "系统提示"},
    {"role": "user", "content": "用户消息"}
  ],
  "temperature": 0.7,
  "stream": false,
  "tools": [...],      // 可选
  "tool_choice": "auto" // 可选: "auto", "none", 或特定工具
}
```

### 2. Tools 格式

**标准 OpenAI 格式**:
```json
{
  "type": "function",
  "function": {
    "name": "function_name",
    "description": "函数描述",
    "parameters": {
      "type": "object",
      "properties": {
        "param1": {
          "type": "string",
          "description": "参数描述"
        }
      },
      "required": ["param1"]
    }
  }
}
```

### 3. 响应格式

**普通响应**:
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "model-name",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "回复内容"
      },
      "finish_reason": "stop"
    }
  ]
}
```

**带工具调用的响应**:
```json
{
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": null,
        "tool_calls": [
          {
            "id": "call_xxx",
            "type": "function",
            "function": {
              "name": "function_name",
              "arguments": "{\\"param1\\": \\"value1\\"}"
            }
          }
        ]
      },
      "finish_reason": "tool_calls"
    }
  ]
}
```

### 4. 工具执行流程

1. 发送用户消息 + tools 定义
2. 接收 AI 响应，检查是否包含 `tool_calls`
3. 执行工具函数，获取结果
4. 将工具结果添加到消息历史：
   - 添加 assistant 消息（包含 tool_calls）
   - 添加 tool 消息（包含执行结果）
5. 再次发送请求获取最终回复

**工具消息格式**:
```json
{
  "role": "tool",
  "tool_call_id": "call_xxx",
  "content": "工具执行结果"
}
```

## 注意事项

1. 不是所有模型都支持工具调用
2. 工具定义必须使用 `"type": "function"` 包装
3. `function.parameters` 必须符合 JSON Schema
4. 工具调用参数是 JSON 字符串，需要解析
5. LM Studio 的兼容性取决于加载的模型

## 常见错误

- `400 Bad Request`: 请求格式错误，检查 tools 格式
- `tool_calls` 为空: 模型不支持工具或提示不够明确
- 参数解析失败: 检查 `arguments` 字段是否为有效 JSON
"""
    
    with open("docs/lmstudio_api_reference.md", "w", encoding="utf-8") as f:
        f.write(doc)
    
    print("\n✓ API 参考文档已保存到: docs/lmstudio_api_reference.md")


if __name__ == "__main__":
    import os
    
    # 创建 docs 目录
    os.makedirs("docs", exist_ok=True)
    
    print("=" * 60)
    print("LM Studio API 测试脚本")
    print("=" * 60)
    print(f"\n默认配置:")
    print(f"  Base URL: {DEFAULT_BASE_URL}")
    print(f"  Model: {DEFAULT_MODEL}")
    print("\n请确保 LM Studio 已启动并加载了模型")
    print("=" * 60)
    
    # 测试模型列表
    client = LMStudioClient()
    models = client.list_models()
    if models:
        print(f"\n✓ 检测到可用模型: {models}")
    else:
        print("\n⚠ 无法获取模型列表，请检查 LM Studio 是否运行")
    
    # 运行测试
    try:
        test_basic_chat()
    except Exception as e:
        print(f"基础聊天测试失败: {e}")
    
    try:
        test_with_tools()
    except Exception as e:
        print(f"工具调用测试失败: {e}")
    
    try:
        test_tool_execution_flow()
    except Exception as e:
        print(f"完整流程测试失败: {e}")
    
    try:
        test_different_tool_formats()
    except Exception as e:
        print(f"格式对比测试失败: {e}")
    
    # 保存文档
    save_api_reference()
    
    print("\n" + "=" * 60)
    print("测试完成")
    print("=" * 60)
