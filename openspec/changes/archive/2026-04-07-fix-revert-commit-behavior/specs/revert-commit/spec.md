## MODIFIED Requirements

### Requirement: Revert commit from history view
When user right-clicks a commit in the history view and selects "还原提交", the system SHALL immediately execute `git revert --no-edit <commit_id>` without showing a confirmation dialog.

#### Scenario: Successful revert
- **WHEN** user selects "还原提交" on a non-merge commit with a clean worktree
- **THEN** system executes `git revert --no-edit`, creates a new revert commit, refreshes the history view, and shows a success toast

#### Scenario: Revert with conflicts
- **WHEN** user selects "还原提交" and the revert produces merge conflicts
- **THEN** system navigates to the conflict resolution view

#### Scenario: Revert on dirty worktree
- **WHEN** user selects "还原提交" but the worktree has uncommitted changes
- **THEN** system shows an error message indicating the worktree must be clean

#### Scenario: Revert merge commit
- **WHEN** user selects "还原提交" on a merge commit (multiple parents)
- **THEN** system shows a message that merge commit revert is not yet supported
