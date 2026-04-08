"""
E2E: 导航栏辅助视图 — Remotes、Tags、Stashes、Settings

测试左侧导航栏底部的辅助面板按钮:
  - 打开/关闭各辅助面板
  - 验证面板显示正确
"""
import driver


# 导航栏按钮相对坐标 (左侧垂直排列，从下往上)
# 基于 1728x1080 窗口，左侧 nav rail 宽约 40px
NAV_CHANGES = (0.012, 0.12)      # Changes 按钮 (顶部)
NAV_REMOTES = (0.012, 0.82)      # Remotes 按钮
NAV_TAGS = (0.012, 0.86)         # Tags 按钮
NAV_STASHES = (0.012, 0.90)      # Stashes 按钮
NAV_REBASE = (0.012, 0.94)       # Rebase 按钮


class TestRemotes面板:
    def test_打开Remotes(self, app):
        """点击导航栏 Remotes 图标。"""
        driver.click_relative(*NAV_REMOTES)
        driver.sleep(1)
        driver.window_screenshot("nav_01_remotes面板")

    def test_关闭Remotes(self, app):
        """ESC 关闭 Remotes 面板。"""
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("nav_02_remotes关闭")


class TestTags面板:
    def test_打开Tags(self, app):
        """点击导航栏 Tags 图标。"""
        driver.click_relative(*NAV_TAGS)
        driver.sleep(1)
        driver.window_screenshot("nav_03_tags面板")

    def test_关闭Tags(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("nav_04_tags关闭")


class TestStashes面板:
    def test_打开Stashes(self, app):
        """点击导航栏 Stashes 图标。"""
        driver.click_relative(*NAV_STASHES)
        driver.sleep(1)
        driver.window_screenshot("nav_05_stashes面板")

    def test_关闭Stashes(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("nav_06_stashes关闭")


class Test回到Changes:
    def test_点击Changes(self, app):
        """确保所有面板关闭后能回到 Changes 主视图。"""
        driver.click_relative(*NAV_CHANGES)
        driver.sleep(1)
        driver.window_screenshot("nav_07_回到changes")
