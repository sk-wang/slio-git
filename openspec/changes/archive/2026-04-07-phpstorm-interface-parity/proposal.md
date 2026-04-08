## Why

当前 `slio-git` 已经在「轻量 IDEA 式 Git 工作台」方向上推进了几轮，但用户仍然明确希望主界面在样式和交互上都更接近 PhpStorm，尤其是顶部 chrome、分支弹层、标签栏、底部工具窗与提交历史区之间的整体节奏。现在继续补齐这一步，可以把“像 JetBrains”从局部相似提升为整体验证通过的日常体验。

## What Changes

- 以用户提供的 PhpStorm 截图为主基线，收敛仓库工作台的整体布局、密度、配色、边框、分割线和选中态节奏。
- 对齐主工作区的关键交互骨架，包括仓库/分支上下文条、编辑区标签栏、左侧工具窗栏、底部 Git/日志工具窗、提交列表与右键/下拉动作菜单。
- 重做分支与提交相关弹层的层级、分组、悬浮态、快捷入口与禁用反馈，使之更接近 PhpStorm 的“列表即操作面板”体验。
- 建立一套面向 PhpStorm parity 的视觉与交互验收清单，确保后续实现不是只改样式，而是同时覆盖高频交互路径。

## Capabilities

### New Capabilities
- `phpstorm-workbench-parity`: 定义仓库工作台在布局、视觉 token、上下文菜单、分支切换面板、标签栏、工具窗与高频交互上的 PhpStorm 对齐要求。

### Modified Capabilities

## Impact

- Affected code: `src-ui/src/views/main_window.rs`, `src-ui/src/views/branch_popup.rs`, `src-ui/src/views/history_view.rs`, `src-ui/src/widgets/changelist.rs`, `src-ui/src/widgets/diff_viewer.rs`, `src-ui/src/widgets/statusbar.rs`, `src-ui/src/widgets/scrollable.rs`, `src-ui/src/theme.rs`, `src-ui/src/state.rs`
- Affected UX surface: 仓库主工作台、分支/提交菜单、底部日志区域、编辑标签栏、左侧工具窗入口、状态提示与滚动条表现
- Dependencies: 继续使用 `iced 0.14` 现有组件体系，不计划引入新的 UI 框架；需要补充截图基线与手工验收步骤
