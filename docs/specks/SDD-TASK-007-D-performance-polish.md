# SDD-TASK-007-D — performance polish

**Backlog item:** TASK-007-D
**Impacted UCs:** UC-001, UC-005

## Scope
- Restructure `src/scanner.rs::walk_one` so each directory iteration takes the shared mutex **once** rather than once per entry. Concretely: collect all `(NodeId-parent, NodeKind, name, size, alloc, accessed_secs)` tuples from `DirEntry` traversal into a local `Vec<EntryStaged>` (built outside the lock), then take the lock and apply the whole batch with `add_child` + `set_accessed` + `bubble_up_add` in sequence, plus push any deferred sub-dirs onto the DFS stack. The lock duration scales with the size of the batch (typical: a few microseconds; pathological: limited by `max_errors` cap and a soft 4096-entry batch cap).
- Pre-allocate `Vec::with_capacity` for the local batch buffer.
- Pre-allocate `Vec::with_capacity` for the rendered row lines in `render_body` (already done; confirm).
- No behaviour change. Error semantics (BR-005) preserved — errors continue to be recorded under the same lock window.

## ACs
- AC-1: Existing 93 tests still pass (no behaviour change observable from the API).
- AC-2: `cargo clippy --locked -- -D warnings` clean.
- AC-3: Lock-acquisition count per directory is **1** (not N) — confirmed by code review of `walk_one`.
- AC-4: Pathological huge-dir case (>4096 entries) still drains in soft batches to avoid holding the lock across an unbounded payload.

## SEC ACs
- SEC-1: BR-001 / BR-005 / BR-002 / BR-003 / BR-004 all preserved.
- SEC-2: No new `unsafe`. No new crate dependency.
- SEC-3: Hardlink set still de-duplicates correctly (one `HashSet<(dev,ino)>` shared across the scan; batch refactor must not let two file entries in the same batch double-count).

## §9.5 cross-agent attack review
Skipped per session-wide rule.
