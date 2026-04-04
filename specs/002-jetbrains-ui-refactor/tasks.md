# Tasks: JetBrains风格Git UI重构

**Input**: Design documents from `/specs/002-jetbrains-ui-refactor/`
**Prerequisites**: plan.md, spec.md (8个用户故事), research.md, data-model.md, contracts/

## 格式: `[ID] [P?] [故事?] 描述`

- **[P]**: 可并行执行（不同文件，无依赖）
- **[故事]**: 属于哪个用户故事 (US1~US8)
- 描述中包含具体文件路径

---

## Phase 1: 基础设施搭建

**Purpose**: 完善现有项目结构，为所有用户故事做准备

- [x] T001 [P] 更新 `src-ui/src/state.rs` 添加 ViewMode 枚举和冲突解决相关状态
- [x] T002 [P] 扩展 `src/git-core/src/diff.rs` 中的 `ConflictHunkType` 枚举，添加 `OursOnly`、`TheirsOnly` 以支持自动合并算法
- [x] T003 [P] 在 `src/git-core/src/diff.rs` 中实现 `get_conflict_diff()` 函数获取所有冲突文件的三路差异
- [x] T004 [P] 在 `src/git-core/src/diff.rs` 中实现 `auto_merge_conflict()` 函数，自动合并非冲突 hunks
- [x] T005 [P] 在 `src/git-core/src/diff.rs` 中实现 `resolve_conflict_hunk()` 函数处理单个 hunk 的冲突解决
- [x] T006 [P] 在 `src/git-core/src/lib.rs` 中导出新的 diff 模块函数

**Checkpoint**: 基础库函数就绪，UI 实现可以开始

---

## Phase 2: 用户故事 1 - 现代化Git工具窗口布局 (P1) 🎯 MVP

**Goal**: 实现 JetBrains 风格的三段式窗口布局：工具栏 + 主内容区 + 状态栏

**Independent Test**: 启动应用后打开 Git 仓库，界面正确显示三段式布局

### 实现

- [x] T007 [P] [US1] 重构 `src-ui/src/views/main_window.rs` 实现三段式布局结构
- [x] T008 [US1] 在 `src-ui/src/views/main_window.rs` 中添加工具栏区域 (FR-001, FR-002)
- [x] T009 [US1] 在 `src-ui/src/views/main_window.rs` 中添加主内容区分栏布局 (左侧变更列表 + 右侧差异面板)
- [x] T010 [US1] 在 `src-ui/src/views/main_window.rs` 中添加底部状态栏区域 (FR-007)
- [x] T011 [US1] 配置窗口默认尺寸 1280x800，最小尺寸 800x600 (SC-007)
- [x] T012 [US1] 配置中文字体 PingFang SC (SC-006)

**Checkpoint**: 主窗口布局完成，呈现 JetBrains 风格

---

## Phase 3: 用户故事 2 - 工具栏设计与交互 (P1)

**Goal**: 实现工具栏按钮：刷新、提交、拉取、推送、暂存全部、取消暂存全部、藏匿

**Independent Test**: 点击工具栏按钮，相应功能正常执行且响应时间 <100ms (SC-002)

### 实现

- [x] T013 [P] [US2] 创建 `src-ui/src/widgets/toolbar.rs` 实现工具栏组件
- [x] T014 [P] [US2] 在 `src-ui/src/widgets/toolbar.rs` 中添加刷新按钮
- [x] T015 [P] [US2] 在 `src-ui/src/widgets/toolbar.rs` 中添加提交按钮（打开提交对话框）
- [x] T016 [P] [US2] 在 `src-ui/src/widgets/toolbar.rs` 中添加拉取按钮
- [x] T017 [P] [US2] 在 `src-ui/src/widgets/toolbar.rs` 中添加推送按钮
- [x] T018 [P] [US2] 在 `src-ui/src/widgets/toolbar.rs` 中添加暂存全部按钮
- [x] T019 [P] [US2] 在 `src-ui/src/widgets/toolbar.rs` 中添加取消暂存全部按钮
- [x] T020 [P] [US2] 在 `src-ui/src/widgets/toolbar.rs` 中添加藏匿按钮（打开藏匿面板）
- [x] T021 [US2] 在 `src-ui/src/state.rs` 中添加 `toolbar_click()` 消息处理函数
- [x] T022 [US2] 在 `src-ui/src/main.rs` 中集成工具栏到主窗口

