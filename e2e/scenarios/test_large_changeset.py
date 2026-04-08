"""
E2E 场景: 大量文件变更

覆盖:
  1. 创建 20 个文件
  2. slio-git 显示所有文件
  3. 全部暂存
  4. 提交
  5. 验证变更列表滚动
"""

import os
import subprocess

import driver


def _git(repo, *args):
    return subprocess.run(
        ["git"] + list(args),
        cwd=repo, capture_output=True, text=True,
    )


class Test大量文件变更:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_创建20个文件(self, app):
        bulk_dir = os.path.join(app, "bulk")
        os.makedirs(bulk_dir, exist_ok=True)
        for i in range(1, 21):
            with open(os.path.join(bulk_dir, f"file_{i:02d}.txt"), "w") as f:
                f.write(f"Bulk file {i} content\nLine 2\nLine 3\n")
        driver.sleep(4)
        driver.window_screenshot("large_01_20个文件")

    def test_截图变更列表(self, app):
        path = driver.region(0.0, 0.08, 0.38, 0.85, "large_02_文件列表")
        assert os.path.exists(path)

    def test_滚动文件列表(self, app):
        """用键盘在左侧列表滚动。"""
        # 点击左侧列表区域聚焦
        driver.click_relative(0.15, 0.20)
        driver.sleep(0.3)
        for _ in range(15):
            driver.press("down")
            driver.sleep(0.1)
        driver.window_screenshot("large_03_滚动后")

    def test_全部暂存(self, app):
        _git(app, "add", ".")
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("large_04_全部暂存")

    def test_提交所有(self, app):
        result = _git(app, "commit", "-m", "e2e: bulk 20 files")
        assert result.returncode == 0
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("large_05_提交后")

    def test_验证提交了20个文件(self, app):
        result = _git(app, "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD")
        files = [f for f in result.stdout.strip().split("\n") if f]
        print(f"提交了 {len(files)} 个文件")
        assert len(files) == 20

    def test_验证工作区干净(self, app):
        result = _git(app, "status", "--porcelain")
        assert result.stdout.strip() == ""
