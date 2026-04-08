"""工具栏操作 — 刷新、提交、设置、Tab 切换。"""

import driver
from .base import auto_screenshot_on_failure


# 工具栏按钮相对坐标 (基于 1728x1080 最大化窗口)
REFRESH_BTN = (0.77, 0.03)
COMMIT_BTN = (0.925, 0.03)
SETTINGS_BTN = (0.975, 0.03)
CHANGES_TAB = (0.045, 0.07)
LOG_TAB = (0.085, 0.07)


@auto_screenshot_on_failure
def click_refresh():
    """点击刷新按钮。"""
    driver.click_relative(*REFRESH_BTN)
    driver.sleep(2)


@auto_screenshot_on_failure
def open_commit_dialog():
    """打开提交对话框。"""
    driver.click_relative(*COMMIT_BTN)
    driver.sleep(1)


@auto_screenshot_on_failure
def open_settings():
    """打开设置面板。"""
    driver.click_relative(*SETTINGS_BTN)
    driver.sleep(1)


@auto_screenshot_on_failure
def switch_to_log_tab():
    """切换到日志 Tab。"""
    driver.click_relative(*LOG_TAB)
    driver.sleep(1)


@auto_screenshot_on_failure
def switch_to_changes_tab():
    """切换到变更 Tab。"""
    driver.click_relative(*CHANGES_TAB)
    driver.sleep(1)
