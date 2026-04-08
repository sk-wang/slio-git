"""
E2E: 键盘快捷键 — 按键精灵风格测试所有快捷键

覆盖:
  - Ctrl+K: 打开提交对话框
  - Ctrl+R: 刷新工作区
  - Ctrl+D: 显示 diff
  - F7 / Shift+F7: hunk 导航
  - Ctrl+S / Ctrl+U: 暂存/取消暂存
"""
import driver


class Test快捷键_提交对话框:
    def test_Ctrl_K打开提交(self, app):
        """Ctrl+K 应打开提交对话框。"""
        driver.hotkey("ctrl", "k")
        driver.sleep(1)
        driver.window_screenshot("kb_01_ctrl_k_提交对话框")

    def test_ESC关闭提交(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("kb_02_esc_关闭提交")


class Test快捷键_刷新:
    def test_Ctrl_R刷新(self, app):
        """Ctrl+R 应刷新工作区（不崩溃即通过）。"""
        driver.window_screenshot("kb_03_刷新前")
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("kb_04_ctrl_r_刷新后")


class Test快捷键_暂存操作:
    def test_Ctrl_Shift_S全部暂存(self, app):
        """Ctrl+Shift+S 暂存所有文件。"""
        driver.hotkey("ctrl", "shift", "s")
        driver.sleep(1)
        driver.window_screenshot("kb_05_ctrl_shift_s_全部暂存")

    def test_Ctrl_Shift_U全部取消暂存(self, app):
        """Ctrl+Shift+U 取消暂存所有文件。"""
        driver.hotkey("ctrl", "shift", "u")
        driver.sleep(1)
        driver.window_screenshot("kb_06_ctrl_shift_u_全部取消暂存")


class Test快捷键_文件导航:
    def test_Ctrl_Alt_Right下一个文件(self, app):
        """Ctrl+Alt+Right 切换到下一个文件。"""
        driver.hotkey("ctrl", "alt", "right")
        driver.sleep(0.5)
        driver.window_screenshot("kb_07_下一个文件")

    def test_Ctrl_Alt_Left上一个文件(self, app):
        """Ctrl+Alt+Left 切换到上一个文件。"""
        driver.hotkey("ctrl", "alt", "left")
        driver.sleep(0.5)
        driver.window_screenshot("kb_08_上一个文件")


class Test快捷键_Diff:
    def test_Ctrl_D显示diff(self, app):
        """Ctrl+D 显示当前文件 diff。"""
        driver.hotkey("ctrl", "d")
        driver.sleep(1)
        driver.window_screenshot("kb_09_ctrl_d_显示diff")

    def test_F7下一个hunk(self, app):
        """F7 跳到下一个 diff hunk。"""
        driver.press("f7")
        driver.sleep(0.5)
        driver.window_screenshot("kb_10_f7_下一个hunk")

    def test_Shift_F7上一个hunk(self, app):
        """Shift+F7 跳到上一个 diff hunk。"""
        driver.hotkey("shift", "f7")
        driver.sleep(0.5)
        driver.window_screenshot("kb_11_shift_f7_上一个hunk")


class Test快捷键_Stash:
    def test_Ctrl_Shift_Z保存stash(self, app):
        """Ctrl+Shift+Z 保存 stash。"""
        driver.hotkey("ctrl", "shift", "z")
        driver.sleep(2)
        driver.window_screenshot("kb_12_stash保存")

    def test_Ctrl_Z恢复stash(self, app):
        """Ctrl+Z pop stash。"""
        driver.hotkey("ctrl", "z")
        driver.sleep(2)
        driver.window_screenshot("kb_13_stash恢复")
