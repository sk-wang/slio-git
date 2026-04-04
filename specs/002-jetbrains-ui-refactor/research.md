# Research: JetBrains风格Git UI重构

**Feature**: JetBrains风格Git UI重构
**Date**: 2026-03-22
**Branch**: `002-jetbrains-ui-refactor`

## Research 1: 自动合并算法 (Auto-Merge)

**Question**: 如何实现 IntelliJ IDEA 风格的自动合并非冲突变更功能？

### Findings

**IntelliJ IDEA 的冲突解决架构** (from `~/git/intellij-community/plugins/git4idea/`):

1. **核心组件**:
   - `GitConflictResolver.java` — 顶层协调器，处理冲突文件列表
   - `GitMergeProvider.java` — 实现 `MergeProvider2` 接口，提供 `loadRevisions()` 加载三路内容
   - `GitMergeUtil.java` — 工具类，提供 `loadMergeData()`, `acceptOneVersion()`, `markConflictResolved()`
   - `MultipleFileMergeDialog.kt` — 多文件合并对话框 UI

2. **Git Stage 机制**:
   - Stage 1 (`:1:<path>`) = ORIGINAL / BASE — 共同祖先
   - Stage 2 (`:2:<path>`) = YOURS / OURS — 当前分支版本
   - Stage 3 (`:3:<path>`) = THEIRS — 被合并的分支版本

3. **获取冲突文件**:
   ```java
   GitLineHandler h = new GitLineHandler(project, root, GitCommand.LS_FILES);
   h.addParameters("--exclude-standard", "--unmerged", "-t", "-z");
   ```
   返回格式: `M <mode> <object> <stage>\t<path>\0`

4. **三路差异加载** (`GitMergeUtil.loadMergeData()`):
   ```java
   byte[] originalContent = loadRevision(project, root, path, ORIGINAL_REVISION_NUM);
   byte[] yoursContent = loadRevision(project, root, path, YOURS_REVISION_NUM);
   byte[] theirsContent = loadRevision(project, root, path, THEIRS_REVISION_NUM);
   ```

5. **自动合并算法**:
   - 对于每个 hunk，比较 Base、Ours、Theirs 三方内容
   - 如果只有一方修改（Ours≠Base && theirs==Base 或反之），自动接受修改
   - 如果两方都修改了同一区域但内容不同，标记为真正冲突，需要手动解决
   - 参考 `classify_line()` 函数（在 `git4idea` 中由 Diff 框架处理）

6. **冲突状态类型**:
   - `DEFAULT` = 三方都修改了
   - `ADDED_ADDED` = Ours 和 Theirs 都新增了
   - `MODIFIED_DELETED` / `DELETED_MODIFIED` = 一方修改一方删除

### Decision

采用与 IntelliJ 相同的算法：
1. 使用 `git ls-files --unmerged` 获取冲突文件列表
2. 对每个冲突文件，使用 `git show :1/:2/:3 <path>` 读取三个 stage 内容
3. 三路 diff 分析后，自动合并非冲突 hunks（只有单侧修改）
4. 剩余真正冲突（两侧修改了同一区域不同内容）保持手动解决

### Rationale

IntelliJ IDEA 的算法经过多年验证，是最用户友好的方式。用户只需要处理真正冲突的部分，而不是所有变更。

### Alternatives Considered

- **Git 的 `git merge --no-commit` + 手动解决**: 太底层，不提供 UI
- **Git 的 `git rerere`**: 仅记录解决，不自动应用
- **手动全量替换 Ours/Theirs**: 用户体验差，需要重复处理非冲突变更

---

## Research 2: Iced UI 框架能力

**Question**: Pure Iced 是否支持 JetBrains 风格 UI 所需的组件？

### Findings

**Iced 0.13 能力**:

1. **布局组件**:
   - `Column`, `Row`, `Container` — 基础布局
   - `Space`, `Rule` — 分隔和填充
   - `Pane` — 面板分割（可用于左右分栏）

2. **组件**:
   - `Text`, `Button`, `TextInput`, `Checkbox`
   - `Scrollable` — 可滚动区域
   - `PickList` / `ComboBox` — 下拉选择
   - `Image`, `Svg` — 静态资源

3. **状态管理**:
   - 应用状态通过 `State` struct 统一管理
   - `Subscription` 处理键盘、鼠标事件
   - `Task` 处理异步操作

4. **已知限制**:
   - 无内置 TreeView（需要自己实现或用 List 模拟）
   - 无内置 Table 组件（需要用 List + custom rendering）
   - 无原生对话框系统（需要自己用 Container + Overlay 实现）
   - 字体需要显式指定（已设置 PingFang SC 支持中文）

### Decision

Iced 0.13 完全满足需求：
- 使用 `Scrollable` + 自定义渲染实现文件树
- 使用 `Container` + `Layer` 实现对话框
- 使用 `Pane::horizontal()` 实现左右分栏差异面板

### Rationale

Pure Rust 架构是 Constitution II 的强制要求。Iced 虽有局限但完全可用，且避免了 Tauri/WebView 的性能和二进制大小开销。

---

## Research 3: 三路差异数据结构

**Question**: 如何在 Rust 中表示三路差异和冲突 hunks？

### Findings

**从 `src/git-core/src/diff.rs` 已有实现**:

```rust
pub struct ConflictHunk {
    pub base_lines: Vec<String>,
    pub ours_lines: Vec<String>,
    pub theirs_lines: Vec<String>,
    pub line_type: ConflictLineType,
}

pub enum ConflictLineType {
    Unchanged,
    OursOnly,      // 只有我们修改了
    TheirsOnly,     // 只有对方修改了
    Modified,       // 双方都修改了（真正冲突）
    ConflictMarker, // 冲突标记
}

pub struct ThreeWayDiff {
    pub path: String,
    pub hunks: Vec<ConflictHunk>,
    pub has_conflicts: bool,
    pub base_content: String,
    pub ours_content: String,
    pub theirs_content: String,
}
```

### Decision

保持现有 `ThreeWayDiff` 和 `ConflictHunk` 结构，但增强 `ConflictLineType` 分类逻辑以支持自动合并：

```rust
pub enum ConflictLineType {
    Unchanged,      // 三方相同
    OursOnly,      // 只有 Ours 修改（自动合并安全）
    TheirsOnly,    // 只有 Theirs 修改（自动合并安全）
    Modified,      // 双方都修改了（真正冲突）
    Empty,
    ConflictMarker,
}
```

### Rationale

现有的 `ConflictHunk` 已经包含三路内容，只需增强分类算法即可支持自动合并。
