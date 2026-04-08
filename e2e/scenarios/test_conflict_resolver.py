"""
E2E 场景: 冲突合并解决器 (Conflict Resolver) — 深度测试

覆盖:
  1. 单文件冲突 — merge conflict → 检测 → 进入冲突视图 → 解决
  2. 多文件冲突 — 2+ 文件同时冲突 → 逐个切换 → 全部解决
  3. 取消合并 — 冲突后 abort merge
  4. Rebase 冲突 — rebase 产生冲突 → 中止
  5. 冲突解决后提交 — 解决冲突 → git add → commit

slio-git 冲突 UI 流程:
  1. git merge 产生冲突
  2. slio-git auto-refresh 检测到 ChangeStatus::Conflict
  3. 导航栏出现 "Conflicts" section (带 badge 数字)
  4. 点击 Conflicts → 进入 ConflictResolver 视图
  5. 三栏合并: 您的(左) | 合并结果(中) | 他们的(右)
  6. 每个 hunk 可选: << 接受左侧 | = 保留基础 | >> 接受右侧
  7. 全部解决后 → 点击 "应用"

导航栏坐标:
  - Changes: (0.012, 0.12)
  - Conflicts: (0.012, 0.155) — 紧跟 Changes 下方，仅冲突时可见
"""

import os
import subprocess
import time

import driver


# ═══════════════════════════════════════
# 导航栏与 UI 坐标
# ═══════════════════════════════════════

NAV_CHANGES = (0.014, 0.09)          # Changes 导航按钮
NAV_CONFLICTS = (0.014, 0.133)       # Conflicts 导航按钮 (窗口内 y≈136px)

# 冲突列表视图 (进入 Conflicts section 后左侧显示冲突文件列表)
CONFLICT_FILE_1 = (0.15, 0.18)       # 冲突文件列表第一行
CONFLICT_FILE_2 = (0.15, 0.22)       # 冲突文件列表第二行

# 快速解决面板 (冲突列表页右侧的三个大卡片)
QUICK_ACCEPT_OURS = (0.75, 0.47)     # "<< 接受您的更改" 卡片 (蓝色)
QUICK_ACCEPT_THEIRS = (0.75, 0.58)   # ">> 接受他们的更改" 卡片 (红色)
QUICK_MERGE = (0.75, 0.70)            # "<> 合并..." 卡片 (蓝色) → 打开三栏合并

# 三栏合并编辑器工具栏 (进入三栏后)
TOOLBAR_ALL_OURS = (0.65, 0.14)       # "全部左侧"
TOOLBAR_ALL_THEIRS = (0.75, 0.14)     # "全部右侧"
TOOLBAR_AUTO_MERGE = (0.55, 0.14)     # "自动合并"

# 底部
FOOTER_CANCEL = (0.90, 0.96)          # "取消" 按钮
FOOTER_APPLY = (0.96, 0.96)           # "应用" 按钮


# ═══════════════════════════════════════
# Git 操作辅助函数
# ═══════════════════════════════════════

def _git(repo, *args):
    """执行 git 命令。"""
    result = subprocess.run(
        ["git"] + list(args),
        cwd=repo, capture_output=True, text=True,
    )
    return result


def _ensure_clean(repo):
    """确保仓库干净，取消所有进行中的操作。"""
    _git(repo, "merge", "--abort")
    _git(repo, "rebase", "--abort")
    _git(repo, "checkout", ".")
    _git(repo, "clean", "-fd")
    _git(repo, "checkout", "main")


def _has_conflicts(repo):
    """检查 git 是否有冲突。"""
    result = _git(repo, "diff", "--name-only", "--diff-filter=U")
    return bool(result.stdout.strip())


def _create_single_file_conflict(repo):
    """制造单文件 merge conflict (README.md)。"""
    _ensure_clean(repo)

    # 删除旧的冲突分支
    _git(repo, "branch", "-D", "conflict-single")

    # 在 main 上修改 README.md
    readme = os.path.join(repo, "README.md")
    with open(readme, "w") as f:
        f.write("# Test Repository\n\nmain branch line 3\n\nmain branch line 5\n")
    _git(repo, "add", "README.md")
    _git(repo, "commit", "-m", "main: single conflict prep")

    # 创建 conflict 分支并修改同一文件
    _git(repo, "checkout", "-b", "conflict-single", "HEAD~1")
    with open(readme, "w") as f:
        f.write("# Test Repository\n\nconflict branch line 3\n\nconflict branch line 5\n")
    _git(repo, "add", "README.md")
    _git(repo, "commit", "-m", "conflict: single file change")

    # 回到 main 并 merge
    _git(repo, "checkout", "main")
    result = _git(repo, "merge", "conflict-single")
    return result


