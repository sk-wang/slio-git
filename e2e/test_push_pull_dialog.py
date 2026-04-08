"""
E2E: Push/Pull 对话框 — 工具栏 Push/Pull 分裂按钮

覆盖:
  - 点击 Push 按钮区域
  - 点击 Pull 按钮区域
  - 验证对话框弹出/关闭
"""
import driver


# 工具栏按钮相对坐标 (从右到左: Settings, Commit, Push, Pull, Refresh)
PULL_BTN = (0.835, 0.03)
PUSH_BTN = (0.88, 0.03)


class TestPull操作:
    def test_点击Pull按钮(self, app):
        """点击 Pull 按钮区域。"""
        driver.click_relative(*PULL_BTN)
        driver.sleep(2)
        driver.window_screenshot("pp_01_pull点击后")

    def test_ESC关闭(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("pp_02_pull关闭")


class TestPush操作:
    def test_点击Push按钮(self, app):
        """点击 Push 按钮区域。"""
        driver.click_relative(*PUSH_BTN)
        driver.sleep(2)
        driver.window_screenshot("pp_03_push点击后")

    def test_ESC关闭(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("pp_04_push关闭")


class TestPush快捷键:
    def test_Ctrl_Shift_K打开Push(self, app):
        """Ctrl+Shift+K 应打开 Push 对话框。"""
        driver.hotkey("ctrl", "shift", "k")
        driver.sleep(2)
        driver.window_screenshot("pp_05_ctrl_shift_k_push")

    def test_ESC关闭(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("pp_06_push关闭")
