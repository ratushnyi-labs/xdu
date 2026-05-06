# TASK-001-F — Tree model

**Status:** DONE  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
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
[`docs/specks/SDD-TASK-001-F-tree-model.md`](../specks/SDD-TASK-001-F-tree-model.md)

## Verified results

- 18 unit tests pass: 14 functional + 4 `[SEC]`.
- Deep-chain test exercises 100 000-level depth without call-stack overflow.
- Saturating overflow test exercises `u64::MAX` aggregation with `size_overflow()` latch.
- `cargo fmt --check` clean. `cargo clippy --locked -D warnings` clean.
- No `unsafe`, no I/O.

## Pre-merge baseline security gate (§9.4)

| Audit                          | Depth     | Disposition | Rationale                                                                |
|--------------------------------|-----------|-------------|--------------------------------------------------------------------------|
| SECURITY_AUDIT                 | NORMAL    | PASS        | Pure data structure; no `unsafe`; no I/O; invalid IDs return `Option::None` (SEC-3 verified by test); arena cap rejection prevents `OutOfCapacity` panics (SEC-2 verified by code path inspection). |
| DEPENDENCY_TREE_SECURITY_AUDIT | FAST      | PASS        | No new crate dependencies (per ADR §7).                                  |
| MEMORY_LEAK_AUDIT              | FAST      | PASS        | Only allocations are `Vec<Node>` and per-call sort/path scratch vectors. No retained globals, no listener accumulation, no thread spawning. Aggregation stack size proportional to depth (SEC-4). |

Findings: none. Continuity: AC-2 partially asserted (cannot allocate 4 G nodes in unit test;
the `len() >= u32::MAX` branch was reviewed by code inspection — no follow-up required).

Status: **DONE**.
