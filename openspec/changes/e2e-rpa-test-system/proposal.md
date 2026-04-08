## Why

slio-git 是一个 Iced 0.14 构建的原生 Git GUI，当前已有基于 Python + PyAutoGUI 的 RPA E2E 框架（14 个测试通过），但测试覆盖仅限于窗口启动、按钮点击等基础场景。随着 diff editor、merge editor、branch popup 等高复杂度组件不断演进，需要一套**按键精灵思维**的 E2E 测试架构——以"录制-回放-断言"为核心范式，让非程序员也能通过截图 + 坐标 + 按键序列快速编写端到端场景测试。

## What Changes

- 重构现有 `e2e/rpa.py` 引擎为分层架构：**Driver 层**（窗口/键鼠/截图）→ **Action 层**（可复用操作片段）→ **Scenario 层**（业务场景脚本）
- 引入 **脚本 DSL**：用 YAML/Python 描述"按键精灵式"操作序列（点击坐标、等待图片、键入文字、截图断言），降低测试编写门槛
- 新增 **参考图管理**：按分辨率/主题分组存储 UI 参考截图，支持模糊匹配阈值配置
- 新增 **Git 场景测试集**：覆盖提交、分支切换、diff 查看、merge conflict 解决、stash、settings 持久化等核心工作流
- 新增 **CI 集成方案**：headless 截图对比 + 测试报告生成

## Capabilities

### New Capabilities
- `rpa-engine-v2`: 分层 RPA 引擎（Driver/Action/Scenario），替代当前单文件 rpa.py
- `script-dsl`: 按键精灵风格的 YAML 脚本 DSL，支持录制回放语义
- `reference-image-management`: UI 参考截图管理与模糊匹配断言系统
- `git-workflow-scenarios`: 核心 Git 操作的端到端场景测试集

### Modified Capabilities

（无现有 spec 需修改）

## Impact

- **代码**：`e2e/` 目录重构，新增 `e2e/actions/`、`e2e/scenarios/`、`e2e/refs/` 子目录
- **依赖**：Python 侧可能新增 PyYAML（DSL 解析）；PyAutoGUI + Pillow 保持不变
- **CI**：需 macOS runner 支持 GUI 测试（或 headless 截图方案）
- **开发流程**：测试编写从"写 Python 代码"降级为"写 YAML 脚本 + 放参考图"
