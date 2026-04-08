"""
E2E 场景: 部分暂存 + 提交

覆盖:
  1. 修改多个文件
  2. 只暂存部分文件
  3. 提交 → 只有暂存文件被提交
  4. 未暂存文件仍在工作区
"""

import os
import subprocess

import driver
from scenarios.conftest import add_unstaged_change


def _git(repo, *args):
    return subprocess.run(
        ["git"] + list(args),
        cwd=repo, capture_output=True, text=True,
    )


class Test部分暂存提交:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_修改两个文件(self, app):
        add_unstaged_change(app, filename="src/main.py",
                           content='def main():\n    print("partial commit A")\n')
        add_unstaged_change(app, filename="README.md",
                           content="# Test Repo\n\nPartial commit B\n")
        driver.sleep(3)
        driver.window_screenshot("partial_01_两个文件变更")

    def test_只暂存一个文件(self, app):
        """用 git 命令只暂存 src/main.py，确保 README.md 未暂存。"""
        _git(app, "reset", "HEAD", "README.md")  # 确保 README.md 未暂存
        _git(app, "add", "src/main.py")
        driver.sleep(2)
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("partial_02_部分暂存")

    def test_验证暂存状态(self, app):
        result = _git(app, "diff", "--cached", "--name-only")
        staged = result.stdout.strip().split("\n") if result.stdout.strip() else []
        result2 = _git(app, "diff", "--name-only")
        unstaged = result2.stdout.strip().split("\n") if result2.stdout.strip() else []
        print(f"暂存: {staged}, 未暂存: {unstaged}")
        assert "src/main.py" in staged, "src/main.py 应已暂存"
        assert "README.md" in unstaged, "README.md 应未暂存"

    def test_提交暂存文件(self, app):
        result = _git(app, "commit", "-m", "e2e: partial commit (only main.py)")
        assert result.returncode == 0, f"提交失败: {result.stderr}"
        print("部分提交成功")

    def test_验证只提交了暂存文件(self, app):
        # 最新提交应只包含 src/main.py
        result = _git(app, "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD")
        committed_files = result.stdout.strip().split("\n")
        print(f"提交的文件: {committed_files}")
        assert "src/main.py" in committed_files
        assert "README.md" not in committed_files

    def test_验证未暂存文件仍在(self, app):
        result = _git(app, "status", "--porcelain")
        print(f"剩余变更: {result.stdout.strip()}")
        assert "README.md" in result.stdout

    def test_刷新slio_git(self, app):
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("partial_03_提交后仍有变更")

    def test_清理(self, app):
        _git(app, "checkout", ".")
        _git(app, "clean", "-fd")
