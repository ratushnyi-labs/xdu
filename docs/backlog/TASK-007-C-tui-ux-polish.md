# TASK-007-C — TUI UX polish

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT

## Result

- `sort_label_for(SortKey, utf8: bool)` introduced; `sort_label(k)` now dispatches via `is_utf8_terminal()`. UTF-8 paths show `size↓` / `size↑` / `name↑` / `name↓` / `count↓` / `count↑`; ASCII fallback unchanged (`size-` etc.).
- New tests:
  - `task007c_sort_label_uses_arrows_in_utf8`
  - `task007c_sort_label_falls_back_to_ascii`
  - `task007c_name_column_width_floors_at_8`
  - `task007c_compose_footer_narrower_than_right_does_not_panic`
  - `task007c_compose_footer_width_zero_is_empty`

`cargo fmt`/`clippy --locked -D warnings`/`test --locked`: all clean. §9.5 skipped per session-wide rule. Pre-merge baseline security gate: PASS — no behavioural change to delete/force-close paths; no new `unsafe`; BR-001 invariant preserved.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description

Targeted UX improvements without changing key bindings or layout structure:

- **Sort indicator clarity:** swap the ASCII `size-` / `size+` for arrow glyphs (`size↓` / `size↑`) in UTF-8 terminals, fall back to ASCII otherwise. Mirrors the bar UTF-8/ASCII dual-mode pattern.
- **Narrow-terminal safety check:** confirm `name_column_width` floors at 8 cells, no panic on `<35`-column terminals. Add a unit test if missing.
- **Spinner-visibility check:** confirm the spinner is rendered in the size column for non-finalized rows under both default and reverse-video styling.
- **Status-bar truncation:** the existing `compose_footer` truncates left when too narrow; confirm the right-segment free-space block is preserved on a 30-column terminal (no panic, no underflow).
- **Help text accuracy:** verify the help overlay still lists all keys (including new `f` force-close in the lock prompt).

**Impacted UCs:** UC-002, UC-003, UC-007, UC-008.
**Impacted BR/WF:** none.

## Estimation
**Complexity:** EASY. Small render tweaks + 1-2 unit tests.

## Speck Reference
[`docs/specks/SDD-TASK-007-C-tui-ux-polish.md`](../specks/SDD-TASK-007-C-tui-ux-polish.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
