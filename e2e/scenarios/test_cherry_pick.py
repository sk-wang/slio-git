"""
E2E 场景: Cherry-pick

覆盖:
  1. 在 feature 分支创建独立提交
  2. 切回 main
  3. 执行 cherry-pick
  4. 验证提交被应用到 main
  5. slio-git 检测到变更
"""

import os
import subprocess

import driver


def _git(repo, *args):
    return subprocess.run(
        ["git"] + list(args),
        cwd=repo, capture_output=True, text=True,
    )


class TestCherryPick:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_创建feature分支和提交(self, app):
        _git(app, "checkout", "-b", "cherry-pick-source")
        filepath = os.path.join(app, "cherry.txt")
        with open(filepath, "w") as f:
            f.write("cherry-pick test content\n")
        _git(app, "add", "cherry.txt")
        _git(app, "commit", "-m", "feat: cherry content")

        # 记录 commit hash
        result = _git(app, "rev-parse", "HEAD")
        cherry_commit = result.stdout.strip()
        print(f"cherry commit: {cherry_commit[:8]}")

        # 保存到文件供后续步骤使用
        with open(os.path.join(app, ".cherry_hash"), "w") as f:
            f.write(cherry_commit)

    def test_切回main(self, app):
        _git(app, "checkout", "main")
        driver.sleep(2)
        driver.hotkey("ctrl", "r")
        driver.sleep(2)

    def test_执行cherry_pick(self, app):
        with open(os.path.join(app, ".cherry_hash")) as f:
            cherry_commit = f.read().strip()

        result = _git(app, "cherry-pick", cherry_commit)
        print(f"cherry-pick: {result.stdout.strip()}")
        assert result.returncode == 0, f"cherry-pick 失败: {result.stderr}"

    def test_等待检测(self, app):
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("cherry_01_pick后")

    def test_验证cherry_pick成功(self, app):
        # cherry.txt 应存在于 main
        assert os.path.exists(os.path.join(app, "cherry.txt"))

        result = _git(app, "log", "--oneline", "-1")
        print(f"最新提交: {result.stdout.strip()}")
        assert "cherry content" in result.stdout

    def test_验证不是同一个commit(self, app):
        """cherry-pick 创建新 commit，hash 应不同。"""
        with open(os.path.join(app, ".cherry_hash")) as f:
            original = f.read().strip()
        current = _git(app, "rev-parse", "HEAD").stdout.strip()
        assert original != current, "cherry-pick 应创建新 commit"
        print(f"原 commit: {original[:8]}, 新 commit: {current[:8]}")

    def test_清理(self, app):
        _git(app, "branch", "-D", "cherry-pick-source")
        os.remove(os.path.join(app, ".cherry_hash"))
        driver.sleep(1)