**Checkpoint**: 工具栏完整可用

---

## Phase 4: 用户故事 3 - 变更文件列表 (P1)

**Goal**: 实现变更文件列表，以树状结构显示，状态图标区分文件状态

**Independent Test**: 在有变更的仓库中，变更列表正确显示所有变更文件及状态图标

### 实现

- [x] T023 [P] [US3] 重构 `src-ui/src/widgets/changelist.rs` 实现树状目录结构
- [x] T024 [P] [US3] 在 `src-ui/src/widgets/changelist.rs` 中添加文件状态图标渲染（绿色=新增，红色=删除，蓝色=修改，黄色=冲突）
- [x] T025 [US3] 在 `src-ui/src/widgets/changelist.rs` 中实现点击文件选中功能
- [x] T026 [US3] 在 `src-ui/src/widgets/changelist.rs` 中实现上下文菜单（暂存、取消暂存、还原）- 替代方案：选中文件后显示操作按钮面板
- [x] T027 [US3] 在 `src-ui/src/widgets/changelist.rs` 中实现空状态提示"工作区是干净的"
- [x] T028 [US3] 更新 `src-ui/src/state.rs` 添加选中变更文件的状态管理

**Checkpoint**: 变更列表完整实现，支持 5 种 Git 状态 (SC-003)

---

## Phase 5: 用户故事 4 - 差异对比面板 (P1)

**Goal**: 实现分栏差异面板，支持统一视图和分栏视图切换，高亮显示差异行

**Independent Test**: 选择任意已修改文件，差异面板正确高亮显示差异行，1000行文件加载时间 <500ms (SC-004)

### 实现

- [x] T029 [P] [US4] 重构 `src-ui/src/widgets/split_diff_viewer.rs` 实现三栏布局（ Ours / Base / Theirs）
- [x] T030 [P] [US4] 在 `src-ui/src/widgets/split_diff_viewer.rs` 中添加统一视图模式
- [x] T031 [P] [US4] 在 `src-ui/src/widgets/split_diff_viewer.rs` 中添加分栏视图模式
- [x] T032 [US4] 在 `src-ui/src/widgets/split_diff_viewer.rs` 中实现差异行高亮（绿色=添加，红色=删除）
- [x] T033 [US4] 在 `src-ui/src/widgets/split_diff_viewer.rs` 中实现行号显示和滚动同步
- [x] T034 [US4] 在 `src-ui/src/state.rs` 中添加差异面板状态管理和消息处理

**Checkpoint**: 差异面板完整实现

**Checkpoint**: 差异面板完整实现

---

## Phase 6: 用户故事 5 - 提交对话框 (P2)

**Goal**: 实现提交对话框，包含消息输入框和文件选择列表

**Independent Test**: 点击提交按钮后对话框弹出，输入消息并提交成功

### 实现

- [x] T035 [P] [US5] 重构 `src-ui/src/views/commit_dialog.rs` 实现提交对话框 UI
- [x] T036 [P] [US5] 在 `src-ui/src/views/commit_dialog.rs` 中添加多行文本输入框
- [x] T037 [P] [US5] 在 `src-ui/src/views/commit_dialog.rs` 中添加变更文件列表（可选择）
- [x] T038 [US5] 在 `src-ui/src/views/commit_dialog.rs` 中添加消息非空验证
- [x] T039 [US5] 在 `src-ui/src/views/commit_dialog.rs` 中集成 git commit 操作
- [x] T040 [US5] 在 `src-ui/src/state.rs` 中添加提交对话框状态和消息处理

