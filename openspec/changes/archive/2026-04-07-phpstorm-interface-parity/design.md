## Context

`slio-git` 已经通过 `006-phpstorm-style-polish`、`008-idea-lite-git` 和 `009-branch-history-actions` 把样式密度、Git 主工作流、分支/提交动作补齐到了“像 JetBrains”的方向，但当前主窗口仍然更像若干功能区的组合，而不是一个一眼就能认出的 PhpStorm Git 工作台。用户这次的目标不是再做一轮局部 polish，而是把整套仓库工作台的样式和交互都收敛到参考截图的节奏。

当前代码里已经有可复用的基础：`state.rs` 中的 `CompactChromeProfile`、`WorkspaceContextStrip`、`AuxiliaryView` 和 `LightweightStatusSurface`，`main_window.rs` 中的顶部 chrome 与底部状态栏，`branch_popup.rs` / `history_view.rs` 中的弹层与历史视图，以及 `theme.rs` / `widgets/*` 中的控件 token。约束条件是继续使用 `iced 0.14` 与现有 Rust UI 架构，保持中文界面，不把产品扩展成真正的代码编辑器。

## Goals / Non-Goals

**Goals:**
- 让默认仓库工作台在布局骨架、视觉密度和交互节奏上明显贴近用户提供的 PhpStorm 截图。
- 统一主窗口、分支弹层、提交/分支菜单、标签栏、底部日志区和滚动条的视觉语言，而不是继续局部修补。
- 保持现有 Git 主工作流能力可达，并让它们自然嵌入新的 PhpStorm 式外壳。
- 为后续实现提供一套可逐步验证的 parity 基线，降低“越改越像不同产品”的风险。

**Non-Goals:**
- 不复制 PhpStorm 的代码编辑、项目结构、运行配置、插件系统等 IDE 能力。
- 不引入新的 UI 框架、WebView 或跨进程渲染方案。
- 不追求逐像素复刻整个原生窗口装饰；优先复刻应用内部工作台与高频交互。
- 不在本次设计中重写 Git 核心逻辑，重点放在工作台组织方式与交互承载层。

## Decisions

**Decision 1: 以统一的 PhpStorm parity token 层驱动视觉收敛。**  
在 `theme.rs`、`widgets/button.rs`、`widgets/text_input.rs`、`widgets/scrollable.rs` 与 `state.rs` 中补齐一组面向 PhpStorm 的尺寸、分隔线、悬浮态、选中态、滚动条和工具窗 token，由这些 token 统一控制密度和视觉重量。  
**Why:** 这次变化横跨主窗口、弹层、菜单、日志区和列表，如果继续逐文件手调，最终很难稳定收敛。  
**Alternative considered:** 继续按 widget 分散改样式；放弃，因为容易造成不同面之间“各像各的”。

**Decision 2: 把主窗口重组为 JetBrains 风格的稳定外壳，而不是继续堆叠页面区块。**  
工作台会固定为“左侧工具窗栏 + 顶部紧凑上下文条 + 编辑区标签栏 + 中央 changes/diff 主区 + 底部 Git/日志工具窗”五块骨架，并通过 `MainWindow` 与 `HistoryView` 的组合保持连续上下文。  
**Why:** 用户要的是一眼像 PhpStorm 的整体轮廓；只有把壳层组织方式固定下来，后续的控件和交互才有共同参照。  
**Alternative considered:** 仅保留当前布局、只调颜色间距；放弃，因为用户当前不满已不仅是“重”，而是整体结构不像 PhpStorm。

**Decision 3: 用共享的列表/菜单原语统一分支、提交和远端动作面板。**  
`branch_popup.rs`、历史提交菜单、顶部 pull/push 菜单将共用同一套分组节奏、行高、悬浮反馈、子菜单指示、禁用原因和危险动作区。  
**Why:** PhpStorm 的好用很大一部分来自“所有菜单都说同一种语言”；当前如果各处菜单继续自成体系，就很难形成一致的交互记忆。  
**Alternative considered:** 维持各个弹层独立实现；放弃，因为这会让 parity 只停留在配色层面。

**Decision 4: 把底部历史/日志区域做成真正的工具窗，而不是附属页面。**  
`AuxiliaryView::History` 与相关状态会被增强为类似 JetBrains 底部工具窗的停靠体验：保持当前仓库/分支上下文、支持标签式切换、切回 changes 时保留当前焦点。  
**Why:** 参考图里底部 Git/日志区域是工作流的一部分，不是临时跳转页。  
**Alternative considered:** 继续把历史区当作替代主区的附属视图；放弃，因为这会破坏“一套工作台连续操作”的目标。

**Decision 5: 用截图基线 + 交互清单做人工验收，而不是只依赖主观感觉。**  
这次变更会显式产出 parity 验收项，覆盖默认窗口、分支弹层、右键菜单、底部日志区、列表密度和滚动条表现。  
**Why:** “像不像 PhpStorm”如果没有固定对照，很容易在实现过程中反复摇摆。  
**Alternative considered:** 只看开发者主观判断；放弃，因为过往几轮已经证明局部最优不能自动得到整体验收。

## Risks / Trade-offs

- [过度贴一张截图，忽略真实流程] → 以截图定骨架和密度，但验收必须覆盖提交流程、分支切换、日志查看和右键菜单等高频路径。
- [`iced` 对复杂停靠/自定义窗口装饰支持有限] → 优先把应用内部工作台做到高度相似，窗口原生边框与平台保留差异作为可接受边界。
- [密度继续提高后，点击目标与可读性可能下降] → 统一最小行高、命中区和对比度 token，对 hover/selected/warning 状态做单独校验。
- [跨多个 view/widget 改动容易引入风格回退] → 按壳层、菜单、底部工具窗、细节控件四个层次拆任务，并在每层完成后做截图对照。

## Migration Plan

本次变更不涉及持久化数据结构或仓库数据迁移。实现时按 UI 表层逐步替换，保持现有 Git 操作逻辑与状态模型可用；如果某一轮 parity 改动导致可用性回退，可以仅回滚对应 view/theme 变更，而不影响仓库状态与数据文件。

## Open Questions

- 是否需要为 macOS 标题栏区域做更深的自定义，以进一步贴近截图里的顶部观感；如果 `iced`/平台能力成本过高，则内部工作台优先。
- 标签栏首轮是否只覆盖当前 changes/diff 与底部工具窗切换，还是同步支持多标签工作流外观但暂不开放完整多页签能力。
