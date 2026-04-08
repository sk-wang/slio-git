"""
slio-git RPA Driver — 统一入口

按键精灵风格的桌面自动化核心，分为四个子模块:
  - window: 窗口管理（激活、定位、最大化）
  - screen: 截图（全屏、窗口、区域、截图断言）
  - image_match: 图像查找（confidence、灰度、区域限定）
  - mouse: 鼠标操作（点击、拖拽、找图点击）
  - keyboard: 键盘操作（按键、组合键、文字输入）
"""

import time

import pyautogui

from .window import Rect, APP_PROCESS, activate, get_bounds, maximize, prepare, is_alive
from .screen import fullscreen, window as window_screenshot, region, assert_screenshot, SCREENSHOT_DIR, OUTPUT_DIR
from .image_match import (
    ImageNotFoundError,
    find_image, wait_image, wait_disappear, compare_images,
)
from .mouse import click, double_click, right_click, click_relative, click_image, drag
from .keyboard import press, hotkey, type_text

# PyAutoGUI 全局设置
pyautogui.FAILSAFE = True
pyautogui.PAUSE = 0.15


def sleep(seconds: float):
    """固定等待。"""
    time.sleep(seconds)


# === 兼容旧 API 的中文别名 ===

激活窗口 = activate
获取窗口区域 = get_bounds
最大化窗口 = maximize
窗口置顶并准备 = prepare
进程存活 = is_alive
全屏截图 = fullscreen
窗口截图 = window_screenshot
区域截图 = region
找图 = find_image
等待图片出现 = wait_image
等待图片消失 = wait_disappear
点击 = click
双击 = double_click
右键 = right_click
窗口内点击 = click_relative
找图并点击 = click_image
拖拽 = drag
按键 = press
组合键 = hotkey
输入文字 = type_text
延时 = sleep
截图对比 = window_screenshot

# 断言函数
def 断言图片存在(image_path: str, msg: str = "", timeout: float = 5, confidence: float = 0.8):
    try:
        result = wait_image(image_path, timeout, confidence)
        return result
    except ImageNotFoundError:
        raise AssertionError(msg or f"屏幕上未找到: {image_path}")

def 断言图片不存在(image_path: str, msg: str = "", timeout: float = 3, confidence: float = 0.8):
    gone = wait_disappear(image_path, timeout, confidence)
    assert gone, msg or f"图片仍在屏幕上: {image_path}"
