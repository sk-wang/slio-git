# Tasks: 工作区工作流补齐与直改回填

## 1. Workspace continuity

- [x] 在 `src-ui/src/state.rs` 中新增最近项目持久化与恢复模型
- [x] 在 `src-ui/src/state.rs` 中实现上次仓库自动恢复逻辑
- [x] 在 `src-ui/src/views/main_window.rs` 中补齐最近项目切换入口
- [x] 为最近项目记忆补充基础单元测试

## 2. Remote reliability

- [x] 在 `src/git-core/src/remote.rs` 中实现 SSH remote 的系统 git 回退
- [x] 在 `src/git-core/src/remote.rs` 中补齐 credential helper / ssh-agent 认证链
- [x] 在 `src/git-core/src/repository.rs` 中暴露当前分支 upstream remote 能力
- [x] 在 `src-ui/src/state.rs` 与 `src-ui/src/main.rs` 中补齐自动远端检查节流与暂停逻辑
- [x] 在 `src-ui/src/state.rs` 与 `src-ui/src/views/main_window.rs` 中补齐成功 toast 反馈链路

## 3. Readable review surfaces

- [x] 在 `src-ui/src/widgets/syntax_highlighting.rs` 中实现共享语法高亮器
- [x] 将 unified diff 接入高亮
- [x] 将 split diff 接入高亮
- [x] 将冲突三栏页面接入高亮
- [x] 在 `src-ui/src/views/commit_dialog.rs` 中补齐文件预览能力

## 4. Conflict workflow depth

- [x] 在 `src/git-core/src/diff.rs` 中补齐三方冲突 diff 与自动合并基础能力
- [x] 在 `src-ui/src/main.rs` 中补齐冲突列表页与操作编排
- [x] 在 `src-ui/src/widgets/conflict_resolver.rs` 中实现三栏冲突工作台
- [x] 支持逐块导航、逐块选择、批量 accept 与自动合并

## 5. Runtime modernization

- [x] 将 workspace iced 依赖升级到 0.14
- [x] 适配新的 application / keyboard / theme / widget API
- [x] 通过 `cargo check -p src-ui`
- [x] 通过 `cargo test --workspace --no-run`
- [x] 通过 `cargo test --workspace`
