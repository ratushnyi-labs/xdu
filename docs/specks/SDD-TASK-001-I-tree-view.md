# SDD-TASK-001-I — Tree view widget

**Backlog item:** TASK-001-I
**Impacted UCs:** UC-002, UC-003, UC-004

## Scope
ratatui widget rendering current dir's children in rows: `<bar:10>  <size:>9>  <name>[/]`. Selection highlight (reverse video). Visible-window scrolling that keeps selection on screen. Color rules from `docs/spec.md` §7. Filename sanitization to escape control chars.

## ACs
- AC-1: Each row format matches the specified column layout.
- AC-2: Directories suffixed with `/`; symlinks with `@`.
- AC-3: Selection highlighted via `Modifier::REVERSED`.
- AC-4: Rows whose share > 25% of dir total render in red; > 10% yellow; otherwise default (or cyan for dirs).
- AC-5: Names are sanitized (control chars → `?`); no terminal injection from filenames.
- AC-6: Header path truncated from the LEFT with `…` when wider than viewport.

## SEC ACs
- SEC-1: Filename rendering escapes ASCII control codes (`\r`, `\n`, etc.) — no terminal sequence injection.
- SEC-2: Width calculations bounded by `area.width`; no panics on narrow terminals.

Implemented in `src/tui.rs`.
