# Feature Specification: JetBrains风格Git UI重构

**Feature Branch**: `002-jetbrains-ui-refactor`
**Created**: 2026-03-22
**Status**: Draft
**Input**: User description: "UI要参考jetbrains的样式来实现，代码参考 ~/git/intellij-community/ 里面"

## User Scenarios & Testing

### User Story 1 - 现代化Git工具窗口布局 (Priority: P1)

用户打开Git工具窗口，看到类似IntelliJ IDEA的经典Git界面布局：顶部工具栏、中间主内容区（变更列表+差异面板）、底部状态栏。

**Why this priority**: 界面是用户每天交互的核心，直接影响使用体验和效率。

**Independent Test**: 启动应用后打开任意Git仓库，界面呈现三段式布局（工具栏、主内容区、状态栏）。

**Acceptance Scenarios**:

1. **Given** 应用已启动，**When** 用户打开一个Git仓库，**Then** 顶部显示工具栏，中间左侧显示变更文件列表，中间右侧显示选中文件的差异内容，底部显示当前分支和仓库状态
2. **Given** 工具栏已显示，**When** 用户点击工具栏按钮（如刷新、提交），**Then** 对应功能正常执行
3. **Given** 变更列表已显示，**When** 用户点击某个文件，**Then** 该文件被选中，差异面板显示其内容

---

### User Story 2 - 工具栏设计与交互 (Priority: P1)

工具栏包含常用的Git操作按钮：刷新、提交、拉取、推送、暂存、取消暂存、藏匿。按钮带有图标和可选文字。

**Why this priority**: 工具栏是最高频的操作入口，需要快速识别和访问。

**Independent Test**: 工具栏在界面上正确渲染，点击任意按钮触发对应操作。

**Acceptance Scenarios**:

1. **Given** 工具栏已显示，**When** 用户点击"刷新"按钮，**Then** 变更列表重新加载
2. **Given** 工具栏已显示，**When** 用户点击"提交"按钮，**Then** 弹出提交对话框
3. **Given** 工具栏已显示，**When** 用户点击"拉取"按钮，**Then** 执行git pull操作
4. **Given** 工具栏已显示，**When** 用户点击"推送"按钮，**Then** 执行git push操作
5. **Given** 工具栏已显示，**When** 用户点击"藏匿"按钮，**Then** 弹出藏匿选项菜单

---

### User Story 3 - 变更文件列表 (Priority: P1)

左侧面板以树状结构显示已修改、已暂存、新增、删除的文件。每个文件前显示状态图标（绿色=新增，红色=删除，蓝色=修改）。

**Why this priority**: 变更列表是查看工作区状态的核心功能。

**Independent Test**: 在有变更的仓库中，变更列表正确显示所有变更文件及其状态图标。

**Acceptance Scenarios**:

1. **Given** 仓库有已修改文件，**When** 界面加载完成，**Then** 文件名和状态图标在列表中显示
2. **Given** 变更列表已显示，**When** 用户点击某个文件，**Then** 该文件被选中，差异面板显示其内容
3. **Given** 变更列表已显示，**When** 用户双击某个文件，**Then** 打开文件对比视图
4. **Given** 变更列表已显示，**When** 用户右键点击文件，**Then** 显示上下文菜单（暂存、取消暂存、还原等）

---

### User Story 4 - 差异对比面板 (Priority: P1)

右侧面板显示选中文件的完整差异内容。使用类似JetBrains的分栏布局：左侧显示修改前内容，右侧显示修改后内容，行号显示在左侧。

**Why this priority**: 差异查看是日常代码审查和冲突解决的核心功能。

**Independent Test**: 选择任意已修改文件，差异面板正确高亮显示添加行（绿色背景）、删除行（红色背景）、修改行。

**Acceptance Scenarios**:

1. **Given** 用户选中一个已修改文件，**When** 差异面板显示，**Then** 添加的代码行显示绿色背景，删除的代码行显示红色背景
2. **Given** 差异面板已显示，**When** 用户滚动查看长文件差异，**Then** 行号跟随滚动，内容正确同步
3. **Given** 用户选中一个新增文件，**When** 差异面板显示，**Then** 所有行显示为新增（绿色背景）
4. **Given** 用户选中一个删除文件，**When** 差异面板显示，**Then** 所有行显示为删除（红色背景）

---

