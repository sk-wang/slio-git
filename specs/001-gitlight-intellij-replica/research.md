# Research: slio-git 技术研究

**Date**: 2026-03-22
**Feature**: IntelliJ-Compatible Git Client

## 1. Iced UI Framework

### Decision: 使用 Iced 的 `Application` trait + `Widget` trait 委托模式

### Rationale
Iced 是纯 Rust 的声明式 UI 框架，支持跨平台 (Windows, macOS, Linux)，适合构建轻量级桌面应用。其 Elm 风格的架构与 Rust 的所有权模型配合良好。

关键发现:
- 使用 `Widget` trait + 委托模式构建自定义组件
- `Application` trait 管理全局状态，通过 `Message` 枚举处理用户交互
- 中文需要 `Text::shaping(iced::text::Shaping::Advanced)`
- Pure Iced 架构，无 Tauri/WebView，文件选择需自实现或用 `rfd` crate
- 大文件渲染使用 `Lazy` widget + 视口裁剪

### Alternatives Considered
- **Yew**: React-like 但 WebView 依赖，不适合原生桌面
- **Dioxus**: 好但生态不如 Iced 成熟
- **Leptos**: 主要面向 Web

## 2. git2-rs API Coverage

### Decision: 以 git2-rs 为主，CLI 为辅

### Rationale
git2-rs (libgit2 的 Rust 绑定) 支持 95%+ 的 Git 操作:

| 优先级 | 操作 | git2-rs 支持 |
|--------|------|-------------|
| P1 | 仓库检测、暂存、提交、分支 | ✅ 完全支持 |
| P2 | 远程、储藏、Diff、合并 | ✅ 完全支持 |
| P3 | 标签、变基 | ⚠️ 部分支持 |

**关键限制**:
- **交互式变基**: 需要实现 `GIT_SEQUENCE_EDITOR` 协议
- **Worktree**: git2-rs API 非常有限，需调用 CLI
- **凭据助手**: 需实现回调模式

### Alternatives Considered
| 方案 | 优点 | 缺点 | 结论 |
|------|------|------|------|
| **gix** (纯 Rust) | 纯 Rust，更好 async | 不够成熟，部分功能缺失 | 未来考虑 |
| **JGit** (Java) | IntelliJ 使用 | 需 JNI，JVM 开销 | 不采用 |
| **libgit2 (直接 C)** | 完整功能 | 无安全保证 | 不采用 |
| **git CLI** | 100% 兼容 | 慢，解析脆弱 | 仅作备选 |

## 3. 跨平台文件监视

### Decision: 使用 `notify` crate

### Rationale
`notify` 是 Rust 生态中最成熟的跨平台文件监视库，支持:
- Windows: ReadDirectoryChangesW
- macOS: FSEvents / kqueue
- Linux: inotify

性能特征:
- 低延迟: 文件变化到通知 < 10ms
- 低开销: 仅监视变化的文件
- 跨平台一致 API

## 4. Iced 中文支持

### Decision: 使用 `Shaping::Advanced` + 系统字体回退

### Rationale
Iced 对 UTF-8 有良好支持，但默认 `Basic` shaping 无法正确渲染中文:

```rust
Text::new("中文")
    .shaping(iced::text::Shaping::Advanced)
```

中文字体回退由系统处理，主流操作系统均支持。

## 5. IntelliJ Git 操作流程

### Decision: 参考 IntelliJ Community 的 git4idea 模块

### Rationale
根据已完成的 IntelliJ git 模块研究，关键架构:

```
plugins/git4idea/
├── src/git4idea/           # 后端 (Java/Kotlin)
│   ├── Git.java          # Git 操作接口
│   ├── GitImpl.java      # 实现
│   ├── repo/             # 仓库管理
│   ├── branch/           # 分支操作
│   └── push/fetch/       # 远程操作
├── shared/               # 共享模型
└── frontend/             # UI 组件
```

**关键复刻点**:
1. **GitRepositoryManager**: 仓库检测和状态管理
2. **GitBranchWorker**: 分支操作的统一入口
3. **GitIndexUtil**: 暂存区操作
4. **统一的事件流**: VFS 监听 → 状态更新 → UI 刷新

## 6. 架构决策总结

| 组件 | 技术选型 | 理由 |
|------|----------|------|
| UI 框架 | Pure Iced | 纯 Rust，无 Tauri/WebView，最小依赖 |
| Git 引擎 | git2-rs + CLI fallback | 95% 覆盖，CLI 处理边界情况 |
| 文件监视 | notify | 跨平台，高性能 |
| 文件选择 | rfd crate | Pure Rust 的原生文件对话框 |
| 中文渲染 | Shaping::Advanced | Iced 内置支持 |
