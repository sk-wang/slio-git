## ADDED Requirements

### Requirement: Squash Commits menu item
The context menu SHALL include "压缩选中提交..." in the 历史重写 group, enabled when multiple commits are selected.

#### Scenario: Multi-select squash
- **WHEN** user selects multiple commits and right-clicks
- **THEN** "压缩选中提交..." is enabled and triggers SquashSelectedCommits

## MODIFIED Requirements

### Requirement: Menu item enable/disable rules match IDEA
Each menu item SHALL follow IDEA's enable/disable rules based on commit properties (merge, root, HEAD-chain membership).

#### Scenario: Merge commit restrictions
- **WHEN** user right-clicks a merge commit
- **THEN** "还原提交" is disabled, "重置当前分支到此处" is enabled, rewrite operations are disabled

#### Scenario: Root commit restrictions
- **WHEN** user right-clicks the root commit (no parents)
- **THEN** "Fixup 到此提交" and "Squash 到此提交" are disabled

#### Scenario: All items clickable on normal commit
- **WHEN** user right-clicks a normal (non-merge, non-root) commit on the current branch
- **THEN** all menu items are enabled and functional (no crash, no silent noop)
