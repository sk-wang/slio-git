"""
slio-git E2E conftest — 按键精灵风格 (v2)

使用新 Driver 层，保持 session-scoped app fixture 兼容旧测试。

fixture 负责:
  1. 启动 app (session scope, 通过 open 命令)
  2. 每个 test 前激活窗口
  3. 失败时自动截图
"""

import os
import subprocess
import time

import pytest
import driver


APP_BUNDLE = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "..", "dist", "slio-git.app",
)


@pytest.fixture(scope="session")
def app():
    """启动 slio-git，测试结束后关闭。"""
    subprocess.run(["pkill", "-x", driver.APP_PROCESS], capture_output=True)
    time.sleep(2)

    result = subprocess.run(["open", APP_BUNDLE], capture_output=True, text=True)
    if result.returncode != 0:
        pytest.fail(f"启动失败: {result.stderr}")
    time.sleep(5)

    driver.prepare()

    yield

    subprocess.run(["pkill", "-x", driver.APP_PROCESS], capture_output=True)


@pytest.fixture(autouse=True)
def _per_test_setup(app):
    """每个测试前: 确认进程存活 + 激活窗口。"""
    assert driver.is_alive(), "slio-git 进程已退出"
    driver.activate()
    driver.sleep(0.3)


# === 失败诊断 ===

# 操作日志（由 pytest hook 收集）
_step_log = []


def log_step(action: str, detail: str = ""):
    """记录操作步骤（供诊断报告使用）。"""
    _step_log.append({
        "time": time.time(),
        "action": action,
        "detail": detail,
    })
    # 只保留最近 50 步
    if len(_step_log) > 50:
        _step_log.pop(0)


@pytest.hookimpl(hookwrapper=True)
def pytest_runtest_makereport(item, call):
    """测试失败时自动截图并保存诊断报告。"""
    outcome = yield
    report = outcome.get_result()

    if report.when == "call" and report.failed:
        os.makedirs(driver.OUTPUT_DIR, exist_ok=True)
        timestamp = int(time.time())
        test_name = item.name.replace(" ", "_")

        # 失败截图
        try:
            driver.fullscreen(f"failure_{test_name}_{timestamp}")
        except Exception:
            pass

        # 操作日志
        log_path = os.path.join(driver.OUTPUT_DIR, f"log_{test_name}_{timestamp}.txt")
        last_steps = _step_log[-5:] if _step_log else []
        with open(log_path, "w") as f:
            f.write(f"Test: {item.nodeid}\n")
            f.write(f"Error: {call.excinfo}\n\n")
            f.write("=== Last 5 Steps ===\n")
            for step in last_steps:
                f.write(f"  [{step['action']}] {step['detail']}\n")
