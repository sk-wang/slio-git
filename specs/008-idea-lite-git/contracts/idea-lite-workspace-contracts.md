# UI Contracts: IDEA 式 Git 工作台主线

**Feature**: IDEA 式 Git 工作台主线  
**Date**: 2026-03-25  
**Branch**: `008-idea-lite-git`

## Contract: Single Git Workspace

```rust
/// Once a repository is opened, the product MUST behave like one Git-focused
/// workspace instead of a collection of unrelated dialogs.
///
/// Required:
/// - one primary workspace centered on current repository context
/// - branch, sync, and risk hints remain visible without leaving the workspace
/// - changes review is the visual center of the first screen
```

## Contract: Review Before Action

```rust
/// The default interaction rhythm MUST be:
/// inspect changes -> decide -> execute git action.
///
/// Required:
/// - a visible change list
/// - immediate diff preview when focus changes
/// - stage / unstage / discard / commit paths attached to review context
///
/// Discouraged:
/// - action-heavy screens that hide current file changes
/// - workflows that require leaving the workspace before understanding changes
```

## Contract: High-Frequency Actions First

```rust
/// High-frequency git actions MUST remain closer to the workspace center than
/// low-frequency management features.
///
/// High-frequency actions include:
/// - stage / unstage
/// - commit
/// - fetch / pull / push
/// - branch switch / search / create
///
/// Lower-frequency context such as history, tags, stashes, and remotes may stay
/// reachable, but must not displace the main review surface.
```

## Contract: Auxiliary Peek Surfaces

```rust
/// History, tags, stashes, remotes, and similar context surfaces MUST behave as
/// auxiliary peeks, not as a replacement for the main workspace.
///
/// Rules:
/// - user can enter quickly when more context is needed
/// - user can return quickly to the previous workspace state
/// - these surfaces help judgment, but do not become the primary home screen
```

## Contract: Risk-State Continuation

```rust
/// When git workflow is blocked by conflicts, authentication issues, merge or
/// rebase progress, the product MUST keep the user inside an understandable
/// continuation path.
///
/// Required:
/// - explain current blocked state in Chinese
/// - provide the next useful action or entry point
/// - refresh workspace state after resolution
///
/// Forbidden:
/// - raw git internals as the only user-facing message
/// - dead-end dialogs with no recovery guidance
```

## Contract: IDEA Lite Scope

```rust
/// The product direction is Git-first IDEA Lite, not a full IDE.
///
/// This feature MUST NOT introduce:
/// - code editing as a primary workflow
/// - plugin-platform thinking in the workspace shell
/// - feature growth that weakens the "review changes + do git" core promise
```
