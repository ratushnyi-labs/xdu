# TASK-001-H — TUI runtime

**Status:** READY  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** PLAN_READY
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
**Scope:** terminal init/teardown (alt-screen, raw mode, mouse off), event loop with keyboard handling, panic-safe restoration (a panic must always restore the terminal — `std::panic::set_hook` + `Drop` guard), 10 Hz progress redraw cap, color detection (`NO_COLOR`, terminal capability), UTF-8 detection.
**Non-goals:** widget content (TASK-001-I, J).
**Impacted UCs:** UC-001 (TUI init), UC-002 (input loop), UC-007.
**Impacted BR/WF:** —
**Depends on:** TASK-001-C.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE | ESTIMATION | DONE | DECOMPOSITION | N/A | DESIGN | DONE | FRONTEND | PENDING | TESTING | PENDING | POST_AUDIT | PENDING |

## Estimation
**Complexity:** ABOVE_EASY  **Justification:** cross-platform terminal lifecycle + panic hook is a known landmine.

## Acceptance (pointers)
- AC-1: launch + immediate Ctrl-C exits cleanly with terminal restored to cooked mode + main screen.
- AC-2: panic during render restores terminal before propagating.
- AC-3: `NO_COLOR` env var disables color output entirely.
- AC-4: ASCII-only fallback when terminal does not advertise UTF-8.
- AC-5: progress redraws no more than 10× per second.
- SEC-1: no input echo to terminal during raw mode (passwords accidentally typed shall not appear).
- SEC-2: window resize handled without crash.

## Speck Reference
`docs/specks/SDD-TASK-001-H-tui-runtime.md` (to be authored).
