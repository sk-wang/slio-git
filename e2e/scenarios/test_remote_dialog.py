"""
E2E 场景: 远程仓库对话框 (Remote Dialog)

流程:
  1. 打开 Remotes 面板
  2. 查看远程列表
  3. 截图对话框布局
  4. 关闭面板

注意: 临时 git 仓库没有远程，所以主要测试空状态 + 面板打开关闭。
可通过 git remote add 添加一个假远程来测试列表显示。
"""

import subprocess

import driver

NAV_REMOTES = (0.012, 0.82)


class TestRemote面板_空状态:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_打开Remotes面板(self, app):
        driver.click_relative(*NAV_REMOTES)
        driver.sleep(1.5)
        driver.window_screenshot("remote_01_面板打开")

    def test_截图面板布局(self, app):
        driver.region(0.03, 0.06, 0.94, 0.90, "remote_02_面板布局")

    def test_关闭面板(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("remote_03_关闭")


class TestRemote面板_有远程:
    """添加一个假远程后查看列表。"""

    def test_添加假远程(self, app):
        """通过 git 命令添加一个假的远程。"""
        subprocess.run(
            ["git", "remote", "add", "origin", "https://example.com/test.git"],
            cwd=app, capture_output=True, check=True,
        )
        result = subprocess.run(
            ["git", "remote", "-v"],
            cwd=app, capture_output=True, text=True,
        )
        print(f"远程列表:\n{result.stdout.strip()}")
        assert "origin" in result.stdout

    def test_刷新并打开Remotes(self, app):
        driver.activate()
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.click_relative(*NAV_REMOTES)
        driver.sleep(1.5)
        driver.window_screenshot("remote_04_有远程")

    def test_截图远程详情(self, app):
        driver.region(0.03, 0.06, 0.94, 0.90, "remote_05_远程详情")

    def test_关闭并删除远程(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        subprocess.run(
            ["git", "remote", "remove", "origin"],
            cwd=app, capture_output=True,
        )
        print("假远程已删除")
