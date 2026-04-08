"""窗口管理 — 激活、定位、最大化、进程检测。"""

import subprocess
import time
from dataclasses import dataclass
from typing import Tuple

APP_PROCESS = "slio-git"


@dataclass
class Rect:
    x: int
    y: int
    w: int
    h: int

    @property
    def center(self) -> Tuple[int, int]:
        return (self.x + self.w // 2, self.y + self.h // 2)

    @property
    def right(self) -> int:
        return self.x + self.w

    @property
    def bottom(self) -> int:
        return self.y + self.h


def activate():
    """将 slio-git 窗口置于最前。"""
    r = subprocess.run(
        ["osascript", "-e", f'''
            tell application "System Events"
                set frontmost of (first process whose name is "{APP_PROCESS}") to true
            end tell
        '''],
        capture_output=True,
    )
    if r.returncode != 0:
        subprocess.run(
            ["osascript", "-e", f'tell application "{APP_PROCESS}" to activate'],
            capture_output=True,
        )
    time.sleep(0.3)


def get_bounds() -> Rect:
    """获取 slio-git 窗口的位置和尺寸。"""
    result = subprocess.run(
        ["osascript", "-e", f'''
            tell application "System Events"
                tell (first process whose name is "{APP_PROCESS}")
                    set win to first window
                    set pos to position of win
                    set sz to size of win
                    return (item 1 of pos) & "," & (item 2 of pos) & "," & (item 1 of sz) & "," & (item 2 of sz)
                end tell
            end tell
        '''],
        capture_output=True, text=True,
    )
    parts = result.stdout.strip().split(",")
    if len(parts) == 4:
        vals = [int(p.strip()) for p in parts]
        return Rect(*vals)

    try:
        import Quartz
        window_list = Quartz.CGWindowListCopyWindowInfo(
            Quartz.kCGWindowListOptionOnScreenOnly, Quartz.kCGNullWindowID
        )
        for win in window_list:
            owner = win.get("kCGWindowOwnerName", "")
            name = win.get("kCGWindowName", "")
            if APP_PROCESS in owner or APP_PROCESS in name:
                bounds = win.get("kCGWindowBounds", {})
                if bounds:
                    return Rect(
                        int(bounds["X"]), int(bounds["Y"]),
                        int(bounds["Width"]), int(bounds["Height"]),
                    )
    except ImportError:
        pass

    raise RuntimeError("无法获取窗口区域: AppleScript 和 CGWindowList 均失败")


def maximize():
    """将窗口尽量铺满屏幕（不用全屏模式，避免动画延迟）。"""
    subprocess.run(
        ["osascript", "-e", f'''
            tell application "System Events"
                tell (first process whose name is "{APP_PROCESS}")
                    set win to first window
                    set position of win to {{0, 25}}
                    set size of win to {{1728, 1080}}
                end tell
            end tell
        '''],
        capture_output=True,
    )
    time.sleep(0.3)


def prepare():
    """激活 + 最大化 + 等待渲染。"""
    activate()
    maximize()
    time.sleep(0.5)


def is_alive() -> bool:
    """检查 slio-git 进程是否存活。"""
    result = subprocess.run(["pgrep", "-f", APP_PROCESS], capture_output=True)
    return result.returncode == 0
