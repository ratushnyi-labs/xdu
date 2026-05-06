# TASK-002 — TUI 3-column layout, ".." parent row, last-access date column

**Status:** DONE  **Type:** TASK
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
Pre-merge baseline security gate: PASS — BR-001 read-only invariant preserved (only `meta.accessed()` consulted, no content reads); no new crate deps; no `unsafe`; date helpers are total functions (no panics on extremes — verified by `sec_format_ymd_no_panic_on_max`). 73 tests pass (67 unit + 6 integration). Multi-target rebuild within size budget (688 KiB / 622 KiB / 491 KiB).
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> so under win user should be able (if has permissions) to navigate to list of disks. under pisix systems to '/'. the run param for app should be the starting folder. e.g. 'xdu /' - start in root. by default starts in current work dir but allow to navigate the directory tree (i think vertical scroll might be useful). so i think we should have 3 columns: fs entity (folder, file, symlink) including where needed the '..' to navigate up, bar chart that shows relative value for for used space, space in numbers, possible one more column - last access date

## Refined Description

**Scope (this leaf, TASK-002):**
- Render the children list with **`name` first**, then `bar`, then `size`, then `last access date` — matching the user's requested column order.
- Insert a synthetic `..` row at the top of the children list whenever ascend would have an effect (back-stack non-empty OR scan-root has a filesystem parent). Pressing `Right` / `l` / Enter on the `..` row triggers ascend.
- Capture last-access time in the scanner (`Metadata::accessed()`) and store it on `Node` as a small unsigned integer (seconds since the Unix epoch; `0` = unknown).
- Refactor date / timestamp formatting helpers into `crate::format` so `export.rs` and the TUI share one canonical implementation.

**Already covered by prior work, surfaced for completeness:**
- "starting folder via run param" — `xdu [path]` (default `.`).
- "vertical scroll" — the body widget already paginates and keeps the selection on screen.
- "`xdu /` to start at the root" — already works post-BUG-001.

**Out of scope for THIS leaf:**
- **Windows drive-list view at the FS root** — moved to TASK-003 (deferred to next turn). On Windows today, ascending from `C:\` shows a status message and stays put; on Unix, ascending from `/` is already a no-op.

**Impacted UCs:** UC-002 (navigation — `..` row), UC-003 (display — column reorder + date column), UC-001 (scanner captures accessed time).
**Impacted BR/WF:** none directly modified; BR-001 read-only invariant preserved (accessed-time read is metadata-only).

## Estimation

**Complexity:** ABOVE_EASY  **Justification:** touches `format`, `tree`, `scanner`, `tui` and several test surfaces; column reorder requires width math; date helper refactor is across files; selection model changes to handle the `..` row.

## Speck Reference
[`docs/specks/SDD-TASK-002-columns-dotdot-date.md`](../specks/SDD-TASK-002-columns-dotdot-date.md)
