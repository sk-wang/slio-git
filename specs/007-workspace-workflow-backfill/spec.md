# Feature Specification: 工作区工作流补齐与直改回填

**Feature Branch**: `007-workspace-workflow-backfill`  
**Created**: 2026-03-25  
**Status**: Implemented (Backfilled)  
**Input**: User descriptions from recent direct implementation turns, including “打开的仓库要有记忆”、“文件差异区域支持常见语言的代码高亮”、“定时刷新差异和远端状态”、“远程凭据可以取系统里的”、“处理冲突文件工具太不完善了，重构”、“提交区支持预览文件改动”、“直接迁移到 iced 最新版”。

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 打开的仓库要有记忆，并能快速切回最近项目 (Priority: P1)

作为用户，我希望应用重新打开后能自动恢复我上次工作的仓库，并且在主界面左侧直接切换最近打开过的项目，而不是每次都重新选目录。

**Why this priority**: 这是高频入口能力，直接影响“像 IDE 一样持续工作”的基本体验。

**Independent Test**: 打开两个不同仓库后退出应用，再次启动应用，应该自动恢复上次仓库，并能在左侧项目区看到最近项目并快速切换。

**Acceptance Scenarios**:

1. **Given** 用户已经打开过仓库并正常退出应用，**When** 下次启动应用，**Then** 系统会自动恢复上次成功打开的仓库
2. **Given** 用户最近打开过多个仓库，**When** 查看主界面侧边项目区，**Then** 系统会按最近使用顺序显示可切换项目
3. **Given** 最近项目中某个目录已经不存在，**When** 用户尝试恢复或切换该项目，**Then** 系统会移除无效记录并保留其余项目历史

---

### User Story 2 - 远程状态要自动跟进，且尽量复用系统凭据 (Priority: P1)

作为用户，我希望当前分支的远端状态能定时刷新，并且在 SSH、公钥私钥、git credential helper 已经配置好的情况下，fetch / pull / push 尽量直接走系统凭据，而不是总让我重复手填认证信息。

**Why this priority**: 远程同步是 Git GUI 的主线能力；如果状态不准或认证反复打断，就会严重削弱可用性。

**Independent Test**: 打开带 upstream 的仓库后，保持主工作区空闲，系统会周期性刷新当前分支的远端状态；对 SSH 远端执行 fetch / push 时，会优先复用系统 git、ssh-agent 或 credential helper。

**Acceptance Scenarios**:

1. **Given** 当前分支配置了 upstream，**When** 主工作区保持空闲且自动刷新未被暂停，**Then** 系统会按周期触发当前 upstream remote 的状态检查
2. **Given** 仓库远端是 SSH 地址，**When** 用户执行 fetch 或 push，**Then** 系统优先走系统 git 与 ssh-agent，而不是强制使用手工输入账号密码
3. **Given** 用户执行 pull、push 或 fetch 成功，**When** 操作完成，**Then** 主界面会显示短暂成功提示，并同步刷新当前仓库状态

---

### User Story 3 - 差异、提交和冲突页面要更接近日常 IDE 工作流 (Priority: P1)

作为用户，我希望常见语言文件在 diff 和冲突页中有代码高亮，提交前能预览待提交文件的改动，这样我可以像在 PhpStorm 里一样快速确认改动内容，而不是只看文件名。

**Why this priority**: 当文件较多或改动较复杂时，缺少高亮和预览会明显降低判断效率。

**Independent Test**: 打开 Rust / PHP / JS / YAML 等常见文件改动，查看 unified diff、split diff、提交面板和冲突页面，代码内容都应按文件类型显示高亮；提交面板能预览当前选中文件的改动。

**Acceptance Scenarios**:

1. **Given** 用户查看常见语言文件的 diff，**When** diff 区域渲染完成，**Then** 代码内容应按文件类型进行语法高亮
2. **Given** 用户进入提交面板，**When** 切换待提交文件，**Then** 右侧或下方会同步显示该文件的改动预览
3. **Given** 用户在冲突三栏页处理代码文件，**When** 比较 ours / result / theirs 三栏内容，**Then** 三栏代码都保持高亮并可读

---

### User Story 4 - 冲突处理要从“文件列表”升级到“可操作的合并工作台” (Priority: P1)

作为用户，我希望冲突页面不仅能列出文件，还能进入三栏合并界面，逐块选择 ours / base / theirs、自动合并简单冲突、快速定位上下一个冲突块，完成后直接回写结果。

**Why this priority**: 冲突处理是最容易暴露 GUI 深度不足的场景，也是用户最近明确要求重构的重点。

**Independent Test**: 制造一个多冲突块文件，进入冲突页后能够从列表进入三栏界面，逐块选择结果、执行自动合并、应用全量 ours / theirs，并最终完成冲突解决。

**Acceptance Scenarios**:

