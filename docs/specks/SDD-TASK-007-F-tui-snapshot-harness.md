# SDD-TASK-007-F — TUI snapshot test harness

**Backlog item:** TASK-007-F
**Impacted UCs:** all rendered UCs indirectly (the harness locks in the
TUI output across refactors).

## Scope
- `src/tui.rs`: add two public test seams:
  - `pub fn render_snapshot(app: &App, width: u16, height: u16) -> String`
    — renders to ratatui's `TestBackend` and returns the cell symbols joined
    by `\n` (one row per terminal line).
  - `pub fn app_from_tree_for_test(root_path: PathBuf, tree: Tree) -> App`
    — wires a hand-built tree into an `App` without spawning a scanner.
- `src/tree.rs`: add `Tree::finalize_all(&mut self)` so the snapshot harness
  can mark every node finalized in one call (avoids exposing the private
  `NodeId` tuple constructor to integration tests).
- `tests/snapshot_test.rs`: 3 initial snapshot cases:
  - `snapshot_main_list_80x24` — populated 3-child fixture at default 80×24.
  - `snapshot_main_list_narrow_40x20` — same data at narrow terminal.
  - `snapshot_empty_tree_80x24` — empty-root layout.
- `docs/ui-baselines/snapshots/*.txt` — committed golden files. Regenerate
  with `XDU_BLESS_SNAPSHOTS=1 cargo test --test snapshot_test`.

## ACs
- AC-1: `cargo test --test snapshot_test` passes against the committed
  baselines.
- AC-2: A trivial change in rendering output (e.g. swapping the spinner
  glyph) causes the snapshot test to fail with a clear diff hint.
- AC-3: First-run convenience: when a baseline file does not yet exist,
  the test writes it and prints a notice to stderr; the test still
  passes. CI runs with the baselines committed so this branch is dead
  code in production.
- AC-4: `XDU_BLESS_SNAPSHOTS=1` env var rewrites baselines instead of
  asserting; without the env var, mismatches always fail.

## SEC ACs
- SEC-1: No new shipping crate dependency. `TestBackend` ships with the
  existing ratatui dep.
- SEC-2: New `pub fn`s are clearly test-helpers per their doc comments and
  do not widen the user-facing API surface.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
