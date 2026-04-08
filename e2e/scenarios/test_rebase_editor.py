"""
E2E 场景: 交互式变基编辑器 (Rebase Editor)

流程:
  1. 在临时仓库创建多个提交 (用于 rebase)
  2. 打开 Rebase 编辑器 (通过导航栏 Rebase 按钮)
  3. 输入 onto 分支
  4. 截图验证 rebase 编辑器 UI
  5. 通过 git 命令验证 rebase 功能

Rebase 编辑器布局:
  - 顶部: 标题 + 状态
  - 中间: 目标分支输入 / Todo 列表
  - 底部: [开始变基] / [继续] [跳过] [中止]
"""

import os
import subprocess

import driver


def _create_commits_for_rebase(repo_path: str):
    """创建多个提交用于 rebase 测试。"""
    def git(*args):
        subprocess.run(
            ["git"] + list(args),
            cwd=repo_path, capture_output=True, check=True,
        )

    # 确保在 main 分支
    git("checkout", "main")

    # 创建 rebase-test 分支
    git("checkout", "-b", "rebase-test")

    # 创建 3 个提交
    for i in range(1, 4):
        filepath = os.path.join(repo_path, f"rebase_file_{i}.txt")
        with open(filepath, "w") as f:
            f.write(f"Content for rebase commit {i}\n")
        git("add", f"rebase_file_{i}.txt")
        git("commit", "-m", f"rebase commit {i}")

    print(f"创建了 3 个提交在 rebase-test 分支")


def _cleanup_rebase(repo_path: str):
    """清理 rebase 状态。"""
    # 如果正在 rebase，中止它
    subprocess.run(["git", "rebase", "--abort"], cwd=repo_path, capture_output=True)
    # 切回 main
    subprocess.run(["git", "checkout", "main"], cwd=repo_path, capture_output=True)
    # 删除测试分支
    subprocess.run(
        ["git", "branch", "-D", "rebase-test"],
        cwd=repo_path, capture_output=True,
    )


# === Rebase 编辑器 UI 坐标 ===
# 导航栏底部的 Rebase 按钮
NAV_REBASE = (0.012, 0.94)

# Rebase 编辑器内
ONTO_INPUT = (0.35, 0.25)            # 目标分支输入框
START_REBASE_BTN = (0.50, 0.92)      # "开始变基" 按钮
ABORT_BTN = (0.75, 0.92)             # "中止" 按钮
CLOSE_BTN = (0.97, 0.07)             # "关闭" 按钮

# Todo 列表行 (交互式 rebase 时)
TODO_ROW_1_ACTION = (0.08, 0.45)     # 第一行的 action 按钮
TODO_ROW_2_ACTION = (0.08, 0.50)     # 第二行的 action 按钮
TODO_ROW_1_MSG = (0.40, 0.45)        # 第一行的消息文本
MOVE_UP_BTN = (0.10, 0.92)           # "上移" 按钮
MOVE_DOWN_BTN = (0.15, 0.92)         # "下移" 按钮


class TestRebase编辑器_打开关闭:
    """测试 Rebase 编辑器的基本打开和关闭。"""

    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_打开Rebase面板(self, app):
        """通过导航栏底部 Rebase 按钮打开编辑器。"""
        driver.click_relative(*NAV_REBASE)
        driver.sleep(1.5)
        driver.window_screenshot("rebase_01_编辑器打开")

    def test_截图编辑器布局(self, app):
        """截取 rebase 编辑器全貌。"""
        driver.region(0.03, 0.06, 0.94, 0.90, "rebase_02_编辑器布局")

    def test_输入onto分支(self, app):
        """在 onto 分支输入框中输入分支名。"""
        driver.click_relative(*ONTO_INPUT)
        driver.sleep(0.3)
        driver.type_text("main", interval=0.08)
        driver.sleep(0.5)
        driver.window_screenshot("rebase_03_输入onto")

    def test_清空输入并关闭(self, app):
        """清空输入，关闭 Rebase 面板。"""
        driver.hotkey("command", "a")
        driver.sleep(0.1)
        driver.press("backspace")
        driver.sleep(0.3)
        driver.press("escape")
        driver.sleep(1)
        driver.window_screenshot("rebase_04_关闭")


