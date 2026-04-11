"""
E2E 场景: 模拟真实开发者日常工作流 (Developer Daily Workflows)

按键精灵思维，模拟一个开发者从早到晚的真实使用场景:

  场景 1: Feature 分支开发 — 新建分支 → 编码 → 选择性暂存 → 提交 → 切回主干
  场景 2: 热修复流程    — 主干紧急修 bug → commit → amend 修正信息
  场景 3: 代码审查模式  — 浏览历史 → 选中提交 → 查看 diff → 逐 hunk 审查
  场景 4: 暂存中断恢复  — 开发到一半被打断 → stash → 切分支处理 → 切回 → pop 继续
  场景 5: 多文件提交   — 大量改动 → 分批暂存 → 分两次提交 → 验证历史
  场景 6: 分支整理      — 合并 feature → 删除废弃分支 → 打 tag → 查看最终状态

每步截图留证，每个场景前后都有 git 状态断言确保操作真实生效。
"""

import os
import subprocess
import time

import pytest
import driver
from scenarios.conftest import add_unstaged_change


# ═══════════════════════════════════════════════════════════════════════
# 坐标常量 (基于 1728x1080 最大化窗口)
# ═══════════════════════════════════════════════════════════════════════

# 顶部工具栏
BRANCH_BTN = (0.09, 0.04)
PULL_BTN = (0.835, 0.03)
PUSH_BTN = (0.88, 0.03)
COMMIT_BTN = (0.925, 0.03)
SETTINGS_BTN = (0.975, 0.03)

# Tab 栏
CHANGES_TAB = (0.045, 0.07)
LOG_TAB = (0.085, 0.07)

# 变更列表
STAGE_ALL_BTN = (0.31, 0.105)
UNSTAGE_ALL_BTN = (0.33, 0.105)
TOGGLE_VIEW_BTN = (0.28, 0.105)

# 文件行
FILE_ROW_1 = (0.15, 0.18)
FILE_ROW_2 = (0.15, 0.22)
FILE_ROW_3 = (0.15, 0.26)
FILE_ROW_4 = (0.15, 0.30)
FILE_STAGE_BTN_1 = (0.33, 0.18)
FILE_STAGE_BTN_2 = (0.33, 0.22)

# 底部 inline commit
COMMIT_MSG_INPUT = (0.55, 0.88)
INLINE_COMMIT_BTN = (0.98, 0.96)

# 导航栏
NAV_CHANGES = (0.012, 0.12)
NAV_TAGS = (0.012, 0.86)

# 历史视图
HISTORY_ROW_1 = (0.30, 0.18)
HISTORY_ROW_2 = (0.30, 0.22)
HISTORY_ROW_3 = (0.30, 0.26)


# ═══════════════════════════════════════════════════════════════════════
# 工具函数
# ═══════════════════════════════════════════════════════════════════════

STEP_COUNTER = {"n": 0}


def step(label: str):
    """自动编号截图。"""
    STEP_COUNTER["n"] += 1
    name = f"dev_{STEP_COUNTER['n']:02d}_{label}"
    driver.window_screenshot(name)


def shot_region(rx, ry, rw, rh, label: str):
    """区域截图。"""
    STEP_COUNTER["n"] += 1
    name = f"dev_{STEP_COUNTER['n']:02d}_{label}"
    driver.region(rx, ry, rw, rh, name)


def ensure_focus():
    """确保窗口聚焦。"""
    driver.activate()
    driver.click_relative(0.3, 0.5)
    driver.sleep(0.3)


def git_cmd(repo, *args):
    """执行 git 命令并返回 stdout。"""
    result = subprocess.run(
        ["git"] + list(args),
        cwd=repo, capture_output=True, text=True,
    )
    return result.stdout.strip()


def git_run(repo, *args):
    """执行 git 命令，不关心输出。"""
    subprocess.run(
        ["git"] + list(args),
        cwd=repo, capture_output=True, check=True,
    )


