"""图像查找与匹配 — confidence 阈值、灰度模式、区域限定搜索。"""

import time
from typing import Optional, Tuple

import pyautogui
from PIL import Image, ImageChops

from .window import Rect


class ImageNotFoundError(Exception):
    """目标图像在屏幕上未找到。"""
    pass


def find_image(
    image_path: str,
    confidence: float = 0.8,
    region: Optional[Tuple[int, int, int, int]] = None,
    grayscale: bool = False,
) -> Optional[Rect]:
    """在屏幕上查找图片，返回匹配区域。找不到返回 None。"""
    try:
        location = pyautogui.locateOnScreen(
            image_path,
            confidence=confidence,
            region=region,
            grayscale=grayscale,
        )
        if location:
            return Rect(location.left, location.top, location.width, location.height)
    except pyautogui.ImageNotFoundException:
        pass
    return None


def wait_image(
    image_path: str,
    timeout: float = 5,
    confidence: float = 0.8,
    interval: float = 0.3,
    grayscale: bool = False,
) -> Rect:
    """反复查找图片直到出现或超时。超时抛出 ImageNotFoundError。"""
    deadline = time.time() + timeout
    while time.time() < deadline:
        result = find_image(image_path, confidence=confidence, grayscale=grayscale)
        if result:
            return result
        time.sleep(interval)
    raise ImageNotFoundError(f"等待超时({timeout}s): {image_path}")


def wait_disappear(
    image_path: str,
    timeout: float = 5,
    confidence: float = 0.8,
    interval: float = 0.3,
) -> bool:
    """等待图片从屏幕消失。超时返回 False。"""
    deadline = time.time() + timeout
    while time.time() < deadline:
        if not find_image(image_path, confidence=confidence):
            return True
        time.sleep(interval)
    return False


def compare_images(
    actual_path: str,
    expected_path: str,
    threshold: float = 0.05,
) -> Tuple[float, Optional[Image.Image]]:
    """对比两张图片，返回 (差异百分比, 差异高亮图)。

    差异百分比 < threshold 视为匹配。
    """
    actual = Image.open(actual_path).convert("RGB")
    expected = Image.open(expected_path).convert("RGB")

    # 统一尺寸
    if actual.size != expected.size:
        expected = expected.resize(actual.size, Image.LANCZOS)

    diff = ImageChops.difference(actual, expected)
    pixels = list(diff.getdata())
    total = len(pixels) * 255 * 3  # max possible difference
    actual_diff = sum(sum(p) for p in pixels)
    diff_pct = actual_diff / total if total > 0 else 0.0

    # 生成差异高亮图（放大差异）
    diff_highlight = diff.point(lambda x: min(x * 10, 255))

    return diff_pct, diff_highlight
