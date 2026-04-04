# Data Model: IDEA Git Feature Parity

**Branch**: `011-idea-git-parity` | **Date**: 2026-04-04

## Entities

### Change (EXISTING - extended)

File-level modification with staging state and diff hunks.

| Field | Type | Description |
| ----- | ---- | ----------- |
| path | String | Relative file path from repository root |
| status | ChangeStatus | Added, Modified, Deleted, Renamed, Untracked, Ignored, Conflict |
| staged | bool | Whether file has staged changes |
| unstaged | bool | Whether file has unstaged changes |
| old_oid | Option\<String\> | Object ID before change |
| new_oid | Option\<String\> | Object ID after change |
| **is_submodule** | **bool** | **NEW: Whether this change is a submodule entry** |
| **submodule_summary** | **Option\<String\>** | **NEW: Commit range summary for submodule changes** |

**State transitions**: Untracked → Staged (add) → Committed. Modified → Staged → Committed. Any → Discarded.

---

### Branch (EXISTING - extended)

Git reference with tracking and display metadata.

| Field | Type | Description |
| ----- | ---- | ----------- |
| name | String | Branch name (e.g., "main", "origin/main") |
| oid | String | Commit hash at branch tip |
| is_remote | bool | Whether this is a remote-tracking branch |
| is_head | bool | Whether this is the current HEAD branch |
| upstream | Option\<String\> | Upstream tracking reference |
| tracking_status | Option\<String\> | Ahead/behind display text |
| sync_hint | Option\<String\> | Sync status hint for display |
| recency_hint | Option\<String\> | Last activity hint |
| last_commit_timestamp | Option\<i64\> | Timestamp of last commit on branch |
| **group_path** | **Option\<Vec\<String\>\>** | **NEW: Hierarchical group path for tree display (e.g., ["feature", "auth"])** |

---

### Commit (EXISTING - extended as HistoryEntry)

Point in history with graph visualization data.

| Field | Type | Description |
| ----- | ---- | ----------- |
| id | String | Full commit hash |
| message | String | Commit message |
| author_name | String | Author name |
| author_email | String | Author email |
| timestamp | i64 | Author timestamp (Unix epoch) |
| parent_ids | Vec\<String\> | Parent commit hashes |
| **committer_name** | **Option\<String\>** | **NEW: Committer name (if different from author)** |
| **committer_email** | **Option\<String\>** | **NEW: Committer email** |
| **refs** | **Vec\<RefLabel\>** | **NEW: Branch/tag labels pointing to this commit** |
| **signature_status** | **Option\<SignatureStatus\>** | **NEW: GPG/SSH verification result** |

---

### GraphNode (NEW)

Visual layout data for a single commit in the graph view.

| Field | Type | Description |
| ----- | ---- | ----------- |
| commit_id | String | Reference to HistoryEntry |
| lane | u32 | Column index for this commit's position |
| parent_edges | Vec\<GraphEdge\> | Edges connecting to parent commits |
| is_merge | bool | Whether this commit has multiple parents |

---

### GraphEdge (NEW)

Visual edge connecting two commits in the graph.

| Field | Type | Description |
| ----- | ---- | ----------- |
| from_lane | u32 | Source lane (child commit) |
| to_lane | u32 | Target lane (parent commit) |
| edge_type | EdgeType | Direct (same lane), Merge (cross-lane), Fork (split) |
| color_index | u8 | Color palette index for branch coloring |

---

### RefLabel (NEW)

Branch or tag label attached to a commit in the log view.

| Field | Type | Description |
| ----- | ---- | ----------- |
| name | String | Reference display name |
| ref_type | RefType | LocalBranch, RemoteBranch, Tag, Head |
| is_current | bool | Whether this is the current HEAD ref |

---

### SignatureStatus (NEW)

GPG/SSH signature verification result for a commit.

