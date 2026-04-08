"""
E2E: 提交对话框高级功能 — amend 模式、文件选择、提交信息历史

覆盖:
  - 打开提交对话框
  - 检查 amend 模式切换 (Alt+M)
  - 验证提交信息输入区域
  - 检查文件变更列表
  - 取消提交
"""
import driver


COMMIT_BTN = (0.925, 0.03)


class Test提交对话框高级:
    def test_打开提交对话框(self, app):
        """通过工具栏按钮打开。"""
        driver.click_relative(*COMMIT_BTN)
        driver.sleep(1)
        driver.window_screenshot("ca_01_提交对话框")

    def test_提交对话框布局截图(self, app):
        """截取提交对话框不同区域验证布局。"""
        import os
        # 左侧文件列表区域
        path = driver.region(0.0, 0.08, 0.3, 0.9, "ca_02_文件列表区域")
        assert os.path.exists(path) and os.path.getsize(path) > 0
        # 右侧提交信息区域
        path = driver.region(0.3, 0.08, 0.7, 0.4, "ca_03_提交信息区域")
        assert os.path.exists(path) and os.path.getsize(path) > 0

    def test_Amend模式切换(self, app):
        """Alt+M 切换 amend 模式。"""
        driver.hotkey("alt", "m")
        driver.sleep(0.5)
        driver.window_screenshot("ca_04_amend模式开")
        # 再次切换回来
        driver.hotkey("alt", "m")
        driver.sleep(0.5)
        driver.window_screenshot("ca_05_amend模式关")

    def test_输入提交信息(self, app):
        """在提交信息框中输入文字。"""
        driver.type_text("test: advanced commit dialog")
        driver.sleep(0.5)
        driver.window_screenshot("ca_06_输入信息")

    def test_清空提交信息(self, app):
        """全选+删除清空提交信息。"""
        driver.hotkey("command", "a")
        driver.sleep(0.2)
        driver.press("backspace")
        driver.sleep(0.3)
        driver.window_screenshot("ca_07_清空信息")

    def test_ESC取消提交(self, app):
        """ESC 关闭对话框。"""
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("ca_08_取消提交")
