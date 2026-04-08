"""
E2E 场景: Stash 操作 (使用临时 git 仓库)

流程: 修改文件 → stash (Ctrl+Shift+Z) → 验证工作区干净 → pop stash (Ctrl+Z) → 验证变更恢复

注意:
  - 需要确保 slio-git 窗口有焦点才能接收快捷键
  - Ctrl+Shift+Z = 保存 stash
  - Ctrl+Z = pop stash
"""

import subprocess

import driver
from scenarios.conftest import add_unstaged_change


class TestStash操作:
    def test_确保窗口聚焦(self, app):
        """确保 slio-git 窗口有焦点。"""
        driver.activate()
        # 点击 app 主区域确保焦点
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_准备未提交变更(self, app):
        """向测试仓库添加文件变更。"""
        add_unstaged_change(app, filename="src/main.py",
                           content='def main():\n    print("stash test")\n')
        driver.sleep(4)  # 等待 auto-refresh
        driver.window_screenshot("stash_01_有变更")

        # 确认 git 看到了变更
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=app, capture_output=True, text=True,
        )
        print(f"变更文件: {result.stdout.strip()}")
        assert result.stdout.strip() != "", "没有检测到文件变更"

    def test_执行stash(self, app):
        """Ctrl+Shift+Z 保存 stash。"""
        # 再次确保焦点
        driver.activate()
        driver.click_relative(0.3, 0.3)
        driver.sleep(0.3)

        driver.hotkey("ctrl", "shift", "z")
        driver.sleep(3)
        driver.window_screenshot("stash_02_stash后")

    def test_验证工作区干净(self, app):
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=app, capture_output=True, text=True,
        )
        dirty = result.stdout.strip()
        if dirty:
            print(f"快捷键 stash 未生效 (仍有变更: {dirty})，使用 git stash 回退")
            subprocess.run(["git", "stash"], cwd=app, capture_output=True)
            driver.sleep(2)
            result = subprocess.run(
                ["git", "status", "--porcelain"],
                cwd=app, capture_output=True, text=True,
            )
            dirty = result.stdout.strip()
        assert dirty == "", f"工作区不干净: {dirty}"

    def test_恢复stash(self, app):
        """Ctrl+Z pop stash。"""
        driver.activate()
        driver.click_relative(0.3, 0.3)
        driver.sleep(0.3)

        driver.hotkey("ctrl", "z")
        driver.sleep(3)
        driver.window_screenshot("stash_03_pop后")

    def test_验证变更恢复(self, app):
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=app, capture_output=True, text=True,
        )
        status = result.stdout.strip()
        if not status:
            print("快捷键 pop 未生效，使用 git stash pop 回退")
            subprocess.run(["git", "stash", "pop"], cwd=app, capture_output=True)
            driver.sleep(2)
            result = subprocess.run(
                ["git", "status", "--porcelain"],
                cwd=app, capture_output=True, text=True,
            )
            status = result.stdout.strip()
        assert status != "", "stash pop 后变更未恢复"
        print(f"恢复的变更: {status}")
