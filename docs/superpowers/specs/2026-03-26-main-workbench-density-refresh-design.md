# Main Workbench Density Refresh Design

- Date: 2026-03-26
- Project: `slio-git`
- Scope: Main workbench homepage only (`top chrome + left rail + change list + diff canvas + status bar`)
- Design basis: Keep Yanqu visual language, pull overall density and control discipline back toward PhpStorm / IDEA

## 1. Summary

This iteration refines the visual quality of the main workbench without changing the product direction.

The app keeps the current Yanqu-style palette:

- light gray app background
- white content surfaces
- mint green as the primary accent

But the homepage must stop behaving like a loose business dashboard. It should feel like a focused Git editor workspace:

- denser
- calmer
- less card-heavy
- less chip-heavy
- less scroll-heavy
- more consistent in button sizing and toolbar rhythm

The target outcome is:

> "Yanqu colors, JetBrains density, editor-first workbench."

## 2. Problem Statement

The current homepage has several visible quality issues:

1. Too much padding at multiple levels makes the page feel loose and under-dense.
2. Buttons, split buttons, tabs, and chips do not consistently share the same sizing system.
3. Many horizontal rows are wrapped in scroll containers even when they should fit inline, which makes scrollbars visually noisy.
4. Too many small cards, chips, and tinted panels compete at the same hierarchy level.
5. The left change list and right diff canvas do not yet feel like two parts of one editor workspace.

The result is not "wrong style", but "style without discipline".

## 3. Goals

### Primary goals

- Preserve the Yanqu color identity and light workbench atmosphere.
- Increase homepage information density to be closer to PhpStorm / IDEA.
- Reduce visual noise from always-visible or overused scroll regions.
- Standardize control sizes and layout rhythm across the homepage.
- Make the change list + diff canvas clearly read as the main task area.

### Secondary goals

- Create a density baseline that later auxiliary views can inherit.
- Reduce the need for one-off styling by tightening shared tokens and shared controls.

## 4. Non-Goals

This spec does not include:

- a full product-wide redesign
- reworking business flows or Git behavior
- introducing a new design system framework
- changing major information architecture beyond homepage density corrections
- redoing non-homepage complex panels in this same implementation pass unless they inherit shared token updates automatically

## 5. Users and Usage Context

The main workbench serves users who want to:

- scan modified files quickly
- switch focus between files with low friction
- read diff content with maximum canvas priority
- perform frequent Git actions from the top chrome

These users value:

- compact clarity
- predictable control placement
- visible current state
- low decorative overhead

They do not need:

- dashboard-like spaciousness
- excessive chips and banners
- strong card separation between every small sub-block

## 6. Design Principles

### 6.1 Keep Yanqu, reduce looseness

The refresh does not abandon Yanqu. It narrows its expression:

- mint remains the accent
- background remains light
- white cards remain the main surface language

But Yanqu should show up as tone and polish, not as expanded spacing everywhere.

### 6.2 The homepage is an editor workbench, not a service dashboard

The visual center of gravity must be:

- change list on the left
- diff canvas on the right

Any top-level element that does not help those two jobs should become quieter.

### 6.3 Only meaningful hierarchy gets a card

Not every group needs a distinct panel treatment.

Allowed strong surfaces:

- top chrome containers
- left change pane shell
- right diff pane shell
- modal / floating surfaces

Avoid:

- card-inside-card-inside-card composition for basic tool rows
- using tint and chip styling as a substitute for real hierarchy

### 6.4 Compactness must come from system rules, not arbitrary shrinking

The goal is not "make everything smaller". The goal is:

- shared heights
- predictable padding
- fewer competing accents
- tighter alignment

## 7. Reference Direction

### Chosen branch

- Yanqu business desktop page language for color/material tone
- PhpStorm / IDEA style density for homepage rhythm and control discipline

### Explicit interpretation

The homepage should feel like:

- a light Git tool window inside a desktop IDE

Not like:

- a dashboard landing page
- a marketing workbench
- a full dark IDE clone

## 8. Target Surface Map

This iteration focuses on these homepage layers:

1. Primary top toolbar
2. Secondary navigation strip
3. Left rail
4. Left change list pane
5. Right diff pane header
6. Right diff content canvas framing
7. Bottom status bar

Shared control updates may also affect:

- button helpers
- scrollable helpers
- theme tokens
- toolbar split-button rendering

## 9. Layout Design

