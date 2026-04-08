"""鼠标操作 — 点击、双击、右键、拖拽、找图点击。"""

import pyautogui

from .window import get_bounds
from .image_match import find_image, wait_image, ImageNotFoundError


def click(x: int, y: int):
    """移动鼠标到指定位置并单击。"""
    pyautogui.click(x, y)


def double_click(x: int, y: int):
    pyautogui.doubleClick(x, y)


def right_click(x: int, y: int):
    pyautogui.rightClick(x, y)


def click_relative(rx: float, ry: float):
    """点击窗口内的相对位置 (0.0~1.0)。"""
    rect = get_bounds()
    x = rect.x + int(rect.w * rx)
    y = rect.y + int(rect.h * ry)
    pyautogui.click(x, y)


def click_image(
    image_path: str,
    timeout: float = 5,
    confidence: float = 0.8,
) -> bool:
    """找到图片后点击其中心。找不到抛出 ImageNotFoundError。"""
    result = wait_image(image_path, timeout, confidence)
    cx, cy = result.center
    pyautogui.click(cx, cy)
    return True


def drag(x1: int, y1: int, x2: int, y2: int, duration: float = 0.5):
    pyautogui.moveTo(x1, y1)
    pyautogui.drag(x2 - x1, y2 - y1, duration=duration)
