# Data Model: slio-git

**Date**: 2026-03-22
**Feature**: IntelliJ-Compatible Git Client

## 实体关系图

```
┌─────────────────────────────────────────────────────────────────────┐
│                            Repository                                │
│  - path: PathBuf                                                     │
│  - workdir: Option<Workdir>                                         │
│  - state: RepositoryState                                           │
└─────────────────────────────────────────────────────────────────────┘
    │
    ├──┬── Branch (1:N)
    │   └── name: String
    │   └── is_remote: bool
    │   └── upstream: Option<Branch>
    │
    ├──├── Commit (1:N)
    │   └── id: Oid
    │   └── message: String
    │   └── author: Signature
    │   └── parents: Vec<Oid>
    │
    ├──├── Remote (1:N)
    │   └── name: String
    │   └── url: String
    │
    ├──├── Tag (1:N)
    │   └── name: String
    │   └── target: Oid
    │   └── message: Option<String>
    │
    └──├── Stash (1:N)
        └── index: u32
        └── message: String
        └── branch: String
```

## 实体定义

### Repository

git 仓库根目录

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `PathBuf` | .git 目录路径 |
| `workdir` | `Option<PathBuf>` | 工作树根目录 |
| `current_branch` | `Branch` | 当前分支 |
| `state` | `RepositoryState` | 仓库状态枚举 |

### RepositoryState

仓库的当前状态

| 变体 | 说明 |
|------|------|
| `Clean` | 无待提交变更 |
| `Dirty` | 有未暂存/未提交变更 |
| `Merging` | 合并中 (存在 MERGE_HEAD) |
| `Rebasing` | 变基中 (存在 rebase 状态) |

### Branch

git 分支引用

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 分支名称 |
| `oid` | `Oid` | 当前指向的提交 |
| `is_remote` | `bool` | 是否远程分支 |
| `is_head` | `bool` | 是否 HEAD |
| `upstream` | `Option<String>` | 上游分支名 |

### Commit

git 提交对象

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `Oid` | 提交 SHA |
| `message` | `String` | 提交消息 |
| `author` | `Signature` | 作者签名 |
| `committer` | `Signature` | 提交者签名 |
| `parents` | `Vec<Oid>` | 父提交列表 |
| `tree_id` | `Oid` | 根树对象 |

### Signature

提交者/作者签名

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 姓名 |
| `email` | `String` | 邮箱 |
| `when` | `Time` | 时间戳 |

### Remote

git 远程仓库

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 远程名称 (origin) |
| `url` | `String` | 远程 URL |
| `fetch_specs` | `Vec<String>` | fetch 规格 |

### Tag

git 标签

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 标签名 |
| `target` | `Oid` | 指向的提交 |
| `message` | `Option<String>` | 标签消息 (annotated) |
| `tagger` | `Option<Signature>` | 标签创建者 |

### Stash

git 储藏

| 字段 | 类型 | 说明 |
|------|------|------|
| `index` | `u32` | 储藏索引 |
| `message` | `String` | 储藏消息 |
| `branch` | `String` | 创建时所在分支 |
| `oid` | `Oid` | 储藏的提交 |

### Change

文件变更状态

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `PathBuf` | 文件路径 |
| `status` | `ChangeStatus` | 变更状态 |
| `old_mode` | `Option<FileMode>` | 旧文件模式 |
| `new_mode` | `Option<FileMode>` | 新文件模式 |
| `old_oid` | `Option<Oid>` | 旧 blob Oid |
| `new_oid` | `Option<Oid>` | 新 blob Oid |

### ChangeStatus

变更状态枚举

| 变体 | 说明 |
|------|------|
| `Modified` | 已修改 |
| `Added` | 新添加 |
| `Deleted` | 已删除 |
| `Renamed` | 已重命名 |
| `Copied` | 已复制 |
| `Untracked` | 未跟踪 |
| `Ignored` | 已忽略 |
| `Conflict` | 冲突状态 |

### IndexEntry

暂存区条目

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `PathBuf` | 文件路径 |
| `oid` | `Oid` | blob Oid |
| `mode` | `FileMode` | 文件模式 |
| `stage` | `u32` | 暂存阶段 (冲突时) |

### DiffHunk

diff 代码块

| 字段 | 类型 | 说明 |
|------|------|------|
| `header` | `String` | 头部信息 (@@ -n,m +n,m @@) |
| `lines` | `Vec<DiffLine>` | 包含的行 |
| `old_start` | `u32` | 旧文件起始行 |
| `new_start` | `u32` | 新文件起始行 |

### DiffLine

diff 单行

| 字段 | 类型 | 说明 |
|------|------|------|
| `content` | `String` | 行内容 |
| `origin` | `DiffLineOrigin` | 行类型 (+/-/ /\) |
| `old_lineno` | `Option<u32>` | 旧文件行号 |
| `new_lineno` | `Option<u32>` | 新文件行号 |

### DiffLineOrigin

| 变体 | 说明 |
|------|------|
| `Context` | 上下文行 |
| `Addition` | 添加行 (+) |
| `Deletion` | 删除行 (-) |
| `Header` | diff 头部 |
| `HunkHeader` | 代码块头部 |

## 状态转换

### RepositoryState 转换

```
Clean <---> Dirty
   │
   ├── Merge Start --> Merging --> Merge Complete --> Clean/Dirty
   │
   └── Rebase Start --> Rebasing --> Rebase Complete --> Clean/Dirty
```

### ChangeStatus 转换

```
Untracked --add--> Staged --commit--> (deleted from index)
    │
Modified --edit--> Modified --add--> Staged --commit--> (deleted from index)
    │
Conflict: 合并/变基时产生，需手动解决后 add
```

## 验证规则

| 实体 | 规则 |
|------|------|
| Branch | 名称不能包含空格或特殊字符 (* ? [ :/) |
| Commit | message 不能为空 |
| Remote | url 必须是有效 URL 或路径 |
| Tag | 名称必须符合 git tag 规范 |
| Stash | 索引必须唯一 |
