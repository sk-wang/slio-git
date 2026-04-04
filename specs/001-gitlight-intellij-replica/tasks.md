# Tasks: slio-git - IntelliJ 兼容 Git 客户端

**输入**: 设计文档 from `/specs/001-gitlight-intellij-replica/`
**前置要求**: plan.md (必须), spec.md (必须), data-model.md, research.md

## 格式: `[ID] [P?] [Story] 描述`

- **[P]**: 可并行运行（不同文件，无依赖）
- **[Story]**: 所属用户故事 (例如 US1, US2, US3)
- 描述中包含确切的文件路径

## 路径约定

- **项目结构**: `src/` (git-core), `src-ui/` (Iced UI), `tests/` 在仓库根目录
- 路径假设为单项目结构 - 根据 plan.md 调整

## 依赖关系与执行顺序

### 阶段依赖

- **阶段 1 (Setup)**: 无依赖 - 可立即开始
- **阶段 2 (Foundational)**: 依赖 Setup 完成 - 阻塞所有用户故事
- **用户故事 (阶段 3+)**: 全部依赖 Foundational 阶段完成
  - 用户故事可以并行进行（如果有团队资源）
  - 或按优先级顺序执行 (P1 → P2 → P3)
- **Polish (最终阶段)**: 依赖所有用户故事完成

### 用户故事依赖

- **用户故事 1 (P1)**: Foundational 完成后可开始 - 无其他故事依赖
- **用户故事 2 (P1)**: Foundational 完成后可开始 - 可与 US1 并行
- **用户故事 3 (P1)**: Foundational 完成后可开始 - 可与 US1/US2 并行
- **用户故事 4 (P1)**: Foundational 完成后可开始 - 可与 US1/US2/US3 并行
- **用户故事 5 (P2)**: Foundational 完成后可开始
- **用户故事 6 (P2)**: Foundational 完成后可开始
- **用户故事 7 (P2)**: Foundational 完成后可开始
- **用户故事 8 (P3)**: Foundational 完成后可开始
- **用户故事 9 (P3)**: Foundational 完成后可开始

---

## 阶段 1: 项目初始化

**目的**: 创建 Pure Iced + git-core 架构的基础项目结构

- [x] T001 创建 Rust workspace 结构，配置文件在 `Cargo.toml`
- [x] T002 初始化 `git-core` crate，配置 `Cargo.toml` 依赖 (git2-rs, notify)
- [x] T003 初始化 `src-ui` crate，配置 `Cargo.toml` 依赖 (iced)
- [x] T004 [P] 配置 src-ui 的中文本地化，文件在 `src-ui/src/i18n.rs`
- [x] T005 [P] 配置项目格式化 (`rustfmt.toml`) 和 lint (`clippy.toml`)
- [x] T006 配置日志系统，文件在 `src-ui/src/logging.rs`
- [x] T007 创建 git-core 基础模块结构 `src/git-core/src/lib.rs`

---

## 阶段 2: 基础设施 (阻塞前置条件)

**目的**: 构建所有用户故事依赖的核心基础设施

**⚠️ 关键**: 用户故事工作在 Foundational 阶段完成前不能开始

### git-core 库基础设施

- [x] T008 创建 Repository 结构体，文件在 `src/git-core/src/repository.rs`
- [x] T009 实现仓库检测逻辑 (`.git` 目录/文件扫描)
- [x] T010 创建 Branch、Commit、Remote、Tag、Stash 结构体
- [x] T011 创建 Change、IndexEntry 结构体用于文件变更跟踪
- [x] T012 实现 git2-rs 初始化和错误处理
- [x] T013 配置 git-core 单元测试框架

### UI 应用基础设施

- [x] T014 [P] 创建 Iced Application 主结构，文件在 `src-ui/src/main.rs`
- [x] T015 [P] 创建 Iced 状态管理，文件在 `src-ui/src/state.rs`
- [x] T016 实现文件选择器 UI (使用 `rfd` crate)
- [x] T017 创建基础 widgets (Button, TextInput, Scrollable 等的包装)
- [x] T018 创建主窗口布局，文件在 `src-ui/src/views/main_window.rs`
- [x] T019 配置 notify 文件监视集成
- [x] T020 实现后台线程池用于 git 操作 (tokio 或标准线程)

