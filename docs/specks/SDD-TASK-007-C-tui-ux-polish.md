# SDD-TASK-007-C — TUI UX polish

**Backlog item:** TASK-007-C
**Impacted UCs:** UC-002, UC-003, UC-007, UC-008

## Scope
- Switch `sort_label` to `↓` / `↑` arrow glyphs under UTF-8, ASCII fallback `-` / `+`. Reuses `is_utf8_terminal()`.
- Add a unit test asserting `name_column_width(small)` floors at 8 cells without panic.
- Add a unit test asserting `compose_footer` handles a width smaller than the right segment without panic.

## ACs
- AC-1: `sort_label_for(SortKey, utf8: bool)` returns `"size↓"` for `(SizeDesc, true)` and `"size-"` for `(SizeDesc, false)`. Symmetric for all 6 keys.
- AC-2: `name_column_width(30)` returns 8 (the floor).
- AC-3: `compose_footer("left", " right ", 5)` does not panic and produces a width-5 result.
- AC-4: Existing 93 tests still pass.

## SEC ACs
- SEC-1: BR-001 invariant preserved.
- SEC-2: No new crate dependency.
- SEC-3: No `unsafe` added.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
