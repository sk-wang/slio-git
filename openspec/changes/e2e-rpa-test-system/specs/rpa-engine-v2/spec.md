## ADDED Requirements

### Requirement: Driver 层封装底层操作原语
Driver 层 SHALL 提供以下原子操作的稳定 API：窗口激活、窗口尺寸设置、全屏/区域截图、鼠标点击（坐标/图像定位）、鼠标拖拽、键盘输入（文本/热键）、图像查找（返回坐标+置信度）、等待图像出现/消失。

#### Scenario: 通过图像定位并点击 UI 元素
- **WHEN** 调用 `driver.click_image("toolbar_commit.png", confidence=0.8)`
- **THEN** 系统 SHALL 在屏幕上查找匹配图像，点击其中心坐标，未找到时抛出 `ImageNotFoundError`

#### Scenario: 等待图像出现（带超时）
- **WHEN** 调用 `driver.wait_image("dialog_title.png", timeout=10)`
- **THEN** 系统 SHALL 每 0.5 秒轮询截图匹配，匹配成功返回坐标；超时抛出 `TimeoutError`

#### Scenario: 区域截图
- **WHEN** 调用 `driver.screenshot(region=(x, y, w, h), save_as="result.png")`
- **THEN** 系统 SHALL 截取指定区域并保存到测试输出目录

### Requirement: Action 层组合可复用操作片段
Action 层 SHALL 将多个 Driver 调用组合为业务语义明确的操作函数。每个 Action 函数 MUST 有清晰的前置条件和后置断言。

#### Scenario: 打开提交对话框
- **WHEN** 调用 `actions.open_commit_dialog()`
- **THEN** 系统 SHALL 点击工具栏提交按钮并等待提交对话框标题图像出现

#### Scenario: Action 失败时提供上下文截图
- **WHEN** Action 执行过程中任何 Driver 调用失败
- **THEN** 系统 SHALL 自动截取当前屏幕状态并保存为 `failure_<action_name>_<timestamp>.png`

### Requirement: Scenario 层编排端到端工作流
Scenario 层 SHALL 调用 Action 层组装完整的用户工作流。每个 Scenario MUST 是一个独立的 pytest 测试函数或类。

#### Scenario: 场景脚本独立运行
- **WHEN** 使用 `pytest e2e/scenarios/test_commit_flow.py` 运行单个场景
- **THEN** 该场景 SHALL 独立完成，不依赖其他场景的执行顺序

#### Scenario: 场景失败生成诊断报告
- **WHEN** 场景测试失败
- **THEN** 系统 SHALL 保存失败截图、操作日志、最后 5 步的步骤回放信息到 `e2e/output/` 目录
