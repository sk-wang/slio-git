# Implementation Plan: 分支视图提交动作补齐

**Branch**: `009-branch-history-actions` | **Date**: 2026-03-25 | **Spec**: [/Users/wanghao/git/slio-git/specs/009-branch-history-actions/spec.md](/Users/wanghao/git/slio-git/specs/009-branch-history-actions/spec.md)
**Input**: Feature specification from `/Users/wanghao/git/slio-git/specs/009-branch-history-actions/spec.md`

## Summary

本轮规划要把 `slio-git` 的分支视图从“分支列表 + 少量分支动作”提升为接近 PhpStorm / IDEA 的提交工作台：用户在选中某个分支后，不仅能看见这条分支上的提交时间线，还能围绕某一条提交直接完成复制、补丁导出、比较、导航、建分支/标签，以及更高价值的摘取、回退、重置、只推到这里和本地未发布提交整理。

技术上继续坚持现有 Rust workspace：`git-core` 负责提交动作、可执行性判定、发布范围与后续 Git 流程状态；`src-ui` 负责在分支视图里复用提交图谱、组织 PhpStorm 式动作分组、展示中文确认与冲突后续入口。为了避免分支视图和历史视图继续各自演化，本轮优先抽出共享的提交时间线 / 提交动作组件，并让危险动作的风险提示与继续处理路径保持统一。

## Technical Context

**Language/Version**: Rust 2021+  
**Primary Dependencies**: iced 0.14（native UI）, git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, syntect 5.3  
**Storage**: Git 仓库与本地文件系统；补丁导出与临时确认结果使用本地文件 / 进程级状态，不引入额外数据库  
**Testing**: `cargo check -p src-ui`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`，以及围绕分支视图提交动作的手工 walkthrough  
**Target Platform**: macOS 优先，同时维持 Windows 10+、Ubuntu 20.04+ 的桌面兼容目标  
**Project Type**: Native desktop application（Rust workspace: `git-core` library + `src-ui` app）  
**Performance Goals**: 分支视图打开后应在一次加载中稳定展示最近 100~200 条提交；打开提交动作菜单、切换选中提交和刷新动作结果都应保持无明显卡顿；动作完成后尽快回写新的时间线与同步状态  
**Constraints**: 必须保持 IntelliJ / PhpStorm 风格 Git 语义；完整中文界面；库优先架构；危险动作执行前必须有清晰影响说明；正在进行的 merge / rebase / cherry-pick / revert 流程不能被静默打断  
**Scale/Scope**: 重点影响 `src-ui` 的分支视图、历史视图、主消息路由和辅助对话框，以及 `git-core` 中提交、历史、分支、标签、远端、rebase、repository 状态模块与集成回归测试

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. IntelliJ Compatibility | ✅ PASS | 本特性直接以 PhpStorm / IDEA 分支视图截图为目标，要求动作分组、启用逻辑、风险提示和后续流程保持 IntelliJ 风格。 |
| II. Rust + Iced Stack | ✅ PASS | 继续保持 Rust + Iced 原生桌面栈，不引入 WebView、Electron 或 Tauri。 |
| III. Library-First Architecture | ✅ PASS | 新的提交动作、可执行性判定、发布范围和后续流程状态继续下沉到 `git-core`，UI 只负责编排与呈现。 |
| IV. Integration Testing for Git Parity | ✅ PASS | 计划扩展 `src/git-core/tests/workflow_regressions.rs`，覆盖 cherry-pick、revert、reset、push-to-here、branch/tag from commit 与 rewrite 范围判定。 |
| V. Observability | ✅ PASS | 所有提交级动作都必须输出结构化日志，并在 UI 中反馈成功、失败或待继续状态，而不是裸露底层错误。 |
| VI. 中文本地化支持 | ✅ PASS | 分支视图、确认弹层、禁用原因、冲突后续入口和导出反馈都保持中文文案与中文字体策略。 |

**Gate Result**: 通过。当前规划围绕“分支视图中的提交级工作流”增强体验深度，但没有偏离 Constitution 对 IntelliJ 兼容性、Rust/Iced、库分层、测试与中文界面的硬约束。

**Post-Design Re-check**: 通过。Phase 1 设计产物明确把提交动作分成共享时间线、资格判定、执行后续状态和回归测试几层，没有把 Git 语义散落到 UI，也没有引入与 Constitution 相冲突的新栈。

## Project Structure

### Documentation (this feature)

```text
/Users/wanghao/git/slio-git/specs/009-branch-history-actions/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── spec.md
├── contracts/
│   └── branch-history-ui-contracts.md
└── tasks.md
```

### Source Code (repository root)

```text
/Users/wanghao/git/slio-git/src/git-core/
├── src/
│   ├── branch.rs
│   ├── commit.rs
│   ├── commit_actions.rs        # new: 提交级动作与资格判定
│   ├── history.rs
│   ├── remote.rs
│   ├── rebase.rs
│   ├── repository.rs
│   ├── tag.rs
│   └── lib.rs
└── tests/
    └── workflow_regressions.rs

