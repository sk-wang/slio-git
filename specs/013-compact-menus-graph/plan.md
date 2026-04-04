# Implementation Plan: 紧凑右键菜单和提交图

**Branch**: `013-compact-menus-graph` | **Date**: 2026-04-04 | **Spec**: [spec.md](spec.md)

## Summary

调整右键菜单和提交图的间距/密度常量以匹配 IDEA 的紧凑布局。纯数值调整，不改功能逻辑。

## Technical Context

**Language/Version**: Rust (edition 2021+)
**Primary Dependencies**: Iced 0.14
**Testing**: cargo test (94 existing tests)
**Scope**: Constants-only changes in 3 files

## Constitution Check

| Principle | Status | Notes |
| --------- | ------ | ----- |
| I. IntelliJ Compatibility | PASS | 直接对标 IDEA 密度参数 |
| II. Rust + Iced Stack | PASS | 纯 Iced 样式调整 |
| III. Library-First | N/A | 无 git-core 改动 |
| IV. Integration Testing | PASS | 零回归验证 |
| V. Observability | N/A | 无逻辑改动 |
| VI. 中文本地化 | N/A | 无文本改动 |

## Project Structure

```text
src-ui/src/
├── views/history_view.rs    # MODIFY: row height, graph constants, menu padding
├── views/branch_popup.rs    # MODIFY: context menu item padding
└── widgets/menu.rs          # MODIFY: action_row padding, remove detail text
```

## Change Map

| Constant | File | Current | Target | IDEA Reference |
| -------- | ---- | ------- | ------ | -------------- |
| HISTORY_ROW_HEIGHT | history_view.rs | 24.0 | 22.0 | 22-24px |
| HISTORY_GRAPH_LANE_WIDTH | history_view.rs | 16.0 | 14.0 | ~14px |
| HISTORY_GRAPH_NODE_RADIUS | history_view.rs | 4.0 | 3.0 | ~3px |
| HISTORY_GRAPH_LINE_WIDTH | history_view.rs | 1.6 | 1.5 | ~1.5px |
| HISTORY_CONTEXT_MENU_WIDTH | history_view.rs | 332.0 | 280.0 | ~280px |
| Commit row padding | history_view.rs | [8, 10] | [4, 8] | [4, 8] |
| Menu action_row padding | menu.rs | [10, 8] | [4, 8] | [4, 8] |
| Menu group padding | menu.rs | [8, 10] | [4, 8] | [4, 8] |
| Menu detail text | menu.rs | Shown | Hidden | Not shown in IDEA |
| Branch menu item padding | branch_popup.rs | [8, 12] | [4, 8] | [4, 8] |

## Complexity Tracking

No constitution violations. All changes are numeric constant adjustments.