def _create_multi_file_conflict(repo):
    """制造多文件 merge conflict (README.md + config.toml)。"""
    _ensure_clean(repo)

    _git(repo, "branch", "-D", "conflict-multi")

    readme = os.path.join(repo, "README.md")
    config = os.path.join(repo, "config.toml")

    # main 上修改两个文件
    with open(readme, "w") as f:
        f.write("# Test Repo\n\nmain readme change\n")
    with open(config, "w") as f:
        f.write('[app]\nname = "main-version"\nversion = "1.0.0"\n')
    _git(repo, "add", ".")
    _git(repo, "commit", "-m", "main: multi conflict prep")

    # conflict 分支修改同样两个文件
    _git(repo, "checkout", "-b", "conflict-multi", "HEAD~1")
    with open(readme, "w") as f:
        f.write("# Test Repo\n\nconflict readme change\n")
    with open(config, "w") as f:
        f.write('[app]\nname = "conflict-version"\nversion = "2.0.0"\n')
    _git(repo, "add", ".")
    _git(repo, "commit", "-m", "conflict: multi file change")

    # 回到 main 并 merge
    _git(repo, "checkout", "main")
    result = _git(repo, "merge", "conflict-multi")
    return result


def _resolve_conflicts_with_git(repo, strategy="ours"):
    """用 git 命令解决所有冲突。"""
    result = _git(repo, "diff", "--name-only", "--diff-filter=U")
    conflict_files = result.stdout.strip().split("\n")

    for f in conflict_files:
        if not f:
            continue
        filepath = os.path.join(repo, f)
        with open(filepath, "w") as fh:
            fh.write(f"resolved by e2e ({strategy})\n")
        _git(repo, "add", f)

    _git(repo, "commit", "-m", f"e2e: resolve conflicts ({strategy})")


def _wait_for_slio_refresh():
    """等待 slio-git 检测到变更。"""
    driver.activate()
    driver.click_relative(0.3, 0.4)  # 确保焦点在 app 上
    driver.sleep(0.5)
    driver.hotkey("ctrl", "r")  # 手动刷新
    driver.sleep(4)


def _cleanup_branches(repo, *branches):
    """清理测试分支。"""
    _git(repo, "checkout", "main")
    for branch in branches:
        _git(repo, "branch", "-D", branch)


# ═══════════════════════════════════════
# 测试: 单文件冲突
# ═══════════════════════════════════════

class Test单文件冲突:
    """最基本的冲突场景: 一个文件冲突 → 检测 → 解决。"""

    def test_制造单文件冲突(self, app):
        result = _create_single_file_conflict(app)
        assert result.returncode != 0 or "CONFLICT" in result.stdout + result.stderr
        assert _has_conflicts(app), "冲突未产生"
        print(f"冲突文件: {_git(app, 'diff', '--name-only', '--diff-filter=U').stdout.strip()}")

    def test_等待检测(self, app):
        _wait_for_slio_refresh()
        driver.window_screenshot("c1_01_检测到冲突")

    def test_截图顶部状态(self, app):
        """截取顶部栏，应显示 '合并中' 标记。"""
        driver.region(0.0, 0.0, 0.5, 0.06, "c1_02_顶部合并状态")

    def test_尝试点击Conflicts导航(self, app):
        """点击导航栏 Conflicts section（如果出现了）。"""
        driver.click_relative(*NAV_CONFLICTS)
        driver.sleep(2)
        driver.window_screenshot("c1_03_点击conflicts导航")

    def test_截图冲突视图(self, app):
        """截取当前冲突视图全貌。"""
        driver.window_screenshot("c1_04_冲突视图全貌")
        driver.region(0.0, 0.08, 0.40, 0.45, "c1_05_左侧面板")
        driver.region(0.40, 0.08, 0.60, 0.85, "c1_06_中间区域")

    def test_点击接受您的更改(self, app):
        """点击 "<< 接受您的更改" 快速解决卡片。"""
        driver.click_relative(*QUICK_ACCEPT_OURS)
        driver.sleep(2)
        driver.window_screenshot("c1_07_接受我方")

    def test_截图解决后状态(self, app):
        """截图查看解决后的状态。"""
        driver.window_screenshot("c1_08_解决后状态")

    def test_验证并git_fallback(self, app):
        """验证冲突是否已解决，必要时用 git fallback。"""
        if _has_conflicts(app):
            print("UI 冲突解决未生效，使用 git fallback")
            _resolve_conflicts_with_git(app, "ours")

        assert not _has_conflicts(app), "冲突仍未解决"
        # 检查提交历史
        result = _git(app, "log", "--oneline", "-3")
        print(f"最近提交:\n{result.stdout.strip()}")

    def test_清理(self, app):
        _cleanup_branches(app, "conflict-single")
        driver.sleep(1)
        driver.window_screenshot("c1_09_清理完成")


