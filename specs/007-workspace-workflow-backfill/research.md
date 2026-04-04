# Research: 工作区工作流补齐与直改回填

## 1. 最近项目记忆

- 需求来源：用户明确要求“打开的仓库要有记忆”，并且后续又要求支持多文件夹项目切换。
- 结论：对当前产品而言，最实用的策略不是引入复杂 workspace model，而是先把“最近项目 + 上次仓库恢复”做稳。
- 采用方案：`PersistedWorkspaceMemory` 使用本地文本文件记录 `last` 与 `recent` 两类路径；启动时通过 `AppState::restore()` 自动恢复。

## 2. 远端认证与状态刷新

- 需求来源：用户明确指出“远程凭据可以取系统里的”，并给出了 SSH 公钥私钥场景；同时又要求“这里只要访问当前分支的远端就行了”。
- 结论：当前阶段不做完整多远端 / 多分支仪表盘，而是聚焦当前分支 upstream remote 的自动状态刷新。
- 采用方案：
  - SSH remote：优先走系统 `git fetch` / `git push`，复用 ssh-agent 与系统 git 环境
  - HTTPS remote：优先走显式用户名、URL 用户名、git credential helper、`Cred::default()` 的回退链
  - 自动远端检查：仅针对 `current_upstream_remote()`，并带有 in-flight 防抖与辅助视图暂停逻辑

## 3. 差异和冲突可读性

- 需求来源：用户要求“文件差异区域支持常见语言的代码高亮”，后来又要求冲突工具参考 IDE 重构。
- 结论：如果 diff 和冲突页继续只是纯文本块，复杂文件的判断效率会很低；代码高亮是投入产出比很高的补齐项。
- 采用方案：引入共享 `syntax_highlighting` 模块，统一服务 unified diff、split diff 和冲突三栏页；对未知语法优雅降级到纯文本。

## 4. 提交流程补齐

- 需求来源：用户指出“待提交这里，要能够预览文件改动”，并强调提交说明输入和提交流程必须可连续使用。
- 结论：在没有 hunk 级提交 UI 之前，至少要先把“文件级勾选 + 预览 diff”补齐，才能形成可用的提交前审阅体验。
- 采用方案：在 `CommitDialogState` 中保留 `selected_files` 与 `previewed_file`，并将当前选中文件的 diff 直接复用到提交流程内。

## 5. 冲突处理深度

- 需求来源：用户多次指出现有冲突处理工具“不完善”，要求参考后续图片重构。
- 结论：单纯文件列表已经无法支撑高频冲突处理，必须补齐三栏合并、逐块导航、逐块选择和自动合并。
- 采用方案：
  - 列表页：显示冲突文件、冲突块数量、是否需要人工处理
  - 三栏页：展示 ours / result / theirs，支持逐块选择结果
  - 自动合并：对仅一侧改动或 base 安全继承的块自动处理，保留真正冲突块

## 6. iced 0.14 升级

- 需求来源：用户明确要求“直接迁移到 iced 最新版”。
- 结论：需要升级到当前稳定版 iced 0.14，同时尽量不破坏已完成的 UI shell 和主题体系。
- 采用方案：
  - 升级 workspace 依赖到 iced 0.14
  - 适配新的 `application` builder、keyboard subscription、theme status 枚举与 widget API
  - 保留必要兼容层，优先保证功能和打包链稳定
