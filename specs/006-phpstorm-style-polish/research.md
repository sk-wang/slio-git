# Research: PhpStorm 风格的轻量化样式收敛

**Feature**: PhpStorm 风格的轻量化样式收敛  
**Date**: 2026-03-23  
**Branch**: `006-phpstorm-style-polish`

## Decision 1: 先统一收敛主题 token，而不是逐个 widget 打补丁

### Decision

优先在 `src-ui/src/theme.rs` 中下调默认圆角、容器层级、控件高度、区块间距和 badge 强度，再由 `main_window.rs`、`branch_popup.rs` 和各 widget 继承新的轻量基线。

### Rationale

- 当前“重”的根因不是单个面板，而是 `theme.rs` 中 `Raised/Panel` 表面、`radius::MD`、`layout::SHELL_PADDING` 等基础 token 普遍偏厚。
- 如果只在 `main_window.rs` 或 `branch_popup.rs` 局部减 padding，其他列表、状态条和 diff 仍会保留旧的厚重观感，最终风格割裂。
- 用户要的是“整体像 PhpStorm 一样轻”，这类目标必须从共享样式系统着手。

### Alternatives considered

- 只调 `main_window.rs`：首屏会轻一些，但弹层、列表和底部状态区仍显得厚。
- 完全改色：颜色不是主要问题，视觉重量更多来自边框、圆角、间距和控件高度。
- 为 006 单独引入第二套 theme：维护成本高，且会放大风格分裂。

## Decision 2: 主工作区压缩为连续的双层 chrome，上层上下文、下层导航/动作

### Decision

将仓库打开后的持久顶部区域收敛为最多两条细窄水平带：第一层承担仓库/分支上下文与分支入口；第二层承担必要导航与极少量高频动作。主体区直接衔接改动树与 diff，不再嵌套厚容器。

### Rationale

- 规范中的 SC-001 明确要求顶部持久样式层不超过两条明显水平带。
- 当前 `src-ui/src/views/main_window.rs` 的上下文按钮、导航按钮、banner 和主体容器叠加后，视觉上仍然像“盒子套盒子”。
- 参考图的关键气质是：窗口 chrome 存在，但它不喧宾夺主，内容区一眼成为焦点。

### Alternatives considered

- 保留现有结构，仅减少文案：结构本身仍然厚，第一屏会继续显得拥挤。
- 彻底隐藏导航：会损失概览/改动/冲突切换的可发现性。
- 把动作全部下沉到底部状态栏：会让状态栏变得过重。

## Decision 3: 分支弹层采用 JetBrains 式“搜索 + 动作列表 + 分组分支列表”构图

### Decision

将 `src-ui/src/views/branch_popup.rs` 调整为轻量列表面板：顶部是搜索与当前分支摘要，中段是紧凑高频动作列表，底部是最近/本地/远程分组；分支项仅保留必要的名称、跟踪关系和当前态提示。

### Rationale

- 用户明确要求以参考图为标准，不希望看到“完整管理页面式”的厚弹层。
- 当前分支面板逻辑已具备 `recent/local/remote` 数据分组能力，最需要调整的是视觉组织方式和信息密度，而不是交互算法。
- JetBrains 弹层的核心特征不是控件种类多，而是统一成列表节奏，说明性文案极少。

### Alternatives considered

- 继续用大按钮块展示动作：会让弹层再次变成卡片页。
- 只缩小字体不改节奏：视觉仍然会显得破碎。
- 把高频动作挪回主界面常驻：违背“主体留给改动树和 diff”的目标。

## Decision 4: 列表、diff 顶栏和状态栏统一向“薄、紧、平”靠拢

### Decision

在 `src-ui/src/widgets/changelist.rs`、`diff_viewer.rs`、`split_diff_viewer.rs`、`statusbar.rs`、`button.rs` 和 `text_input.rs` 中统一降低行高、padding、按钮边框存在感和 badge 对比度，使改动列表、diff 顶栏与底部状态层表现为连续 IDE 控件，而不是独立卡片。

### Rationale

- 如果主框架变轻，但文件行、badge 和状态条仍然偏厚，用户仍会感知整体“重”。
- 视觉基线来自 PhpStorm 的 Changes 视图，而它的列表、工具条、状态条本身就拥有统一密度。
- 这些部件已经模块化，适合在 Phase 1 里用统一 contract 约束。

### Alternatives considered

- 只改顶部，不改列表：第一眼改善后会立刻在操作过程中暴露风格断层。
- 只隐藏 badge：会损失当前状态可读性，不如把 badge 收敛成弱化样式。

## Decision 5: 空状态、错误态和超长文本采用“紧凑但清晰”的边界规则

### Decision

空仓库、无改动、长分支名、长仓库路径和错误/冲突提示统一遵循轻量规则：默认不使用大卡片；长文本优先截断、滚动或次级展示；只有冲突/失败等高风险状态使用更强强调。

### Rationale

- 用户要的是轻量，而不是“只有理想 happy path 才轻”。
- 规范中的 Edge Cases 明确要求处理窄窗口、长名称、空状态和冲突场景。
- Constitution V 与 VI 要求错误必须可理解、中文必须完整，因此不能通过简单隐藏来追求极简。

### Alternatives considered

- 所有空状态都改成极小纯文本：在首次使用或无仓库状态下会缺乏必要引导。
- 错误也完全做成弱文本：会损失关键风险感知。
