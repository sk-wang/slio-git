"""Action 基础设施 — 失败自动截图装饰器。"""

import functools
import os
import time

from driver import screen


def auto_screenshot_on_failure(func):
    """装饰器：Action 失败时自动截取当前屏幕状态。"""
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except Exception:
            os.makedirs(screen.OUTPUT_DIR, exist_ok=True)
            timestamp = int(time.time())
            name = f"failure_{func.__name__}_{timestamp}"
            try:
                screen.fullscreen(name)
            except Exception:
                pass  # 截图失败不应掩盖原始异常
            raise
    return wrapper
