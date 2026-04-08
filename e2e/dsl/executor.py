"""DSL 执行器 — 将 step 类型映射到 Driver/Action 方法并执行。"""

import importlib
from typing import Dict

import driver
from .parser import Script, Step, ScriptError, parse_script


def _execute_step(step: Step, script_file: str = ""):
    """执行单个步骤。"""
    t = step.type
    v = step.value

    if t == "click_image":
        driver.click_image(v)

    elif t == "click_at":
        if isinstance(v, dict):
            driver.click_relative(v["rx"], v["ry"])
        elif isinstance(v, list) and len(v) == 2:
            driver.click(v[0], v[1])
        else:
            raise ScriptError(f"click_at 格式错误: {v}", file=script_file, line=step.line)

    elif t == "type_text":
        driver.type_text(str(v))

    elif t == "hotkey":
        if isinstance(v, list):
            driver.hotkey(*v)
        elif isinstance(v, str):
            driver.hotkey(*v.split("+"))
        else:
            raise ScriptError(f"hotkey 格式错误: {v}", file=script_file, line=step.line)

    elif t == "wait_image":
        if isinstance(v, dict):
            driver.wait_image(v["image"], timeout=v.get("timeout", 5))
        else:
            driver.wait_image(v)

    elif t == "wait_disappear":
        if isinstance(v, dict):
            driver.wait_disappear(v["image"], timeout=v.get("timeout", 5))
        else:
            driver.wait_disappear(v)

    elif t == "screenshot":
        if isinstance(v, dict):
            save_as = v.get("save_as", "dsl_screenshot")
            expect = v.get("expect_image")
            if expect:
                driver.screen.assert_screenshot(save_as, expect, threshold=v.get("threshold", 0.05))
            else:
                driver.window_screenshot(save_as)
        else:
            driver.window_screenshot(str(v))

    elif t == "sleep":
        driver.sleep(float(v))

    elif t == "call_action":
        _call_action(v, script_file, step.line)


def _call_action(value, script_file: str, line: int):
    """动态调用 Action 层函数。"""
    if isinstance(value, dict):
        name = value.get("name")
        args = value.get("args", {})
    elif isinstance(value, str):
        name = value
        args = {}
    else:
        raise ScriptError(f"call_action 格式错误: {value}", file=script_file, line=line)

    # 从 actions 模块动态查找函数
    try:
        actions_mod = importlib.import_module("actions")
        func = getattr(actions_mod, name)
    except (ImportError, AttributeError):
        raise ScriptError(
            f"Action '{name}' 不存在。请检查 actions/__init__.py 中的导出。",
            file=script_file, line=line,
        )

    if isinstance(args, dict):
        func(**args)
    else:
        func(args)


def execute_script(script: Script):
    """执行已解析的脚本。"""
    for i, step in enumerate(script.steps, 1):
        try:
            _execute_step(step, script.file)
        except ScriptError:
            raise
        except Exception as e:
            raise ScriptError(
                f"步骤 {i} ({step.type}) 执行失败: {e}",
                file=script.file, line=step.line,
            ) from e


def run(source, variables: Dict[str, str] = None):
    """解析并执行 YAML 脚本（一站式入口）。"""
    script = parse_script(source, variables)
    execute_script(script)
