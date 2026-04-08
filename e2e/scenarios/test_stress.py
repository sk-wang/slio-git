"""
E2E 场景: 快速交互压力测试

覆盖:
  1. 快速切换 Tab (Changes ↔ Log) 多次
  2. 快速打开/关闭辅助面板
  3. 快速刷新多次
  4. 快速暂存/取消暂存
  5. 验证 app 未崩溃
"""

import driver


class Test快速Tab切换:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_快速切换Tab_10次(self, app):
        """在 Changes 和 Log 之间快速切换。"""
        for i in range(10):
            driver.click_relative(0.085, 0.07)  # Log tab
            driver.sleep(0.2)
            driver.click_relative(0.045, 0.07)  # Changes tab
            driver.sleep(0.2)
        driver.sleep(0.5)
        driver.window_screenshot("stress_01_tab切换后")

    def test_进程仍存活(self, app):
        assert driver.is_alive(), "app 崩溃了！"


class Test快速面板切换:
    def test_快速打开关闭辅助面板(self, app):
        """快速在 Remotes/Tags/Stashes/Rebase 间切换。"""
        panels = [
            (0.012, 0.82),   # Remotes
            (0.012, 0.86),   # Tags
            (0.012, 0.90),   # Stashes
            (0.012, 0.94),   # Rebase
        ]
        for _ in range(3):
            for px, py in panels:
                driver.click_relative(px, py)
                driver.sleep(0.3)
        # 回到 Changes
        driver.click_relative(0.014, 0.09)
        driver.sleep(0.5)
        driver.window_screenshot("stress_02_面板切换后")

    def test_进程仍存活(self, app):
        assert driver.is_alive()


class Test快速刷新:
    def test_连续刷新10次(self, app):
        """快速按 Ctrl+R 刷新 10 次。"""
        for _ in range(10):
            driver.hotkey("ctrl", "r")
            driver.sleep(0.3)
        driver.sleep(1)
        driver.window_screenshot("stress_03_连续刷新后")

    def test_进程仍存活(self, app):
        assert driver.is_alive()


class Test快速暂存操作:
    def test_创建文件变更(self, app):
        import os
        from scenarios.conftest import add_unstaged_change
        add_unstaged_change(app, filename="src/main.py")
        driver.sleep(3)

    def test_快速暂存取消10次(self, app):
        """快速 Ctrl+Shift+S / Ctrl+Shift+U 循环 10 次。"""
        for _ in range(10):
            driver.hotkey("ctrl", "shift", "s")
            driver.sleep(0.2)
            driver.hotkey("ctrl", "shift", "u")
            driver.sleep(0.2)
        driver.sleep(0.5)
        driver.window_screenshot("stress_04_暂存循环后")

    def test_进程仍存活(self, app):
        assert driver.is_alive()

    def test_清理(self, app):
        import subprocess
        subprocess.run(["git", "checkout", "."], cwd=app, capture_output=True)
        subprocess.run(["git", "clean", "-fd"], cwd=app, capture_output=True)


class Test快速键盘操作:
    def test_快速按键序列(self, app):
        """快速按各种快捷键。"""
        keys = [
            ("ctrl", "r"),       # 刷新
            ("ctrl", "k"),       # 提交对话框
            ("escape",),         # 关闭
            ("ctrl", "d"),       # diff
            ("f7",),             # next hunk
            ("shift", "f7"),     # prev hunk
            ("ctrl", "r"),       # 刷新
        ]
        for _ in range(3):
            for key_combo in keys:
                if len(key_combo) == 1:
                    driver.press(key_combo[0])
                else:
                    driver.hotkey(*key_combo)
                driver.sleep(0.15)
        driver.sleep(0.5)
        driver.window_screenshot("stress_05_快速键盘后")

    def test_进程仍存活(self, app):
        assert driver.is_alive()
        driver.window_screenshot("stress_06_最终状态")
