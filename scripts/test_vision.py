#!/usr/bin/env python3
"""
LM Studio Vision/OCR 测试脚本
用于测试视觉模型的图片识别功能
"""

import json
import base64
import requests
from typing import List, Dict, Any, Optional
from pathlib import Path

# LM Studio 默认配置
DEFAULT_BASE_URL = "http://localhost:1234/v1"
DEFAULT_MODEL = "qwen2.5-7b-instruct"  # 需要替换为视觉模型


class LMStudioVisionClient:
    """LM Studio Vision API 客户端"""

    def __init__(self, base_url: str = DEFAULT_BASE_URL, api_key: str = "not-needed"):
        self.base_url = base_url.rstrip("/")
        self.headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}",
        }

    def encode_image_to_base64(self, image_path: str) -> str:
        """将图片编码为 base64"""
        with open(image_path, "rb") as f:
            return base64.b64encode(f.read()).decode("utf-8")

    def chat_with_image(
        self,
        image_path: str,
        prompt: str,
        model: str = DEFAULT_MODEL,
    ) -> Dict:
        """发送图片聊天请求 (使用 base64 编码)"""

        # 编码图片
        base64_image = self.encode_image_to_base64(image_path)

        # 构建多模态消息
        messages = [
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {"url": f"data:image/jpeg;base64,{base64_image}"},
                    },
                    {"type": "text", "text": prompt},
                ],
            }
        ]

        payload = {"model": model, "messages": messages, "max_tokens": 2048}

        print("=" * 60)
        print(f"请求: {model}")
        print(f"图片: {image_path}")
        print(f"提示: {prompt}")
        print("=" * 60)

        try:
            response = requests.post(
                f"{self.base_url}/chat/completions",
                headers=self.headers,
                json=payload,
                timeout=120,
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

    def chat_with_image_url(
        self,
        image_url: str,
        prompt: str,
        model: str = DEFAULT_MODEL,
    ) -> Dict:
        """发送图片聊天请求 (使用 URL)"""

        messages = [
            {
                "role": "user",
                "content": [
                    {"type": "image_url", "image_url": {"url": image_url}},
                    {"type": "text", "text": prompt},
                ],
            }
        ]

        payload = {"model": model, "messages": messages, "max_tokens": 2048}

        print("=" * 60)
        print(f"请求: {model}")
        print(f"图片 URL: {image_url}")
        print(f"提示: {prompt}")
        print("=" * 60)

        try:
            response = requests.post(
                f"{self.base_url}/chat/completions",
                headers=self.headers,
                json=payload,
                timeout=120,
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
                f"{self.base_url}/models", headers=self.headers, timeout=10
            )
            if response.status_code == 200:
                data = response.json()
                models = [m.get("id", "unknown") for m in data.get("data", [])]
                return models
            return []
        except Exception as e:
            print(f"获取模型列表失败: {e}")
            return []


def test_vision_model():
    """测试视觉模型"""
    print("\n" + "=" * 60)
    print("测试: Vision/OCR 功能")
    print("=" * 60)

    client = LMStudioVisionClient()

    # 获取模型列表
    models = client.list_models()
    print(f"\n可用模型: {models}")

    # 查找视觉模型
    vision_models = [
        m
        for m in models
        if "vision" in m.lower() or "vl" in m.lower() or "qwen2-vl" in m.lower()
    ]
    print(f"视觉模型: {vision_models}")

    return models, vision_models


def test_ocr(image_path: str, prompt: str = "请描述这张图片中的文字内容"):
    """测试 OCR 功能"""
    print("\n" + "=" * 60)
    print("测试: OCR 图片文字识别")
    print("=" * 60)

    client = LMStudioVisionClient()

    # 尝试查找视觉模型
    models = client.list_models()
    print(f"\n可用模型: {models}")

    # 查找视觉模型
    vision_model = None
    for m in models:
        if "vision" in m.lower() or "vl" in m.lower() or "qwen2-vl" in m.lower():
            vision_model = m
            break

    if not vision_model:
        # 尝试常用视觉模型名
        for m in models:
            if "qwen" in m.lower() or "llama" in m.lower() or "phi" in m.lower():
                vision_model = m
                break

    if not vision_model:
        print("未找到视觉模型，请先在 LM Studio 中加载")
        return None

    print(f"\n使用模型: {vision_model}")
    print(f"图片路径: {image_path}")

    # 测试 OCR
    result = client.chat_with_image(
        image_path=image_path, prompt=prompt, model=vision_model
    )

    if "choices" in result:
        content = result["choices"][0].get("message", {}).get("content", "")
        print(f"\nOCR 结果: {content}")

    return result


if __name__ == "__main__":
    import sys
    import os

    # 创建 docs 目录
    os.makedirs("docs", exist_ok=True)

    print("=" * 60)
    print("LM Studio Vision/OCR 测试脚本")
    print("=" * 60)

    # 先列出模型
    models, vision_models = test_vision_model()

    # 如果有命令行参数，使用指定图片测试
    if len(sys.argv) > 1:
        image_path = sys.argv[1]
        prompt = sys.argv[2] if len(sys.argv) > 2 else "请描述这张图片中的文字内容"

        if not os.path.exists(image_path):
            print(f"\n错误: 图片文件不存在: {image_path}")
            sys.exit(1)

        test_ocr(image_path, prompt)
    else:
        print("\n用法:")
        print("  python test_vision.py <图片路径> [提示词]")
        print("\n示例:")
        print('  python test_vision.py test.png "请提取图片中的文字"')
        print("\n或运行基础测试:")
        print("  python test_vision.py")
