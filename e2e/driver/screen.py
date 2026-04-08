"""截图 — 全屏、窗口、区域截图及截图断言。"""

import os
import subprocess

import pyautogui
from PIL import Image

from .window import Rect, get_bounds
from .image_match import compare_images

SCREENSHOT_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "screenshots")
OUTPUT_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "output")


def _get_screen_scale() -> int:
    result = subprocess.run(
        ["system_profiler", "SPDisplaysDataType"],
        capture_output=True, text=True,
    )
    return 2 if "Retina" in result.stdout else 1


def fullscreen(name: str = "screen") -> str:
    """截取全屏并保存。返回文件路径。"""
    os.makedirs(SCREENSHOT_DIR, exist_ok=True)
    path = os.path.join(SCREENSHOT_DIR, f"{name}.png")
    img = pyautogui.screenshot()
    img.save(path)
    return path


def window(name: str = "window") -> str:
    """截取 slio-git 窗口区域。"""
    os.makedirs(SCREENSHOT_DIR, exist_ok=True)
    rect = get_bounds()
    img = pyautogui.screenshot(region=(rect.x, rect.y, rect.w, rect.h))
    path = os.path.join(SCREENSHOT_DIR, f"{name}.png")
    img.save(path)
    return path


def region(rx: float, ry: float, rw: float, rh: float, name: str = "region") -> str:
    """截取窗口内指定比例区域 (0.0~1.0)。"""
    os.makedirs(SCREENSHOT_DIR, exist_ok=True)
    rect = get_bounds()
    x = rect.x + int(rect.w * rx)
    y = rect.y + int(rect.h * ry)
    w = int(rect.w * rw)
    h = int(rect.h * rh)
    img = pyautogui.screenshot(region=(x, y, w, h))
    path = os.path.join(SCREENSHOT_DIR, f"{name}.png")
    img.save(path)
    return path


def assert_screenshot(
    name: str,
    expected_path: str,
    threshold: float = 0.05,
) -> str:
    """截图并与参考图对比。差异超阈值断言失败并保存差异图。"""
    actual_path = window(name)
    diff_pct, diff_img = compare_images(actual_path, expected_path, threshold)

    if diff_pct > threshold:
        os.makedirs(OUTPUT_DIR, exist_ok=True)
        diff_path = os.path.join(OUTPUT_DIR, f"diff_{name}.png")
        diff_img.save(diff_path)
        raise AssertionError(
            f"截图断言失败: {name} 差异 {diff_pct:.1%} > 阈值 {threshold:.1%}\n"
            f"  实际: {actual_path}\n"
            f"  期望: {expected_path}\n"
            f"  差异图: {diff_path}"
        )

    return actual_path
