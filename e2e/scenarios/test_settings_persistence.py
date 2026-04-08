"""
E2E 场景: 设置持久化

流程: 打开设置 → 修改选项 → 保存 → 重启应用 → 验证设置保持
"""

import os

import driver
from actions import toolbar
from actions.app import restart_app, wait_app_ready


class Test设置持久化:
    def test_打开设置面板(self, app):
        toolbar.open_settings()
        driver.window_screenshot("settings_01_设置面板")

    def test_勾选签署提交(self, app):
        # checkbox 在设置面板左上方
        driver.click_relative(0.06, 0.19)
        driver.sleep(0.5)
        driver.window_screenshot("settings_02_勾选后")

    def test_点击保存(self, app):
        driver.click_relative(0.975, 0.965)
        driver.sleep(1)
        driver.window_screenshot("settings_03_保存后")

    def test_设置文件存在(self, app):
        candidates = [
            os.path.expanduser("~/Library/Application Support/slio-git/git-settings-v1.txt"),
            os.path.expanduser("~/.local/share/slio-git/git-settings-v1.txt"),
        ]
        found = next((p for p in candidates if os.path.exists(p)), None)
        assert found, f"设置文件未找到: {candidates}"
        content = open(found).read()
        assert "sign_off_commit" in content

    def test_重启后设置保持(self, app):
        restart_app(repo_path=app)
        wait_app_ready(timeout=15)
        driver.window_screenshot("settings_04_重启后")
        # 重新打开设置面板验证
        toolbar.open_settings()
        driver.sleep(1)
        driver.window_screenshot("settings_05_重启后设置面板")
