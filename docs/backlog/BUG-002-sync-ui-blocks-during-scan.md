# BUG-002 — synchronous scan blocks the UI

**Status:** DONE  **Type:** BUG
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
Pre-merge baseline security gate: PASS — BR-001 read-only invariant preserved (worker only reads metadata); no `unsafe`; no new crate dep; lock guards held for narrow scopes; running flag flipped only as the worker's last act. 77 tests pass (71 unit + 6 integration) — including new `scanner::scan_async_eventually_finalizes`, `scanner::after_scan_running_is_false_and_root_finalized`, `tui::bug002_async_scan_progresses_to_finalized`, `tui::spinner_char_is_one_of_the_frames`. Multi-target rebuild within size budget (646 KiB / 647 KiB / 503 KiB).
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> ok, now i see one more real issue. the sync interface. so when user navigates the interface should not block UI until all is calculated; you can write ??? when not yet calculated and redraw needed cells

(Follow-up clarification:)
> instead of ??? you can render spinner

## Reproduction

1. Run `xdu /large/path` against any directory tree large enough that the scan takes more than ~200 ms.
2. The terminal stays blank (or in `--help` text) until the scan completes; navigation keys cannot be pressed because the TUI has not started.
3. After the scan completes, the TUI opens with the fully-aggregated tree.

## Expected

- The TUI opens immediately. The user sees the scan root and can navigate as soon as the first directory listing is available.
- Cells whose value is not yet determined (in particular, directory totals while the scan is still walking that subtree) render an animated spinner (cycle of `| / - \` at ~10 fps). As the scanner finishes each subtree, the cell's real value appears on the next redraw.
- The footer shows a `scanning…` status while the worker is running.
- Pressing keys (navigation, sort, rescan, export, quit) works at all times — never blocks waiting for a scan to finish.

## Refined Description

**Scope:** introduce a worker-thread-based scanner. The TUI holds an `Arc<Mutex<ScanResult>>`; the worker thread mutates the shared state in place; the TUI redraws every loop iteration (already at ~10 Hz via the existing `event::poll(100ms)` cycle), seeing the latest state under a brief lock.

**Eager aggregation:** to make partial sums meaningful, scanner now bubbles `(size, alloc, +1 entries)` up the parent chain on every `add_child`. This replaces the post-scan single-shot `tree.aggregate()` call inside the scanner.

**Per-node `finalized` flag:** every node gains a boolean. Files, symlinks, and "other" entries are finalized at insert time (their size is known). Directories start `finalized = false` and get marked `true` after the scanner completes their subtree (a `Pre` / `Post` frame pair on the explicit DFS stack handles the post-visit). The TUI renders `???` (and a blank bar) for non-finalized rows.

**Public scanner API:** `scan(...)` (synchronous, unchanged for tests) and `scan_async(root, opts) -> Result<ScanHandle, ScanError>` (validates synchronously, then spawns the worker). Both share a private `scan_into` body.

**Impacted UCs:** UC-001 (scan can be asynchronous), UC-002 (navigation responsive during scan), UC-003 (display `???` for non-finalized).
**Impacted BR/WF:** BR-001 read-only invariant preserved (worker only reads metadata).

## Estimation

**Complexity:** ABOVE_EASY  **Justification:** touches `tree`, `scanner`, `tui`, `main` and several test surfaces; introduces a worker thread + `Arc<Mutex<...>>`. No new crate dependencies (uses `std::sync` + `std::thread`).

## Speck Reference
[`docs/specks/SDD-BUG-002-async-scanner.md`](../specks/SDD-BUG-002-async-scanner.md)
