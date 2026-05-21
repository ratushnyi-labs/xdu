# TASK-007-D — performance polish

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT

## Result

`src/scanner.rs::walk_one` refactored to stage entries into a local `Vec<StagedEntry>` and apply the batch under a single `state.lock()` per directory iteration. Previously the lock was taken once per file (N acquisitions); now it is 1 (or ⌈N / 4096⌉ for pathological huge directories — the `WALK_BATCH_CAP` soft cap).

- Lock acquisitions per directory: **N → 1** (or ⌈N / 4096⌉).
- Deferred errors (`read_dir` failure, `entry.metadata()` failure, individual `entry` iteration error) are batched into a local `Vec<(PathBuf, io::Error)>` and applied under the same lock.
- Stack pushes for sub-directories happen **after** the lock is released to keep the critical section short.
- All BR invariants preserved (BR-001 read-only, BR-002 hardlink dedup unchanged — `compute_sizes` still runs outside the lock against the scanner's local `HashSet`, BR-003 cross-fs check unchanged, BR-004 symlinks not followed, BR-005 errors don't abort).
- No new `unsafe`. No new crate dependency.

Tests: 98 pass (existing 93 + 5 new TASK-007-C). `cargo fmt`/`clippy --locked -D warnings`/`test --locked`: all clean.

Pre-merge baseline security gate: PASS — refactor preserves the documented contract; `cargo test --locked` covers the scanner end-to-end (`scans_simple_tree`, `permission_denied_subdir_is_recorded_not_fatal`, `hardlinks_are_deduplicated`, `symlinks_are_not_followed`, `scan_async_eventually_finalizes`). §9.5 skipped per session-wide rule.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description

Performance polish driven by inspection rather than synthetic benchmarks (the project
has no benchmark harness in v0.1 — adding one is `LATER-011` candidate). Targets the
two hottest paths:

- **Scanner mutex acquisitions.** Currently the scanner takes the `state` mutex once
  per `add_child` call. For typical 100-entry directories that is 100 lock cycles per
  dir. Change to a **single lock per directory iteration** in `walk_one`, accumulating
  insertions into a small local vector and applying them under one lock at the end of
  the loop. Keep the lock duration small (microseconds) by doing all metadata reads
  outside the lock.
- **Pre-allocate Vec capacities** where the size is known: render row vectors, the
  scanner's local accumulator, and the tree path breadcrumb.

These changes are observable (lock-acquisition count) but the behaviour-level effect
is invisible to the user. No new dependency. No `unsafe`.

**Non-goals (v0.1):**
- Parallel directory walking (ADR §4 — Phase 3 candidate).
- SIMD or custom allocator.
- Benchmark harness (separate task).

**Impacted UCs:** UC-001, UC-005.
**Impacted BR/WF:** none (BR-001 preserved; BR-005 error model preserved — errors still pushed under lock as before).

## Estimation
**Complexity:** ABOVE_EASY. Refactors the inner loop of `walk_one`; needs care to keep
the lock contract identical for readers.

## Speck Reference
[`docs/specks/SDD-TASK-007-D-performance-polish.md`](../specks/SDD-TASK-007-D-performance-polish.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
