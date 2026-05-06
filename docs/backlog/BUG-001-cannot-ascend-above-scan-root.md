# BUG-001 — cannot ascend above the scan root

**Status:** DONE  **Type:** BUG
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
Pre-merge baseline security gate: PASS — read-only invariant (BR-001) preserved (parent rescan goes through `xdu::scanner::scan` which is metadata-only); no new crate deps; no `unsafe`. Verified by 68 tests including `bug001_ascend_at_root_walks_filesystem_parent`. Multi-target rebuild within size budget (688 KiB / 620 KiB / 490 KiB).
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> bug. it does not allow user to do up in the directory tree

## Reproduction

1. Run `xdu /some/path`.
2. Without descending into any subdirectory (or after ascending all the way back to the scan root), press `Left` / `h` / `Backspace`.
3. **Observed:** nothing happens. The TUI stays at the scan root with no feedback.
4. **Expected:** `xdu` walks up to the filesystem parent of the scan root and re-scans, allowing the user to continue exploring upward.

## Root cause

`src/tui.rs::App::ascend` only consults the in-memory `back_stack`. When the user is at `tree.root()`, `back_stack` is empty, `pop()` returns `None`, and the function exits silently. The TUI offers no signal that the keypress was recognized, and the user has no path to walk above the scan root short of relaunching `xdu` with a parent path.

The behavior at the scan root was never specified in `docs/spec.md` UC-002 (only "Validation: descending into a non-directory entry is a no-op"). This is a spec gap as well as an implementation gap — fixed by amending UC-002 first per project rule §3.5.

## Refined Description

**Scope:** when `App::ascend` is called and the back-stack is empty, walk to the filesystem parent of `App::root_path`, re-scan with the existing `ScanOpts`, replace `App::scan` and reset navigation. If the current root is already at the filesystem root (e.g. `/` on Unix or `C:\` on Windows), surface a status message instead of failing silently.

**Non-goals:** asynchronous scan with progress display (parent rescan is synchronous, matching the existing `r` rescan flow); preserving any portion of the previous scan tree across the ascend (the in-memory tree is fully replaced).

**Impacted UCs:** UC-002 (navigation), UC-005 (rescan — same code path).
**Impacted BR/WF:** BR-001 (read-only — preserved; the new path performs only metadata reads). Re-confirms BR-005 (errors don't abort).

## Estimation

**Complexity:** EASY  **Justification:** single method change + one accessory unit test + a one-line spec amendment. No new dependencies, no architecture impact.

## Speck Reference
[`docs/specks/SDD-BUG-001-ascend-walks-fs-parent.md`](../specks/SDD-BUG-001-ascend-walks-fs-parent.md)
