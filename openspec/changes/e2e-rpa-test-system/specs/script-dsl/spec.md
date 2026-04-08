## ADDED Requirements

### Requirement: YAML 脚本描述操作序列
系统 SHALL 支持 YAML 格式的脚本文件，每个脚本包含 `name`（脚本名称）和 `steps`（操作步骤列表）。

#### Scenario: 解析并执行 YAML 脚本
- **WHEN** 调用 `dsl.run("scripts/commit_flow.yaml")`
- **THEN** 系统 SHALL 按顺序解析并执行每个 step，将 step 类型映射到对应的 Driver/Action 方法

#### Scenario: 脚本语法错误时报告位置
- **WHEN** YAML 脚本包含未知的 step 类型（如 `unknown_action: foo`）
- **THEN** 系统 SHALL 抛出 `ScriptError` 并指明文件名和行号

### Requirement: 支持核心操作指令集
YAML DSL SHALL 支持以下操作指令：`click_image`（图像点击）、`click_at`（坐标点击）、`type_text`（文本输入）、`hotkey`（快捷键）、`wait_image`（等待图像出现）、`wait_disappear`（等待图像消失）、`screenshot`（截图断言）、`sleep`（固定等待）、`call_action`（调用 Action 层函数）。

#### Scenario: click_image 指令执行
- **WHEN** 脚本包含 `- click_image: refs/toolbar/commit_btn.png`
- **THEN** 系统 SHALL 调用 `driver.click_image()` 定位并点击该图像

#### Scenario: call_action 调用 Python Action
- **WHEN** 脚本包含 `- call_action: { name: switch_branch, args: { branch: "develop" } }`
- **THEN** 系统 SHALL 调用 `actions.switch_branch(branch="develop")`

#### Scenario: screenshot 断言指令
- **WHEN** 脚本包含 `- screenshot: { save_as: "result.png", expect_image: "refs/expected_state.png" }`
- **THEN** 系统 SHALL 截图保存，并与参考图对比，相似度低于阈值时标记测试失败

### Requirement: 脚本支持变量替换
YAML 脚本 SHALL 支持 `${VAR}` 语法引用环境变量或脚本参数，用于动态内容（如分支名、提交信息）。

#### Scenario: 变量替换文本输入
- **WHEN** 脚本包含 `- type_text: "fix: ${COMMIT_MSG}"` 且 `COMMIT_MSG=update readme`
- **THEN** 系统 SHALL 输入 `fix: update readme`
