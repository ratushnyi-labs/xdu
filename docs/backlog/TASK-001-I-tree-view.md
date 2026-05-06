# TASK-001-I — Tree view widget

**Status:** READY  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** PLAN_READY
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
**Scope:** ratatui widget rendering current directory's children as rows: `<bar:10>  <size:>9>  <name>[/]`. Selection highlight, scroll window, breadcrumb header, path display, color application from token table (spec §7).
**Non-goals:** sorting (TASK-001-K via UC-004 logic in F); modals (J); rescan (K).
**Impacted UCs:** UC-002, UC-003, UC-004.
**Impacted BR/WF:** —
**Depends on:** TASK-001-F (tree), TASK-001-G (formatter), TASK-001-H (runtime).

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE | ESTIMATION | DONE | DECOMPOSITION | N/A | DESIGN | DONE | FRONTEND | PENDING | TESTING | PENDING | POST_AUDIT | PENDING |

## Estimation
**Complexity:** ABOVE_EASY  **Justification:** widget + scrolling + color rules + small-terminal truncation.

## Acceptance (pointers)
- AC-1: rows visible in 80×24 terminal show `bar (10) + 2sp + size (9) + 2sp + name`.
- AC-2: selection highlighted via reverse-video.
- AC-3: scroll keeps selection on screen.
- AC-4: directories suffixed `/`; symlinks suffixed `@` (data-only — never followed).
- AC-5: `<40 col` terminal: name truncated with `…`; bar shrinks.
- SEC-1: name rendering escapes control characters (no `\r\n` injection from filenames).

## Speck Reference
`docs/specks/SDD-TASK-001-I-tree-view.md` (to be authored).
