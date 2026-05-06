# TASK-001-K — Rescan / refresh flow

**Status:** READY  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** PLAN_READY
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
**Scope:** `r` keybinding rescans the focused subtree, builds a new subtree off-screen, atomically swaps it into the parent. Spinner shown during rescan.
**Impacted UCs:** UC-005.
**Impacted BR/WF:** WF-001 (partial).
**Depends on:** TASK-001-E, TASK-001-I.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE | ESTIMATION | DONE | DECOMPOSITION | N/A | DESIGN | DONE | INTEG | PENDING | TESTING | PENDING | POST_AUDIT | PENDING |

## Estimation
**Complexity:** EASY

## Acceptance (pointers)
- AC-1: post-rescan focused subtree reflects current FS state; siblings unchanged.
- AC-2: target deleted between scans → entry removed from parent + reported in error log.
- AC-3: spinner visible during operation; UI not blocked (input still polled — but only `q` and ESC accepted).
- SEC-1: rescan thread cannot mutate tree concurrently with main render — swap is atomic.

## Speck Reference
`docs/specks/SDD-TASK-001-K-rescan.md` (to be authored).
