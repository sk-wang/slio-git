# Data Model: JetBrains-Styled Diff File List Panel

**Feature**: 003-jetbrains-diff-ui
**Date**: 2026-03-22

## Entity Definitions

### 1. ChangedFile

Represents a file with uncommitted changes in the repository.

| Field | Type | Validation | Notes |
|-------|------|------------|-------|
| `path` | String | Non-empty, valid file path | Relative path from repository root |
| `status` | FileStatus | Enum variant | One of: Modified, Added, Deleted, Renamed |
| `selection_state` | bool | Default: false | Whether file is currently selected |
| `old_path` | Option<String> | Required if status=Renamed | Previous path for renamed files |

**State Transitions**:
- `selection_state`: false → true (when user clicks)
- `selection_state`: true → false (when another file is selected)

### 2. FileStatus

Enumeration of possible file change types.

| Variant | Color (Hex) | Description |
|---------|-------------|-------------|
| `Modified` | `#6897BB` | File contents changed |
| `Added` | `#629755` | New untracked file |
| `Deleted` | `#6C6C6C` | File removed |
| `Renamed` | `#3A8484` | File moved/renamed |

### 3. DiffContent

Represents the diff output for a single file.

| Field | Type | Validation | Notes |
|-------|------|------------|-------|
| `file_path` | String | Non-empty | Path to the file |
| `hunks` | Vec<DiffHunk> | Non-empty if file has changes | Grouped changes |
| `old_content` | String | May be empty | Original file content |
| `new_content` | String | May be empty | New file content |

### 4. DiffHunk

Represents a group of related line changes.

| Field | Type | Validation | Notes |
|-------|------|------------|-------|
| `header` | String | Non-empty | Hunk header (e.g., "@@ -1,3 +1,4 @@") |
| `lines` | Vec<DiffLine> | Non-empty | Individual lines in hunk |
| `old_start` | u32 | >= 1 | Starting line in old file |
| `old_count` | u32 | >= 0 | Number of lines from old |
| `new_start` | u32 | >= 1 | Starting line in new file |
| `new_count` | u32 | >= 0 | Number of lines from new |

### 5. DiffLine

Represents a single line in a diff.

| Field | Type | Validation | Notes |
|-------|------|------------|-------|
| `content` | String | Any | Line text content |
| `line_type` | LineType | Enum variant | One of: Context, Added, Deleted |
| `old_line_num` | Option<u32> | Present if applicable | Line number in old file |
| `new_line_num` | Option<u32> | Present if applicable | Line number in new file |

### 6. LineType

Enumeration of diff line types.

| Variant | Background Color | Description |
|---------|------------------|-------------|
| `Context` | `#1E1E1E` | Unchanged line |
| `Added` | `#2B6742` | Line added |
| `Deleted` | `#6C6C6C` | Line removed |

## UI State Model

### DiffPanelState

| Field | Type | Notes |
|-------|------|-------|
| `files` | Vec<ChangedFile> | All changed files |
| `selected_index` | Option<usize> | Currently selected file index |
| `filter` | Option<String> | Optional file path filter |
| `sort_order` | SortOrder | How files are sorted |

### SortOrder

| Variant | Description |
|---------|-------------|
| `ByPath` | Alphabetical by file path |
| `ByStatus` | Grouped by status type |
| `ByChangeTime` | By most recent change |

## Relationships

```
DiffPanelState
├── files: Vec<ChangedFile>
│   └── ChangedFile.status: FileStatus
├── selected_index → files[index]
└── DiffContent (loaded on selection)
    ├── hunks: Vec<DiffHunk>
    │   └── DiffHunk.lines: Vec<DiffLine>
    │       └── DiffLine.line_type: LineType
    └── old_content, new_content
```

## Validation Rules

1. **ChangedFile.path**: Must be a valid, non-empty string
2. **ChangedFile.status**: Must be one of the defined FileStatus variants
3. **Renamed files**: Must have `old_path` set to the original path
4. **DiffHunk**: `old_count` lines must be `Deleted`, `new_count` lines must be `Added`
5. **DiffLine**: Added lines have no `old_line_num`, Deleted lines have no `new_line_num`
