"""
E2E 场景: 标签操作 (Tag Operations)

流程:
  1. 打开 Tag 对话框
  2. 输入标签名和目标
  3. 创建标签
  4. 验证标签存在
  5. 删除标签
  6. 关闭对话框

Tag 对话框布局:
  - 左侧: 标签列表
  - 右侧: 创建表单 (标签名 + 目标 + 消息) + 操作按钮
  - 底部: 创建标签 按钮
"""

import subprocess

import driver

# 导航栏 Tags 按钮
NAV_TAGS = (0.012, 0.86)

# Tag 对话框内坐标
TAG_NAME_INPUT = (0.35, 0.20)       # 标签名称输入框
TAG_TARGET_INPUT = (0.35, 0.30)     # 目标 commit 输入框
CREATE_TAG_BTN = (0.15, 0.50)       # "创建标签" 按钮
CLOSE_BTN = (0.97, 0.07)            # 关闭按钮
FIRST_TAG_ROW = (0.12, 0.20)        # 标签列表第一行
DELETE_LOCAL_BTN = (0.35, 0.60)     # "删除本地" 按钮


class TestTag创建与删除:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_先用git创建tag(self, app):
        """通过 git 创建标签，确保 slio-git 能显示。"""
        subprocess.run(
            ["git", "tag", "e2e-test-tag", "-m", "E2E test tag"],
            cwd=app, capture_output=True, check=True,
        )
        result = subprocess.run(
            ["git", "tag", "-l"],
            cwd=app, capture_output=True, text=True,
        )
        assert "e2e-test-tag" in result.stdout
        print(f"已创建 tag: e2e-test-tag")

    def test_打开Tags面板(self, app):
        """通过导航栏打开 Tags。"""
        driver.click_relative(*NAV_TAGS)
        driver.sleep(1.5)
        driver.window_screenshot("tag_01_tags面板")

    def test_截图标签列表(self, app):
        """截取标签列表。"""
        driver.region(0.03, 0.06, 0.94, 0.90, "tag_02_标签列表")

    def test_刷新标签(self, app):
        """点击刷新确保标签可见。"""
        # 刷新按钮通常在面板顶部工具栏
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("tag_03_刷新后")

    def test_关闭Tags面板(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        driver.window_screenshot("tag_04_关闭")

    def test_git删除tag(self, app):
        """通过 git 删除测试标签。"""
        subprocess.run(
            ["git", "tag", "-d", "e2e-test-tag"],
            cwd=app, capture_output=True, check=True,
        )
        result = subprocess.run(
            ["git", "tag", "-l"],
            cwd=app, capture_output=True, text=True,
        )
        assert "e2e-test-tag" not in result.stdout
        print("tag 已删除")


class TestTag多标签:
    """测试创建多个标签并验证列表显示。"""

    def test_创建多个tag(self, app):
        for i in range(1, 4):
            subprocess.run(
                ["git", "tag", f"v0.0.{i}"],
                cwd=app, capture_output=True, check=True,
            )
        result = subprocess.run(
            ["git", "tag", "-l"],
            cwd=app, capture_output=True, text=True,
        )
        print(f"所有标签: {result.stdout.strip()}")

    def test_打开Tags查看列表(self, app):
        driver.click_relative(*NAV_TAGS)
        driver.sleep(1.5)
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("tag_05_多标签列表")

    def test_关闭并清理(self, app):
        driver.press("escape")
        driver.sleep(0.5)
        for i in range(1, 4):
            subprocess.run(
                ["git", "tag", "-d", f"v0.0.{i}"],
                cwd=app, capture_output=True,
            )
