"""
E2E 场景: Diff 视图切换 (统一/分栏)

覆盖:
  1. 修改文件 → 查看 unified diff
  2. 切换到分栏 (split) 视图
  3. 截图对比两种视图
  4. 切回统一视图
"""

import os

import driver
from scenarios.conftest import add_unstaged_change

# 右上角的 统一/分栏 切换按钮
UNIFIED_BTN = (0.92, 0.10)
SPLIT_BTN = (0.96, 0.10)


class TestDiff视图切换:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_创建文件变更(self, app):
        add_unstaged_change(
            app,
            filename="src/main.py",
            content='def main():\n    print("diff toggle test")\n    x = 1\n    y = 2\n    return x + y\n',
        )
        driver.sleep(3)

    def test_选中文件查看diff(self, app):
        driver.click_relative(0.12, 0.18)
        driver.sleep(1)
        driver.window_screenshot("difftoggle_01_unified视图")

    def test_截图unified区域(self, app):
        path = driver.region(0.35, 0.08, 0.60, 0.60, "difftoggle_02_unified区域")
        assert os.path.exists(path)

    def test_切换到分栏(self, app):
        """点击 "分栏" 按钮。"""
        driver.click_relative(*SPLIT_BTN)
        driver.sleep(1)
        driver.window_screenshot("difftoggle_03_split视图")

    def test_截图split区域(self, app):
        path = driver.region(0.35, 0.08, 0.60, 0.60, "difftoggle_04_split区域")
        assert os.path.exists(path)

    def test_切回统一(self, app):
        driver.click_relative(*UNIFIED_BTN)
        driver.sleep(1)
        driver.window_screenshot("difftoggle_05_回到unified")

    def test_清理(self, app):
        import subprocess
        subprocess.run(["git", "checkout", "."], cwd=app, capture_output=True)
"""
"""
