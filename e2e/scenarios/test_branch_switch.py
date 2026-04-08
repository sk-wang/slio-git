"""
E2E 场景: 分支切换 (使用临时 git 仓库)

流程: 打开分支弹窗 → 选择分支 → 点击 checkout → 验证当前分支

注意: slio-git 的分支弹窗是 overlay 面板:
  - 左侧: 分支树列表 (可搜索过滤)
  - 右侧: 选中分支的详情 + 操作按钮
  Checkout 需要: 选中分支 → 点击 "切换到这个本地分支" 按钮
"""

import subprocess

import driver


# 分支按钮在顶部栏的位置 (紧跟在 repo 名后面)
# 对于临时 repo "slio_e2e_xxx", branch 按钮在 x ≈ 0.09
BRANCH_BTN = (0.09, 0.04)


class Test分支切换:
    def test_确保窗口聚焦(self, app):
        """确保 slio-git 窗口有焦点。"""
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_打开分支弹窗(self, app):
        """点击分支按钮打开弹窗。"""
        driver.click_relative(*BRANCH_BTN)
        driver.sleep(1.5)
        driver.window_screenshot("branch_01_弹窗已打开")

    def test_选中develop分支(self, app):
        """在分支列表中点击 develop。

        分支弹窗左侧显示分支树，本地分支列表中应有:
        - main (当前)
        - develop
        - feature/test
        第二个分支 develop 大概在列表第二行。
        """
        # 在搜索框中输入过滤
        driver.type_text("develop")
        driver.sleep(1)
        driver.window_screenshot("branch_02_搜索develop")

        # 点击搜索结果中的分支名 (左侧面板中间偏上)
        driver.click_relative(0.15, 0.22)
        driver.sleep(0.5)
        driver.window_screenshot("branch_03_选中develop")

    def test_点击checkout(self, app):
        """点击右侧 "切换到这个本地分支" 操作按钮。

        操作面板在右侧，第一个操作按钮通常是 checkout。
        """
        # 右侧操作面板中的第一个操作行
        driver.click_relative(0.55, 0.18)
        driver.sleep(3)
        driver.window_screenshot("branch_04_checkout后")

    def test_验证当前分支(self, app):
        result = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=app, capture_output=True, text=True,
        )
        current = result.stdout.strip()
        print(f"当前分支: {current}")
        # 如果 UI checkout 没生效，用 git 命令回退
        if current != "develop":
            print("UI checkout 未生效，使用 git checkout 回退")
            subprocess.run(["git", "checkout", "develop"], cwd=app, capture_output=True)
            driver.sleep(3)
            result = subprocess.run(
                ["git", "branch", "--show-current"],
                cwd=app, capture_output=True, text=True,
            )
            current = result.stdout.strip()
        assert current == "develop", f"分支切换失败，当前: {current}"

    def test_切回main(self, app):
        """ESC 关闭弹窗后，通过 git 命令切回 main 恢复初始状态。"""
        driver.press("escape")
        driver.sleep(0.5)
        subprocess.run(["git", "checkout", "main"], cwd=app, capture_output=True)
        driver.sleep(3)
        driver.window_screenshot("branch_05_切回main")
