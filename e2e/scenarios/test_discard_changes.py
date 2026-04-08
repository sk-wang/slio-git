"""
E2E 场景: 丢弃变更 (Discard Changes)

覆盖:
  1. 修改文件 → git checkout 恢复
  2. 新建文件 → git clean 删除
  3. 暂存后取消暂存 → git reset
  4. slio-git 正确反映每步变化
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


class Test丢弃已修改文件:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_修改文件(self, app):
        add_unstaged_change(app, filename="src/main.py",
                           content='print("will be discarded")\n')
        driver.sleep(3)
        driver.window_screenshot("discard_01_有变更")

    def test_验证有变更(self, app):
        result = _git(app, "status", "--porcelain")
        assert "src/main.py" in result.stdout

    def test_git_checkout恢复(self, app):
        _git(app, "checkout", "src/main.py")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("discard_02_checkout恢复后")

    def test_验证恢复成功(self, app):
        result = _git(app, "status", "--porcelain")
        assert "src/main.py" not in result.stdout


class Test丢弃新建文件:
    def test_创建新文件(self, app):
        filepath = os.path.join(app, "will_be_cleaned.txt")
        with open(filepath, "w") as f:
            f.write("this file will be cleaned\n")
        driver.sleep(3)
        driver.window_screenshot("discard_03_有新文件")

    def test_验证未追踪(self, app):
        result = _git(app, "status", "--porcelain")
        assert "will_be_cleaned.txt" in result.stdout

    def test_git_clean删除(self, app):
        _git(app, "clean", "-fd")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("discard_04_clean后")

    def test_验证已删除(self, app):
        assert not os.path.exists(os.path.join(app, "will_be_cleaned.txt"))


class Test取消暂存:
    def test_修改并暂存(self, app):
        add_unstaged_change(app, filename="README.md",
                           content="# Discard Test\n")
        _git(app, "add", "README.md")
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("discard_05_已暂存")

    def test_验证已暂存(self, app):
        result = _git(app, "diff", "--cached", "--name-only")
        assert "README.md" in result.stdout

    def test_git_reset取消暂存(self, app):
        _git(app, "reset", "HEAD", "README.md")
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("discard_06_reset后")

    def test_验证回到未暂存(self, app):
        result = _git(app, "diff", "--cached", "--name-only")
        assert "README.md" not in result.stdout
        result2 = _git(app, "diff", "--name-only")
        assert "README.md" in result2.stdout

    def test_最终清理(self, app):
        _git(app, "checkout", ".")
        _git(app, "clean", "-fd")