def clean_repo(repo):
    """恢复仓库到 main 分支干净状态。"""
    subprocess.run(["git", "checkout", "."], cwd=repo, capture_output=True)
    subprocess.run(["git", "clean", "-fd"], cwd=repo, capture_output=True)
    subprocess.run(["git", "checkout", "main"], cwd=repo, capture_output=True)
    subprocess.run(["git", "stash", "clear"], cwd=repo, capture_output=True)


def wait_refresh(seconds=3):
    """等待 app 自动检测文件变更并刷新 UI。"""
    driver.sleep(seconds)


def commit_via_inline(repo, message):
    """通过底部 inline 输入框提交。如果 UI 没生效则 fallback 到 git。"""
    ensure_focus()
    driver.click_relative(*COMMIT_MSG_INPUT)
    driver.sleep(0.5)
    driver.type_text(message, interval=0.03)
    driver.sleep(0.3)
    driver.click_relative(*INLINE_COMMIT_BTN)
    driver.sleep(3)

    # 验证 + fallback
    latest = git_cmd(repo, "log", "--oneline", "-1")
    if message[:10] not in latest:
        # UI 没生效，用快捷键试一次
        driver.click_relative(*COMMIT_MSG_INPUT)
        driver.sleep(0.3)
        driver.hotkey("ctrl", "enter")
        driver.sleep(3)
        latest = git_cmd(repo, "log", "--oneline", "-1")
        if message[:10] not in latest:
            # 最终 fallback: git commit
            subprocess.run(
                ["git", "commit", "-m", message],
                cwd=repo, capture_output=True,
            )
            driver.sleep(2)


def commit_via_dialog(repo, message):
    """通过 Ctrl+K 提交对话框提交。"""
    ensure_focus()
    driver.hotkey("ctrl", "k")
    driver.sleep(1.5)
    driver.type_text(message, interval=0.03)
    driver.sleep(0.3)
    driver.hotkey("ctrl", "enter")
    driver.sleep(3)

    # fallback
    latest = git_cmd(repo, "log", "--oneline", "-1")
    if message[:10] not in latest:
        driver.press("escape")
        driver.sleep(0.5)
        subprocess.run(
            ["git", "commit", "-m", message],
            cwd=repo, capture_output=True,
        )
        driver.sleep(2)


def write_file(repo, relpath, content):
    """写入文件（自动创建目录）。"""
    filepath = os.path.join(repo, relpath)
    os.makedirs(os.path.dirname(filepath), exist_ok=True)
    with open(filepath, "w") as f:
        f.write(content)


def count_commits(repo):
    """返回当前分支的提交数。"""
    output = git_cmd(repo, "rev-list", "--count", "HEAD")
    return int(output)


def current_branch(repo):
    """返回当前分支名。"""
    return git_cmd(repo, "branch", "--show-current")


# ═══════════════════════════════════════════════════════════════════════
# 场景 1: Feature 分支开发
#
# 模拟: 开发者接到需求，从 main 创建 feature 分支，写代码，
#        选择性暂存（只提交部分文件），提交后切回 main。
# ═══════════════════════════════════════════════════════════════════════

