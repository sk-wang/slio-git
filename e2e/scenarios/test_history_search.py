"""
E2E 场景: 历史视图搜索与交互 (History Search)

流程:
  1. 切换到 Log tab
  2. 搜索提交关键词
  3. 点击选中提交查看详情
  4. 右键提交查看上下文菜单
  5. 复制提交哈希
  6. 清除搜索

History 视图布局:
  - 顶部: 搜索框 + 搜索/清除 按钮 + 刷新
  - 主体: 提交列表 (graph | hash | message | author | time)
  - 底部/侧边: 提交详情面板
"""

import subprocess

import driver

LOG_TAB = (0.085, 0.07)
CHANGES_TAB = (0.045, 0.07)

# 历史视图坐标
SEARCH_INPUT = (0.25, 0.115)         # 搜索框
SEARCH_BTN = (0.42, 0.115)           # "搜索" 按钮
CLEAR_BTN = (0.47, 0.115)            # "清除" 按钮
REFRESH_BTN = (0.52, 0.115)          # "刷新" 按钮

# 提交列表行
COMMIT_ROW_1 = (0.30, 0.18)         # 第一个提交行
COMMIT_ROW_2 = (0.30, 0.22)         # 第二个提交行
COMMIT_ROW_3 = (0.30, 0.26)         # 第三个提交行


class Test历史搜索:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_先创建多个提交(self, app):
        """创建几个提交以丰富历史记录。"""
        import os
        for i in range(1, 4):
            filepath = os.path.join(app, f"history_test_{i}.txt")
            with open(filepath, "w") as f:
                f.write(f"History search test file {i}\n")
            subprocess.run(["git", "add", filepath], cwd=app, capture_output=True)
            subprocess.run(
                ["git", "commit", "-m", f"feat: add history file {i}"],
                cwd=app, capture_output=True,
            )
        driver.sleep(2)
        driver.hotkey("ctrl", "r")
        driver.sleep(2)

    def test_切换到Log_tab(self, app):
        driver.click_relative(*LOG_TAB)
        driver.sleep(1.5)
        driver.window_screenshot("history_01_log_tab")

    def test_截图提交列表(self, app):
        driver.region(0.03, 0.10, 0.94, 0.85, "history_02_提交列表全貌")

    def test_点击第一个提交(self, app):
        """选中第一个提交查看详情。"""
        driver.click_relative(*COMMIT_ROW_1)
        driver.sleep(1)
        driver.window_screenshot("history_03_选中提交")

    def test_键盘下移选中(self, app):
        """用键盘 Down 切换选中。"""
        for _ in range(3):
            driver.press("down")
            driver.sleep(0.3)
        driver.window_screenshot("history_04_键盘导航")

    def test_右键提交查看菜单(self, app):
        """右键点击提交行，弹出上下文菜单。"""
        driver.right_click(
            int(0.30 * 1728), int(0.22 * 1080)
        )
        driver.sleep(1)
        driver.window_screenshot("history_05_右键菜单")

    def test_ESC关闭菜单(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("history_06_菜单关闭")

    def test_切回Changes(self, app):
        driver.click_relative(*CHANGES_TAB)
        driver.sleep(1)
        driver.window_screenshot("history_07_回到changes")


class Test历史滚动:
    """测试长历史列表的滚动。"""

    def test_切换到Log(self, app):
        driver.click_relative(*LOG_TAB)
        driver.sleep(1)

    def test_滚动到底部(self, app):
        """用 End 键或多次 Down 滚动到底部。"""
        driver.press("end")
        driver.sleep(1)
        driver.window_screenshot("history_08_滚动到底部")

    def test_滚动到顶部(self, app):
        """用 Home 键回到顶部。"""
        driver.press("home")
        driver.sleep(1)
        driver.window_screenshot("history_09_滚动到顶部")

    def test_切回Changes(self, app):
        driver.click_relative(*CHANGES_TAB)
        driver.sleep(0.5)
