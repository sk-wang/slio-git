## ADDED Requirements

### Requirement: Workspace shell SHALL match the PhpStorm Git workbench baseline
系统 MUST 在默认仓库窗口中呈现接近用户参考图的 PhpStorm 式工作台骨架，包括左侧工具窗栏、紧凑顶部上下文条、编辑区标签栏、中央 changes/diff 主区，以及底部 Git/日志工具窗，而不是多个厚重卡片和独立页面的拼接。

#### Scenario: Default repository layout mirrors the reference rhythm
- **WHEN** 用户在默认窗口尺寸下打开一个本地 Git 仓库
- **THEN** 系统展示连续的暗色工作台骨架，主视觉焦点落在变更列表、差异预览和底部 Git/日志区域，而不是大块容器装饰

### Requirement: Branch and commit action surfaces SHALL use PhpStorm-style list-first interactions
系统 MUST 将当前分支入口、分支列表、提交列表以及相关上下文菜单实现为接近 PhpStorm 的列表优先交互面板，包含搜索、分组动作、悬浮态、选中态、子菜单指示、禁用原因与危险动作分区。

#### Scenario: Branch popup behaves like a JetBrains action panel
- **WHEN** 用户点击顶部当前分支控件
- **THEN** 系统打开一个以搜索框和分组列表为核心的紧凑弹层，并在同一表面中呈现最近分支、本地分支、远程分支与高频 Git 动作

#### Scenario: Context menu keeps the source row and action grouping clear
- **WHEN** 用户在分支行或提交行上打开右键菜单或行尾动作菜单
- **THEN** 系统保持触发行可辨认，并以统一分组顺序展示比较、切换、创建、同步和危险动作

### Requirement: The workbench SHALL preserve context across changes, history, and bottom tool windows
系统 MUST 让 changes、diff、历史/日志和其他底部工具窗在同一个工作台中连续切换，切换过程中保留当前仓库、分支、选中文件或提交等关键上下文，而不是把这些区域视为互相替代的独立页面。

#### Scenario: Switching to the log area does not discard current workspace context
- **WHEN** 用户在查看某个改动文件时打开底部 Git/日志工具窗
- **THEN** 系统在停靠式底部区域展示日志内容，同时保留顶部仓库上下文和当前变更工作区，且用户返回 changes 时仍可回到原来的焦点

### Requirement: Visual tokens SHALL be dense, low-noise, and clearly readable
系统 MUST 在标签栏、列表项、菜单项、滚动条、状态栏、计数和分隔线中使用接近 PhpStorm 的深色配色、细分隔、低干扰滚动条、紧凑行高和克制反馈，同时保持 hover、selected、warning、error 与当前上下文状态清晰可辨。

#### Scenario: Hover and selection remain clear without heavy chrome
- **WHEN** 用户依次悬浮或选中文件行、分支行、标签页、日志项和菜单项
- **THEN** 系统使用统一且清晰的轻量反馈表达焦点状态，而不会退回厚边框、大徽章或高噪音装饰

### Requirement: High-frequency Git workflows SHALL remain reachable inside the parity shell
系统 MUST 在新的 PhpStorm 式工作台中持续提供提交、暂存/取消暂存、刷新、拉取、推送、分支切换、比较和历史查看等高频 Git 能力，并保证用户不需要返回旧页面结构或终端才能完成主流程。

#### Scenario: User completes a commit-oriented workflow inside one workbench
- **WHEN** 用户从 changes 列表审阅改动并准备提交
- **THEN** 系统在同一套工作台中提供暂存、填写提交信息、提交以及继续推送或切换分支的可达入口
