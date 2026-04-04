# Feature Specification: JetBrains-Styled Diff File List Panel

**Feature Branch**: `003-jetbrains-diff-ui`
**Created**: 2026-03-22
**Status**: Draft
**Input**: User description: "现在没有差异文件列表，界面的样式要和jetbrains配色主题一样，UI也要和idea源码里一样。所有UI都要jetbrains风格，重构一下，现在太丑了"

## Clarifications

### Session 2026-03-22

- Q: 点击文件后应该发生什么？ → A: 选中文件并打开差异对比视图（选项B）
- Q: UI重构的范围是什么？ → A: 整个应用程序的每个UI元素都要完全匹配IDEA的视觉效果（选项C）
- Q: 使用哪种JetBrains主题风格？ → A: Darcula 暗色主题（选项A）
- Q: 对IDEA源码的UI结构和布局参考程度？ → A: 完全复制IDEA的UI结构和布局，包括菜单结构、工具栏、面板分割方式等（选项C）

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Changed Files List (Priority: P1)

As a user, I want to see a list of all files that have been modified, added, or deleted in my repository, so I can quickly understand what has changed.

**Why this priority**: This is the core functionality requested - without a diff file list, users cannot review their changes efficiently.

**Independent Test**: Can be fully tested by opening a repository with changes and verifying the changed files appear in the panel with correct status indicators.

**Acceptance Scenarios**:

1. **Given** a git repository with modified files, **When** the user opens the diff file list, **Then** all modified files are displayed with a "modified" indicator
2. **Given** a git repository with new untracked files, **When** the user opens the diff file list, **Then** all new files are displayed with an "added" indicator
3. **Given** a git repository with deleted files, **When** the user opens the diff file list, **Then** all deleted files are displayed with a "deleted" indicator
4. **Given** a git repository with no changes, **When** the user opens the diff file list, **Then** an empty state is shown indicating no changes

---

### User Story 2 - JetBrains Visual Theme (Priority: P1)

As a user, I want the entire application UI to match the JetBrains color scheme and visual style of IntelliJ IDEA, so the interface feels familiar and consistent with other JetBrains tools I use.

**Why this priority**: User explicitly requested JetBrains styling to ensure visual consistency with their IDE experience.

**Independent Test**: Can be tested by comparing the application's color palette, typography, and UI elements against IntelliJ IDEA's appearance.

**Acceptance Scenarios**:

1. **Given** the application is running, **When** the diff file list is displayed, **Then** it uses the JetBrains Darcula dark theme color palette (background #2B2B2B, text #ABB2BF, accent #6897BB)
2. **Given** the application is running, **When** the diff file list is displayed, **Then** file status colors match IDEA conventions (blue for modified, green for added, red/orange for deleted)
3. **Given** the application is running, **When** the UI is displayed, **Then** the window structure, menu layout, toolbar arrangement, and panel split layout match IDEA's native structure
4. **Given** the application is running, **When** the UI is displayed, **Then** the UI components (borders, separators, icons) match IDEA's visual style

---

### User Story 3 - Interactive File Selection (Priority: P2)

As a user, I want to click on a file in the diff list to select it, so I can focus on reviewing specific changes.

**Why this priority**: Enables user interaction with the diff list for practical code review workflows.

**Independent Test**: Can be tested by clicking on files in the list and verifying selection state is visually indicated.

**Acceptance Scenarios**:

1. **Given** the diff file list contains files, **When** the user clicks on a file, **Then** that file becomes selected with a highlighted background and the diff view opens showing the file's content changes
2. **Given** a file is selected, **When** the user clicks on another file, **Then** the selection moves to the newly clicked file and the diff view updates to show the newly selected file's changes

---

### User Story 4 - File Status Icons (Priority: P2)

As a user, I want to see intuitive icons indicating the type of change for each file, so I can quickly scan the list without reading text labels.

**Why this priority**: Visual indicators improve scanning efficiency and match IDEA's established patterns.

**Independent Test**: Can be tested by verifying correct icons appear for modified, added, and deleted files.

**Acceptance Scenarios**:

1. **Given** a file is modified, **When** it appears in the list, **Then** it shows a modified indicator icon
2. **Given** a file is new, **When** it appears in the list, **Then** it shows an added indicator icon
3. **Given** a file is deleted, **When** it appears in the list, **Then** it shows a deleted indicator icon

---

### Edge Cases

- What happens when the repository has a very large number of changed files (100+)? Does the list scroll smoothly?
- How does the system handle files with special characters in their names?
- What happens when files are renamed versus simply added/deleted?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display a panel listing all files with uncommitted changes in the current repository
- **FR-002**: System MUST indicate the status of each file (modified, added, deleted)
- **FR-003**: System MUST apply JetBrains IDE color scheme to ALL UI components in the application
- **FR-004**: System MUST use JetBrains IDEA visual styling for ALL UI elements (window frame, panels, borders, backgrounds, text, icons)
- **FR-005**: Users MUST be able to select a file from the list by clicking
- **FR-006**: System MUST provide file status icons that match JetBrains conventions
- **FR-007**: System MUST show an empty state when there are no changes in the repository
- **FR-008**: System MUST scroll smoothly when the list exceeds the visible area

### Key Entities *(include if feature involves data)*

- **ChangedFile**: Represents a file that has been modified, added, or deleted. Attributes: path (string), status (enum: modified|added|deleted|renamed), selectionState (boolean)
- **DiffPanel**: The UI container component that displays the list of ChangedFile entities

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view all changed files in the repository within 1 second of opening the panel
- **SC-002**: The diff file list matches JetBrains color palette within 95% accuracy (RGB values)
- **SC-003**: All file status indicators are correctly displayed for modified, added, and deleted files
- **SC-004**: Users can select files in the list and receive immediate visual feedback
- **SC-005**: The empty state is displayed when repository has no changes
