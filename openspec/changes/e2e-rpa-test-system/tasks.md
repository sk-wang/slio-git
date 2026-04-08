## 1. Driver 层重构

- [x] 1.1 创建 `e2e/driver/` 目录，将 `rpa.py` 中窗口管理、截图、鼠标、键盘操作拆分为 `window.py`、`screen.py`、`mouse.py`、`keyboard.py`、`image_match.py`
- [x] 1.2 实现 `image_match.py`：图像查找（confidence 阈值 + 灰度模式）、区域限定搜索、`ImageNotFoundError` 异常
- [x] 1.3 实现 `screen.py`：全屏截图、区域截图、截图对比（差异百分比 + 差异高亮图生成）
- [x] 1.4 创建 `e2e/driver/__init__.py` 暴露统一 `Driver` 类，聚合所有子模块方法
- [x] 1.5 为 Driver 层编写单元测试 `e2e/tests/test_driver.py`

## 2. Action 层实现

- [x] 2.1 创建 `e2e/actions/` 目录结构，定义 `base.py`（Action 基类，含失败自动截图逻辑）
- [x] 2.2 实现 `toolbar.py`：`click_refresh()`、`open_commit_dialog()`、`open_settings()` 等工具栏操作
- [x] 2.3 实现 `branch.py`：`open_branch_popup()`、`search_branch(name)`、`switch_branch(name)`、`close_branch_popup()`
- [x] 2.4 实现 `commit.py`：`open_commit_dialog()`、`type_commit_message(msg)`、`confirm_commit()`、`cancel_commit()`
- [x] 2.5 实现 `stash.py`：`stash_changes()`、`pop_stash()`
- [x] 2.6 实现 `app.py`：`launch_app()`、`quit_app()`、`restart_app()`、`wait_app_ready()`

## 3. 参考图管理

- [x] 3.1 创建 `e2e/refs/` 目录结构：`toolbar/`、`branch_popup/`、`commit_dialog/`、`diff_editor/`、`settings/`、`common/`
- [ ] 3.2 截取并存储核心 UI 元素参考图（工具栏按钮、对话框标题、分支弹窗标识等）<!-- 需 GUI 环境手动截取 -->
- [x] 3.3 编写 `e2e/refs/README.md` 说明参考图命名规范和更新流程

## 4. YAML 脚本 DSL

- [x] 4.1 创建 `e2e/dsl/` 目录，实现 `parser.py`（YAML 解析 + step 类型校验 + 行号错误报告）
- [x] 4.2 实现 `executor.py`（step 类型到 Driver/Action 方法的映射与执行）
- [x] 4.3 实现变量替换引擎（`${VAR}` 语法，支持环境变量和脚本参数）
- [x] 4.4 实现 `call_action` 指令（动态调用 Action 层函数）
- [x] 4.5 编写 DSL 单元测试 `e2e/tests/test_dsl.py`

## 5. Git 工作流场景测试

- [x] 5.1 创建 `e2e/scenarios/` 目录和 `conftest.py`（测试仓库 fixture：创建临时 git repo + 预置文件变更）
- [x] 5.2 实现 `test_commit_flow.py`：正常提交工作流场景
- [x] 5.3 实现 `test_branch_switch.py`：分支切换 + 搜索分支场景
- [x] 5.4 实现 `test_diff_view.py`：文件变更 diff 查看场景
- [x] 5.5 实现 `test_stash_flow.py`：stash 保存与恢复场景
- [x] 5.6 实现 `test_settings_persistence.py`：设置修改后重启保持场景

## 6. 测试基础设施与 CI

- [x] 6.1 更新 `e2e/conftest.py`：集成新 Driver 层，保持 session-scoped app fixture 兼容
- [x] 6.2 实现失败诊断报告：失败截图 + 操作日志 + 最后 5 步回放保存到 `e2e/output/`
- [x] 6.3 更新 `e2e/run.sh`：支持按场景/按标签选择性运行测试
- [x] 6.4 编写 `e2e/requirements.txt`（PyAutoGUI、Pillow、PyYAML、pytest）