**检查点**: 基础设施就绪 - 用户故事实现可以开始

---

## 阶段 3: 用户故事 1 - 仓库管理 (优先级: P1) 🎯 MVP

**目标**: 用户可以打开或初始化 git 仓库，应用程序自动检测仓库状态

**独立测试**: 打开一个已存在的 git 仓库，验证所有状态信息正确检测和显示

### 实现

- [x] T021 [P] [US1] 在 `src/git-core/src/repository.rs` 实现 `open_repository` 方法
- [x] T022 [P] [US1] 实现 `init_repository` 方法创建新仓库
- [x] T023 [US1] 实现 RepositoryManager 跟踪多个仓库实例
- [x] T024 [US1] 实现 worktree 检测逻辑
- [x] T025 [US1] 在 UI 中添加"打开仓库"和"初始化仓库"按钮
- [x] T026 [US1] 实现仓库状态显示面板 (分支、远程、文件状态)
- [x] T027 [US1] 添加日志记录仓库操作 (FR-005)

**检查点**: 此时用户故事 1 应该完全可用，可以独立测试

---

## 阶段 4: 用户故事 2 - 文件变更与暂存 (优先级: P1) 🎯 MVP

**目标**: 用户可以查看所有文件变更并暂存/取消暂存，与 IntelliJ 行为完全一致

**独立测试**: 修改仓库中的文件，验证变更检测和暂存行为与 IntelliJ 一致

### 实现

- [x] T028 [P] [US2] 在 `src/git-core/src/index.rs` 实现暂存区操作
- [x] T029 [P] [US2] 实现 `stage_file` 和 `unstage_file` 方法
- [x] T030 [US2] 实现 hunk 级别的暂存 (解析 diff 生成 hunks)
- [x] T031 [US2] 实现文件状态检测 (git2-rs StatusOptions)
- [x] T032 [US2] 实现 Conflict 检测 (检查 MERGE_HEAD)
- [x] T033 [US2] 创建变更列表 UI 组件，文件在 `src-ui/src/widgets/changelist.rs`
- [x] T034 [US2] 实现变更面板视图，文件在 `src-ui/src/views/changes_panel.rs`
- [x] T035 [US2] 添加冲突文件的三向 diff 显示支持
- [x] T036 [US2] 添加暂存/取消暂存的快捷键支持

**检查点**: 此时用户故事 1 和 2 应该都可用

---

## 阶段 5: 用户故事 3 - 提交操作 (优先级: P1) 🎯 MVP

**目标**: 用户可以创建提交、修改提交、查看历史，与 IntelliJ 完全一致

**独立测试**: 创建各种格式的提交，验证生成的 git 对象符合预期状态

### 实现

- [x] T037 [P] [US3] 在 `src/git-core/src/commit.rs` 实现提交创建
- [x] T038 [P] [US3] 实现 `create_commit` 和 `amend_commit` 方法
- [x] T039 [US3] 实现 Signature 创建 (作者/提交者信息)
- [x] T040 [US3] 实现提交历史读取 (`git log`)
- [x] T041 [US3] 创建提交对话框 UI，文件在 `src-ui/src/views/commit_dialog.rs`
- [x] T042 [US3] 实现提交消息编辑器组件
- [x] T043 [US3] 实现 Diff 预览面板
- [x] T044 [US3] 实现"修改上次提交"功能
- [x] T045 [US3] 实现提交历史视图，文件在 `src-ui/src/views/history_view.rs`

**检查点**: P1 功能 (US1-US3) 全部可用 - MVP 完成

---

## 阶段 6: 用户故事 4 - 分支操作 (优先级: P1) 🎯 MVP

**目标**: 用户可以创建、切换、合并、删除、重命名分支，与 IntelliJ 分支对话框一致