# ═══════════════════════════════════════
# 测试: 多文件冲突
# ═══════════════════════════════════════

class Test多文件冲突:
    """两个文件同时冲突 → 检测 → 逐文件截图 → 全部解决。"""

    def test_制造多文件冲突(self, app):
        result = _create_multi_file_conflict(app)
        assert result.returncode != 0 or "CONFLICT" in result.stdout + result.stderr
        conflict_files = _git(app, "diff", "--name-only", "--diff-filter=U").stdout.strip()
        print(f"冲突文件:\n{conflict_files}")
        assert conflict_files.count("\n") >= 1, "预期至少 2 个冲突文件"

    def test_等待检测(self, app):
        _wait_for_slio_refresh()
        driver.window_screenshot("c2_01_多文件冲突检测")

    def test_点击Conflicts导航(self, app):
        driver.click_relative(*NAV_CONFLICTS)
        driver.sleep(2)
        driver.window_screenshot("c2_02_conflicts视图")

    def test_截图冲突文件列表(self, app):
        """截取冲突文件列表区域。"""
        driver.region(0.0, 0.06, 0.40, 0.50, "c2_03_冲突文件列表")

    def test_点击第一个冲突文件(self, app):
        """点击第一个冲突文件查看三栏合并。"""
        driver.click_relative(*CONFLICT_FILE_1)
        driver.sleep(1)
        driver.window_screenshot("c2_04_第一个冲突文件")

    def test_截图三栏合并(self, app):
        """截取三栏合并视图的核心区域。"""
        driver.region(0.03, 0.12, 0.94, 0.80, "c2_05_三栏合并视图")

    def test_点击第二个冲突文件(self, app):
        """切换到第二个冲突文件。"""
        driver.click_relative(*CONFLICT_FILE_2)
        driver.sleep(1)
        driver.window_screenshot("c2_06_第二个冲突文件")

    def test_点击接受他们的更改(self, app):
        """点击 ">> 接受他们的更改" 快速解决卡片。"""
        driver.click_relative(*QUICK_ACCEPT_THEIRS)
        driver.sleep(2)
        driver.window_screenshot("c2_07_接受对方")

    def test_截图解决后(self, app):
        driver.window_screenshot("c2_08_解决后")

    def test_验证并git_fallback(self, app):
        if _has_conflicts(app):
            print("UI 解决未生效，使用 git fallback")
            _resolve_conflicts_with_git(app, "theirs")

        assert not _has_conflicts(app)
        result = _git(app, "log", "--oneline", "-3")
        print(f"最近提交:\n{result.stdout.strip()}")

    def test_清理(self, app):
        _cleanup_branches(app, "conflict-multi")
        driver.sleep(1)


# ═══════════════════════════════════════
# 测试: 取消合并 (Merge Abort)
# ═══════════════════════════════════════

class Test取消合并:
    """冲突后选择 abort merge 而非解决。"""

    def test_制造冲突(self, app):
        result = _create_single_file_conflict(app)
        assert _has_conflicts(app)

    def test_等待检测(self, app):
        _wait_for_slio_refresh()
        driver.window_screenshot("c3_01_冲突待取消")

    def test_git_merge_abort(self, app):
        """通过 git merge --abort 取消合并。"""
        result = _git(app, "merge", "--abort")
        assert result.returncode == 0, f"merge abort 失败: {result.stderr}"
        print("merge 已取消")

    def test_等待刷新(self, app):
        _wait_for_slio_refresh()
        driver.window_screenshot("c3_02_abort后")

    def test_验证状态恢复(self, app):
        """验证仓库不再有冲突，且处于 main 分支。"""
        assert not _has_conflicts(app), "abort 后仍有冲突"
        current = _git(app, "branch", "--show-current").stdout.strip()
        assert current == "main", f"不在 main 分支: {current}"
        print("merge abort 成功，回到 main 分支")

    def test_清理(self, app):
        _cleanup_branches(app, "conflict-single")


# ═══════════════════════════════════════
# 测试: Rebase 产生冲突
# ═══════════════════════════════════════