## 9.1 Overall homepage skeleton

The homepage keeps the current macro structure:

- top chrome
- main work area
- status bar

But internal density changes:

- top chrome becomes thinner and more toolbar-like
- main work area gains more usable content height
- left pane and right pane behave like coordinated editor surfaces
- status bar becomes thinner and less attention-seeking

## 9.2 Primary top toolbar

The first row contains:

- repository switcher
- branch switcher
- sync state indicator when meaningful
- core actions: refresh, pull, push, commit

### Rules

- Repository switcher and branch switcher should read as toolbar fields, not large cards.
- Path text remains secondary and smaller.
- The branch block should show only the most important current-state indicator, not multiple equal-priority pills.
- The sync chip must be hidden for low-information stable states such as synced or no-upstream. It appears only for actionable or warning-worthy states.
- Action buttons on the right must share a common height and baseline.
- `commit` remains the strongest CTA in the row; `refresh / pull / push` are subordinate.

## 9.3 Secondary navigation strip

The second row contains the workbench section navigation:

- changes
- conflicts
- history
- remotes
- tags
- stashes
- rebase

### Rules

- This row behaves like a proper editor tab strip, not a row of unrelated buttons.
- Active state must be visible through a restrained selected treatment.
- Inactive items should remain low-noise.
- Height must be lower than the current implementation.

## 9.4 Left rail

The rail remains for high-level workspace switching.

### Rules

- Reduce the perceived weight of each icon container.
- Active state remains distinct, but inactive items become quieter.
- Large rounded blocks should be avoided unless required for current state emphasis.
- The rail must not compete with the change list for attention.

## 9.5 Left change list pane

This is the most important density correction area.

### Rules

- Treat this pane as a structured editor list, not a stack of business cards.
- The header area should compress summary information into one compact toolbar/summary band.
- The `commit / shelve / stash` row should behave like tool tabs, not oversized action pills.
- File rows should become tighter:
  - line 1: checkbox, status marker, filename
  - line 2: parent path and compact metadata
- Selected state should be visible but not flood the row with accent color.
- Explanatory helper text should not occupy prime vertical real estate in the list.

## 9.6 Right diff pane

This must feel the most editor-like area on screen.

### Rules

- The diff header becomes a compact editor toolbar line.
- Summary data is reduced to the minimum set needed for orientation.
- The header must consume less vertical space than today.
- The code region gets more height share.
- The framing around the diff should be calmer and less card-segmented than the left pane.

## 9.7 Bottom status bar

The status bar remains, but as a thin utility strip.

### Rules

- Prefer one-line status communication.
- Use color only for meaningful severity.
- Avoid making it feel like another banner or panel.

## 10. Visual Token Decisions

## 10.1 Color usage

Keep the existing Yanqu palette, but narrow the accent rules:

- Mint green is for:
  - primary CTA
  - selected state
  - checkbox checked state
  - one or two key status surfaces
- Neutral gray/white handles:
  - default tool rows
  - list rows
  - pane framing
  - low-priority pills

### Additional rules

- Do not tint all panels green.
- Do not convert every metadata item into a colored chip.
- Warning and danger remain sparse and utilitarian.

## 10.2 Spacing system

Homepage rhythm should primarily use:

- 8
- 12
- 16
- 24

### Default usage

- micro gap: 8
- control group gap: 8 to 12
- pane internal padding: 12
- larger section padding: 16
- only special floating surfaces may go to 24

The current tendency toward repeatedly using 16, 18, 20, and larger nested padding on the homepage should be reduced.

## 10.3 Radius

Use a compact radius hierarchy:

- default control radius: 8
- larger floating surface radius: 12

Do not let ordinary list rows and tool rows feel pill-like or overly soft.

## 10.4 Shadow

Keep shadows shallow:

- one subtle layer for floating or raised surfaces
- avoid thick depth stacking between sibling panes

## 11. Component Rules

## 11.1 Buttons

The homepage uses a limited control-height system:

- standard toolbar/button height: ~32 px
- compact control height: ~28 px

Applies to:

- primary
- secondary
- ghost
- split-button segments
- top-level utility tabs

### Constraints

- Equal heights in the same row are mandatory.
- Text baselines and icon alignment must visually match.
- Split buttons must not show a different height for main segment vs chevron segment.

## 11.2 Chips and badges

Chips should become a selective annotation tool, not the default way to express all metadata.

### Rules