### User Story 5 - 提交对话框 (Priority: P2)

弹出式对话框包含：提交消息输入框（多行文本）、变更文件列表（可选中要提交的文件）、提交按钮、取消按钮。

**Why this priority**: 提交是Git工作流的核心操作，需要便捷高效。

**Independent Test**: 点击提交按钮后，对话框正确弹出，用户可以输入消息并提交。

**Acceptance Scenarios**:

1. **Given** 用户点击提交按钮，**When** 对话框弹出，**Then** 消息输入框获得焦点
2. **Given** 提交对话框已打开，**When** 用户输入有效的提交消息并点击提交，**Then** git commit执行成功，对话框关闭
3. **Given** 提交对话框已打开，**When** 用户点击取消，**Then** 对话框关闭，无操作执行
4. **Given** 提交对话框已打开，**When** 用户未输入消息就点击提交，**Then** 显示错误提示，提交不执行

---

### User Story 6 - 分支选择器 (Priority: P2)

工具栏旁边显示当前分支名称，点击后弹出分支选择面板，显示本地分支、远程分支列表，支持创建新分支、切换分支、删除分支。

**Why this priority**: 分支操作是Git工作流的基础操作。

**Independent Test**: 点击分支名称后，弹出分支选择面板，正确显示所有分支。

**Acceptance Scenarios**:

1. **Given** 工具栏已显示，**When** 当前分支为main，**Then** 工具栏显示"main"
2. **Given** 用户点击分支名称，**When** 分支选择面板弹出，**Then** 面板显示本地分支列表和远程分支列表
3. **Given** 分支选择面板已显示，**When** 用户双击某个分支，**Then** 切换到该分支，面板关闭
4. **Given** 分支选择面板已显示，**When** 用户点击"新建分支"，**Then** 弹出输入框，创建新分支

---

### User Story 7 - 底部状态栏 (Priority: P3)

底部状态栏显示：当前仓库路径、当前分支名称、是否有未提交的变更、远程仓库状态（ahead/behind）、推送状态图标。

**Why this priority**: 状态栏提供全局Git状态的快速概览。

**Independent Test**: 底部状态栏始终可见，显示正确的仓库和分支状态信息。

**Acceptance Scenarios**:

1. **Given** 应用已打开仓库，**When** 界面加载完成，**Then** 底部状态栏显示仓库路径和分支名称
2. **Given** 状态栏已显示，**When** 有未提交的变更，**Then** 状态栏显示变更数量或指示器
3. **Given** 状态栏已显示，**When** 远程分支ahead或behind，**Then** 显示对应箭头图标和数量

---

### Edge Cases

- 当仓库没有变更时，变更列表显示空状态提示："工作区是干净的"
- 当仓库没有配置远程仓库时，推送按钮显示禁用状态
- 当网络错误导致拉取/推送失败时，显示错误消息通知
- 当有合并冲突时，冲突文件显示特殊图标，点击可打开冲突解决视图

---

### User Story 8 - 冲突解决与自动合并 (Priority: P1)

当用户遇到合并冲突时，提供一个三路合并（three-way merge）界面，支持：
1. **自动合并**：点击"自动合并"按钮，系统自动分析并合并非冲突的变更（只有单侧修改的 hunks 自动应用）
2. **手动解决**：对于真正的冲突（两侧修改了同一行的不同内容），显示三个面板（ Ours / Base / Theirs），用户逐个 hunk 手动选择
3. **整体接受**：用户也可以直接点击"接受我的"或"接受对方的"来整体接受某一侧的变更

参考 IntelliJ IDEA 的合并对话框架构（`MultipleFileMergeDialog` + `MergeProvider2`）:
- 使用 git stage 的三个版本（stage 1=ours, stage 2=theirs, stage 3=base）
- 通过 `git ls-files --unmerged` 获取冲突文件列表
- 通过 `git show :1/:2/:3 <path>` 读取三个版本的原始内容

**Why this priority**: 冲突解决是 Git 最复杂也是最容易出错的操作，IntelliJ IDEA 的自动合并不冲突变更功能深受用户喜爱，这个功能是核心差异化竞争力。

**Independent Test**: 创建一个有冲突的合并场景，自动合并功能正确识别并合并非冲突部分，剩余冲突可手动解决。

**Acceptance Scenarios**:

