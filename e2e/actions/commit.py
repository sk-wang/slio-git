"""提交操作 — 输入信息、确认、取消。"""

import driver
from .base import auto_screenshot_on_failure
from .toolbar import open_commit_dialog as _open_commit


@auto_screenshot_on_failure
def type_commit_message(msg: str):
    """在提交对话框中输入提交信息。"""
    driver.type_text(msg)
    driver.sleep(0.3)


@auto_screenshot_on_failure
def confirm_commit():
    """Cmd+Enter 确认提交。"""
    driver.hotkey("ctrl", "enter")
    driver.sleep(2)


@auto_screenshot_on_failure
def cancel_commit():
    """ESC 取消提交。"""
    driver.press("escape")
    driver.sleep(0.5)
