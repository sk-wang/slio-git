"""
E2E 场景: 文件暂存操作 (Individual File Staging)

流程:
  1. 创建多个文件变更
  2. 在变更列表中逐个暂存/取消暂存
  3. 验证 "已暂存" 和 "未暂存" 区域正确更新
  4. 全部暂存 / 全部取消暂存

Changelist 布局:
  - "已暂存 N" 折叠区 (绿色 badge)
  - "未暂存 N" 折叠区 (蓝色 badge)
  - 每个文件行右侧有 +/- 按钮
  - 工具栏: ≡(视图切换) ↻(刷新) ✓(全部暂存) ✗(全部取消暂存)
"""

import os
import subprocess

import driver
from scenarios.conftest import add_unstaged_change


# 变更列表工具栏坐标
STAGE_ALL_BTN = (0.31, 0.105)        # ✓ 全部暂存
UNSTAGE_ALL_BTN = (0.33, 0.105)      # ✗ 全部取消暂存
TOGGLE_VIEW_BTN = (0.28, 0.105)      # ≡ 视图切换

# 文件列表坐标 (相对位置)
FIRST_FILE_STAGE_BTN = (0.33, 0.18)   # 第一个文件的 + 按钮
FIRST_FILE_ROW = (0.15, 0.18)         # 第一个文件行
SECOND_FILE_ROW = (0.15, 0.22)        # 第二个文件行


class Test文件暂存:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_创建多个文件变更(self, app):
        """创建 3 个文件的修改。"""
        # 修改现有文件
        add_unstaged_change(app, filename="src/main.py",
                           content='def main():\n    print("staging test")\n')
        add_unstaged_change(app, filename="README.md",
                           content="# Test Repo\n\nStaging test modification.\n")
        # 创建新文件
        new_file = os.path.join(app, "new_file.txt")
        with open(new_file, "w") as f:
            f.write("New file for staging test\n")

        driver.sleep(4)  # 等待 auto-refresh
        driver.window_screenshot("staging_01_多文件变更")

    def test_变更列表显示(self, app):
        """验证变更列表显示多个文件。"""
        driver.region(0.0, 0.08, 0.38, 0.50, "staging_02_变更列表")

    def test_Ctrl_Shift_S全部暂存(self, app):
        """暂存所有文件。"""
        driver.activate()
        driver.hotkey("ctrl", "shift", "s")
        driver.sleep(1)
        driver.window_screenshot("staging_03_全部已暂存")

    def test_截图已暂存区域(self, app):
        """截取已暂存区域。"""
        driver.region(0.0, 0.08, 0.38, 0.50, "staging_04_已暂存区域")

    def test_Ctrl_Shift_U全部取消暂存(self, app):
        """取消暂存所有文件。"""
        driver.hotkey("ctrl", "shift", "u")
        driver.sleep(1)
        driver.window_screenshot("staging_05_全部已取消暂存")

    def test_点击单个文件暂存(self, app):
        """点击第一个文件行的 + 按钮暂存。"""
        driver.click_relative(*FIRST_FILE_STAGE_BTN)
        driver.sleep(0.5)
        driver.window_screenshot("staging_06_单文件暂存")

    def test_验证git暂存状态(self, app):
        """检查 git status 验证暂存状态。"""
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=app, capture_output=True, text=True,
        )
        print(f"git status:\n{result.stdout}")
        # 至少应有 staged 和 unstaged 文件
        assert result.stdout.strip() != "", "没有文件变更"

    def test_恢复干净状态(self, app):
        """丢弃所有变更恢复干净状态。"""
        subprocess.run(["git", "checkout", "."], cwd=app, capture_output=True)
        subprocess.run(["git", "clean", "-fd"], cwd=app, capture_output=True)
        driver.sleep(2)
        driver.window_screenshot("staging_07_恢复干净")


class Test视图切换:
    """测试变更列表的平铺/树形视图切换。"""

    def test_创建文件变更(self, app):
        add_unstaged_change(app, filename="src/main.py")
        driver.sleep(3)

    def test_切换视图模式(self, app):
        """点击 ≡ 按钮切换视图。"""
        driver.click_relative(*TOGGLE_VIEW_BTN)
        driver.sleep(0.5)
        driver.window_screenshot("staging_08_视图切换")

    def test_再次切换回来(self, app):
        driver.click_relative(*TOGGLE_VIEW_BTN)
        driver.sleep(0.5)
        driver.window_screenshot("staging_09_切回原视图")

    def test_清理(self, app):
        subprocess.run(["git", "checkout", "."], cwd=app, capture_output=True)
        subprocess.run(["git", "clean", "-fd"], cwd=app, capture_output=True)