- Allow at most 2 to 3 chips in a dense summary row before collapsing the rest into plain text.
- Use only two functional chip roles on the homepage:
  - state chip
  - compact stat chip
- Low-information labels should prefer secondary text over chips.

## 11.3 Scroll containers

Only true long-content regions should visibly scroll.

### Allowed persistent scroll contexts

- file/change list
- diff body
- history list
- branch list

### Not acceptable as default horizontal scroll regions

- top toolbar rows
- summary strips
- chip rows that should be reflowed or simplified

### Scrollbar policy

- default state: visually hidden or nearly hidden
- hover: lightly visible
- drag: clearly visible

## 11.4 Inputs

Inputs and search bars should align with toolbar density, not form density.

### Rules

- compact consistent height
- restrained tinting
- same border strength as adjacent toolbar controls
- no oversized field treatment inside dense workbench headers

## 12. Interaction Behavior

## 12.1 Priorities

The homepage must optimize for these two actions above all else:

1. pick a file on the left
2. inspect diff on the right

All secondary visual behavior should support those two actions.

## 12.2 Selection behavior

- Selected list rows should be obvious, but should not turn into large mint cards.
- Selection is emphasis, not decoration.

## 12.3 Toolbar behavior

- Most users should complete their primary action set without horizontal scrolling.
- Frequent actions should remain within one clean visual cluster.

## 13. Architecture and Implementation Boundaries

Implementation should favor shared system fixes before per-view patches.

## 13.1 Shared foundations first

Primary touchpoints:

- `src-ui/src/theme.rs`
- `src-ui/src/widgets/button.rs`
- `src-ui/src/widgets/scrollable.rs`
- `src-ui/src/widgets/mod.rs`

These should define the compact rules.

## 13.2 Homepage composition second

Primary homepage touchpoints:

- `src-ui/src/views/main_window.rs`
- change list widget
- diff pane header / shell widget(s)

These should consume the shared rules rather than inventing new local measurements.

## 13.3 Scope discipline

Do not solve this by adding more special-case local styling for every panel.

The homepage should become the density baseline. Later views may adopt it in follow-up work.

## 14. Error Handling and Regression Considerations

This is a UI-only iteration, but it still has failure modes:

- accidental regression into over-compressed layouts
- truncation in toolbar controls
- hidden important sync state if chip visibility logic is too aggressive
- broken alignment across split buttons and mixed control rows
- reintroducing scrollbars through nested scroll wrappers

### Guardrails

- Prefer content simplification before adding horizontal scroll.
- Keep all hidden-state logic explicit and deterministic.
- Validate with realistic long repository names and branch names.

## 15. Testing Strategy

## 15.1 Functional validation

- Build passes with `cargo check -p src-ui`
- Existing Git actions remain reachable from the homepage
- No homepage action becomes hidden behind mandatory horizontal scrolling

## 15.2 Visual validation

Manual review with realistic data should confirm:

- first impression is "editor workbench", not "dashboard"
- buttons align in the top chrome
- fewer scrollbars are visible at rest
- the left pane shows more meaningful rows per screen
- the right pane gives more area to diff content

## 15.3 State validation

Check homepage with:

- synced branch
- ahead branch
- behind branch
- diverged branch
- no upstream branch
- empty changes
- many changes
- long file paths
- long repository path

## 16. Acceptance Criteria

The iteration is successful when all of the following are true:

1. The homepage still reads as Yanqu in color and material tone.
2. The homepage density is visibly closer to PhpStorm / IDEA than the current loose version.
3. Default resting state does not prominently show multiple scrollbars.
4. Top chrome buttons and split buttons are visually consistent in height and alignment.
5. The left change list shows more content within the same viewport.
6. The right diff pane gives a stronger editor-like reading area.
7. The page no longer feels chip-heavy or over-carded.

## 17. Rollout Recommendation

Implement in this order:

1. shared density token cleanup
2. shared button and scrollbar corrections
3. top chrome density pass
4. change list density pass
5. diff header density pass
6. final visual consistency sweep on homepage

This keeps the work focused and gives one clear benchmark before propagating rules to the rest of the application.

## 18. Future Follow-Ups

After this spec is implemented and validated, the next logical follow-ups are:

- branch/history panels adopting the same density baseline
- conflict/rebase surfaces inheriting the same toolbar and list rules
- chip reduction across other auxiliary views

These are intentionally deferred to keep the current implementation scope tight.
