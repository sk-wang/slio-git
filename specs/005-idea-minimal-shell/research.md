# Research: IDEA 风格的极简 Git 工作台

**Feature**: IDEA 风格的极简 Git 工作台  
**Date**: 2026-03-23  
**Branch**: `005-idea-minimal-shell`

## Decision 1: 主工作区采用“单一上下文入口 + 改动/差异主体”的信息架构

### Decision

仓库工作区顶部不再长期显示产品名、标语、多组芯片和重复说明，而是收敛为一个主上下文入口，只表达当前仓库与当前分支；其余空间优先留给改动列表和差异预览。

### Rationale

- 用户明确指出“像顶部的 slio-git 这种完全没必要显示”。
- 当前 `src-ui/src/views/main_window.rs` 的 toolbar 同时承载产品标题、标语、section chip、next-step chip、仓库 chip 和一排动作，明显挤压主工作区。
- IntelliJ 的 Changes/Git 区域更强调“当前仓库/分支上下文 + 工作内容”，而不是额外品牌展示。

### Alternatives considered

- 仅隐藏产品标题，保留其余 chip 与说明：噪音仍然存在，冗余感不会根治。
- 完全移除顶部上下文区：会损失仓库与分支识别效率，不符合 spec。
- 将所有上下文都挪入侧栏：用户需要跨区域对照，不如单点入口直接。

## Decision 2: 次要 Git 动作收纳到类似 IDEA 分支弹层的上下文切换器

### Decision

将分支切换、常用 Git 动作和次要入口集中到一个上下文切换器/弹层中，结构参考 IntelliJ 的 `DvcsBranchPopup` 与 `GitBranchPopupActions`：优先展示当前分支、搜索、最近项、本地/远程分组和高频动作，其它能力保持渐进展开。

### Rationale

- IDEA 的 `DvcsBranchPopup` 将分支与操作放入一个可聚焦 popup，而不是常驻在主工作区顶部。
- `GitBranchPopupActions` 展示了本地/远程分组、最近分支、incoming/outgoing 状态和单分支操作的组织方式，适合本项目“参考 IDEA 但更克制”的目标。
- 当前 `slio-git` 已有 `branch_popup.rs`、`remote_dialog.rs`、`stash_panel.rs` 等独立视图，可在入口层统一收纳，而不必删除功能。

### Alternatives considered

- 继续把 Pull/Push/Commit/Stash 作为顶部常驻按钮：高频但占位过重，会持续稀释主工作区。
- 把所有能力塞到侧栏：会让侧栏从导航变成命令堆栈，失去层次。
- 只做分支弹层，不纳入其他常用动作：仍然无法解决主界面按钮过多的问题。

## Decision 3: 持久反馈从“宽 banner + 说明块”改为“短时、就近、状态化”

### Decision

保留结构化日志与错误可见性，但将主工作区中的持久反馈层改为更克制的状态表达：稳定状态时不显示大块引导文案，只有在加载、成功、失败或阻断时才短时出现明显反馈。

### Rationale

- 当前 `main_window.rs` 与 `state.rs` 组合会在 toolbar、sidebar 和 content header 中重复表达 section/next-step/context，导致用户一直被“解释页面”包围。
- 用户诉求是“尽量精简交互和 UI 元素”，反馈层必须遵循同一原则。
- Constitution V 要求可观测性，但不要求 UI 长时间常驻大体积反馈，只要求错误可追踪、信息可操作。

### Alternatives considered

- 完全取消 UI 反馈，只保留日志：会损害可用性。
- 保持现有 banner 模式，只减少文案：仍然会留下固定占位问题。
- 所有反馈都改为模态弹窗：过于侵入，不符合桌面工作流。

## Decision 4: 保留 004 中已完成的功能修复，但重新组织其入口层级

### Decision

不回退 `004-ui-usability-refresh` 中已经完成的提交、分支、历史、远程、标签、储藏、冲突与 rebase 修复；本特性只调整这些能力在 UI 中的显露方式和默认可见层级。

### Rationale

- `004` 已把多条主流程和回归测试修通，重新砍掉会造成倒退。
- 本特性的问题是“太冗杂”，不是“能力太多”。
- 通过入口重组而非回退功能，能在不损失能力的前提下实现极简化。

### Alternatives considered

- 临时移除一部分能力入口：会造成功能不可达，违反新 spec 的 FR-010 / FR-011。
- 在主工作区保留所有入口但缩小尺寸：只是视觉压缩，不是真正的交互收敛。

## Decision 5: 用状态模型收敛重复上下文，而不是在多个视图重复渲染

### Decision

扩展 `AppState` 中的壳层状态，使“当前仓库、当前分支、当前工作区、当前高优先级反馈、当前上下文动作入口”只需要被建模一次，然后由 `main_window.rs` 在唯一主上下文区域中渲染，不再让顶部、侧栏和正文各自重复生成一份。

### Rationale

- 当前状态结构已具备壳层状态中心，只是渲染层重复消费同一语义。
- 先做状态收敛，再做 UI 收敛，才能长期避免冗余反弹。
- 这也让后续 `tasks.md` 能按“状态收敛 → 入口收纳 → 视图瘦身 → 回归验证”拆分。

### Alternatives considered

- 单纯改视觉，不改状态结构：容易在后续维护中重新长出重复 UI。
- 为每个 view 单独定义极简模式：会让信息层级再次分裂。
