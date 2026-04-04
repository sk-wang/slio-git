# Data Model: 工作区工作流补齐与直改回填

## PersistedWorkspaceMemory

- **Location**: `src-ui/src/state.rs`
- **Fields**:
  - `last_open_repository: Option<PathBuf>`
  - `recent_paths: Vec<PathBuf>`
- **Purpose**: 记录最近项目与最后一次成功打开的仓库，用于启动恢复与侧边栏切换。
- **Rules**:
  - 去重
  - 保持最近顺序
  - 超出上限时截断
  - 恢复失败时清理无效路径

## ProjectEntry

- **Location**: `src-ui/src/state.rs`
- **Fields**:
  - `name: String`
  - `path: PathBuf`
- **Purpose**: 作为最近项目条目在主界面导航区展示和切换。

## AutoRefreshState

- **Location**: `src-ui/src/state.rs`
- **Fields**:
  - `last_workspace_refresh_at: Option<Instant>`
  - `last_remote_check_at: Option<Instant>`
  - `remote_check_in_flight_for: Option<PathBuf>`
- **Purpose**: 负责工作区自动刷新节流、远端状态轮询节流和 in-flight 保护。
- **Rules**:
  - 辅助视图、工具菜单、冲突编辑器打开时暂停自动刷新
  - 同一仓库同一时刻只允许一个远端检查任务

## ToastNotificationState

- **Location**: `src-ui/src/state.rs`
- **Fields**:
  - `level: FeedbackLevel`
  - `title: String`
  - `detail: Option<String>`
  - `expires_at: Instant`
- **Purpose**: 为 pull / push / commit 等成功操作提供短时反馈。

## Remote Credential Resolution Chain

- **Location**: `src/git-core/src/remote.rs`
- **Inputs**:
  - 显式输入用户名 / 密码
  - URL 内用户名
  - git config / credential helper
  - ssh-agent
  - 系统 git
- **Purpose**: 在尽量少打断用户的前提下完成远端认证。
- **Decision Order**:
  1. 手工显式输入
  2. URL 用户名
  3. credential helper / config 用户名
  4. ssh-agent / `Cred::ssh_key_from_agent`
  5. `Cred::default()`
  6. 对 SSH remote 直接回退到系统 git 命令

## FileSyntaxHighlighter / CodeSyntaxHighlighter

- **Location**: `src-ui/src/widgets/syntax_highlighting.rs`
- **Purpose**: 提供 diff / merge 共用的语法识别与 token 着色能力。
- **Coverage**:
  - Rust, Python, PHP, JS/TS, JSX/TSX, YAML, Dockerfile, Shell, Makefile, CMake, Kotlin 等常见文件
- **Fallback**: 无法识别时退回纯文本。

## CommitDialogState

- **Location**: `src-ui/src/views/commit_dialog.rs`
- **Fields**:
  - `message`
  - `message_editor`
  - `is_amend`
  - `commit_to_amend`
  - `diff`
  - `staged_files`
  - `selected_files`
  - `previewed_file`
- **Purpose**: 承载提交流程中的说明、文件勾选与改动预览。

## ConflictResolver

- **Location**: `src-ui/src/widgets/conflict_resolver.rs`
- **Fields / Sub-state**:
  - `diff: ThreeWayDiff`
  - `selected_hunk_index`
  - `resolutions`
  - `is_auto_merged`
- **Purpose**: 作为冲突处理的交互工作台，支持逐块选择、批量接受和自动合并。

## ThreeWayDiff / ConflictHunk

- **Location**: `src/git-core/src/diff.rs`
- **Purpose**: 描述 ours / base / theirs 三方内容和冲突块结构，供冲突列表与三栏页面复用。
