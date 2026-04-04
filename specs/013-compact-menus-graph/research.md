# Research: 紧凑右键菜单和提交图

**Branch**: `013-compact-menus-graph` | **Date**: 2026-04-04

## R1: IDEA Menu Item Density

**Decision**: Menu items use 4px vertical padding, no description text, 22-24px total height

**Rationale**: IDEA's context menus show only the action name (12px font) with minimal padding. No secondary description text is shown — the action name is self-explanatory. This allows 12+ items to fit in a single screen without scrolling.

**Current slio-git**: 10px vertical padding + 10px detail text → ~34px per item
**Target**: 4px vertical padding, no detail → ~24px per item (~30% reduction)

## R2: IDEA Git Log Row Density

**Decision**: Commit rows use 22px height with 4px vertical padding

**Rationale**: IDEA's git log uses a fixed 22px row height with minimal padding to maximize the number of visible commits. The graph uses ~14px lane width with 3px node radius.

**Current slio-git**: 24px height, 16px lanes, 4px nodes
**Target**: 22px height, 14px lanes, 3px nodes (~8% more rows visible)
