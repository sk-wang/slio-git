"""YAML 脚本 DSL — 按键精灵风格的声明式操作序列。"""

from .parser import parse_script, ScriptError
from .executor import execute_script, run
