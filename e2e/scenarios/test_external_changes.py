"""
E2E 场景: 外部文件变更检测

覆盖:
  1. slio-git 运行时，通过 shell 命令修改文件
  2. 等待 auto-refresh (2s interval) 或手动刷新
  3. 验证 slio-git 检测到变更
  4. 通过 shell 执行 git commit，验证 slio-git 检测到新提交
"""

import os
import subprocess
import time

import driver


def _git(repo, *args):
    return subprocess.run(["git"] + list(args), cwd=repo, capture_output=True, text=True)


class Test外部文件修改:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_通过shell修改文件(self, app):
        """不通过 slio-git，直接用 shell 写文件。"""
        filepath = os.path.join(app, "src/main.py")
        with open(filepath, "w") as f:
            f.write('def main():\n    print("externally modified")\n')
        print(f"已通过 shell 修改 {filepath}")

    def test_等待auto_refresh(self, app):
        """slio-git 有 2s auto-refresh，等 4s 应该够了。"""
        driver.sleep(4)
        driver.window_screenshot("external_01_自动检测")

    def test_验证slio检测到变更(self, app):
        """手动刷新后再截图确认。"""
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("external_02_刷新后")

    def test_清理(self, app):
        _git(app, "checkout", ".")


class Test外部Git操作:
    """在 slio-git 运行时，通过 shell 执行 git 操作。"""

    def test_通过shell创建并提交文件(self, app):
        filepath = os.path.join(app, "external_commit.txt")
        with open(filepath, "w") as f:
            f.write("committed externally\n")
        _git(app, "add", "external_commit.txt")
        _git(app, "commit", "-m", "e2e: external commit via shell")
        print("已通过 shell 执行 git commit")

    def test_等待并刷新(self, app):
        driver.sleep(3)
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("external_03_外部commit后")

    def test_通过shell创建分支(self, app):
        _git(app, "branch", "external-branch")
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("external_04_外部branch后")

    def test_验证分支存在(self, app):
        result = _git(app, "branch")
        assert "external-branch" in result.stdout

    def test_通过shell删除分支(self, app):
        _git(app, "branch", "-D", "external-branch")
        driver.hotkey("ctrl", "r")
        driver.sleep(2)

    def test_通过shell修改已提交文件(self, app):
        """修改已追踪文件，不暂存。"""
        filepath = os.path.join(app, "README.md")
        with open(filepath, "a") as f:
            f.write("\n\nAppended externally.\n")
        driver.sleep(4)
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("external_05_外部修改追踪文件")

    def test_清理(self, app):
        _git(app, "checkout", ".")
        _git(app, "clean", "-fd")


class Test外部Stash操作:
    """通过 shell 执行 stash，验证 slio-git 检测。"""

    def test_创建变更(self, app):
        with open(os.path.join(app, "src/main.py"), "a") as f:
            f.write("\n# external stash test\n")

    def test_shell执行stash(self, app):
        result = _git(app, "stash", "push", "-m", "external stash")
        assert result.returncode == 0
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("external_06_外部stash后")

    def test_验证工作区干净(self, app):
        result = _git(app, "status", "--porcelain")
        assert result.stdout.strip() == ""

    def test_shell执行stash_pop(self, app):
        result = _git(app, "stash", "pop")
        assert result.returncode == 0
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("external_07_外部stash_pop后")

    def test_清理(self, app):
        _git(app, "checkout", ".")