**独立测试**: 执行分支操作，验证仓库状态符合预期分支配置

### 实现

- [x] T046 [P] [US4] 在 `src/git-core/src/branch.rs` 实现分支操作
- [x] T047 [P] [US4] 实现 `create_branch`, `delete_branch`, `rename_branch`
- [x] T048 [US4] 实现 `checkout_branch` 和 `get_current_branch`
- [x] T049 [US4] 实现 `merge_branch` 方法
- [x] T050 [US4] 实现分支列表获取 (本地和远程跟踪分支)
- [x] T051 [US4] 创建分支弹出窗口 UI，文件在 `src-ui/src/views/branch_popup.rs`
- [x] T052 [US4] 实现分支选择器组件
- [x] T053 [US4] 实现合并对话框
- [x] T054 [US4] 实现分支删除确认对话框

**检查点**: 分支操作功能完整

---

## 阶段 7: 用户故事 5 - 远程操作 (优先级: P2)

**目标**: 用户可以执行 fetch、pull、push，认证处理与 IntelliJ 一致

**独立测试**: 对测试远程执行远程操作，验证数据传输和分支跟踪正确

### 实现

- [x] T055 [P] [US5] 在 `src/git-core/src/remote.rs` 实现远程操作
- [x] T056 [P] [US5] 实现 `fetch`, `push` 方法
- [x] T057 [US5] 实现 `pull` 方法 (fetch + merge)
- [x] T058 [US5] 实现 SSH 密钥和凭据助手认证回调
- [x] T059 [US5] 实现进度报告回调
- [x] T060 [US5] 创建远程操作对话框 UI，文件在 `src-ui/src/views/remote_dialog.rs`
- [x] T061 [US5] 实现凭据输入对话框
- [x] T062 [US5] 添加远程分支显示到分支弹出窗口

**检查点**: 远程操作功能完整

---

## 阶段 8: 用户故事 6 - 储藏管理 (优先级: P2)

**目标**: 用户可以储藏、列出、应用、删除储藏，与 IntelliJ 储藏面板一致

**独立测试**: 储藏变更，验证储藏列表和应用/删除功能

### 实现

- [x] T063 [P] [US6] 在 `src/git-core/src/stash.rs` 实现储藏操作
- [x] T064 [P] [US6] 实现 `stash_save`, `stash_pop`, `stash_drop`
- [x] T065 [US6] 实现储藏列表获取
- [x] T066 [US6] 创建储藏面板 UI，文件在 `src-ui/src/views/stash_panel.rs`
- [x] T067 [US6] 实现储藏详情视图
- [x] T068 [US6] 添加储藏相关的快捷键支持

**检查点**: 储藏管理功能完整

---

## 阶段 9: 用户故事 7 - Diff 与历史查看器 (优先级: P2)

**目标**: 用户可以查看文件和提交 diff、浏览历史、搜索提交，与 IntelliJ 一致

**独立测试**: 查看提交和文件的 diff，验证视觉呈现符合预期格式

### 实现

- [x] T069 [P] [US7] 在 `src/git-core/src/diff.rs` 实现 diff 生成
- [x] T070 [P] [US7] 实现 `diff_commit`, `diff_file`, `diff_workdir` 方法
- [x] T071 [US7] 实现 DiffHunk 和 DiffLine 结构
- [x] T072 [US7] 创建 Diff 查看器 UI，文件在 `src-ui/src/widgets/diff_viewer.rs`
- [x] T073 [US7] 实现统一 diff 视图
- [x] T074 [US7] 实现分屏 diff 视图
- [ ] T075 [US7] 实现语法高亮 (使用 `syntect` 或类似库)
- [x] T076 [US7] 实现历史搜索功能
- [x] T077 [US7] 实现提交比较功能

**检查点**: Diff 和历史查看功能完整

---

## 阶段 10: 用户故事 8 - 变基操作 (优先级: P3)