**Checkpoint**: 提交对话框完整实现

---

## Phase 7: 用户故事 6 - 分支选择器 (P2)

**Goal**: 实现分支选择器，显示当前分支名称，支持切换分支

**Independent Test**: 点击分支名称后弹出选择面板，正确显示所有分支

### 实现

- [x] T041 [P] [US6] 重构 `src-ui/src/views/branch_popup.rs` 实现分支选择面板
- [x] T042 [P] [US6] 在 `src-ui/src/views/branch_popup.rs` 中添加本地分支列表显示
- [x] T043 [P] [US6] 在 `src-ui/src/views/branch_popup.rs` 中添加远程分支列表显示
- [x] T044 [US6] 在 `src-ui/src/views/branch_popup.rs` 中实现双击切换分支
- [x] T045 [US6] 在 `src-ui/src/views/branch_popup.rs` 中实现新建分支功能
- [x] T046 [US6] 在工具栏上显示分支名称

**Checkpoint**: 分支选择器完整实现

---

## Phase 8: 用户故事 7 - 底部状态栏 (P3)

**Goal**: 实现底部状态栏，显示仓库路径、分支、变更数量、同步状态

**Independent Test**: 底部状态栏始终可见，显示正确信息

### 实现

- [x] T047 [P] [US7] 创建 `src-ui/src/widgets/statusbar.rs` 实现状态栏组件
- [x] T048 [P] [US7] 在 `src-ui/src/widgets/statusbar.rs` 中添加仓库路径显示
- [x] T049 [P] [US7] 在 `src-ui/src/widgets/statusbar.rs` 中添加当前分支名称显示
- [x] T050 [US7] 在 `src-ui/src/widgets/statusbar.rs` 中添加未提交变更数量显示
- [x] T051 [US7] 在 `src-ui/src/widgets/statusbar.rs` 中添加远程同步状态显示（ahead/behind 箭头）
- [x] T052 [US7] 在 `src-ui/src/views/main_window.rs` 中集成状态栏

**Checkpoint**: 状态栏完整实现

---

## Phase 9: 用户故事 8 - 冲突解决与自动合并 (P1)

**Goal**: 实现三路合并界面，支持自动合并非冲突变更

**Independent Test**: 创建有冲突的合并场景，自动合并功能正确识别并合并非冲突部分

### 实现

- [x] T053 [P] [US8] 重构 `src-ui/src/widgets/conflict_resolver.rs` 实现三路合并 UI
- [x] T054 [P] [US8] 在 `src-ui/src/widgets/conflict_resolver.rs` 中添加 Ours/Base/Theirs 三栏显示
- [x] T055 [P] [US8] 在 `src-ui/src/widgets/conflict_resolver.rs` 中添加结果预览面板
- [x] T056 [US8] 在 `src-ui/src/widgets/conflict_resolver.rs` 中添加"接受我的"按钮
- [x] T057 [US8] 在 `src-ui/src/widgets/conflict_resolver.rs` 中添加"接受对方的"按钮
- [x] T058 [US8] 在 `src-ui/src/widgets/conflict_resolver.rs` 中添加"自动合并"按钮
- [x] T059 [US8] 在 `src-ui/src/widgets/conflict_resolver.rs` 中实现逐 hunk 选择功能
- [x] T060 [US8] 集成 `src/git-core/src/diff.rs` 中的 `auto_merge_conflict()` 函数
- [x] T061 [US8] 在 `src-ui/src/state.rs` 中添加冲突解决状态管理
- [x] T062 [US8] 在 `src-ui/src/main.rs` 中添加冲突检测和自动打开冲突解决视图

**Checkpoint**: 冲突解决完整实现，支持自动合并算法

---

## Phase 10: 收尾与跨领域优化

**Purpose**: 跨用户故事的优化和收尾工作