/Users/wanghao/git/slio-git/src-ui/src/
├── main.rs
├── state.rs
├── views/
│   ├── branch_popup.rs
│   ├── history_view.rs
│   ├── rebase_editor.rs
│   └── tag_dialog.rs
└── widgets/
    ├── commit_action_menu.rs    # new: 提交动作菜单与确认入口
    ├── commit_timeline.rs       # new: 分支/历史共享的提交图谱列表
    ├── commit_compare.rs
    ├── diff_viewer.rs
    └── scrollable.rs
```

**Structure Decision**: 保持现有双 crate 架构，不新建第三层应用服务。`git-core` 新增提交动作模块，统一封装复制辅助信息以外的 Git 语义、资格判定和后续流程状态；`src-ui` 通过共享的提交时间线与动作菜单组件，把分支视图和历史视图拉回同一交互骨架，避免两套提交列表继续分叉。

## Implementation Slices

1. **Shared commit timeline surface**
   - 将 `history_view.rs` 里的提交图谱与行渲染抽成共享 `commit_timeline` 组件
   - 让 `branch_popup.rs` 在选中分支后展示对应提交时间线，而不仅是分支摘要与右侧空状态
   - 统一提交选中态、滚动保持、父/子导航与动作锚点
   - 同步收敛时间线、详情区、比较区的滚动条视觉与横向溢出策略，避免出现叠压正文的双横向滚动条
   - 让相关预览区在选中新文件 / 未跟踪文件时直接渲染整文件内容或明确占位说明，而不是落入“没有变更”空态

2. **Commit action menu and eligibility engine**
   - 定义 PhpStorm 风格的动作分组：基础信息、比较/导航、派生、当前分支动作、历史整理
   - 为每条提交计算可执行性：是否属于当前分支、是否已发布、是否有上游、是否存在进行中的 Git 流程、是否为 merge/root commit
   - 对不可用动作给出中文禁用原因，而不是简单隐藏或报错

3. **Library-first commit actions**
   - 在 `git-core` 中补齐创建补丁、基于提交建分支/标签、checkout commit、compare base 解析、cherry-pick、revert、reset current branch to here、push current branch to selected commit 等能力
   - 新增 ancestry / published-range / current-branch ownership 等判定辅助，避免 UI 自己猜测资格
   - 对高风险动作统一返回影响说明与 follow-up state，使 UI 能正确提示继续 / 跳过 / 中止

4. **Guided rewrite flows**
   - 为 reword、fixup、squash、drop、undo last commit、start interactive rebase from here 建立受限但清晰的 rewrite 范围模型
   - 尽量复用 `rebase_editor.rs` 作为 rewrite 后续流程入口，而不是再做一套孤立流程
   - 把“本地未发布提交”作为 rewrite 的默认边界，对已发布提交、远端提交和多父提交严格限制或要求额外确认

5. **Feedback and regression coverage**
   - 所有提交级动作执行后刷新分支时间线、当前分支位置、标签/分支装饰和同步状态
   - 扩展 `workflow_regressions.rs` 覆盖典型成功 / 冲突 / 禁止场景
   - 为共享提交时间线和动作菜单补充 UI 单元测试，避免再次出现分支视图显示错乱或动作错误启用的回归

## Phase Framing

### MVP

- 分支视图内的共享提交时间线
- 复制版本号、创建补丁、与当前本地上下文比较
- 父/子提交导航
- 基于提交创建分支 / 标签

### V1

- 摘取、回退、重置到这里
- 仅针对当前分支上游的“推送到这里”
- 提交动作禁用原因、危险确认和动作后状态刷新

### V2

- 修改提交说明、fixup、squash、drop、undo last commit、从这里开始整理
- rewrite 后续流程与 rebase editor 融合
- 更完整的冲突 / 待继续状态承接

## Complexity Tracking

| Decision | Why Needed | Simpler Alternative Rejected Because |
|----------|------------|--------------------------------------|
| 抽出共享 `commit_timeline` 组件，而不是在 `branch_popup.rs` 里复制 `history_view.rs` 的图谱代码 | 分支视图和历史视图都需要提交图谱与选中态，复用才能减少回归面 | 复制现有历史视图代码会让两处提交列表继续漂移，后续修 bug 成本更高 |
| 单独增加 `git-core::commit_actions` 模块，而不是把所有动作散落进 `branch.rs` / `commit.rs` / `main.rs` | 该特性引入的提交级动作横跨比较、派生、发布、rewrite，需要统一资格判定和 follow-up state | 把规则写散会让 UI 被迫理解 Git 语义，违反 library-first 原则 |
| “推送到这里”仅面向当前分支的上游和选中的祖先提交 | 这与用户当前仓库主线和 IntelliJ 心智最一致，也最容易解释风险 | 一上来支持任意远端 / 任意 refspec 会让确认、禁用和错误解释急剧复杂化 |
| rewrite 动作默认只作用于当前分支的本地未发布提交 | 这是最符合 PhpStorm 风格也最安全的默认边界 | 对已发布提交开放 rewrite 会放大误操作风险，并让 UI 需要承接更多不可逆场景 |
| 继续复用现有 `rebase_editor` 承接 rewrite 后续流程 | 当前项目已经有 continue / skip / abort 的流程骨架，复用能降低重复实现 | 另起一套 rewrite 后续界面会制造新的状态孤岛，也更容易和现有 rebase 状态打架 |
