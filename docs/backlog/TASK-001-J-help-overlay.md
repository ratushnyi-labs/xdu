# TASK-001-J — Header / footer / help overlay

**Status:** READY  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** PLAN_READY
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
**Scope:** persistent header (root path + breadcrumb), footer (sort indicator, error count, key hint), help overlay listing all keybindings.
**Non-goals:** modal export prompt (TASK-001-L).
**Impacted UCs:** UC-002, UC-004, UC-007.
**Depends on:** TASK-001-H.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE | ESTIMATION | DONE | DECOMPOSITION | N/A | DESIGN | DONE | FRONTEND | PENDING | TESTING | PENDING | POST_AUDIT | PENDING |

## Estimation
**Complexity:** EASY

## Acceptance (pointers)
- AC-1: header shows root + breadcrumb truncated from left when too long.
- AC-2: footer shows `sort: <key><dir>  errors: <n>  ?: help  q: quit`.
- AC-3: help overlay lists every key from UC-002 / UC-004 / UC-005 / UC-006 / UC-007.
- AC-4: overlay dismissible by any keypress.

## Speck Reference
`docs/specks/SDD-TASK-001-J-help-overlay.md` (to be authored).