1. **Given** 用户在合并过程中遇到冲突，**When** 冲突解决面板打开，**Then** 显示所有冲突文件列表
2. **Given** 冲突文件列表已显示，**When** 用户点击某个文件，**Then** 显示三路差异（Ours / Base / Theirs），非冲突 hunks 预填充在结果面板
3. **Given** 某个冲突 hunks 两侧修改了同一行不同内容，**When** 用户选择接受 Ours，**Then** 该 hunk 使用 Ours 内容
4. **Given** 用户点击"接受我的"，**When** 对于所有冲突文件，**Then** 所有内容替换为 Ours 版本
5. **Given** 用户点击"自动合并"，**When** 系统分析冲突，**Then** 非冲突的 hunks 自动合并，剩余冲突保持手动解决状态

## Requirements

### Functional Requirements

- **FR-001**: 应用主窗口必须包含三个主要区域：顶部工具栏、中间主内容区、底部状态栏
- **FR-002**: 工具栏必须包含以下操作的快速访问按钮：刷新、提交、拉取、推送、暂存全部、取消暂存全部、藏匿
- **FR-003**: 变更列表必须以树状结构显示文件，每个文件必须显示对应的Git状态图标
- **FR-004**: 差异面板必须高亮显示添加行（绿色背景）、删除行（红色背景）
- **FR-005**: 分支选择器必须显示当前分支名称，并支持切换到任意本地或远程分支
- **FR-006**: 提交对话框必须包含消息输入框和文件选择列表
- **FR-007**: 状态栏必须显示：仓库路径、当前分支、未提交变更数量、远程同步状态
- **FR-008**: 所有文本必须使用支持中文的字体（PingFang SC / Microsoft YaHei / Noto Sans CJK）
- **FR-009**: 冲突解决必须支持三路合并（three-way merge），显示 Ours、Base、Theirs 三个面板
- **FR-010**: 冲突解决必须提供"自动合并"功能，自动合并非冲突的 hunks（只有单侧修改的变更自动应用）
- **FR-011**: 冲突解决必须提供"接受我的"/"接受对方的"按钮，整体接受某一侧的变更
- **FR-012**: 冲突解决必须逐 hunk 显示差异，用户可以针对每个冲突 hunk 选择使用哪一侧的内容

### Key Entities

- **Repository**: Git仓库，包含路径、当前分支、变更列表、远程配置
- **Change**: 单一文件的变更，包含路径、状态（新增/修改/删除/重命名/冲突）、差异内容
- **Branch**: Git分支，包含名称、是否为当前分支、是否为远程分支
- **Commit**: Git提交，包含哈希、作者、日期、消息、变更列表
- **ConflictHunk**: 冲突片段，包含在 Base、Ours、Theirs 中的行内容、是否为真正冲突的标记
- **ThreeWayDiff**: 三路差异，包含文件路径、冲突片段列表、Base/Ours/Theirs 三个版本的完整内容

## Success Criteria

### Measurable Outcomes

- **SC-001**: 用户可以在启动应用后5秒内看到一个完整的JetBrains风格Git界面
- **SC-002**: 工具栏的所有按钮必须在点击后100ms内响应
- **SC-003**: 变更列表必须正确显示至少5种Git状态：新增、修改、删除、重命名、冲突
- **SC-004**: 差异面板必须正确高亮差异行，加载1000行文件差异的时间不超过500ms
- **SC-005**: 分支切换操作必须在2秒内完成并更新界面显示
- **SC-006**: 界面中文文本必须正确显示，不出现方块乱码
- **SC-007**: 窗口最小尺寸为800x600，默认尺寸为1280x800
- **SC-008**: 自动合并功能必须在500ms内完成对单个文件的非冲突 hunks 识别和合并
- **SC-009**: 冲突解决界面必须显示 Base / Ours / Theirs 三个面板，正确高亮冲突行

## Clarifications

### Session 2026-03-22

- Q: 冲突解决是否需要自动合并非冲突的变更（参考 IntelliJ IDEA 的功能）？
  → A: **混合模式**（选项 C）— 提供一个"自动合并"按钮，用户可以先自动合并非冲突部分，再手动解决剩余冲突。参考 `~/git/intellij-community/plugins/git4idea/` 中的 `GitMergeProvider.java`、`GitMergeUtil.java` 和 `MultipleFileMergeDialog.kt` 实现。