- [x] T063 [P] 运行 `cargo clippy` 并修复所有警告
- [x] T064 [P] 运行 `cargo fmt` 格式化代码
- [x] T065 运行 `cargo test --workspace` 确保所有测试通过
- [x] T066 [P] 更新 `src-ui/src/i18n.rs` 确保所有中文文本正确
- [x] T067 更新 `specs/002-jetbrains-ui-refactor/quickstart.md` 添加新的构建说明
- [ ] T068 验证 SC-001 ~ SC-009 所有成功标准（需要运行时手动验证）

---

## 依赖关系与执行顺序

### 阶段依赖

- **Phase 1 (基础设施)**: 无依赖，可立即开始
- **Phase 2~9 (用户故事)**: 依赖 Phase 1 完成
- **Phase 10 (收尾)**: 依赖所有用户故事完成

### 用户故事依赖

| 用户故事 | 依赖 | 说明 |
|---------|------|------|
| US1 窗口布局 | Phase 1 | 基础库函数就绪后才能实现布局 |
| US2 工具栏 | US1, Phase 1 | 依赖主窗口布局 |
| US3 变更列表 | US1, Phase 1 | 依赖主窗口布局 |
| US4 差异面板 | US1, US3, Phase 1 | 依赖主窗口布局和变更列表 |
| US5 提交对话框 | US1, US3, Phase 1 | 依赖主窗口布局和变更列表 |
| US6 分支选择器 | US1, Phase 1 | 依赖主窗口布局 |
| US7 状态栏 | US1, Phase 1 | 依赖主窗口布局 |
| US8 冲突解决 | Phase 1 | 依赖基础库函数（自动合并算法） |

### 并行机会

- Phase 1 中所有任务可并行（T001~T006）
- US1 的 T007~T012 可部分并行（T007 是基础结构，T008~T012 可在 T007 后并行）
- US2 的 T013~T022 可并行（T013 是基础组件，T014~T020 按钮实现可并行）
- US3~US7 之间可并行（各自独立）
- US8 依赖 Phase 1，可与 US1~US7 并行

---

## MVP 范围建议

**最小可行产品**: Phase 1 + US1 + US2 + US3

执行步骤:
1. 完成 Phase 1 (T001~T006) — 基础库函数
2. 完成 US1 (T007~T012) — 主窗口三段式布局
3. 完成 US2 (T013~T022) — 工具栏按钮
4. 完成 US3 (T023~T028) — 变更文件列表
5. **验证**: 可打开仓库、显示变更列表、点击工具栏按钮

---

## 任务统计

| 用户故事 | 任务数 | 优先级 |
|---------|-------|-------|
| Phase 1 基础设施 | 6 | - |
| US1 窗口布局 | 6 | P1 |
| US2 工具栏 | 10 | P1 |
| US3 变更列表 | 6 | P1 |
| US4 差异面板 | 6 | P1 |
| US5 提交对话框 | 6 | P2 |
| US6 分支选择器 | 6 | P2 |
| US7 状态栏 | 6 | P3 |
| US8 冲突解决 | 10 | P1 |
| Phase 10 收尾 | 6 | - |
| **总计** | **68** | |

---

## 独立测试标准

每个用户故事完成后的独立测试标准:

- **US1**: 启动应用 → 打开仓库 → 界面显示三段式布局（工具栏、主内容区、状态栏）
- **US2**: 点击工具栏每个按钮 → 验证对应操作执行（刷新、提交对话框弹出等）
- **US3**: 修改几个文件 → 验证变更列表显示所有文件及正确状态图标
- **US4**: 点击变更列表中的文件 → 验证差异面板显示并高亮差异行
- **US5**: 点击提交 → 输入消息 → 选择文件 → 提交成功
- **US6**: 点击分支名称 → 弹出选择面板 → 切换分支成功
- **US7**: 观察底部状态栏 → 验证显示仓库路径、分支、变更数量、同步状态
- **US8**: 创建冲突合并 → 点击"自动合并" → 验证非冲突部分自动合并，剩余冲突手动解决
