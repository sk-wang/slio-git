"""Stash 操作 — 保存、恢复。"""

import driver
from .base import auto_screenshot_on_failure


@auto_screenshot_on_failure
def stash_changes():
    """通过键盘快捷键 stash 当前变更。"""
    # slio-git 的 stash 快捷键
    driver.hotkey("command", "shift", "s")
    driver.sleep(2)


@auto_screenshot_on_failure
def pop_stash():
    """通过键盘快捷键恢复最近的 stash。"""
    driver.hotkey("command", "shift", "p")
    driver.sleep(2)
