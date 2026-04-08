## Why

当前仓库工作台已经具备了 JetBrains/PhpStorm 风格骨架，但默认界面仍然偏松散：顶部 chrome、列表行高、工具栏按钮、提交/历史面板与底部工具窗之间的留白明显大于 PhpStorm。用户明确希望在不牺牲可用性的前提下，把默认 UI 压得更紧凑，让主界面和高频交互更接近 PhpStorm 的信息密度与操作节奏。

## What Changes

- 收紧主工作台的基础密度 token，包括间距、控件高度、标签栏高度、滚动条、边框与分割线权重。
- 压缩 changes/diff/history/commit 等高频工作区的行高、标题栏、说明文案与面板 padding，减少“卡片感”，更贴近 PhpStorm 的连续工作台。
- 统一弹层、菜单、底部工具窗和提交对话框的紧凑样式，确保不同交互面板在视觉和命中区上说同一种语言。
- 补充一份针对“紧凑度像不像 PhpStorm”的验收清单与证据，明确哪些区域必须达到更高的信息密度。

## Capabilities

### New Capabilities
- `compact-phpstorm-workbench`: 定义仓库工作台、弹层、工具窗和提交/历史面板需要达到的 PhpStorm 式紧凑密度要求。

### Modified Capabilities

## Impact

- Affected code: `/Users/wanghao/git/slio-git/src-ui/src/theme.rs`, `/Users/wanghao/git/slio-git/src-ui/src/views/main_window.rs`, `/Users/wanghao/git/slio-git/src-ui/src/views/commit_dialog.rs`, `/Users/wanghao/git/slio-git/src-ui/src/views/history_view.rs`, `/Users/wanghao/git/slio-git/src-ui/src/views/branch_popup.rs`, `/Users/wanghao/git/slio-git/src-ui/src/widgets/button.rs`, `/Users/wanghao/git/slio-git/src-ui/src/widgets/text_input.rs`, `/Users/wanghao/git/slio-git/src-ui/src/widgets/scrollable.rs`, `/Users/wanghao/git/slio-git/src-ui/src/widgets/changelist.rs`, `/Users/wanghao/git/slio-git/src-ui/src/widgets/diff_viewer.rs`
- Affected UX surface: 主窗口 shell、顶部工具栏、编辑标签、变更列表、diff 区、底部历史/日志工具窗、分支与远端菜单、提交对话框
- Dependencies: 延续 `iced 0.14` 组件体系与现有 Rust UI 架构，不引入新的 UI 框架；需要人工截图对照来验证紧凑度是否达标
