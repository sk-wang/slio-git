"""
E2E 场景: Detached HEAD 状态

覆盖:
  1. checkout 到一个 commit hash → 进入 detached HEAD
  2. slio-git 显示 "detached HEAD" 状态
  3. 在 detached HEAD 上创建提交
  4. 切回分支
"""

import os
import subprocess

import driver


def _git(repo, *args):
    return subprocess.run(["git"] + list(args), cwd=repo, capture_output=True, text=True)


class TestDetachedHEAD:
    def test_获取当前HEAD_hash(self, app):
        result = _git(app, "rev-parse", "--short", "HEAD")
        commit_hash = result.stdout.strip()
        with open(os.path.join(app, ".detach_hash"), "w") as f:
            f.write(commit_hash)
        print(f"HEAD hash: {commit_hash}")

    def test_checkout到commit_hash(self, app):
        with open(os.path.join(app, ".detach_hash")) as f:
            commit_hash = f.read().strip()
        result = _git(app, "checkout", commit_hash)
        assert result.returncode == 0
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("detached_01_进入detached")

    def test_验证detached状态(self, app):
        result = _git(app, "symbolic-ref", "HEAD")
        assert result.returncode != 0, "应处于 detached HEAD (symbolic-ref 应失败)"
        result2 = _git(app, "status")
        assert "HEAD detached" in result2.stdout or "detached" in result2.stdout.lower()
        print("已确认 detached HEAD 状态")

    def test_截图顶部状态栏(self, app):
        """顶部应显示 'detached HEAD' 而非分支名。"""
        driver.region(0.0, 0.0, 0.25, 0.06, "detached_02_顶部状态")

    def test_在detached上创建提交(self, app):
        filepath = os.path.join(app, "detached_commit.txt")
        with open(filepath, "w") as f:
            f.write("committed in detached HEAD\n")
        _git(app, "add", "detached_commit.txt")
        result = _git(app, "commit", "-m", "e2e: detached HEAD commit")
        assert result.returncode == 0
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("detached_03_detached提交")

    def test_切回main(self, app):
        _git(app, "checkout", "main")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("detached_04_回到main")

    def test_验证回到分支(self, app):
        result = _git(app, "symbolic-ref", "--short", "HEAD")
        assert result.stdout.strip() == "main"
        # detached 上的文件不应在 main 上
        assert not os.path.exists(os.path.join(app, "detached_commit.txt"))

    def test_清理(self, app):
        os.remove(os.path.join(app, ".detach_hash"))
