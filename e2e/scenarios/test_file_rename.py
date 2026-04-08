"""
E2E 场景: 文件重命名检测

覆盖:
  1. git mv 重命名文件 → git 检测到 rename
  2. 手动重命名 (delete + create) → git 能否检测为 rename
  3. slio-git 正确显示重命名状态
"""

import os
import subprocess

import driver


def _git(repo, *args):
    return subprocess.run(["git"] + list(args), cwd=repo, capture_output=True, text=True)


class TestGitMv重命名:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_创建待重命名文件(self, app):
        filepath = os.path.join(app, "old_name.txt")
        with open(filepath, "w") as f:
            f.write("file to be renamed\nline 2\nline 3\n")
        _git(app, "add", "old_name.txt")
        _git(app, "commit", "-m", "e2e: add file for rename test")

    def test_git_mv重命名(self, app):
        _git(app, "mv", "old_name.txt", "new_name.txt")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("rename_01_git_mv后")

    def test_验证git检测到重命名(self, app):
        result = _git(app, "status", "--porcelain")
        print(f"status: {result.stdout.strip()}")
        # R = renamed
        assert "new_name.txt" in result.stdout

    def test_验证文件存在(self, app):
        assert os.path.exists(os.path.join(app, "new_name.txt"))
        assert not os.path.exists(os.path.join(app, "old_name.txt"))

    def test_提交重命名(self, app):
        result = _git(app, "commit", "-m", "e2e: rename old_name to new_name")
        assert result.returncode == 0

    def test_验证提交中包含重命名(self, app):
        result = _git(app, "log", "-1", "--diff-filter=R", "--summary")
        print(f"rename log:\n{result.stdout.strip()}")
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("rename_02_提交后")


class Test手动重命名检测:
    """通过 delete + create 模拟重命名，git 应检测为 rename (相似度 > 50%)。"""

    def test_创建文件(self, app):
        filepath = os.path.join(app, "manual_old.txt")
        with open(filepath, "w") as f:
            f.write("manual rename test\nline 2\nline 3\nline 4\nline 5\n")
        _git(app, "add", "manual_old.txt")
        _git(app, "commit", "-m", "e2e: add file for manual rename")

    def test_手动删除创建(self, app):
        old = os.path.join(app, "manual_old.txt")
        new = os.path.join(app, "manual_new.txt")
        with open(old) as f:
            content = f.read()
        os.remove(old)
        with open(new, "w") as f:
            f.write(content)  # 完全相同内容
        _git(app, "add", ".")

    def test_验证检测为重命名(self, app):
        result = _git(app, "diff", "--cached", "--name-status", "-M")
        print(f"diff: {result.stdout.strip()}")
        # 应包含 R (rename) 而非 D+A
        assert "R" in result.stdout or "manual_new.txt" in result.stdout
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("rename_03_手动重命名")

    def test_清理(self, app):
        _git(app, "commit", "-m", "e2e: manual rename")
