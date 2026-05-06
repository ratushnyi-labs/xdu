# SDD-BUG-001 — ascend at scan root walks the filesystem parent

**Backlog item:** BUG-001
**Impacted UCs:** UC-002 (amended this Speck), UC-005 (existing)

## Scope
When `App::ascend` runs out of back-stack entries, attempt to scan the
filesystem parent of `App::root_path` and reset navigation to that scan.
Fall back to a status message when there is no FS parent (root of the
filesystem).

## ACs
- AC-1: At any descended position, `Left` / `h` / `Backspace` pops the back-stack and shows the previous directory (existing behavior preserved).
- AC-2: At the scan root with an empty back-stack, the same key triggers a synchronous `scan()` of `root_path.parent()`. On success the App's `root_path`, `scan`, `current`, `selection`, and `back_stack` are all replaced with the parent-rooted state.
- AC-3: When `root_path.parent()` is `None` (filesystem root), the App sets a status message ("at filesystem root — cannot go up") and stays put.
- AC-4: A scan failure on the parent path surfaces in the App's `status` line and leaves the existing tree intact.
- AC-5: The footer status string in `render_footer` shows the new status message until the next keypress (existing status mechanism).

## SEC ACs
- SEC-1: Read-only invariant (BR-001) preserved — the parent rescan path uses `xdu::scanner::scan`, which performs metadata reads only.
- SEC-2: No new `unsafe`. No new crate dependency.
- SEC-3: Path resolution uses `Path::canonicalize` to defeat surprising `.` / relative-path edge cases; if canonicalization fails (e.g., the path was deleted), fall back to the original `root_path`.

## Test
- Unit test inside `src/tui.rs::tests` builds a temp dir with a nested subdir, scans the inner dir, constructs `App`, calls `ascend()`, verifies `app.root_path` now points at the parent and the new tree's root contains the previously-scanned inner subtree.
- Existing 67 tests must continue to pass unchanged.

## Pre-merge baseline security gate
Will record after fix: SECURITY_AUDIT, DEPENDENCY_TREE_SECURITY_AUDIT, MEMORY_LEAK_AUDIT.
