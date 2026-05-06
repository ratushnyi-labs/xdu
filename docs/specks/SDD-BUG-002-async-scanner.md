# SDD-BUG-002 — async scanner with progressive TUI

**Backlog item:** BUG-002
**Impacted UCs:** UC-001, UC-002, UC-003

## Scope

### `src/tree.rs`
- `Node` gains `pub finalized: bool`. Default at `add_child`: `!matches!(kind, NodeKind::Dir)` — files / symlinks / other are finalized at insert.
- `Tree::new` initializes the root with `finalized = false` (it is a `Dir`; the scanner will mark it `true` after its subtree completes).
- New mutator: `pub fn set_finalized(&mut self, id: NodeId, val: bool)`.
- New mutator: `pub fn bubble_up_add(&mut self, from: NodeId, size: u64, alloc: u64, entries: u64)` — walks the parent chain from `from` and saturating-adds each delta to every ancestor; latches `size_overflow` on saturation.
- Existing `aggregate` / `aggregate_subtree` retained — used by tests and as a one-shot consistency pass when needed.

### `src/scanner.rs`
- `ScanResult` gains `pub running: bool`. Synchronous `scan()` returns with `running = false`; `scan_async()` returns a handle whose locked state shows `running = true` until the worker exits.
- New `pub struct ScanHandle { pub state: Arc<Mutex<ScanResult>>, _join: Option<JoinHandle<()>> }`.
- New `pub fn scan_async(root: PathBuf, opts: ScanOpts) -> Result<ScanHandle, ScanError>`. Validation (NUL byte, exists, is-dir) runs synchronously so callers get an early `Err`. After validation, the worker is spawned.
- Synchronous `pub fn scan(root, opts) -> Result<ScanResult, ScanError>` retained — it now delegates to the same private `scan_into` core.
- DFS stack uses `Frame::Pre(NodeId, PathBuf)` and `Frame::Post(NodeId)`. After processing all children of a directory, the scanner pops a `Post` frame and calls `tree.set_finalized(id, true)`.
- Eager aggregation: each successful `add_child` is followed by `tree.bubble_up_add(id, size, alloc, 1)`. The scanner no longer calls `tree.aggregate()` at the end.

### `src/tui.rs`
- `App` replaces its owned `ScanResult` with `state: Arc<Mutex<ScanResult>>` plus `_scan_thread: Option<JoinHandle<()>>` (kept to express ownership; the worker is left to run to completion when the App is dropped).
- `App::new` accepts a `ScanHandle`.
- Every method that touches the tree / errors locks the mutex for the duration of the call — single-threaded App, brief critical sections.
- Render: when `node.finalized == false`, the size column shows an animated **spinner** (cycle of `| / - \` at ~10 fps based on wall-clock time) and the bar is rendered as blanks. Date column renders normally (set at insert). The spinner is the primary "not yet calculated" affordance per user preference; `???` is the spec-level fallback wording for terminals where spinner glyphs render unpredictably.
- Footer: while `state.running == true`, shows `scanning ({n} entries) …` instead of the regular sort/error/key hints.
- `rescan` and `scan_filesystem_parent` invoke `scan_async()` and replace `App::state` / `_scan_thread` with the new handle. Old worker thread, if still running, becomes orphaned and finishes on its own (no-op cleanup; v0.2 will add a quit-signal pattern).

### `src/main.rs`
- Replaces `scan(...)` + `tui::run(scan_result, ...)` with `scan_async(...)` + `tui::run(handle, ...)`.

## ACs

- AC-1: `xdu /any/path` opens the TUI **before** the scan completes for any directory tree large enough that scanning takes >50 ms.
- AC-2: Pressing navigation / sort / quit keys responds within 100 ms regardless of scan progress.
- AC-3: Directories that have not been fully scanned render `???` in the size column and a blank 10-cell bar.
- AC-4: After `state.running` flips to `false`, every directory in scope shows a real size; no `???` remains.
- AC-5: The footer shows a distinct `scanning …` indicator while `state.running == true` and switches to the regular layout otherwise.
- AC-6: `rescan` (`r`) replaces the state and starts a new worker; old workers are not joined but do not corrupt the new state (they hold an Arc on the old state which is no longer referenced by the App).
- AC-7: BR-001 read-only invariant preserved — no new write syscall paths are introduced.
- AC-8: All 73 prior tests continue to pass (after adapting any tests that destructure `ScanResult` to ignore the new `running` field).

## SEC ACs

- SEC-1: `Mutex` is `std::sync::Mutex` — no new crate dep.
- SEC-2: Lock guards held for narrow scopes (single method) so the worker never starves the TUI redraw.
- SEC-3: No `unsafe`. No `std::mem::transmute`. No FFI.
- SEC-4: ScanHandle is `Send` (Arc + Mutex + PathBuf + ScanOpts are all Send). The worker closure is also `Send + 'static`. Tested by compilation.
- SEC-5: Race on `running` → `false`: scanner sets `running = false` *as the very last action* before exiting, after every other state mutation. TUI sees a fully consistent state when it observes `running == false`.

## Test Matrix

| AC    | Unit | Integration                    |
|-------|------|--------------------------------|
| AC-1  | —    | smoke (manual: launch, observe) |
| AC-2  | —    | smoke                          |
| AC-3  | ✅   | new tui test (renders ??? when finalized=false) |
| AC-4  | ✅   | new scanner test (after scan completes, every node finalized) |
| AC-5  | review | —                            |
| AC-6  | review | —                            |
| AC-7  | static| —                            |
| AC-8  | full test suite — must run green |
| SEC-1..5 | review | —                         |