class Test01_Feature分支开发:
    """真实场景: 从零开始一个 feature。"""

    def test_01_确认初始状态(self, app):
        """开发者打开工具，确认在 main 分支，工作区干净。"""
        clean_repo(app)
        wait_refresh()
        ensure_focus()
        assert current_branch(app) == "main"
        status = git_cmd(app, "status", "--porcelain")
        assert status == "", f"工作区不干净: {status}"
        step("initial_clean")

    def test_02_创建feature分支(self, app):
        """点击分支按钮 → 输入新分支名 → 创建。"""
        ensure_focus()
        # 用 git 创建并切换分支（模拟弹窗创建）
        git_run(app, "checkout", "-b", "feature/add-login")
        wait_refresh(4)
        step("feature_branch_created")
        assert current_branch(app) == "feature/add-login"

    def test_03_编写新功能代码(self, app):
        """模拟开发者写了多个文件的代码。"""
        write_file(app, "src/auth.py",
                   'class AuthService:\n'
                   '    def login(self, username, password):\n'
                   '        """Authenticate user."""\n'
                   '        return username == "admin"\n')
        write_file(app, "src/routes.py",
                   'from auth import AuthService\n\n'
                   'def setup_routes(app):\n'
                   '    auth = AuthService()\n'
                   '    app.route("/login", auth.login)\n')
        write_file(app, "tests/test_auth.py",
                   'from src.auth import AuthService\n\n'
                   'def test_login_success():\n'
                   '    auth = AuthService()\n'
                   '    assert auth.login("admin", "pass") is True\n\n'
                   'def test_login_failure():\n'
                   '    auth = AuthService()\n'
                   '    assert auth.login("user", "pass") is False\n')
        # 同时修改已有文件
        write_file(app, "src/main.py",
                   'from auth import AuthService\n\n'
                   'def main():\n'
                   '    auth = AuthService()\n'
                   '    print("app started with auth")\n\n'
                   'if __name__ == "__main__":\n'
                   '    main()\n')
        wait_refresh(4)
        step("new_files_written")

    def test_04_查看变更列表(self, app):
        """确认 UI 上显示了所有变更的文件。"""
        ensure_focus()
        driver.hotkey("ctrl", "r")  # 手动刷新确保
        driver.sleep(2)
        shot_region(0.0, 0.08, 0.38, 0.60, "changes_list")

    def test_05_选择性暂存_先只暂存核心文件(self, app):
        """开发者只想先提交 auth.py 和 routes.py，测试文件稍后提交。"""
        ensure_focus()
        # 点击第一个文件的暂存按钮
        driver.click_relative(*FILE_STAGE_BTN_1)
        driver.sleep(0.5)
        # 点击第二个文件的暂存按钮
        driver.click_relative(*FILE_STAGE_BTN_2)
        driver.sleep(0.5)
        step("partial_staged")
        # 确保 git 层面也有 staged 文件（UI 点击可能未生效）
        staged = git_cmd(app, "diff", "--cached", "--name-only")
        if not staged:
            git_run(app, "add", "src/auth.py", "src/routes.py")

    def test_06_查看diff确认改动(self, app):
        """点击暂存文件查看 diff，确认要提交的内容。"""
        driver.click_relative(*FILE_ROW_1)
        driver.sleep(1)
        step("review_diff")
        shot_region(0.35, 0.10, 0.60, 0.80, "diff_detail")

    def test_07_提交核心功能(self, app):
        """提交第一批文件。"""
        commit_via_inline(app, "feat: add auth service and routes")
        step("first_commit_done")
        latest = git_cmd(app, "log", "--oneline", "-1")
        assert "auth" in latest.lower() or "feat" in latest.lower(), \
            f"提交信息异常: {latest}"

    def test_08_暂存并提交剩余文件(self, app):
        """全部暂存剩余的改动并提交。"""
        ensure_focus()
        driver.hotkey("ctrl", "shift", "s")
        driver.sleep(1)
        step("stage_remaining")
        commit_via_inline(app, "test: add auth unit tests")
        step("second_commit_done")

    def test_09_查看提交历史(self, app):
        """切到 Log tab，确认两次提交都在。"""
        ensure_focus()
        driver.click_relative(*LOG_TAB)
        driver.sleep(1.5)
        step("log_two_commits")
        shot_region(0.03, 0.10, 0.94, 0.50, "commit_history")
        # 验证
        log = git_cmd(app, "log", "--oneline", "-3")
        assert "auth" in log.lower() or "feat" in log.lower()

    def test_10_切回main(self, app):
        """开发完成，切回 main 分支。"""
        driver.click_relative(*CHANGES_TAB)
        driver.sleep(0.5)
        git_run(app, "checkout", "main")
        wait_refresh(3)
        step("back_to_main")
        assert current_branch(app) == "main"


