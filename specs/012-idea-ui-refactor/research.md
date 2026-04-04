# Research: IDEA Git 视图重构

**Branch**: `012-idea-ui-refactor` | **Date**: 2026-04-04

## R1: Uncommit 操作实现

**Decision**: 使用 `git reset --soft <commit>^` 实现 uncommit（软重置到目标提交的父提交）

**Rationale**: IDEA 的 Uncommit 本质是 soft reset，保留所有改动在暂存区。对于选中的提交（非 HEAD），等价于 `git reset --soft <selected_commit>^`，撤销从 HEAD 到该提交的所有提交。git2-rs 的 `repo.reset()` 支持 Soft 模式。

**Alternatives considered**:
- Interactive rebase with drop（太重，且需要冲突处理）
- Shell out to `git reset --soft`（可行但不一致，其他重置操作使用 git2）

## R2: Keep Index Stash 参数

**Decision**: 在 `stash_save_with_options` 的 git CLI 调用中添加 `--keep-index` 参数

**Rationale**: git2-rs 的 stash API 支持 `STASH_KEEP_INDEX` flag，但现有实现使用 shell command。保持一致，在 `git stash push` 命令中添加 `--keep-index`。

**Alternatives considered**:
- 使用 git2-rs native stash API（需要重写整个 stash_save 函数，风险高）

## R3: Unstash As Branch 操作

**Decision**: 使用 `git stash branch <branch_name> stash@{N}` 实现

**Rationale**: Git 原生支持 `git stash branch` 命令，从贮藏创建新分支并应用改动。这是最安全的实现方式，与 IDEA 行为完全一致。

**Alternatives considered**:
- 手动创建分支 + stash apply（两步操作，不原子，可能中间状态不一致）

## R4: 全文预览渲染策略

**Decision**: 当 diff 为空或文件为新增时，构造一个"全文 addition" diff，所有行标记为 Addition origin

**Rationale**: 现有 DiffViewer 已支持按行渲染 Addition/Deletion/Context。将整个文件内容包装为全部 Addition 行的 FileDiff，复用现有渲染逻辑，无需新建预览组件。对于二进制文件，检测文件扩展名或内容中的 null byte，显示友好提示。

**Alternatives considered**:
- 新建 FilePreviewWidget（大量重复代码，维护成本高）
- 使用 Iced TextEditor（缺少行号和语法高亮）

## R5: 拖拽重排实现策略

**Decision**: 使用 Iced mouse events + 行索引交换 + 视觉反馈（高亮目标位置）

**Rationale**: Iced 0.14 无内置 DnD。在 rebase editor 的表格中：(1) mouse_area 检测 ButtonPressed 开始拖拽，(2) 跟踪鼠标位置计算目标行，(3) 高亮插入位置，(4) ButtonReleased 完成交换。与 011 中 changelist 的 DnD 方案一致。

**Alternatives considered**:
- 仅用上移/下移按钮（不满足 IDEA 拖拽交互要求）

## R6: 行内编辑实现策略

**Decision**: 双击消息列切换为 TextInput 组件，失焦或 Enter 确认编辑

**Rationale**: Iced 的 TextInput 可以动态显示/隐藏。双击时将该行的消息列从 Text 切换为 TextInput，编辑完成后切回 Text。这是标准的行内编辑模式，与 IDEA 的表格行为一致。

**Alternatives considered**:
- 弹出对话框编辑（不满足"行内编辑"要求）
