"""
E2E: 窗口操作 — 调整大小、最小化/恢复、验证布局响应

覆盖:
  - 获取窗口初始状态
  - 调整窗口尺寸
  - 验证缩小后布局
  - 恢复窗口
"""
import subprocess
import driver
from driver.window import APP_PROCESS


class Test窗口尺寸:
    def test_初始窗口状态(self, app):
        rect = driver.get_bounds()
        assert rect.w >= 800 and rect.h >= 600, f"窗口太小: {rect.w}x{rect.h}"
        driver.window_screenshot("win_01_初始状态")
        print(f"初始窗口: ({rect.x}, {rect.y}) {rect.w}x{rect.h}")

    def test_缩小窗口(self, app):
        """将窗口缩小到 1024x768 并截图。"""
        subprocess.run(
            ["osascript", "-e", f'''
                tell application "System Events"
                    tell (first process whose name is "{APP_PROCESS}")
                        set win to first window
                        set position of win to {{100, 100}}
                        set size of win to {{1024, 768}}
                    end tell
                end tell
            '''],
            capture_output=True,
        )
        driver.sleep(1)
        driver.window_screenshot("win_02_缩小到1024x768")
        rect = driver.get_bounds()
        print(f"缩小后: ({rect.x}, {rect.y}) {rect.w}x{rect.h}")

    def test_最小尺寸(self, app):
        """将窗口缩小到最小尺寸 800x600。"""
        subprocess.run(
            ["osascript", "-e", f'''
                tell application "System Events"
                    tell (first process whose name is "{APP_PROCESS}")
                        set win to first window
                        set position of win to {{100, 100}}
                        set size of win to {{800, 600}}
                    end tell
                end tell
            '''],
            capture_output=True,
        )
        driver.sleep(1)
        driver.window_screenshot("win_03_最小尺寸800x600")

    def test_恢复最大化(self, app):
        """恢复到标准 1728x1080。"""
        driver.maximize()
        driver.sleep(1)
        driver.window_screenshot("win_04_恢复最大化")
        rect = driver.get_bounds()
        assert rect.w >= 1700, f"恢复后宽度不够: {rect.w}"