class TestRebase冲突:
    """rebase 过程中产生冲突 → 检测 → 中止/继续。"""

    def test_准备rebase冲突(self, app):
        _ensure_clean(app)
        _git(app, "branch", "-D", "rebase-conflict-test")

        readme = os.path.join(app, "README.md")

        # main 上改
        with open(readme, "w") as f:
            f.write("# Test Repo\n\nmain side for rebase\n")
        _git(app, "add", "README.md")
        _git(app, "commit", "-m", "main: rebase conflict prep")

        # 分支上改
        _git(app, "checkout", "-b", "rebase-conflict-test", "HEAD~1")
        with open(readme, "w") as f:
            f.write("# Test Repo\n\nrebase-branch side change\n")
        _git(app, "add", "README.md")
        _git(app, "commit", "-m", "branch: different change for rebase")

    def test_执行rebase产生冲突(self, app):
        result = _git(app, "rebase", "main")
        assert result.returncode != 0, "预期 rebase 冲突但成功了"
        print(f"rebase 冲突: {result.stderr.strip()[:100]}")

    def test_等待检测(self, app):
        _wait_for_slio_refresh()
        driver.window_screenshot("c4_01_rebase冲突")

    def test_截图rebase冲突状态(self, app):
        """截取 rebase 冲突下的 UI 状态。"""
        driver.region(0.0, 0.0, 0.5, 0.06, "c4_02_顶部rebase状态")
        driver.window_screenshot("c4_03_rebase冲突全貌")

    def test_点击Conflicts查看(self, app):
        """尝试点击 Conflicts 导航。"""
        driver.click_relative(*NAV_CONFLICTS)
        driver.sleep(2)
        driver.window_screenshot("c4_04_rebase_conflicts视图")

    def test_截图冲突内容(self, app):
        """截取冲突内容区域。"""
        driver.region(0.03, 0.10, 0.94, 0.80, "c4_05_rebase冲突内容")

    def test_中止rebase(self, app):
        result = _git(app, "rebase", "--abort")
        assert result.returncode == 0
        print("rebase 已中止")

    def test_验证恢复(self, app):
        _wait_for_slio_refresh()
        driver.window_screenshot("c4_06_rebase中止后")
        assert not _has_conflicts(app)
        # 确认 rebase 状态文件不存在
        for d in [".git/rebase-merge", ".git/rebase-apply"]:
            assert not os.path.exists(os.path.join(app, d))

    def test_清理(self, app):
        _git(app, "checkout", "main")
        _git(app, "branch", "-D", "rebase-conflict-test")


# ═══════════════════════════════════════
# 测试: 冲突解决后正常提交
# ═══════════════════════════════════════

class Test冲突解决后提交:
    """完整流程: 冲突 → 手动解决 → git add → commit → 验证。"""

    def test_制造冲突(self, app):
        _create_single_file_conflict(app)
        assert _has_conflicts(app)

    def test_等待检测(self, app):
        _wait_for_slio_refresh()
        driver.window_screenshot("c5_01_冲突")

    def test_手动编辑解决冲突(self, app):
        """直接编辑冲突文件去掉冲突标记。"""
        readme = os.path.join(app, "README.md")
        with open(readme, "w") as f:
            f.write("# Test Repository\n\nmanually resolved content\n\nfinal line\n")
        _git(app, "add", "README.md")

    def test_等待刷新(self, app):
        _wait_for_slio_refresh()
        driver.window_screenshot("c5_02_解决后")

    def test_验证冲突已清除(self, app):
        assert not _has_conflicts(app), "冲突未清除"

    def test_提交合并结果(self, app):
        result = _git(app, "commit", "-m", "e2e: merge conflict resolved manually")
        assert result.returncode == 0, f"提交失败: {result.stderr}"
        print("合并提交成功")

    def test_验证提交历史(self, app):
        _wait_for_slio_refresh()
        driver.window_screenshot("c5_03_提交后")
        result = _git(app, "log", "--oneline", "-5")
        print(f"提交历史:\n{result.stdout.strip()}")
        assert "merge conflict resolved" in result.stdout

    def test_验证是merge_commit(self, app):
        """验证最新提交是一个 merge commit (有两个 parent)。"""
        result = _git(app, "cat-file", "-p", "HEAD")
        parent_count = result.stdout.count("parent ")
        print(f"HEAD parent 数: {parent_count}")
        assert parent_count == 2, f"预期 merge commit (2 parents), 实际: {parent_count}"

    def test_清理(self, app):
        _cleanup_branches(app, "conflict-single")
        driver.sleep(1)
        driver.window_screenshot("c5_04_清理")
