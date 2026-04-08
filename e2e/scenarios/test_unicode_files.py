"""
E2E 场景: Unicode 文件名

覆盖:
  1. 创建中文文件名的文件
  2. 创建含特殊字符的文件名
  3. slio-git 正确显示这些文件
  4. 暂存和提交含中文名的文件
"""

import os
import subprocess

import driver


def _git(repo, *args):
    return subprocess.run(["git"] + list(args), cwd=repo, capture_output=True, text=True)


class TestUnicode文件名:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_创建中文文件名(self, app):
        files = {
            "测试文件.txt": "中文内容\n",
            "配置说明.md": "# 配置说明\n\n这是一个测试。\n",
            "数据/报告.csv": "名称,值\n测试,100\n",
        }
        for name, content in files.items():
            filepath = os.path.join(app, name)
            os.makedirs(os.path.dirname(filepath), exist_ok=True)
            with open(filepath, "w", encoding="utf-8") as f:
                f.write(content)

        driver.sleep(3)
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("unicode_01_中文文件名")

    def test_验证git检测到文件(self, app):
        result = _git(app, "status", "--porcelain")
        print(f"git status:\n{result.stdout}")
        # git 可能用引号包裹 unicode 路径
        assert len(result.stdout.strip().split("\n")) >= 3

    def test_暂存中文文件(self, app):
        _git(app, "add", ".")
        result = _git(app, "diff", "--cached", "--name-only")
        print(f"暂存文件:\n{result.stdout}")
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("unicode_02_暂存中文文件")

    def test_提交中文文件(self, app):
        result = _git(app, "commit", "-m", "e2e: 添加中文文件名文件")
        assert result.returncode == 0
        print(f"提交: {result.stdout.strip()}")

    def test_验证提交成功(self, app):
        result = _git(app, "log", "--oneline", "-1")
        assert "中文文件名" in result.stdout
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("unicode_03_提交后")

    def test_验证文件内容可读(self, app):
        with open(os.path.join(app, "测试文件.txt"), encoding="utf-8") as f:
            assert "中文" in f.read()


class TestUnicode分支名:
    """创建含中文的分支名。"""

    def test_创建中文分支(self, app):
        result = _git(app, "checkout", "-b", "功能/测试分支")
        assert result.returncode == 0
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("unicode_04_中文分支")

    def test_验证分支(self, app):
        result = _git(app, "branch", "--show-current")
        current = result.stdout.strip()
        print(f"当前分支: {current}")
        assert "测试分支" in current

    def test_切回main(self, app):
        _git(app, "checkout", "main")
        _git(app, "branch", "-D", "功能/测试分支")
        driver.sleep(1)
