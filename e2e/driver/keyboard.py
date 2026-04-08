"""键盘操作 — 按键、组合键、文本输入。"""

import pyautogui


def press(key: str):
    """按下单个键。"""
    pyautogui.press(key)


def hotkey(*keys: str):
    """组合键，如 hotkey('command', 'c')。"""
    pyautogui.hotkey(*keys)


def type_text(text: str, interval: float = 0.05):
    """逐字输入（仅 ASCII）。默认 interval=0.05 避免丢字。"""
    pyautogui.typewrite(text, interval=interval)
