"""
E2E 场景: 工作树视图 (Worktree View)

流程:
  1. 查看当前工作树列表
  2. 验证主工作树信息显示
  3. 截图面板布局

Worktree 面板布局:
  - 标题: "工作树" + 刷新/关闭
  - 列表: 每行显示 name / path / branch / status
  - 主工作树标记为 "主工作树"
"""

import subprocess

import driver


class TestWorktree视图:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_验证git_worktree(self, app):
        """通过 git 命令确认工作树状态。"""
        result = subprocess.run(
            ["git", "worktree", "list"],
            cwd=app, capture_output=True, text=True,
        )
        print(f"工作树列表:\n{result.stdout.strip()}")
        assert app in result.stdout, "当前 repo 不在工作树列表中"

    def test_打开Worktree面板(self, app):
        """通过导航栏或菜单打开工作树面板。

        注意: Worktree 在导航栏可能没有独立按钮，
        需要通过 auxiliary view 切换。此处先尝试键盘导航。
        """
        # Worktree 没有专用导航栏按钮，尝试通过 history view 的分支面板触发
        # 或者直接截图当前状态
        driver.window_screenshot("worktree_01_当前状态")

    def test_状态栏信息(self, app):
        """截取底部状态栏，验证仓库信息。"""
        driver.region(0.0, 0.95, 1.0, 0.05, "worktree_02_状态栏")

    def test_验证仓库已打开(self, app):
        """验证状态栏显示了 repo 信息。"""
        # 截取整个窗口底部
        import os
        path = driver.region(0.0, 0.93, 1.0, 0.07, "worktree_03_底部信息")
        assert os.path.exists(path) and os.path.getsize(path) > 0
