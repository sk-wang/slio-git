"""
E2E 场景: Commit Amend (修正提交)

覆盖:
  1. 创建提交
  2. 修改文件后 amend 到上一个提交
  3. 验证提交消息和文件内容
  4. 验证 commit hash 已改变
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


class TestCommitAmend:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_创建初始提交(self, app):
        filepath = os.path.join(app, "amend_test.txt")
        with open(filepath, "w") as f:
            f.write("initial content\n")
        _git(app, "add", "amend_test.txt")
        _git(app, "commit", "-m", "e2e: initial amend target")

        result = _git(app, "rev-parse", "HEAD")
        original_hash = result.stdout.strip()
        with open(os.path.join(app, ".amend_hash"), "w") as f:
            f.write(original_hash)
        print(f"原始 commit: {original_hash[:8]}")

    def test_修改文件(self, app):
        filepath = os.path.join(app, "amend_test.txt")
        with open(filepath, "w") as f:
            f.write("initial content\namended line\n")
        _git(app, "add", "amend_test.txt")
        driver.sleep(2)
        driver.window_screenshot("amend_01_修改后暂存")

    def test_执行amend(self, app):
        result = _git(app, "commit", "--amend", "-m", "e2e: amended commit message")
        assert result.returncode == 0, f"amend 失败: {result.stderr}"
        print(f"amend: {result.stdout.strip()}")

    def test_等待检测(self, app):
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("amend_02_amend后")

    def test_验证消息已更新(self, app):
        result = _git(app, "log", "--oneline", "-1")
        print(f"最新提交: {result.stdout.strip()}")
        assert "amended commit message" in result.stdout

    def test_验证hash已改变(self, app):
        with open(os.path.join(app, ".amend_hash")) as f:
            original = f.read().strip()
        current = _git(app, "rev-parse", "HEAD").stdout.strip()
        assert original != current, "amend 应改变 commit hash"
        print(f"原 hash: {original[:8]} → 新 hash: {current[:8]}")

    def test_验证文件内容正确(self, app):
        with open(os.path.join(app, "amend_test.txt")) as f:
            content = f.read()
        assert "amend" in content.lower()

    def test_验证只有一个提交(self, app):
        """amend 不应增加提交数量。"""
        result = _git(app, "log", "--oneline")
        commits = result.stdout.strip().split("\n")
        amend_commits = [c for c in commits if "amend" in c]
        assert len(amend_commits) == 1, f"应只有 1 个 amend 提交，实际: {amend_commits}"

    def test_清理(self, app):
        os.remove(os.path.join(app, ".amend_hash"))
