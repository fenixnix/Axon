# LM Studio API 参考文档

> **更新时间**: 2026-03-17
> **测试模型**: qwen/qwen3-vl-30b (视觉模型)

---

## 基础信息

| 项目 | 值 |
|------|-----|
| **Base URL** | `http://localhost:1234/v1` |
| **协议** | OpenAI Compatible API |
| **认证** | 不需要或任意 API key |

---

## 可用模型列表

```
qwen/qwen3.5-9b           # ⚠️ 意外支持视觉/OCR!
qwen/qwen3-vl-30b         # 视觉模型 (推荐 OCR)
qwen/qwen2.5-vl-7b        # 视觉模型
qwen-2-vl-7b-ocr          # OCR 专用模型
google/gemma-3-4b           # 文本模型
zai-org/glm-4.7-flash      # 文本模型
```

---

## 视觉模型 (Vision/OCR)

### 支持的图片格式

| 格式 | 支持 | 备注 |
|------|------|------|
| JPEG | ✅ 推荐 | 兼容性最好 |
| PNG | ✅ | 支持 |
| WebP | ⚠️ | 需要转换为 JPEG |

### 关键发现

1. **图片转换**: WebP 格式需要转换为 JPEG
2. **Base64 格式**: `data:image/jpeg;base64,{base64编码}`
3. **模型选择**: `qwen/qwen3-vl-30b` 推荐用于 OCR
4. **意外发现**: `qwen/qwen3.5-9b` 竟然也支持图片输入！

---

## OCR 测试示例

### Python 代码

```python
import json
import base64
import requests
from PIL import Image
import io

def ocr_image(image_path: str, prompt: str = "请提取这张图片中的文字") -> str:
    """使用 LM Studio 视觉模型进行 OCR"""
    
    # 转换为 JPEG 格式
    img = Image.open(image_path)
    buffer = io.BytesIO()
    img.save(buffer, format='JPEG')
    base64_image = base64.b64encode(buffer.getvalue()).decode('utf-8')
    
    messages = [
        {
            "role": "user",
            "content": [
                {
                    "type": "image_url",
                    "image_url": {
                        "url": f"data:image/jpeg;base64,{base64_image}"
                    }
                },
                {
                    "type": "text",
                    "text": prompt
                }
            ]
        }
    ]
    
    payload = {
        "model": "qwen/qwen3-vl-30b",
        "messages": messages,
        "max_tokens": 2048
    }
    
    response = requests.post(
        "http://127.0.0.1:1234/v1/chat/completions",
        json=payload,
        timeout=300
    )
    
    result = response.json()
    return result["choices"][0]["message"]["content"]

# 使用
result = ocr_image("sample.png", "请提取图片中的文字")
print(result)
```

### 测试结果

| 模型 | 状态 | OCR 结果 |
|------|------|----------|
| qwen/qwen3.5-9b | ✅ | 太平天国 |
| qwen/qwen3-vl-30b | ✅ | 太平天国 |
| qwen/qwen2.5-vl-7b | ✅ | 太平天国 |

---

## 消息格式详解

### 多模态消息结构

```json
{
  "role": "user",
  "content": [
    {
      "type": "image_url",
      "image_url": {
        "url": "data:image/jpeg;base64,/9j/4AAQSkZJRg..."
      }
    },
    {
      "type": "text",
      "text": "请描述这张图片"
    }
  ]
}
```

### 关键要点

1. **content 是数组**: 支持多张图片 + 文本混合
2. **type 字段**: 必须指定 `image_url` 或 `text`
3. **url 格式**: `data:image/jpeg;base64,{base64编码}`
4. **模型参数**: 视觉模型需要使用对应的模型 ID

---

## 常见错误

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `'url' field must be a base64 encoded image` | 图片格式不支持 | 转换为 JPEG |
| `Invalid url` | base64 格式错误 | 检查编码 |
| `Operation canceled` | 模型未加载 | 等待模型加载 |
| 400 Bad Request | 格式错误 | 检查 JSON 结构 |

---

## Axon 集成配置

```yaml
llm:
  model: "qwen/qwen3-vl-30b"  # 视觉模型
  api_key: ""
  base_url: "http://127.0.0.1:1234/v1"
  timeout_secs: 120
```

### 未来: Axon 视觉支持

计划在 Phase 5 添加视觉原子:

```rust
// 计划新增 atom
define_atom!(
    ImageOCR,
    "从图片中提取文字",
    |args: Value| async move {
        let path = args["path"].as_str().ok_or_else(|| ...)?;
        // 调用视觉模型进行 OCR
    }
);
```
