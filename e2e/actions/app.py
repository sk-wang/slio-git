"""应用生命周期管理 — 启动、退出、重启。"""

import os
import subprocess
import time

from driver import window
from .base import auto_screenshot_on_failure

APP_BUNDLE = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "..", "dist", "slio-git.app",
)


@auto_screenshot_on_failure
def launch_app(repo_path: str = None):
    """启动 slio-git。可选传入仓库路径。"""
    subprocess.run(["pkill", "-x", window.APP_PROCESS], capture_output=True)
    time.sleep(2)

    cmd = ["open", APP_BUNDLE]
    if repo_path:
        cmd.extend(["--args", repo_path])

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"启动失败: {result.stderr}")

    time.sleep(5)
    window.prepare()


@auto_screenshot_on_failure
def quit_app():
    """关闭 slio-git。"""
    subprocess.run(["pkill", "-x", window.APP_PROCESS], capture_output=True)
    time.sleep(1)


@auto_screenshot_on_failure
def restart_app(repo_path: str = None):
    """重启 slio-git。"""
    quit_app()
    time.sleep(1)
    launch_app(repo_path)


@auto_screenshot_on_failure
def wait_app_ready(timeout: float = 10):
    """等待应用就绪（进程存活 + 窗口可获取）。"""
    deadline = time.time() + timeout
    while time.time() < deadline:
        if window.is_alive():
            try:
                rect = window.get_bounds()
                if rect.w > 100 and rect.h > 100:
                    return
            except RuntimeError:
                pass
        time.sleep(0.5)
    raise TimeoutError(f"应用未在 {timeout}s 内就绪")