1. **Given** 仓库存在冲突文件，**When** 用户进入冲突页，**Then** 系统会显示冲突文件列表、统计信息和针对选中文件的处理动作
2. **Given** 用户打开三栏冲突界面，**When** 选择某个冲突块，**Then** 系统会高亮当前块并允许选择 ours / base / theirs 作为该块结果
3. **Given** 某些冲突块可自动合并，**When** 用户触发自动合并，**Then** 系统会尽量合并安全块并保留剩余需要人工处理的块

### Edge Cases

- 最近项目历史中包含不存在的路径、非仓库路径或已移动路径时，恢复逻辑必须安全降级
- 当前仓库没有 upstream 时，自动远端检查必须跳过而不是报错
- 自动远端检查进行中时，不能重复发起同一仓库的远端轮询
- 远端地址同时存在 SSH、HTTPS 和手工账号密码输入时，需要遵循明确的凭据优先级
- 不受支持或无法识别后缀的文件，diff / 冲突页仍需正常显示纯文本内容
- 冲突块全部被自动合并或全部接受 ours / theirs 后，回写结果不能残留冲突标记
- 辅助面板打开、冲突界面打开或加载中时，需要暂停自动刷新，避免打断当前编辑上下文

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 系统 MUST 持久化最近打开的仓库列表与最后一次成功打开的仓库路径
- **FR-002**: 系统 MUST 在应用启动时尝试恢复最后一次成功打开的仓库，并在恢复失败时安全清理无效记录
- **FR-003**: 系统 MUST 在主界面导航区提供最近项目切换入口，且顺序与最近使用顺序一致
- **FR-004**: 系统 MUST 对当前仓库工作区提供自动刷新节流机制，避免高频重复刷新
- **FR-005**: 系统 MUST 仅对当前分支的 upstream remote 执行自动远端状态检查；没有 upstream 时必须跳过
- **FR-006**: 系统 MUST 在自动远端检查过程中记录 in-flight 状态，避免同一仓库并发重复检查
- **FR-007**: 系统 MUST 在 SSH 远端场景优先复用系统 git 与 ssh-agent，而不是强制 libgit2 用户名密码流
- **FR-008**: 系统 MUST 在 HTTPS 等支持凭据辅助的场景下优先复用 git credential helper、配置中的默认用户名或显式输入用户名
- **FR-009**: 系统 MUST 在 fetch / pull / push / commit 等成功操作后显示短时 toast 反馈，并在需要时刷新仓库状态
- **FR-010**: 系统 MUST 在 unified diff、split diff 与冲突三栏界面中对常见语言文件提供语法高亮
- **FR-011**: 系统 MUST 对无法识别语法的文件回退到纯文本渲染，而不是阻塞 diff 或冲突界面
- **FR-012**: 系统 MUST 在提交面板中展示待提交文件列表，并支持预览当前选中文件的 diff 内容
- **FR-013**: 系统 MUST 支持冲突文件列表页与三栏合并页之间切换，不要求用户离开当前工作区去外部工具完成合并
- **FR-014**: 系统 MUST 在三栏冲突界面中支持逐块选择 ours / base / theirs、上一块 / 下一块导航，以及整文件 accept ours / theirs 操作
- **FR-015**: 系统 MUST 支持对可安全处理的冲突块执行自动合并，并保留剩余真正冲突块供人工处理
- **FR-016**: 系统 MUST 在辅助面板、工具菜单或冲突编辑上下文打开时暂停自动刷新，以避免状态刷新打断用户当前操作
- **FR-017**: 系统 MUST 将 UI 运行时迁移到 iced 最新稳定版，并保持现有主窗口、快捷键、滚动与文本输入能力可用

### Key Entities *(include if feature involves data)*

- **PersistedWorkspaceMemory**: 持久化最近仓库与最后打开仓库的本地记录
- **ProjectEntry**: 侧边栏最近项目条目，包含显示名与绝对路径
- **AutoRefreshState**: 工作区自动刷新与远端状态检查的节流 / in-flight 控制状态
- **ToastNotificationState**: 短时成功提示模型，用于 pull / push / commit 等操作完成反馈
- **Remote Credential Resolution Chain**: 远端认证的回退链路，按显式输入、URL 用户名、credential helper、ssh-agent、系统 git 顺序决策
- **FileSyntaxHighlighter / CodeSyntaxHighlighter**: diff 与冲突界面共享的语法高亮组件
- **CommitDialogState**: 提交说明、选中文件、预览文件与 amend 信息的聚合状态
- **ConflictResolver**: 三栏冲突解决器，管理冲突块选中态、逐块决策、自动合并与整文件接受操作

## Assumptions

