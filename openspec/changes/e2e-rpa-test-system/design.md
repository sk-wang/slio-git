## Context

slio-git 是一个 Rust + Iced 0.14 构建的 Git GUI 桌面应用。当前 E2E 测试基于 `e2e/rpa.py`（~314 行），使用 PyAutoGUI 进行图像识别点击和键盘模拟，pytest 作为运行器。已有 14 个测试覆盖启动、刷新、提交对话框、分支弹窗、设置持久化等基础场景。

现有 `rpa.py` 是单文件设计，所有操作（窗口管理、截图、点击、等待）混在一起。随着测试场景增多，需要分层重构并引入声明式脚本 DSL，让测试编写更接近"按键精灵"的录制-回放体验。

## Goals / Non-Goals

**Goals:**
- 将 RPA 引擎分为 Driver → Action → Scenario 三层，各层职责明确
- 提供 YAML DSL 描述操作序列，降低测试编写门槛
- 建立参考图管理规范，支持模糊匹配与多分辨率
- 覆盖核心 Git 工作流的 E2E 场景（commit、branch、diff、merge、stash）
- 保持与现有 pytest 基础设施兼容

**Non-Goals:**
- 不做跨平台支持（当前仅 macOS）
- 不做 GUI 录制器工具（手写 YAML 即可）
- 不替换 git-core 的 Rust 单元测试/集成测试
- 不做性能基准测试

## Decisions

### 1. 保持 Python + PyAutoGUI 技术栈

**选择**：继续使用 Python，不迁移到 Rust 测试框架。

**理由**：Iced 没有内建的 UI 测试/accessibility API，无法像 Electron/Qt 那样通过 DOM/widget tree 驱动。图像识别 + 键鼠模拟是目前唯一可行的 E2E 方案。Python 生态的 PyAutoGUI + Pillow 成熟可靠。

**替代方案**：(a) Rust accessibility crate — Iced 0.14 尚无支持；(b) macOS Accessibility API + Swift — 引入额外语言，维护成本高。

### 2. 三层架构：Driver / Action / Scenario

**选择**：
- **Driver 层**（`e2e/driver/`）：封装 PyAutoGUI 原语——窗口管理、截图、鼠标、键盘、图像匹配。对外暴露稳定 API。
- **Action 层**（`e2e/actions/`）：组合 Driver 操作为可复用动作片段（如 `open_commit_dialog()`、`switch_branch(name)`）。
- **Scenario 层**（`e2e/scenarios/`）：端到端场景脚本，调用 Action 层组装完整工作流。

**理由**：按键精灵的核心思想就是"操作片段复用"。三层分离让 Driver 变更（如换底层库）不影响业务场景，Action 可跨场景共享。

### 3. YAML 脚本 DSL

**选择**：用 YAML 文件描述操作序列，Python 解释器逐步执行。

```yaml
name: 提交文件
steps:
  - click_image: refs/toolbar_commit_btn.png
  - wait_image: refs/commit_dialog_title.png
  - type_text: "test commit message"
  - hotkey: [cmd, enter]
  - wait_disappear: refs/commit_dialog_title.png
  - screenshot: result_after_commit.png
```

**理由**：YAML 可读性好，非程序员（QA）也能编写和维护。比纯 Python 测试更接近"按键精灵脚本"的心智模型。

**替代方案**：(a) 纯 Python — 灵活但门槛高；(b) JSON — 可读性差；(c) 自定义 DSL — 解析器维护成本高。YAML 是最佳平衡点。

### 4. 参考图按目录分组

**选择**：`e2e/refs/<category>/` 按功能分组存储参考截图（如 `refs/toolbar/`、`refs/branch_popup/`、`refs/commit_dialog/`）。

**理由**：按功能而非分辨率分组更直观。模糊匹配（confidence 阈值）可吸收小的像素差异。如未来需多分辨率支持，可加 `@2x` 后缀。

### 5. 场景测试用 pytest 类组织

**选择**：每个场景文件是一个 pytest 文件，既支持 YAML DSL 调用也支持 Python 直接调用 Action 层。

**理由**：保持与现有测试运行器兼容，CI 无需额外工具。YAML 和 Python 混合使用让简单场景用 YAML、复杂断言用 Python。

## Risks / Trade-offs

- **[图像匹配脆弱性]** → 使用 confidence 阈值（默认 0.8）+ 灰度匹配减少误报；关键断言提供 `region` 参数限制搜索区域
- **[分辨率/DPI 敏感]** → 固定测试窗口尺寸（1728x1080）；参考图在标准 DPI 下截取
- **[CI 环境无 GUI]** → macOS runner 支持 GUI；或使用 `screencapture` + virtual display fallback
- **[YAML DSL 表达力有限]** → 复杂逻辑（条件判断、循环）回退到 Python Action 层，YAML 仅处理线性序列
