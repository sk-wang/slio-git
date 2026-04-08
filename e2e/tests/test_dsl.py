"""DSL 解析器单元测试 — 不依赖 GUI。"""

import os
import tempfile

import pytest
from dsl.parser import parse_script, ScriptError, VALID_STEP_TYPES


def test_parse_simple_script():
    data = {
        "name": "测试脚本",
        "steps": [
            {"sleep": 0.1},
            {"type_text": "hello"},
        ]
    }
    script = parse_script(data)
    assert script.name == "测试脚本"
    assert len(script.steps) == 2
    assert script.steps[0].type == "sleep"
    assert script.steps[0].value == 0.1
    assert script.steps[1].type == "type_text"


def test_parse_from_yaml_file():
    with tempfile.NamedTemporaryFile(mode='w', suffix='.yaml', delete=False) as f:
        f.write("name: file_test\nsteps:\n  - sleep: 1\n  - type_text: hello\n")
        f.flush()
        script = parse_script(f.name)
        assert script.name == "file_test"
        assert len(script.steps) == 2
    os.unlink(f.name)


def test_unknown_step_type():
    data = {
        "name": "bad",
        "steps": [{"unknown_action": "foo"}]
    }
    with pytest.raises(ScriptError, match="未知操作类型"):
        parse_script(data)


def test_invalid_step_format():
    data = {
        "name": "bad",
        "steps": ["not a dict"]
    }
    with pytest.raises(ScriptError, match="单键字典"):
        parse_script(data)


def test_variable_substitution():
    data = {
        "name": "var_test",
        "steps": [
            {"type_text": "fix: ${MSG}"},
        ]
    }
    script = parse_script(data, variables={"MSG": "update readme"})
    assert script.steps[0].value == "fix: update readme"


def test_variable_from_env(monkeypatch):
    monkeypatch.setenv("E2E_VAR", "from_env")
    data = {
        "name": "env_test",
        "steps": [
            {"type_text": "${E2E_VAR}"},
        ]
    }
    script = parse_script(data)
    assert script.steps[0].value == "from_env"


def test_script_level_variables():
    data = {
        "name": "script_vars",
        "variables": {"BRANCH": "develop"},
        "steps": [
            {"type_text": "${BRANCH}"},
        ]
    }
    script = parse_script(data)
    assert script.steps[0].value == "develop"


def test_passed_variables_override_script_vars():
    data = {
        "name": "override",
        "variables": {"X": "script"},
        "steps": [
            {"type_text": "${X}"},
        ]
    }
    script = parse_script(data, variables={"X": "passed"})
    assert script.steps[0].value == "passed"


def test_unresolved_variable_kept():
    data = {
        "name": "unresolved",
        "steps": [
            {"type_text": "${NONEXISTENT_VAR_12345}"},
        ]
    }
    script = parse_script(data)
    assert script.steps[0].value == "${NONEXISTENT_VAR_12345}"


def test_all_valid_step_types_recognized():
    """确保所有声明的 step 类型都能被解析。"""
    for step_type in VALID_STEP_TYPES:
        data = {
            "name": f"test_{step_type}",
            "steps": [{step_type: "test_value"}]
        }
        script = parse_script(data)
        assert script.steps[0].type == step_type


def test_empty_steps():
    data = {"name": "empty", "steps": []}
    script = parse_script(data)
    assert len(script.steps) == 0


def test_call_action_dict_format():
    data = {
        "name": "action_test",
        "steps": [
            {"call_action": {"name": "switch_branch", "args": {"name": "develop"}}}
        ]
    }
    script = parse_script(data)
    assert script.steps[0].type == "call_action"
    assert script.steps[0].value["name"] == "switch_branch"


def test_hotkey_list_format():
    data = {
        "name": "hotkey_test",
        "steps": [
            {"hotkey": ["cmd", "enter"]}
        ]
    }
    script = parse_script(data)
    assert script.steps[0].value == ["cmd", "enter"]


def test_non_dict_root():
    with pytest.raises(ScriptError, match="字典格式"):
        parse_script(["not", "a", "dict"])
