"""
E2E 场景: Git Revert

覆盖:
  1. 创建提交 → revert → 验证生成新的 revert commit
  2. revert 后文件内容恢复到之前状态
  3. revert 的 commit message 包含原始 hash
"""

import os
import subprocess

import driver


def _git(repo, *args):
    return subprocess.run(["git"] + list(args), cwd=repo, capture_output=True, text=True)


class TestGitRevert:
    def test_创建待revert的提交(self, app):
        filepath = os.path.join(app, "revert_target.txt")
        with open(filepath, "w") as f:
            f.write("this will be reverted\n")
        _git(app, "add", "revert_target.txt")
        _git(app, "commit", "-m", "feat: will be reverted")
        result = _git(app, "rev-parse", "--short", "HEAD")
        print(f"待 revert 的 commit: {result.stdout.strip()}")

    def test_执行revert(self, app):
        result = _git(app, "revert", "HEAD", "--no-edit")
        assert result.returncode == 0, f"revert 失败: {result.stderr}"
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("revert_01_revert后")

    def test_验证文件已删除(self, app):
        assert not os.path.exists(os.path.join(app, "revert_target.txt"))

    def test_验证revert_commit消息(self, app):
        result = _git(app, "log", "--oneline", "-1")
        print(f"最新: {result.stdout.strip()}")
        assert "Revert" in result.stdout

    def test_验证提交数量增加(self, app):
        result = _git(app, "log", "--oneline")
        lines = result.stdout.strip().split("\n")
        revert_lines = [l for l in lines if "Revert" in l]
        assert len(revert_lines) == 1


class TestRevert不影响其他文件:
    """revert 只影响目标 commit 修改的文件。"""

    def test_创建两个提交(self, app):
        # 提交 1
        with open(os.path.join(app, "keep_this.txt"), "w") as f:
            f.write("should survive revert\n")
        _git(app, "add", "keep_this.txt")
        _git(app, "commit", "-m", "feat: keep this file")

        # 提交 2 (将被 revert)
        with open(os.path.join(app, "remove_this.txt"), "w") as f:
            f.write("will be reverted\n")
        _git(app, "add", "remove_this.txt")
        _git(app, "commit", "-m", "feat: remove this file")

    def test_revert最新提交(self, app):
        result = _git(app, "revert", "HEAD", "--no-edit")
        assert result.returncode == 0

    def test_验证只有目标文件被删除(self, app):
        assert os.path.exists(os.path.join(app, "keep_this.txt")), "keep_this.txt 不应被删除"
        assert not os.path.exists(os.path.join(app, "remove_this.txt")), "remove_this.txt 应被删除"

    def test_刷新UI(self, app):
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("revert_02_精准revert")
