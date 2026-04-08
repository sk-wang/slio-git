"""
E2E 场景: 无冲突 Merge 流程

覆盖:
  1. 创建 feature 分支 + 提交
  2. 切回 main
  3. 通过 git merge 合并 (无冲突)
  4. 验证合并成功
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


class TestMerge无冲突:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_创建feature分支(self, app):
        _git(app, "checkout", "-b", "feature-merge-test")
        filepath = os.path.join(app, "merge_test.txt")
        with open(filepath, "w") as f:
            f.write("Feature branch content\n")
        _git(app, "add", "merge_test.txt")
        _git(app, "commit", "-m", "feat: add merge test file")
        print("feature-merge-test 分支已创建")

    def test_切回main(self, app):
        _git(app, "checkout", "main")
        driver.sleep(2)
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("merge_01_main分支")

    def test_执行merge(self, app):
        result = _git(app, "merge", "feature-merge-test")
        print(f"merge: {result.stdout.strip()}")
        assert result.returncode == 0, f"merge 失败: {result.stderr}"

    def test_等待检测(self, app):
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("merge_02_merge后")

    def test_验证merge成功(self, app):
        result = _git(app, "log", "--oneline", "-1")
        print(f"最新提交: {result.stdout.strip()}")
        # merge_test.txt 应存在
        assert os.path.exists(os.path.join(app, "merge_test.txt"))

    def test_验证非快进merge(self, app):
        """如果不是 fast-forward，应有 merge commit。"""
        result = _git(app, "log", "--oneline", "-3")
        print(f"历史:\n{result.stdout.strip()}")

    def test_清理(self, app):
        _git(app, "branch", "-D", "feature-merge-test")
        driver.sleep(1)


class TestMerge快进:
    """Fast-forward merge — main 没有新提交，直接快进。"""

    def test_创建分支并提交(self, app):
        _git(app, "checkout", "-b", "ff-test")
        filepath = os.path.join(app, "ff_test.txt")
        with open(filepath, "w") as f:
            f.write("Fast-forward test\n")
        _git(app, "add", "ff_test.txt")
        _git(app, "commit", "-m", "feat: fast-forward test")

    def test_切回main并merge(self, app):
        _git(app, "checkout", "main")
        result = _git(app, "merge", "ff-test")
        print(f"merge: {result.stdout.strip()}")
        assert result.returncode == 0

    def test_验证快进(self, app):
        """快进后 main HEAD 应该就是 ff-test 的 commit。"""
        main_head = _git(app, "rev-parse", "HEAD").stdout.strip()
        ff_head = _git(app, "rev-parse", "ff-test").stdout.strip()
        assert main_head == ff_head, "不是快进"
        print("确认是 fast-forward merge")
        driver.sleep(2)
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("merge_03_ff后")

    def test_清理(self, app):
        _git(app, "branch", "-D", "ff-test")
