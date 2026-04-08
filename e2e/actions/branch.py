"""分支操作 — 弹窗、搜索、切换。"""

import driver
from .base import auto_screenshot_on_failure

BRANCH_NAME_BTN = (0.14, 0.03)


@auto_screenshot_on_failure
def open_branch_popup():
    """点击分支名打开分支弹窗。"""
    driver.click_relative(*BRANCH_NAME_BTN)
    driver.sleep(1)


@auto_screenshot_on_failure
def close_branch_popup():
    """ESC 关闭分支弹窗。"""
    driver.press("escape")
    driver.sleep(0.5)


@auto_screenshot_on_failure
def search_branch(name: str):
    """在分支弹窗搜索框中输入分支名。"""
    driver.type_text(name)
    driver.sleep(0.5)


@auto_screenshot_on_failure
def switch_branch(name: str):
    """打开分支弹窗 → 搜索 → 回车选择。"""
    open_branch_popup()
    search_branch(name)
    driver.press("enter")
    driver.sleep(2)
