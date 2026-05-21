# TASK-007-F — TUI snapshot test harness (LATER-004)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-21  **Updated:** 2026-05-21

## Refined Description
Resolves `LATER-004` from `docs/spec.md` §9. Adds a snapshot-style test harness
for the TUI so we can lock in the rendered output of representative scenes and
detect accidental visual regressions.

Approach:
- Use `ratatui::backend::TestBackend` (ships with ratatui; no new dep).
- Add a helper in a new `tests/snapshot_test.rs`:
  - Construct an `App` from a fixed in-memory fixture (no I/O).
  - Render to a `TestBackend` buffer at a given (width, height).
  - Stringify the buffer cell-by-cell into a `\n`-joined text grid.
- Compare against golden files under `docs/ui-baselines/snapshots/`.
- Bless mode: env var `XDU_BLESS_SNAPSHOTS=1` rewrites the golden file instead
  of asserting (a development convenience; CI never sets it).

Initial baselines (small — 3 scenes is enough to demonstrate the harness):
- `main_list_80x24.txt` — populated list view at default 80x24.
- `main_list_narrow_40x20.txt` — same data at narrow terminal (column widths flex).
- `confirm_delete_80x24.txt` — delete confirmation modal with default focus on `[ No ]`.

The harness exercises:
- Header line, list rendering, footer composition.
- The sort-key indicator label (utf-8 arrow vs ASCII).
- Confirmation modal layout from TASK-005.

**Why this matters:** the README claims a stable TUI; we lock it in. Future
refactors that change rendering trigger a clear baseline-update review instead
of silent drift.

## Estimation
**Complexity:** ABOVE_EASY. New test-only helpers + 3 baselines + small App
fixture construction.

## Speck Reference
[`docs/specks/SDD-TASK-007-F-tui-snapshot-harness.md`](../specks/SDD-TASK-007-F-tui-snapshot-harness.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
