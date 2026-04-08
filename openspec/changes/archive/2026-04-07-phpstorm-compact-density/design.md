## Context

`slio-git` 刚完成一轮 PhpStorm workbench parity，对主窗口骨架、分支弹层、底部历史工具窗和核心 diff/changelist 做了 JetBrains 风格统一，但当前默认界面仍然比 PhpStorm 更“松”：工具栏高度偏高，标题/说明文字过多，面板 padding 和列表行高较厚，导致同样面积内能看到的信息更少。用户这次的反馈非常聚焦——不是要新功能，而是要把现有界面进一步压实，让默认仓库工作台、提交对话框和历史/菜单交互都更接近 PhpStorm 的紧凑信息密度。

当前可复用基础已经存在：`theme.rs` 里有集中式 token，`main_window.rs` 和 `history_view.rs` 已经是稳定的 JetBrains 壳层，`branch_popup.rs`、`commit_dialog.rs` 与 `widgets/*` 已有共享样式入口。约束是继续使用 `iced 0.14`，保持中文界面，不为了“更像”而引入新的窗口系统或复杂停靠框架。

## Goals / Non-Goals

**Goals:**
- 让默认仓库工作台、提交对话框和底部工具窗达到更接近 PhpStorm 的行高、留白和视觉重量。
- 用统一 token 驱动紧凑化，避免每个 view 单独微调后再次失真。
- 保持当前 Git 主工作流可达，不因压缩界面而牺牲点击命中区和状态可读性。
- 产出可执行的紧凑度验收标准，便于后续实现和截图对照。

**Non-Goals:**
- 不新增 IDE 范围外的功能，如多编辑器工作区、代码运行配置或项目树。
- 不追求操作系统窗口装饰逐像素复刻。
- 不在本次变更中重写 Git 核心逻辑或状态存储。
- 不为了紧凑化牺牲无障碍对比度或关键按钮的最小可点击区域。

## Decisions

**Decision 1: 以“紧凑密度档位”统一驱动壳层和控件。**  
在 `theme.rs` 中补充更低的 spacing、control height、tab height、panel padding 和 caption 字号，并让 `button.rs`、`text_input.rs`、`scrollable.rs`、`statusbar.rs` 等组件从同一套 token 读取默认值。  
**Why:** 这次目标横跨工作台、菜单、对话框和工具窗，只有集中式密度 token 才能保证整体一起收紧。  
**Alternative considered:** 逐个页面手调高度和 padding；放弃，因为后续很容易出现“主窗口紧凑、弹窗仍然偏胖”的回退。

**Decision 2: 优先压缩高频区域，而不是全局无差别缩小。**  
主窗口顶部 chrome、标签栏、变更列表、diff 文件头、底部工具窗标题栏、提交对话框和分支/远端菜单会优先压缩；正文 diff 内容、错误/警告提示和空状态文案保留足够可读性。  
**Why:** 用户感知“像不像 PhpStorm”主要来自轮廓和高频触点，不需要把所有文本都压到最小。  
**Alternative considered:** 所有字号和间距统一缩小；放弃，因为这会伤害可读性，也容易让状态文案变得拥挤。

**Decision 3: 把说明文案改成单行提示或弱化说明，减少卡片感。**  
提交对话框、底部工具窗和菜单面板中的说明文字改为单行 caption、辅助 chip 或更轻的次级文案，避免每个 panel 都像独立 marketing 卡片。  
**Why:** PhpStorm 的紧凑感很大程度来自信息直接贴在操作控件旁，而不是额外占据一整行说明。  
**Alternative considered:** 保留现有多行说明，仅缩小字号；放弃，因为视觉上仍然会显得臃肿。

**Decision 4: 以验收清单约束“更紧凑但不难点”。**  
设计中要求所有被压缩的交互面仍然保留稳定 hover/selected 状态、危险动作区分和最小点击高度，并通过 checklist 明确验证。  
**Why:** 紧凑化很容易变成纯视觉压缩，必须同时守住可用性底线。  
**Alternative considered:** 仅在实现后凭主观感觉验收；放弃，因为容易把“好看”误当成“好用”。

## Risks / Trade-offs

- [界面过紧导致点击命中区下降] → 对按钮、菜单项、列表项保留最小高度，并在验收中覆盖 hover/selected/disabled 状态。
- [不同 view 压缩幅度不一致，形成新的割裂感] → 所有高频 surface 统一从密度 token 读取尺寸，避免局部硬编码。
- [提交/历史等信息密集面板压缩后可读性下降] → 仅压缩标题栏、padding 和冗余说明，不压缩 diff 正文和关键状态颜色对比。
- [后续实现继续围绕截图微调，缺少稳定基线] → 用 spec 明确哪些区域必须更紧凑、哪些可接受保留现状。

## Migration Plan

这次变更不涉及数据迁移。实现时按“主题 token → 主工作台 → 对话框/菜单 → 验收证据”顺序推进；如果紧凑化某一层导致可用性回退，可以单独回滚对应 view/theme 变更，而不会影响仓库状态或 Git 数据。

## Open Questions

- 提交对话框是否需要与主工作台共用完全相同的 toolbar/section 密度，还是允许略微放宽以照顾多行消息输入。
- 是否需要为历史/日志工具窗补充更明显的可展开 affordance，避免在更紧凑的标题栏下不够显眼。