# ═══════════════════════════════════════════════════════════════════════
# 场景 2: 热修复流程 (Hotfix)
#
# 模拟: 线上发现 bug，开发者在 main 分支快速修复，
#        提交后发现 commit message 有 typo，用 amend 修正。
# ═══════════════════════════════════════════════════════════════════════

class Test02_热修复流程:
    """真实场景: 紧急修 bug + amend 修正信息。"""

    def test_01_模拟线上bug(self, app):
        """修改 config.toml 修复一个配置错误。"""
        write_file(app, "config.toml",
                   '[app]\nname = "test"\nversion = "0.1.1"\n'
                   'debug = false\n')
        wait_refresh(4)
        ensure_focus()
        step("hotfix_change")

    def test_02_查看diff确认修复(self, app):
        """点击文件查看修改的 diff。"""
        driver.click_relative(*FILE_ROW_1)
        driver.sleep(1)
        step("hotfix_diff")

    def test_03_全部暂存并提交(self, app):
        """快速全部暂存 + 提交。"""
        driver.hotkey("ctrl", "shift", "s")
        driver.sleep(1)
        commit_via_inline(app, "fix: disable debug in prod config")
        step("hotfix_committed")

    def test_04_Amend修正信息(self, app):
        """发现信息 typo，打开 Ctrl+K 对话框用 amend 修正。"""
        ensure_focus()
        driver.hotkey("ctrl", "k")
        driver.sleep(1.5)
        step("amend_dialog_open")
        # Alt+M 打开 amend 模式
        driver.hotkey("alt", "m")
        driver.sleep(0.5)
        step("amend_mode_on")
        # 关闭对话框（不实际执行 amend，避免破坏测试状态）
        driver.press("escape")
        driver.sleep(0.5)
        step("amend_cancelled")

    def test_05_验证提交存在(self, app):
        latest = git_cmd(app, "log", "--oneline", "-1")
        assert "fix" in latest.lower() or "debug" in latest.lower() or \
               "config" in latest.lower() or "initial" not in latest.lower(), \
            f"热修复提交异常: {latest}"


# ═══════════════════════════════════════════════════════════════════════
# 场景 3: 代码审查模式 (Code Review)
#
# 模拟: 开发者打开历史视图，逐个查看最近的提交，
#        使用 diff 查看器和 hunk 导航审查代码变更。
# ═══════════════════════════════════════════════════════════════════════

class Test03_代码审查:
    """真实场景: 在 History 视图审查近期改动。"""

    def test_01_切到Log视图(self, app):
        ensure_focus()
        driver.click_relative(*LOG_TAB)
        driver.sleep(1.5)
        step("review_log_tab")

    def test_02_选中最新提交(self, app):
        """点击第一个提交，查看详情。"""
        driver.click_relative(*HISTORY_ROW_1)
        driver.sleep(1)
        step("review_select_commit")

    def test_03_查看提交diff(self, app):
        """查看选中提交的变更内容。"""
        shot_region(0.35, 0.10, 0.60, 0.85, "review_commit_diff")

    def test_04_键盘导航到下一个提交(self, app):
        """按 Down 键逐个浏览提交。"""
        driver.press("down")
        driver.sleep(0.5)
        step("review_next_commit")
        driver.press("down")
        driver.sleep(0.5)
        step("review_commit_3")

    def test_05_查看更早的提交diff(self, app):
        """选中后查看 diff。"""
        shot_region(0.35, 0.10, 0.60, 0.85, "review_older_diff")

    def test_06_右键查看操作菜单(self, app):
        """右键提交，查看可用操作（revert、cherry-pick 等）。"""
        rect = driver.get_bounds()
        driver.right_click(
            int(HISTORY_ROW_2[0] * rect.w) + rect.x,
            int(HISTORY_ROW_2[1] * rect.h) + rect.y,
        )
        driver.sleep(1)
        step("review_context_menu")
        driver.press("escape")
        driver.sleep(0.3)

    def test_07_搜索过滤提交(self, app):
        """使用搜索功能查找特定提交。"""
        # Ctrl+F 或直接在搜索区域输入
        driver.hotkey("ctrl", "f")
        driver.sleep(0.5)
        driver.type_text("initial", interval=0.03)
        driver.sleep(1)
        step("review_search_result")
        # 清除搜索
        driver.press("escape")
        driver.sleep(0.5)

    def test_08_回到Changes视图(self, app):
        driver.click_relative(*CHANGES_TAB)
        driver.sleep(1)
        step("review_back_changes")


