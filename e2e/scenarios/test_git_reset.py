"""
E2E 场景: Git Reset 操作

覆盖:
  1. soft reset — 保留变更在暂存区
  2. mixed reset — 保留变更在工作区
  3. hard reset — 丢弃所有变更
  4. slio-git 正确检测每种 reset 后的仓库状态
"""

import os
import subprocess

import driver


def _git(repo, *args):
    return subprocess.run(
        ["git"] + list(args),
        cwd=repo, capture_output=True, text=True,
    )


def _commit_file(repo, name, content, msg):
    filepath = os.path.join(repo, name)
    os.makedirs(os.path.dirname(filepath), exist_ok=True)
    with open(filepath, "w") as f:
        f.write(content)
    _git(repo, "add", name)
    _git(repo, "commit", "-m", msg)


class TestSoftReset:
    """git reset --soft HEAD~1: 回退提交但保留暂存。"""

    def test_创建提交(self, app):
        _commit_file(app, "reset_soft.txt", "soft reset test\n", "e2e: soft reset target")
        result = _git(app, "log", "--oneline", "-2")
        print(f"提交:\n{result.stdout.strip()}")

    def test_执行soft_reset(self, app):
        _git(app, "reset", "--soft", "HEAD~1")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("reset_01_soft后")

    def test_验证文件在暂存区(self, app):
        result = _git(app, "diff", "--cached", "--name-only")
        print(f"暂存: {result.stdout.strip()}")
        assert "reset_soft.txt" in result.stdout

    def test_验证提交已回退(self, app):
        result = _git(app, "log", "--oneline", "-1")
        assert "soft reset target" not in result.stdout

    def test_清理(self, app):
        _git(app, "checkout", ".")
        _git(app, "clean", "-fd")


class TestMixedReset:
    """git reset HEAD~1 (mixed): 回退提交，变更在工作区。"""

    def test_创建提交(self, app):
        _commit_file(app, "reset_mixed.txt", "mixed reset test\n", "e2e: mixed reset target")

    def test_执行mixed_reset(self, app):
        _git(app, "reset", "HEAD~1")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("reset_02_mixed后")

    def test_验证文件未暂存(self, app):
        result = _git(app, "diff", "--cached", "--name-only")
        assert "reset_mixed.txt" not in result.stdout, "mixed reset 后不应在暂存区"

    def test_验证文件在工作区(self, app):
        assert os.path.exists(os.path.join(app, "reset_mixed.txt"))

    def test_清理(self, app):
        _git(app, "checkout", ".")
        _git(app, "clean", "-fd")


class TestHardReset:
    """git reset --hard HEAD~1: 完全丢弃。"""

    def test_创建提交(self, app):
        _commit_file(app, "reset_hard.txt", "hard reset test\n", "e2e: hard reset target")
        assert os.path.exists(os.path.join(app, "reset_hard.txt"))

    def test_执行hard_reset(self, app):
        _git(app, "reset", "--hard", "HEAD~1")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("reset_03_hard后")

    def test_验证文件已删除(self, app):
        assert not os.path.exists(os.path.join(app, "reset_hard.txt"))

    def test_验证工作区干净(self, app):
        result = _git(app, "status", "--porcelain")
        assert result.stdout.strip() == "", f"不干净: {result.stdout}"
