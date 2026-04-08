"""
E2E 场景: 多分支工作流

覆盖:
  1. 创建多个分支并在每个上提交
  2. 切换分支时验证工作区变化
  3. 删除分支
  4. 分支间比较 (通过 log)
"""

import os
import subprocess

import driver


def _git(repo, *args):
    return subprocess.run(
        ["git"] + list(args),
        cwd=repo, capture_output=True, text=True,
    )


class Test多分支创建与切换:
    def test_确保在main(self, app):
        _git(app, "checkout", "main")
        driver.activate()
        driver.sleep(0.5)

    def test_创建三个feature分支(self, app):
        for i in range(1, 4):
            _git(app, "checkout", "-b", f"feature-{i}", "main")
            filepath = os.path.join(app, f"feature_{i}.txt")
            with open(filepath, "w") as f:
                f.write(f"Feature {i} content\n")
            _git(app, "add", f"feature_{i}.txt")
            _git(app, "commit", "-m", f"feat: feature {i}")
            _git(app, "checkout", "main")

        result = _git(app, "branch")
        branches = [b.strip().lstrip("* ") for b in result.stdout.strip().split("\n")]
        print(f"分支列表: {branches}")
        assert all(f"feature-{i}" in branches for i in range(1, 4))

    def test_切换到feature_1(self, app):
        _git(app, "checkout", "feature-1")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("multibranch_01_feature1")
        assert os.path.exists(os.path.join(app, "feature_1.txt"))
        assert not os.path.exists(os.path.join(app, "feature_2.txt"))

    def test_切换到feature_2(self, app):
        _git(app, "checkout", "feature-2")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("multibranch_02_feature2")
        assert os.path.exists(os.path.join(app, "feature_2.txt"))
        assert not os.path.exists(os.path.join(app, "feature_1.txt"))

    def test_切换到feature_3(self, app):
        _git(app, "checkout", "feature-3")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("multibranch_03_feature3")
        assert os.path.exists(os.path.join(app, "feature_3.txt"))

    def test_切回main(self, app):
        _git(app, "checkout", "main")
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        # main 上不应有任何 feature 文件
        for i in range(1, 4):
            assert not os.path.exists(os.path.join(app, f"feature_{i}.txt"))

    def test_合并所有feature到main(self, app):
        for i in range(1, 4):
            result = _git(app, "merge", f"feature-{i}")
            assert result.returncode == 0, f"merge feature-{i} 失败: {result.stderr}"
        # 现在 main 应有所有文件
        for i in range(1, 4):
            assert os.path.exists(os.path.join(app, f"feature_{i}.txt"))
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("multibranch_04_全部合并")

    def test_查看合并后历史(self, app):
        result = _git(app, "log", "--oneline", "-6")
        print(f"历史:\n{result.stdout.strip()}")
        for i in range(1, 4):
            assert f"feature {i}" in result.stdout

    def test_删除feature分支(self, app):
        for i in range(1, 4):
            _git(app, "branch", "-d", f"feature-{i}")
        result = _git(app, "branch")
        print(f"剩余分支: {result.stdout.strip()}")
        assert "feature-" not in result.stdout
        driver.sleep(1)