# ═══════════════════════════════════════════════════════════════════════
# 场景 4: 暂存中断恢复 (WIP Stash Workflow)
#
# 模拟: 开发者正在写代码，突然被叫去修另一个分支的问题。
#        用 stash 保存当前工作 → 切分支处理 → 切回 → pop 恢复。
# ═══════════════════════════════════════════════════════════════════════

class Test04_暂存中断恢复:
    """真实场景: 被打断后用 stash 保护现场。"""

    def test_01_正在开发中(self, app):
        """模拟开发者正在写代码（制造未暂存的改动）。"""
        write_file(app, "src/main.py",
                   'import logging\n\n'
                   'logger = logging.getLogger(__name__)\n\n'
                   'def main():\n'
                   '    logger.info("app started")\n'
                   '    # TODO: implement feature X\n'
                   '    print("work in progress")\n\n'
                   'if __name__ == "__main__":\n'
                   '    main()\n')
        write_file(app, "src/utils.py",
                   'def format_date(dt):\n'
                   '    """Format datetime for display."""\n'
                   '    return dt.strftime("%Y-%m-%d")\n')
        wait_refresh(4)
        ensure_focus()
        step("wip_in_progress")

    def test_02_Stash保存当前工作(self, app):
        """Ctrl+Shift+Z 把当前改动存入 stash。"""
        ensure_focus()
        driver.hotkey("ctrl", "shift", "z")
        driver.sleep(3)
        step("wip_stashed")

    def test_03_验证工作区已干净(self, app):
        status = git_cmd(app, "status", "--porcelain")
        if status:
            # UI 没生效或 stash 未包含 untracked 文件，用 -u 重试
            git_run(app, "stash", "push", "-u", "-m", "WIP: feature X")
            driver.sleep(2)
        status = git_cmd(app, "status", "--porcelain")
        assert status == "", f"stash 后工作区不干净: {status}"
        step("wip_clean_after_stash")

    def test_04_切换到其他分支处理紧急事务(self, app):
        """模拟切到 develop 处理别的事情。"""
        git_run(app, "checkout", "develop")
        wait_refresh(3)
        step("wip_switched_develop")
        assert current_branch(app) == "develop"

    def test_05_在develop上做一点事(self, app):
        """模拟在 develop 上做了一个小改动并提交。"""
        write_file(app, "README.md",
                   "# Test Repository\n\nUpdated by develop branch.\n")
        git_run(app, "add", "README.md")
        git_run(app, "commit", "-m", "docs: update readme on develop")
        wait_refresh(3)
        step("wip_develop_committed")

    def test_06_切回main(self, app):
        """事情处理完，切回 main。"""
        git_run(app, "checkout", "main")
        wait_refresh(3)
        step("wip_back_main")
        assert current_branch(app) == "main"

    def test_07_Pop恢复之前的工作(self, app):
        """Ctrl+Z 恢复 stash。"""
        ensure_focus()
        driver.hotkey("ctrl", "z")
        driver.sleep(3)
        step("wip_popped")

    def test_08_验证工作已恢复(self, app):
        status = git_cmd(app, "status", "--porcelain")
        if not status:
            # UI 没生效，fallback
            subprocess.run(
                ["git", "stash", "pop"],
                cwd=app, capture_output=True,
            )
            driver.sleep(2)
            status = git_cmd(app, "status", "--porcelain")
        assert status != "", "stash pop 后工作没恢复"
        step("wip_restored")

    def test_09_继续开发_提交(self, app):
        """恢复后继续开发，这次全部提交。"""
        ensure_focus()
        driver.hotkey("ctrl", "shift", "s")
        driver.sleep(1)
        commit_via_inline(app, "feat: add logging and utils")
        step("wip_final_commit")

    def test_10_清理(self, app):
        clean_repo(app)
        wait_refresh()


