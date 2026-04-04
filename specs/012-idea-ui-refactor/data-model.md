# Data Model: IDEA Git 视图重构

**Branch**: `012-idea-ui-refactor` | **Date**: 2026-04-04

## 新增/修改实体

### CommitContextAction (新增枚举)

提交历史右键菜单的操作类型，按 IDEA Git.Log.ContextMenu 分组排列。

| 值 | 中文标签 | IDEA 对应 | 需确认 |
| -- | -------- | --------- | ------ |
| ResetToCommit | 重置当前分支到此处 | Git.Reset.In.Log | 是(选择重置模式) |
| RevertCommit | 还原提交 | Git.Revert.In.Log | 否 |
| UncommitToHere | 撤销提交到此处 | Git.Uncommit | 是(影响多个提交) |
| RewordCommit | 修改提交消息 | Git.Reword.Commit | 是(编辑消息) |
| FixupToCommit | Fixup 到此提交 | Git.Fixup.To.Commit | 否 |
| SquashIntoCommit | Squash 到此提交 | Git.Squash.Into.Commit | 是(编辑消息) |
| DropCommit | 丢弃提交 | Git.Drop.Commits | 是(不可逆) |
| SquashCommits | 压缩选中提交 | Git.Squash.Commits | 是(多选+编辑消息) |
| InteractiveRebase | 交互式变基 | Git.Interactive.Rebase | 是(打开编辑器) |
| PushUpToCommit | 推送到此提交 | Git.PushUpToCommit | 否 |
| CreateBranch | 从此提交新建分支 | Git.BranchOperationGroup | 是(输入分支名) |
| CreateTag | 从此提交创建标签 | Git.CreateNewTag | 是(输入标签信息) |

---

### BranchContextAction (新增枚举)

分支右键菜单的操作类型，按 IDEA Git.Branch.Backend 排列。

| 值 | 中文标签 | 适用范围 | 当前分支可用 |
| -- | -------- | -------- | ------------ |
| Checkout | 检出 | 本地+远程 | 否(禁用) |
| NewBranchFrom | 新建分支从此 | 本地+远程 | 是 |
| MergeIntoCurrent | 合并到当前 | 本地+远程 | 否(禁用) |
| RebaseOnto | 变基当前到此 | 本地+远程 | 否(禁用) |
| CompareWithCurrent | 比较 | 本地+远程 | 否(禁用) |
| TrackBranch | 跟踪分支 | 本地 | 是 |
| PullBranch | 拉取 | 本地(有上游) | 是 |
| PushBranch | 推送 | 本地 | 是 |
| Rename | 重命名 | 本地 | 是 |
| Delete | 删除 | 本地 | 否(禁用) |

---

### StashContextAction (新增枚举)

贮藏右键菜单的操作类型，对应 IDEA Git.Stash.Operations.ContextMenu。

| 值 | 中文标签 | 需确认 |
| -- | -------- | ------ |
| Pop | 弹出 | 否 |
| Apply | 应用 | 否 |
| UnstashAs | 应用到新分支 | 是(输入分支名) |
| Drop | 丢弃 | 是(不可逆) |
| ClearAll | 清空所有 | 是(不可逆) |

---

### TagContextAction (新增枚举)

标签右键菜单的操作类型。

| 值 | 中文标签 | 需确认 |
| -- | -------- | ------ |
| PushToRemote | 推送到远程 | 否 |
| DeleteLocal | 删除本地 | 是 |
| DeleteRemote | 删除远程 | 是 |
| DeleteBoth | 删除本地和远程 | 是 |

---

### FilePreview (新增概念)

当文件无差异（新增/未跟踪）时的预览数据。

| 字段 | 类型 | 描述 |
| ---- | ---- | ---- |
| path | String | 文件相对路径 |
| content | String | 文件完整文本内容（截断到 5000 行） |
| is_binary | bool | 是否为二进制文件 |
| is_truncated | bool | 是否因超大而被截断 |
| syntax_name | Option\<String\> | 语法高亮类型（由扩展名推断） |

## 关系

```
CommitContextAction → HistoryEntry (目标提交)
BranchContextAction → Branch (目标分支)
StashContextAction → StashInfo (目标贮藏)
TagContextAction → TagInfo (目标标签)
FilePreview → Change (当 diff 为空时生成)
```
