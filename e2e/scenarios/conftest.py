"""
场景测试 conftest — 创建临时 git 仓库，通过 workspace memory 指向该仓库启动 app。

核心策略:
  1. session scope: 创建临时 git repo + 预置文件 + 初始提交
  2. 修改 workspace-memory-v1.txt 指向临时仓库
  3. 启动 slio-git（自动打开 last repo）
  4. 每个测试前激活窗口、确认进程存活
  5. session 结束后恢复原 workspace memory 并清理临时仓库
"""

import os
import shutil
import subprocess
import tempfile
import time

import pytest

import driver

# slio-git 持久化路径
WORKSPACE_MEMORY_DIR = os.path.expanduser("~/.local/share/slio-git")
WORKSPACE_MEMORY_FILE = os.path.join(WORKSPACE_MEMORY_DIR, "workspace-memory-v1.txt")
# macOS 也可能在这里
WORKSPACE_MEMORY_DIR_ALT = os.path.expanduser("~/Library/Application Support/slio-git")
WORKSPACE_MEMORY_FILE_ALT = os.path.join(WORKSPACE_MEMORY_DIR_ALT, "workspace-memory-v1.txt")

APP_BUNDLE = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "..", "dist", "slio-git.app",
)


def _find_workspace_memory_file():
    """找到实际使用的 workspace memory 文件路径。"""
    # 优先检查哪个目录已存在
    for path in [WORKSPACE_MEMORY_FILE, WORKSPACE_MEMORY_FILE_ALT]:
        if os.path.exists(path):
            return path
    # 都不存在，用 dirs::data_local_dir() 的默认路径
    # macOS: ~/Library/Application Support/
    return WORKSPACE_MEMORY_FILE_ALT


def _write_workspace_memory(repo_path: str, memory_file: str):
    """写入 workspace memory 指向指定仓库。"""
    os.makedirs(os.path.dirname(memory_file), exist_ok=True)
    with open(memory_file, "w") as f:
        f.write(f"last\t{repo_path}\n")
        f.write(f"recent\t{repo_path}\n")


@pytest.fixture(scope="session")
def test_repo():
    """创建临时 git 仓库，预置文件和初始提交。"""
    tmpdir = tempfile.mkdtemp(prefix="slio_e2e_")
    print(f"\n=== 临时测试仓库: {tmpdir} ===")

    # git init
    subprocess.run(["git", "init", tmpdir], capture_output=True, check=True)
    subprocess.run(
        ["git", "config", "user.email", "e2e@test.local"],
        cwd=tmpdir, capture_output=True, check=True,
    )
    subprocess.run(
        ["git", "config", "user.name", "E2E Test"],
        cwd=tmpdir, capture_output=True, check=True,
    )

    # 创建初始文件并提交
    readme = os.path.join(tmpdir, "README.md")
    with open(readme, "w") as f:
        f.write("# Test Repository\n\nCreated by slio-git E2E tests.\n")

    src_dir = os.path.join(tmpdir, "src")
    os.makedirs(src_dir)
    with open(os.path.join(src_dir, "main.py"), "w") as f:
        f.write('def main():\n    print("hello")\n\nif __name__ == "__main__":\n    main()\n')

    with open(os.path.join(tmpdir, "config.toml"), "w") as f:
        f.write('[app]\nname = "test"\nversion = "0.1.0"\n')

    subprocess.run(["git", "add", "."], cwd=tmpdir, capture_output=True, check=True)
    subprocess.run(
        ["git", "commit", "-m", "initial commit"],
        cwd=tmpdir, capture_output=True, check=True,
    )

    # 创建 develop 分支
    subprocess.run(
        ["git", "branch", "develop"],
        cwd=tmpdir, capture_output=True, check=True,
    )

    # 创建 feature/test 分支 (用于搜索测试)
    subprocess.run(
        ["git", "branch", "feature/test"],
        cwd=tmpdir, capture_output=True, check=True,
    )

    yield tmpdir

    # 清理
    shutil.rmtree(tmpdir, ignore_errors=True)


@pytest.fixture(scope="session")
def app(test_repo):
    """通过修改 workspace memory 启动 slio-git 打开临时测试仓库。"""
    # 备份原有 workspace memory
    memory_file = _find_workspace_memory_file()
    backup_file = memory_file + ".e2e_backup"
    if os.path.exists(memory_file):
        shutil.copy2(memory_file, backup_file)

    # 写入临时仓库路径
    _write_workspace_memory(test_repo, memory_file)

    # 杀掉已有实例
    subprocess.run(["pkill", "-x", driver.APP_PROCESS], capture_output=True)
    time.sleep(2)

    # 启动 app
    result = subprocess.run(["open", APP_BUNDLE], capture_output=True, text=True)
    if result.returncode != 0:
        # 恢复备份
        if os.path.exists(backup_file):
            shutil.move(backup_file, memory_file)
        pytest.fail(f"启动失败: {result.stderr}")

    time.sleep(5)
    driver.prepare()

    yield test_repo

    # 关闭 app
    subprocess.run(["pkill", "-x", driver.APP_PROCESS], capture_output=True)
    time.sleep(1)

    # 恢复原有 workspace memory
    if os.path.exists(backup_file):
        shutil.move(backup_file, memory_file)


@pytest.fixture(autouse=True)
def _per_test_setup(app):
    """每个测试前: 确认进程存活 + 激活窗口。"""
    assert driver.is_alive(), "slio-git 进程已退出"
    driver.activate()
    driver.sleep(0.3)


def add_unstaged_change(repo_path: str, filename: str = "src/main.py", content: str = None):
    """工具函数：向测试仓库添加未暂存的变更。"""
    filepath = os.path.join(repo_path, filename)
    if content is None:
        with open(filepath, "a") as f:
            f.write(f"\n# change at {time.time()}\n")
    else:
        os.makedirs(os.path.dirname(filepath), exist_ok=True)
        with open(filepath, "w") as f:
            f.write(content)
