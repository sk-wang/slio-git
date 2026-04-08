"""YAML 脚本解析器 — 校验 step 类型，报告错误位置。"""

import os
import re
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Union

import yaml


class ScriptError(Exception):
    """脚本解析或执行错误，包含文件名和行号。"""
    def __init__(self, message: str, file: str = "", line: int = 0):
        self.file = file
        self.line = line
        loc = f"{file}:{line}" if file and line else (file or "")
        super().__init__(f"{loc}: {message}" if loc else message)


# 支持的 step 类型
VALID_STEP_TYPES = {
    "click_image",
    "click_at",
    "type_text",
    "hotkey",
    "wait_image",
    "wait_disappear",
    "screenshot",
    "sleep",
    "call_action",
}


@dataclass
class Step:
    """一个操作步骤。"""
    type: str
    value: Any
    line: int = 0


@dataclass
class Script:
    """解析后的脚本。"""
    name: str
    steps: List[Step]
    variables: Dict[str, str] = field(default_factory=dict)
    file: str = ""


def _substitute_vars(value: Any, variables: Dict[str, str]) -> Any:
    """递归替换 ${VAR} 变量。"""
    if isinstance(value, str):
        def replacer(match):
            var_name = match.group(1)
            if var_name in variables:
                return variables[var_name]
            # 回退到环境变量
            env_val = os.environ.get(var_name)
            if env_val is not None:
                return env_val
            return match.group(0)  # 保持原样
        return re.sub(r'\$\{(\w+)\}', replacer, value)
    elif isinstance(value, dict):
        return {k: _substitute_vars(v, variables) for k, v in value.items()}
    elif isinstance(value, list):
        return [_substitute_vars(v, variables) for v in value]
    return value


def parse_script(source: Union[str, dict], variables: Dict[str, str] = None) -> Script:
    """解析 YAML 脚本文件或字典。

    Args:
        source: YAML 文件路径或已解析的字典
        variables: 变量替换映射
    """
    variables = variables or {}
    file_path = ""

    if isinstance(source, str):
        file_path = source
        with open(source, 'r', encoding='utf-8') as f:
            raw = f.read()
        try:
            data = yaml.safe_load(raw)
        except yaml.YAMLError as e:
            raise ScriptError(f"YAML 解析错误: {e}", file=file_path)
    else:
        data = source

    if not isinstance(data, dict):
        raise ScriptError("脚本必须是 YAML 字典格式", file=file_path)

    name = data.get("name", "unnamed")
    raw_steps = data.get("steps", [])

    if not isinstance(raw_steps, list):
        raise ScriptError("steps 必须是列表", file=file_path)

    # 合并脚本级变量
    script_vars = data.get("variables", {})
    if isinstance(script_vars, dict):
        merged_vars = {**script_vars, **variables}  # 传入变量优先
    else:
        merged_vars = variables

    steps = []
    for i, raw_step in enumerate(raw_steps, 1):
        if not isinstance(raw_step, dict) or len(raw_step) != 1:
            raise ScriptError(
                f"步骤 {i}: 每个 step 必须是单键字典 (如 `- click_image: xxx`)",
                file=file_path, line=i,
            )

        step_type = list(raw_step.keys())[0]
        step_value = raw_step[step_type]

        if step_type not in VALID_STEP_TYPES:
            raise ScriptError(
                f"步骤 {i}: 未知操作类型 '{step_type}'。"
                f"支持: {', '.join(sorted(VALID_STEP_TYPES))}",
                file=file_path, line=i,
            )

        # 变量替换
        step_value = _substitute_vars(step_value, merged_vars)

        steps.append(Step(type=step_type, value=step_value, line=i))

    return Script(name=name, steps=steps, variables=merged_vars, file=file_path)
