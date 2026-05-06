# SDD-TASK-001-K — Rescan / refresh flow

**Backlog item:** TASK-001-K
**Impacted UCs:** UC-005

## Scope
`r` keybinding triggers a synchronous rescan of the focused subtree. v0.1.0 implementation is **synchronous** (UI freezes briefly during rescan; spinner deferred). Async progress is recorded as a follow-up — keeping the integration simple and reducing concurrency surface.

## ACs
- AC-1: Pressing `r` rebuilds the focused subtree from the live FS.
- AC-2: Sibling subtrees are unchanged (only the focused subtree's children are replaced via `Tree::aggregate_subtree` recompute).
- AC-3: If the focused subtree no longer exists on disk, an error is appended to the error log; selection rolls up to parent.
- AC-4: Synchronous rescan: UI does not respond to keystrokes during the call; this is acceptable for v0.1.0 and documented in the help overlay.

## v0.1.0 simplification
For minimal scope, rescan is implemented inline within the `r` key handler in `src/tui.rs`. It performs a fresh `scan()` of the focused dir's path and replaces the subtree contents. Atomic-swap with backstack-rebuild and async spinner are recorded as v0.2 follow-ups.

Implemented in `src/tui.rs`.