| Field | Type | Description |
| ----- | ---- | ----------- |
| is_signed | bool | Whether the commit has a signature |
| is_verified | bool | Whether the signature verified successfully |
| signer_name | Option\<String\> | Name from the signing key |
| key_id | Option\<String\> | Key fingerprint/ID |
| signature_type | SignatureType | GPG, SSH, or Unknown |

---

### BlameEntry (NEW)

Per-hunk blame attribution data.

| Field | Type | Description |
| ----- | ---- | ----------- |
| commit_id | String | Commit that last modified these lines |
| author_name | String | Author of the commit |
| author_email | String | Author email |
| timestamp | i64 | Commit timestamp |
| message | String | First line of commit message |
| start_line | u32 | First line number (1-based) |
| line_count | u32 | Number of lines in this hunk |

---

### Stash (EXISTING - extended as StashInfo)

Saved work-in-progress state.

| Field | Type | Description |
| ----- | ---- | ----------- |
| index | u32 | Stash index (0 = most recent) |
| message | String | Stash message |
| branch | String | Branch name when stashed |
| oid | String | Stash commit hash |
| **timestamp** | **Option\<i64\>** | **NEW: When the stash was created** |
| **includes_untracked** | **bool** | **NEW: Whether untracked files are included** |

---

### Tag (EXISTING - as TagInfo)

Named reference to a commit. No changes needed from existing model.

| Field | Type | Description |
| ----- | ---- | ----------- |
| name | String | Tag name |
| target | String | Target commit hash |
| message | Option\<String\> | Annotation message (None for lightweight) |
| tagger_name | Option\<String\> | Tagger name (annotated only) |
| tagger_email | Option\<String\> | Tagger email (annotated only) |
| tagged_time | Option\<i64\> | Tag creation time (annotated only) |

---

### Conflict (EXISTING - as ThreeWayDiff)

Merge conflict state. No changes needed from existing model.

---

### WorkingTree (NEW)

Linked git worktree.

| Field | Type | Description |
| ----- | ---- | ----------- |
| name | String | Worktree name |
| path | PathBuf | Absolute path to worktree directory |
| branch | Option\<String\> | Branch checked out in worktree |
| is_main | bool | Whether this is the main worktree |
| is_locked | bool | Whether the worktree is locked |
| is_valid | bool | Whether the worktree path exists and is valid |

---

### LogTab (NEW - UI state only)

State for a single tab in the multi-tab log view.

| Field | Type | Description |
| ----- | ---- | ----------- |
| id | usize | Unique tab identifier |
| label | String | Display label (branch name or "All") |
| is_closable | bool | False for "All" tab |
| branch_filter | Option\<String\> | Branch to filter by (None = all) |
| text_filter | String | Text search filter |
| author_filter | Option\<String\> | Author name filter |
| date_range | Option\<(i64, i64)\> | Date range filter (start, end) |
| path_filter | Option\<String\> | File path filter |
| scroll_offset | f32 | Vertical scroll position |
| selected_commit | Option\<String\> | Currently selected commit hash |

---

### CommitMessageHistory (NEW - persistence)

Recent commit messages for reuse.

| Field | Type | Description |
| ----- | ---- | ----------- |
| repo_path | String | Repository path as key |
| messages | Vec\<String\> | Last 10 commit messages, newest first |

**Storage**: JSON file at `~/.config/slio-git/commit-messages.json`

## Relationships

```
Repository 1──* Change (staged/unstaged files)
Repository 1──* Branch (local + remote)
Repository 1──* Stash
Repository 1──* Tag
Repository 1──* WorkingTree
Repository 1──1 CommitMessageHistory

Branch 1──* Commit (reachable history)
Commit 1──* Change (files modified in commit)
Commit 0..1── SignatureStatus
Commit 1──1 GraphNode (in log view)
GraphNode 1──* GraphEdge (to parents)
Commit *──* RefLabel (branches/tags pointing here)

Change 0..1── Conflict (when status = Conflict)
Change 0..1── BlameEntry * (when annotate is active)
```