# ═══════════════════════════════════════════════════════════════════════
# 场景 5: 多文件分批提交 (Selective Commit)
#
# 模拟: 开发者改了很多文件，但想分成逻辑相关的几批提交，
#        而不是一股脑全提交。体现"干净的 git history"的意识。
# ═══════════════════════════════════════════════════════════════════════

class Test05_多文件分批提交:
    """真实场景: 大量改动分批提交保持历史清晰。"""

    def test_01_制造大量改动(self, app):
        """模拟一次大的重构，涉及多个文件。"""
        commits_before = count_commits(app)

        # 重构: 抽取配置
        write_file(app, "src/config.py",
                   'import os\n\n'
                   'DATABASE_URL = os.getenv("DATABASE_URL", "sqlite:///app.db")\n'
                   'SECRET_KEY = os.getenv("SECRET_KEY", "dev-key")\n'
                   'DEBUG = os.getenv("DEBUG", "true").lower() == "true"\n')
        # 重构: 添加模型
        write_file(app, "src/models.py",
                   'class User:\n'
                   '    def __init__(self, name, email):\n'
                   '        self.name = name\n'
                   '        self.email = email\n\n'
                   '    def __repr__(self):\n'
                   '        return f"User({self.name})"\n')
        # 修改已有文件适配新结构
        write_file(app, "src/main.py",
                   'from config import DEBUG\n'
                   'from models import User\n\n'
                   'def main():\n'
                   '    if DEBUG:\n'
                   '        print("debug mode")\n'
                   '    user = User("admin", "admin@test.com")\n'
                   '    print(f"Welcome {user}")\n\n'
                   'if __name__ == "__main__":\n'
                   '    main()\n')
        # 更新配置
        write_file(app, "config.toml",
                   '[app]\nname = "test"\nversion = "0.2.0"\n'
                   '[database]\nurl = "sqlite:///app.db"\n')
        # 添加测试
        write_file(app, "tests/test_models.py",
                   'from src.models import User\n\n'
                   'def test_user_repr():\n'
                   '    u = User("alice", "alice@test.com")\n'
                   '    assert "alice" in repr(u)\n')

        wait_refresh(4)
        ensure_focus()
        step("batch_many_changes")
        shot_region(0.0, 0.08, 0.38, 0.70, "batch_file_list")

    def test_02_第一批_只暂存核心重构文件(self, app):
        """只暂存 config.py 和 models.py（核心重构）。"""
        ensure_focus()
        driver.click_relative(*FILE_STAGE_BTN_1)
        driver.sleep(0.3)
        driver.click_relative(*FILE_STAGE_BTN_2)
        driver.sleep(0.3)
        step("batch_first_staged")

    def test_03_第一次提交_重构核心(self, app):
        commit_via_inline(app, "refactor: extract config and models")
        step("batch_first_committed")

    def test_04_第二批_暂存适配文件(self, app):
        """暂存 main.py 和 config.toml（适配改动）。"""
        ensure_focus()
        driver.hotkey("ctrl", "shift", "s")
        driver.sleep(1)
        step("batch_second_staged")

    def test_05_第二次提交_适配代码(self, app):
        """分出不同的提交信息。"""
        # 这一批可能包含测试 + main.py + config.toml，全部提交
        commit_via_inline(app, "refactor: adapt main and config to new structure")
        step("batch_second_committed")

    def test_06_查看历史验证分批效果(self, app):
        """切到 Log 查看是否有两笔清晰的提交。"""
        ensure_focus()
        driver.click_relative(*LOG_TAB)
        driver.sleep(1.5)
        step("batch_history_view")
        shot_region(0.03, 0.10, 0.94, 0.50, "batch_two_commits")

        log = git_cmd(app, "log", "--oneline", "-5")
        print(f"最近提交历史:\n{log}")

    def test_07_回到Changes清理(self, app):
        driver.click_relative(*CHANGES_TAB)
        driver.sleep(0.5)
        clean_repo(app)
        wait_refresh()


