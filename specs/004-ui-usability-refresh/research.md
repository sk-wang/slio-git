# Research: 主界面可用性与视觉改造

**Feature**: 主界面可用性与视觉改造  
**Date**: 2026-03-22  
**Branch**: `004-ui-usability-refresh`

## Decision 1: 以共享 Darcula 设计令牌驱动全应用视觉统一

### Decision

继续使用 `src-ui/src/theme.rs` 中已存在的 Darcula 方向，但将其从“颜色常量集合”升级为全应用共享设计令牌层，统一颜色、字体、间距、边框、选中态、空状态、错误状态和 diff/highlight 规则。

### Rationale

- 仓库里已经有 `theme.rs` 和中文字体选择逻辑，说明主题与本地化基础设施已存在，不需要另起体系。
- 当前问题不只是配色，而是不同界面在层级、留白、按钮密度和反馈样式上不统一。共享设计令牌是全应用一致化的最低成本路线。
- 单一 Darcula 主题符合用户澄清与 Constitution VI，不需要在这次范围里同时解决双主题切换。

### Alternatives considered

- 逐个界面手工修样式：短期快，但会继续产生视觉漂移。
- 完整逐像素复刻 IntelliJ：对当前产品适配空间太小，不利于用户已批准的导航重构。
- 同时支持深浅两套主题：会显著放大测试面和设计复杂度，与当前需求不符。

## Decision 2: 重做应用壳层与导航，但保留 `git-core` Git 行为边界

### Decision

允许在 `src-ui` 中自由重构欢迎页、仓库工作区、导航分组、入口命名和主要操作流；`git-core` 中的仓库发现、状态读取、diff、冲突处理、branch/commit/remote/stash/rebase/tag 行为仍保持独立库边界与 IntelliJ-compatible 算法。

### Rationale

- 这是满足用户“可以自由重做导航、入口命名和主要操作流”的唯一方式。
- Constitution I 要求 IntelliJ parity，Constitution III 要求 library-first。最佳平衡是：UI 壳层可重做，但 Git 行为算法与库边界不动摇。
- 当前 `src-ui/src/main.rs` 与 `src-ui/src/views/main_window.rs` 已经呈现明显“壳层/状态/行为”分离趋势，适合继续把导航和视觉重构集中在 UI 层。

### Alternatives considered

- 同时重构 UI 和 Git 核心算法：风险过高，且会破坏 parity 验证边界。
- 保持现有壳层只换皮：无法满足“可自由重做导航和主要操作流”的用户澄清。
- 将更多 Git 行为搬入 UI：违反 Constitution III。

## Decision 3: 将“仓库内发现的功能问题”纳入同一交付流，但通过统一缺陷台账控制执行

### Decision

本 feature 默认包含 defect sweep：在设计、实现和验证过程中，凡是在仓库中发现的现有功能问题都纳入同一交付流。执行上采用统一缺陷台账管理，按发现位置、严重性、复现路径和影响范围记录，并在 tasks 阶段拆分为明确可验证任务。

### Rationale

- 用户已经明确要求“只要在仓库里发现问题，不管是否属于当前主流程都全部修”。
- 如果不建立台账，这个范围会在实现期无限膨胀，难以闭环，也无法在计划阶段与后续任务对齐。
- 用统一缺陷台账管理，不是缩小范围，而是给这个激进范围建立可追踪的执行机制。

### Alternatives considered

- 只修 UI 触及区域的问题：与用户澄清冲突。
- 发现问题但单独延后开 feature：会让“本次顺手修”落空。
- 不记录台账、边做边修：最终很难验证“所有发现的问题都处理了”。

## Decision 4: 采用“全局壳层 + 多视图一致契约 + 回归场景矩阵”验证策略

### Decision

Phase 1 产出统一 UI 契约，覆盖欢迎页、仓库主工作区、提交/分支/stash/history/remote/tag/rebase/conflict 等全部现有界面，并在 quickstart 中定义一套跨视图回归矩阵，要求对每个界面同时验证视觉一致性、状态反馈和原有功能行为。

### Rationale

- Spec 的范围已经扩展到“全部现有 UI 界面 + 仓库内发现的功能问题”，单点验证不够。
- 当前代码库已有大量独立 `views/` 和 `widgets/` 文件，说明视图拆分粒度已经形成，适合通过统一契约约束而不是零散描述。
- Constitution IV 与 V 要求 parity 测试和可观察性；回归矩阵能把 UI 重构和 defect 修复串在一起验证。

### Alternatives considered

- 仅依赖手工目测：无法证明原有功能未退化。
- 只测 `git-core` 不测 UI：无法覆盖本次 UI 壳层重构风险。
- 只给主界面做契约：与“全应用 UI”范围不符。

## Decision 5: 以 `AppState` 为中心扩展应用壳层状态，而不是为每个视图各自发明状态系统

### Decision

沿用 `src-ui/src/state.rs` 的 `AppState`/`ViewMode` 作为全应用状态中枢，并扩展导航、布局、反馈和 defect-sweep 所需的壳层状态，而不是为每个视图建立割裂的局部状态模型。

### Rationale

- 当前 `AppState` 已经持有 repository、loading、error、view_mode、changes、diff、conflict 等核心状态，是现成的全局状态中心。
- 全应用 UI 改造最怕多个页面各自维护命名、错误、加载和选中逻辑，继续扩展 `AppState` 更有利于统一反馈契约。
- 这也让 `main.rs` 的消息流与新导航结构保持单一入口，更适合后续 tasks 分解。

### Alternatives considered

- 每个 view 自己维护状态：会让跨界面一致性和回归验证变难。
- 引入新的状态管理框架：当前没有必要，也会放大实施成本。
- 把所有状态都塞进 widgets：会削弱 views 的组合能力。
