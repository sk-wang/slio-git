"""
E2E 场景: Diff 查看

流程: 修改文件 → 等待变更列表更新 → 点击文件 → 截图验证 diff 区域
"""

import driver
from scenarios.conftest import add_unstaged_change


class TestDiff查看:
    def test_准备文件变更(self, app):
        add_unstaged_change(
            app,
            filename="README.md",
            content="# Test Repository\n\nModified for diff test.\n\nNew line added.\n",
        )
        driver.sleep(3)
        driver.window_screenshot("diff_01_有变更")

    def test_点击变更文件查看diff(self, app):
        # 变更列表通常在左侧面板，点击第一个文件
        driver.click_relative(0.12, 0.20)
        driver.sleep(1)
        driver.window_screenshot("diff_02_diff显示")

    def test_diff区域有内容(self, app):
        """截取 diff 区域并保存，用于人工验证。"""
        # diff editor 通常在右侧主区域
        path = driver.region(0.3, 0.1, 0.65, 0.8, "diff_03_diff区域")
        import os
        assert os.path.exists(path) and os.path.getsize(path) > 0