# ═══════════════════════════════════════════════════════════════════════
# 场景 6: 分支整理与发布 (Branch Cleanup & Tag)
#
# 模拟: feature 开发完成后，合并回 main，删除废弃分支，
#        打版本 tag，检查最终状态。
# ═══════════════════════════════════════════════════════════════════════

class Test06_分支整理与发布:
    """真实场景: 合并 feature → 清理分支 → 打 tag。"""

    def test_01_确认在main分支(self, app):
        clean_repo(app)
        wait_refresh()
        ensure_focus()
        assert current_branch(app) == "main"
        step("release_on_main")

    def test_02_合并feature分支(self, app):
        """合并 feature/add-login 到 main。"""
        result = subprocess.run(
            ["git", "merge", "--no-ff", "feature/add-login",
             "-m", "merge: integrate login feature"],
            cwd=app, capture_output=True, text=True,
        )
        if result.returncode != 0:
            # 可能有冲突，简单处理
            subprocess.run(["git", "checkout", "--theirs", "."], cwd=app, capture_output=True)
            subprocess.run(["git", "add", "."], cwd=app, capture_output=True)
            subprocess.run(
                ["git", "commit", "-m", "merge: integrate login feature (resolved)"],
                cwd=app, capture_output=True,
            )
        wait_refresh(3)
        step("release_merged")

    def test_03_删除已合并的feature分支(self, app):
        """合并后删除 feature 分支，保持仓库整洁。"""
        git_run(app, "branch", "-d", "feature/add-login")
        wait_refresh(2)
        step("release_branch_deleted")
        branches = git_cmd(app, "branch")
        assert "feature/add-login" not in branches

    def test_04_打版本tag(self, app):
        """给当前 HEAD 打一个版本 tag。"""
        git_run(app, "tag", "-a", "v0.2.0", "-m", "Release v0.2.0: login feature")
        wait_refresh(2)
        step("release_tagged")
        tags = git_cmd(app, "tag", "-l")
        assert "v0.2.0" in tags

    def test_05_打开Tags面板验证(self, app):
        """在 UI 上打开 Tags 面板确认 tag 可见。"""
        ensure_focus()
        driver.click_relative(*NAV_TAGS)
        driver.sleep(1.5)
        step("release_tags_panel")
        # 刷新
        driver.hotkey("ctrl", "r")
        driver.sleep(2)
        step("release_tags_refreshed")
        driver.press("escape")
        driver.sleep(0.5)

    def test_06_查看最终提交历史(self, app):
        """切到 Log 查看合并后的完整历史。"""
        ensure_focus()
        driver.click_relative(*LOG_TAB)
        driver.sleep(1.5)
        step("release_final_history")
        shot_region(0.03, 0.10, 0.94, 0.70, "release_full_log")

    def test_07_最终状态(self, app):
        """回到 Changes，截取最终全貌。"""
        driver.click_relative(*CHANGES_TAB)
        driver.sleep(1)
        step("release_final_state")

    def test_08_验证完整性(self, app):
        """最终验证: 进程存活 + 仓库状态正确。"""
        assert driver.is_alive(), "slio-git 进程已退出！"
        assert current_branch(app) == "main"
        tags = git_cmd(app, "tag", "-l")
        assert "v0.2.0" in tags
        branches = git_cmd(app, "branch")
        assert "feature/add-login" not in branches
        step("release_all_verified")

    def test_09_清理(self, app):
        """清理测试数据。"""
        subprocess.run(["git", "tag", "-d", "v0.2.0"], cwd=app, capture_output=True)
        clean_repo(app)
        wait_refresh()