**目标**: 用户可以执行交互式变基，使用与 IntelliJ 相同的基于编辑器的工作流程

**独立测试**: 执行交互式变基操作，验证最终的提交历史

### 实现

- [x] T078 [P] [US8] 实现 GIT_SEQUENCE_EDITOR 协议处理
- [x] T079 [US8] 在 `src/git-core/src/rebase.rs` 实现变基操作
- [x] T080 [US8] 实现 `rebase_start`, `rebase_continue`, `rebase_abort`
- [x] T081 [US8] 实现变基冲突解决流程
- [x] T082 [US8] 创建变基待办列表编辑器
- [x] T083 [US8] 添加变基进度和状态显示

**检查点**: 变基操作功能完整

---

## 阶段 11: 用户故事 9 - 标签操作 (优先级: P3)

**目标**: 用户可以创建、删除、查看标签，与 IntelliJ 工作流程相同

**独立测试**: 创建带注释和轻量级标签，验证出现在 git tag 输出中

### 实现

- [x] T084 [P] [US9] 在 `src/git-core/src/tag.rs` 实现标签操作
- [x] T085 [P] [US9] 实现 `create_tag`, `delete_tag`, `list_tags`
- [x] T086 [US9] 创建标签对话框 UI，文件在 `src-ui/src/views/tag_dialog.rs`
- [x] T087 [US9] 实现标签详情视图

**检查点**: 标签操作功能完整

---

## 阶段 N: 完善与跨领域问题

**目的**: 影响多个用户故事的改进

- [ ] T088 [P] 完善中文本地化所有 UI 文本
- [ ] T089 [P] 性能优化 (大文件 diff 渲染、虚拟滚动)
- [x] T090 [P] 添加单元测试覆盖 git-core 所有 public API
- [ ] T091 实现 IntelliJ parity 测试框架
- [ ] T092 [P] 添加集成测试 fixtures
- [ ] T093 代码清理和重构
- [ ] T094 跨平台构建配置 (Windows, macOS, Linux)
- [ ] T095 更新 README.md 和文档

---

## 并行执行示例

### 用户故事 1 内部并行:
```
Task: T021 - 实现 open_repository
Task: T022 - 实现 init_repository (可并行)
```

### 用户故事 1 和 2 并行:
```
开发者 A: 实现 US1 (T021-T027)
开发者 B: 实现 US2 (T028-T036)
```

### MVP (P1 用户故事) 并行:
```
开发者 A: 实现 US1 (T021-T027)
开发者 B: 实现 US2 (T028-T036)
开发者 C: 实现 US3 (T037-T045)
开发者 D: 实现 US4 (T046-T054)
```

---

## 实施策略

### MVP 优先 (用户故事 1 のみ)

1. 完成阶段 1: Setup
2. 完成阶段 2: Foundational
3. 完成阶段 3: 用户故事 1
4. **停止并验证**: 测试用户故事 1 独立工作

### 增量交付

1. 完成 Setup + Foundational → 基础设施就绪
2. 添加用户故事 1 → 独立测试 → 部署/演示 (MVP!)
3. 添加用户故事 2 → 独立测试 → 部署/演示
4. 添加用户故事 3 → 独立测试 → 部署/演示
5. 添加用户故事 4 → 独立测试 → 部署/演示
6. 每个故事添加价值，不破坏之前的故事

### 团队并行策略

多开发者时:

1. 团队完成 Setup + Foundational 一起
2. Foundational 完成后:
   - 开发者 A: 用户故事 1
   - 开发者 B: 用户故事 2
   - 开发者 C: 用户故事 3
   - 开发者 D: 用户故事 4
3. 故事完成并独立测试

---

## 备注

- [P] 任务 = 不同文件，无依赖
- [Story] 标签将任务映射到特定用户故事以便追溯
- 每个用户故事应该可以独立完成和测试
- 在实现前验证测试失败
- 每个任务后提交
- 在任何检查点停止以独立验证故事
- 避免: 模糊任务、相同文件冲突、破坏独立性的跨故事依赖
