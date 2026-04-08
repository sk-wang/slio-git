"""
E2E 场景: .gitignore 行为

覆盖:
  1. 创建 .gitignore 规则
  2. 被忽略的文件不出现在 git status
  3. slio-git 不在变更列表显示被忽略文件
  4. 修改 .gitignore 后刷新
"""

import os
import subprocess

import driver


def _git(repo, *args):
    return subprocess.run(["git"] + list(args), cwd=repo, capture_output=True, text=True)


class TestGitignore基本:
    def test_确保窗口聚焦(self, app):
        driver.activate()
        driver.click_relative(0.3, 0.5)
        driver.sleep(0.5)

    def test_创建gitignore(self, app):
        with open(os.path.join(app, ".gitignore"), "w") as f:
            f.write("*.log\n*.tmp\nbuild/\n")
        _git(app, "add", ".gitignore")
        _git(app, "commit", "-m", "e2e: add gitignore")

    def test_创建应被忽略的文件(self, app):
        with open(os.path.join(app, "debug.log"), "w") as f:
            f.write("debug log content\n")
        with open(os.path.join(app, "temp.tmp"), "w") as f:
            f.write("temp file\n")
        os.makedirs(os.path.join(app, "build"), exist_ok=True)
        with open(os.path.join(app, "build", "output.bin"), "w") as f:
            f.write("build output\n")
        # 也创建一个不应被忽略的文件
        with open(os.path.join(app, "normal.txt"), "w") as f:
            f.write("normal file\n")

    def test_验证git忽略了文件(self, app):
        result = _git(app, "status", "--porcelain")
        print(f"git status:\n{result.stdout.strip()}")
        assert "debug.log" not in result.stdout
        assert "temp.tmp" not in result.stdout
        assert "build/" not in result.stdout
        assert "normal.txt" in result.stdout

    def test_slio检测(self, app):
        driver.hotkey("ctrl", "r")
        driver.sleep(3)
        driver.window_screenshot("gitignore_01_忽略后状态")

    def test_git_check_ignore确认(self, app):
        for f in ["debug.log", "temp.tmp", "build/output.bin"]:
            result = _git(app, "check-ignore", f)
            assert result.returncode == 0, f"{f} 应被忽略"
        result = _git(app, "check-ignore", "normal.txt")
        assert result.returncode != 0, "normal.txt 不应被忽略"

    def test_清理(self, app):
        _git(app, "clean", "-fd")


class TestGitignore动态修改:
    """修改 .gitignore 后新规则立即生效。"""

    def test_创建文件(self, app):
        with open(os.path.join(app, "data.csv"), "w") as f:
            f.write("a,b,c\n1,2,3\n")
        result = _git(app, "status", "--porcelain")
        assert "data.csv" in result.stdout, "修改 gitignore 前应可见"

    def test_添加忽略规则(self, app):
        with open(os.path.join(app, ".gitignore"), "a") as f:
            f.write("*.csv\n")

    def test_验证文件被忽略(self, app):
        result = _git(app, "status", "--porcelain")
        # data.csv 应不在 status 中 (被新规则忽略)
        untracked = [l for l in result.stdout.strip().split("\n") if "data.csv" in l and "??" in l]
        assert len(untracked) == 0, f"data.csv 应被新规则忽略，但仍显示: {untracked}"
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        driver.window_screenshot("gitignore_02_动态忽略")

    def test_清理(self, app):
        _git(app, "checkout", ".gitignore")
        _git(app, "clean", "-fd")