class TestRebase编辑器_交互式:
    """测试交互式 rebase 流程 (通过 git 创建提交 + UI 查看)。"""

    def test_准备多提交分支(self, app):
        """创建 rebase-test 分支 + 3 个提交。"""
        _create_commits_for_rebase(app)
        driver.sleep(3)  # 等待 auto-refresh
        driver.window_screenshot("rebase_05_准备完成")

    def test_切换到rebase_test分支(self, app):
        """通过 git 切换到 rebase-test 分支。"""
        subprocess.run(
            ["git", "checkout", "rebase-test"],
            cwd=app, capture_output=True, check=True,
        )
        driver.sleep(3)
        driver.activate()
        driver.hotkey("ctrl", "r")  # 刷新
        driver.sleep(2)
        driver.window_screenshot("rebase_06_在rebase_test分支")

    def test_打开Rebase编辑器(self, app):
        """打开 Rebase 编辑器。"""
        driver.click_relative(*NAV_REBASE)
        driver.sleep(1.5)
        driver.window_screenshot("rebase_07_编辑器_有提交")

    def test_截图todo列表(self, app):
        """截取 rebase 编辑器中的 todo 列表区域。"""
        driver.region(0.03, 0.30, 0.94, 0.55, "rebase_08_todo列表")

    def test_关闭编辑器(self, app):
        """关闭 Rebase 编辑器。"""
        driver.press("escape")
        driver.sleep(1)
        driver.window_screenshot("rebase_09_关闭")

    def test_git验证rebase能力(self, app):
        """通过 git 命令验证 rebase 能力 (不通过 UI 执行)。"""
        # 验证分支上有 3 个提交 (超出 initial commit)
        result = subprocess.run(
            ["git", "log", "--oneline"],
            cwd=app, capture_output=True, text=True,
        )
        commits = result.stdout.strip().split("\n")
        print(f"当前分支提交数: {len(commits)}")
        print(f"提交列表:\n{result.stdout.strip()}")

        # 至少应该有 initial + 3 个 rebase commit = 4
        rebase_commits = [c for c in commits if "rebase commit" in c]
        assert len(rebase_commits) == 3, \
            f"预期 3 个 rebase 提交，实际: {len(rebase_commits)}"

    def test_git执行rebase(self, app):
        """用 git 命令执行非交互式 rebase 验证功能正常。"""
        # 确保工作区干净
        subprocess.run(["git", "stash", "--include-untracked"], cwd=app, capture_output=True)

        # 在 rebase-test 上 rebase onto main (此时应该是 fast-forward)
        result = subprocess.run(
            ["git", "rebase", "main"],
            cwd=app, capture_output=True, text=True,
        )
        print(f"rebase 结果: {result.stdout.strip()}")
        assert result.returncode == 0, f"rebase 失败: {result.stderr}"
        driver.sleep(2)
        driver.hotkey("ctrl", "r")  # 刷新
        driver.sleep(2)
        driver.window_screenshot("rebase_10_rebase完成")

    def test_清理(self, app):
        """清理测试分支。"""
        _cleanup_rebase(app)
        driver.sleep(2)
        driver.window_screenshot("rebase_11_清理完成")


class TestRebase编辑器_中止:
    """测试 rebase 中止流程。"""

    def test_准备并制造rebase冲突(self, app):
        """创建一个会冲突的 rebase 场景。"""
        def git(*args):
            subprocess.run(
                ["git"] + list(args),
                cwd=app, capture_output=True, check=True,
            )

        # 确保干净状态
        subprocess.run(["git", "rebase", "--abort"], cwd=app, capture_output=True)
        subprocess.run(["git", "checkout", "main"], cwd=app, capture_output=True)
        subprocess.run(["git", "branch", "-D", "rebase-conflict"], cwd=app, capture_output=True)

        readme = os.path.join(app, "README.md")

        # 在 main 上修改
        with open(readme, "w") as f:
            f.write("# Test Repository\n\nmain side for rebase conflict\n")
        git("add", "README.md")
        git("commit", "-m", "main: prep for rebase conflict")

        # 创建分支并做不同修改
        git("checkout", "-b", "rebase-conflict", "HEAD~1")
        with open(readme, "w") as f:
            f.write("# Test Repository\n\nrebase-conflict side change\n")
        git("add", "README.md")
        git("commit", "-m", "conflict: different change")

        print("已准备 rebase 冲突场景")
        driver.sleep(2)
        driver.window_screenshot("rebase_12_准备冲突")

    def test_执行会冲突的rebase(self, app):
        """git rebase main → 产生冲突。"""
        result = subprocess.run(
            ["git", "rebase", "main"],
            cwd=app, capture_output=True, text=True,
        )
        print(f"rebase stdout: {result.stdout.strip()}")
        print(f"rebase stderr: {result.stderr.strip()}")
        # 应该产生冲突
        assert result.returncode != 0, "预期 rebase 冲突但成功了"
        driver.sleep(3)
        driver.activate()
        driver.hotkey("ctrl", "r")  # 刷新
        driver.sleep(3)
        driver.window_screenshot("rebase_13_rebase冲突")

    def test_截图冲突状态(self, app):
        """截取 rebase 冲突状态下的 UI。"""
        driver.window_screenshot("rebase_14_冲突状态UI")

    def test_中止rebase(self, app):
        """通过 git 命令中止 rebase。"""
        result = subprocess.run(
            ["git", "rebase", "--abort"],
            cwd=app, capture_output=True, text=True,
        )
        assert result.returncode == 0, f"abort 失败: {result.stderr}"
        driver.sleep(3)
        driver.hotkey("ctrl", "r")  # 刷新
        driver.sleep(2)
        driver.window_screenshot("rebase_15_中止后")

    def test_验证中止成功(self, app):
        """验证 rebase 已中止，仓库状态正常。"""
        # 不应该再处于 rebase 状态
        rebase_dir = os.path.join(app, ".git", "rebase-merge")
        rebase_apply = os.path.join(app, ".git", "rebase-apply")
        assert not os.path.exists(rebase_dir), "rebase-merge 目录仍存在"
        assert not os.path.exists(rebase_apply), "rebase-apply 目录仍存在"
        print("rebase 已成功中止")

    def test_清理(self, app):
        """清理测试分支。"""
        subprocess.run(["git", "checkout", "main"], cwd=app, capture_output=True)
        subprocess.run(
            ["git", "branch", "-D", "rebase-conflict"],
            cwd=app, capture_output=True,
        )
        driver.sleep(1)
        driver.window_screenshot("rebase_16_清理完成")
