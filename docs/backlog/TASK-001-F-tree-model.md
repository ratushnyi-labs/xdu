# TASK-001-F — Tree model

**Status:** READY  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** PLAN_READY
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
**Scope:** in-memory arena tree storing scan results. `Vec<Node>` indexed by `NodeId(u32)`, parent / first_child / next_sibling links, name interning, size + alloc + entry count aggregation, stable sort by configured key (UC-004).
**Non-goals:** persistence; parallel mutation.
**Impacted UCs:** UC-002, UC-003, UC-004, UC-005, UC-006.
**Impacted BR/WF:** BR-002, BR-006.
**Depends on:** TASK-001-C.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE | ESTIMATION | DONE | DECOMPOSITION | N/A | DESIGN | DONE | BACKEND | PENDING | TESTING | PENDING | POST_AUDIT | PENDING |

## Estimation
**Complexity:** ABOVE_EASY  **Justification:** arena indexing + path resolution + stable sort + memory budget.

## Acceptance (pointers)
- AC-1: `NodeId` is `u32`; tree supports 4 G entries before rejection.
- AC-2: aggregation: parent.size = sum(children.size); parent.entries = sum(children.entries) + 1.
- AC-3: sort is stable, ties broken by name asc.
- AC-4: full path lookup `O(depth)`.
- SEC-1: integer overflow on size sums saturates at u64::MAX with a flag.
- SEC-2: rejects insertion past `NodeId::MAX` with explicit error (no panic).

## Speck Reference
`docs/specks/SDD-TASK-001-F-tree-model.md` (to be authored).
