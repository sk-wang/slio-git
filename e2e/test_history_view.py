"""
E2E: 历史视图 — 切换到 Log tab，浏览提交历史

覆盖:
  - 切换到 Log tab
  - 滚动历史列表
  - 点击某个 commit 查看详情
  - 切回 Changes tab
"""
import driver


LOG_TAB = (0.085, 0.07)
CHANGES_TAB = (0.045, 0.07)


class Test历史视图:
    def test_切换到Log(self, app):
        """点击 Log tab。"""
        driver.click_relative(*LOG_TAB)
        driver.sleep(1)
        driver.window_screenshot("hist_01_log_tab")

    def test_历史列表可见(self, app):
        """截取历史列表区域，确认有内容。"""
        import os
        path = driver.region(0.03, 0.10, 0.94, 0.85, "hist_02_历史列表")
        assert os.path.exists(path) and os.path.getsize(path) > 0

    def test_点击第一个commit(self, app):
        """点击历史列表中第一个 commit。"""
        driver.click_relative(0.30, 0.15)
        driver.sleep(1)
        driver.window_screenshot("hist_03_选中commit")

    def test_点击第二个commit(self, app):
        """点击第二个 commit。"""
        driver.click_relative(0.30, 0.19)
        driver.sleep(1)
        driver.window_screenshot("hist_04_选中commit2")

    def test_滚动历史列表(self, app):
        """用键盘下箭头滚动。"""
        for _ in range(5):
            driver.press("down")
            driver.sleep(0.2)
        driver.window_screenshot("hist_05_滚动后")

    def test_切回Changes(self, app):
        """切回 Changes tab。"""
        driver.click_relative(*CHANGES_TAB)
        driver.sleep(1)
        driver.window_screenshot("hist_06_回到changes")