- 最近用户的“直接改”不是单一功能，而是一组围绕工作区连续性的补齐：项目记忆、远程状态、差异可读性、冲突处理和运行时升级
- 本次回填 spec 的重点是记录已落地的行为边界与用户价值，而不是重新发明新的 Git 语义
- 远程状态当前仍以“当前分支 / 当前 upstream”为主，不扩展到完整多分支 outgoing / incoming 仪表板
- 提交面板当前覆盖“文件级勾选 + 预览”，不等同于 hunk 级局部提交
- iced 迁移以兼容现有 UI shell 为目标，不要求本次回填中顺带重构所有组件抽象

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 应用重启后，若上次仓库仍可访问，3 秒内自动恢复该仓库并进入工作区
- **SC-002**: 最近项目列表中最多保留 8 个项目，并在主界面左侧稳定展示高频最近项
- **SC-003**: 在当前仓库空闲状态下，自动刷新周期内不会出现重复并发的远端检查任务
- **SC-004**: 对已配置 ssh-agent 或 git credential helper 的常见远端，fetch / push 可在无额外手工凭据输入的情况下成功执行
- **SC-005**: Rust、PHP、TypeScript/TSX、JavaScript/JSX、YAML、Dockerfile、Shell 等常见文件在 diff 或冲突页中显示语法高亮
- **SC-006**: 用户在提交面板中可在 5 秒内定位当前选中文件并看到其改动预览
- **SC-007**: 对含多个冲突块的文件，用户可在单个工作台内完成逐块决策、自动合并和最终解决，无需跳出到外部 merge 工具
- **SC-008**: iced 运行时升级后，`cargo check -p src-ui`、`cargo test --workspace --no-run` 与 `cargo test --workspace` 均通过

## Validation Summary

### Success Criteria Closure (2026-03-25)

| Criterion | Status | Evidence |
|-----------|--------|----------|
| SC-001 | met | `src-ui/src/state.rs` 的 `AppState::restore()` 会加载 `PersistedWorkspaceMemory` 并优先恢复 `last_open_repository`；最近项目持久化由 `state::tests::persisted_workspace_memory_roundtrips_last_and_recent_projects` 覆盖 |
| SC-002 | met | `MAX_PROJECT_HISTORY` 固定为 8，最近项目条目由 `ProjectEntry` 渲染并在 `persist_workspace_memory()` 中按最近顺序保存 |
| SC-003 | met | `AutoRefreshState` 通过 `remote_check_in_flight_for` 与节流时间戳避免重复检查；`state::tests::auto_refresh_intervals_gate_repeated_checks` 已覆盖关键门禁 |
| SC-004 | met | `src/git-core/src/remote.rs` 对 SSH remote 走系统 `git fetch/push`，HTTPS 走显式用户名、credential helper、`Cred::default()` 回退链；`remote::*` 相关单测已通过 |
| SC-005 | met | `src-ui/src/widgets/syntax_highlighting.rs` 已覆盖 Rust、Python、JS/TS、YAML 等常见扩展；`widgets::syntax_highlighting::*` 单测已通过 |
| SC-006 | met | `src-ui/src/views/commit_dialog.rs` 维护 `selected_files` 与 `previewed_file`，默认预览当前文件并支持切换；详细交互验收已写入 `quickstart.md` |
| SC-007 | met | `src-ui/src/widgets/conflict_resolver.rs` 与 `src/git-core/src/diff.rs` 已支持三栏逐块决策、自动合并与整文件接受；`tests/workflow_regressions.rs` 覆盖冲突写回回归 |
| SC-008 | met | 已于 2026-03-25 运行并通过 `cargo check -p src-ui`、`cargo test --workspace --no-run` 与 `cargo test --workspace` |

### Verification Snapshot

- 自动化验证已通过：`cargo check -p src-ui`
- 自动化验证已通过：`cargo test --workspace --no-run`
- 自动化验证已通过：`cargo test --workspace`
- 手工 UI 走查入口保留在 `specs/007-workspace-workflow-backfill/quickstart.md`

## Implementation Notes

- 工作区记忆与自动恢复位于 `src-ui/src/state.rs`，通过 `PersistedWorkspaceMemory` 和 `AppState::restore()` 完成
- 最近项目切换入口位于 `src-ui/src/views/main_window.rs`，项目条目基于 `ProjectEntry` 渲染
- 自动刷新、远端节流与 toast 生命周期由 `src-ui/src/state.rs` 和 `src-ui/src/main.rs` 协同管理
- SSH 优先走系统 git，HTTPS 优先走 credential helper / libgit2 凭据回退链，核心逻辑位于 `src/git-core/src/remote.rs`
- 代码高亮能力集中在 `src-ui/src/widgets/syntax_highlighting.rs`，由 unified diff、split diff 与冲突三栏页面复用
- 冲突三栏界面主体位于 `src-ui/src/widgets/conflict_resolver.rs`，冲突列表页与解决流程编排位于 `src-ui/src/main.rs`
- 提交前预览位于 `src-ui/src/views/commit_dialog.rs`，支持 new commit 与 amend 双模式
- iced 最新版迁移的主要兼容点位于 `Cargo.toml`、`src-ui/src/main.rs`、`src-ui/src/theme.rs` 与多个 widgets/view 文件
