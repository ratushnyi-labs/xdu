# SDD-TASK-001-F — Tree model (arena store)

**Backlog item:** TASK-001-F
**Impacted UCs:** UC-002, UC-003, UC-004, UC-005, UC-006
**Impacted BR/WF:** BR-002, BR-006

## Scope / Non-goals

**In scope:** `src/tree.rs`:
- `NodeId(u32)` — opaque arena index; `u32` allows up to 4 G−1 entries before rejection.
- `NodeKind { Dir, File, Symlink, Other }`.
- `Node { kind, name: Box<str>, size, alloc, entries, error, parent, first_child, next_sibling }`.
- `Tree { nodes: Vec<Node>, root, overflow, size_overflow }`.
- `Tree::new`, `add_child`, `mark_error`, `get`, `root`, `len`, `is_empty`, `children` iterator, `path` breadcrumb, `sort`, `aggregate`.
- `SortKey { SizeDesc, SizeAsc, NameAsc, NameDesc, CountDesc, CountAsc }`.
- `TreeError { OutOfCapacity, InvalidParent }`.
- Iterative post-order aggregation (stack-safe — does not recurse on the call stack).
- Saturating `u64` arithmetic on aggregate sums; `size_overflow` flag latched on saturation.
- Stable sort with name-ascending tiebreak.
- Comprehensive unit tests + `[SEC]` cases.

**Out of scope:** scanner integration (TASK-001-E), JSON serialization (TASK-001-L), TUI rendering (TASK-001-I), name interning / path arena.

## Acceptance Criteria

- AC-1: `Tree::new(NodeKind::Dir, "/")` produces a tree of length 1 whose root has the supplied name and `entries = 1`.
- AC-2: `NodeId` is `u32`-backed; the tree rejects insertion when `len() >= u32::MAX` with `TreeError::OutOfCapacity` and sets `overflow() == true`.
- AC-3: `add_child` to an unknown parent returns `TreeError::InvalidParent`; the tree is unchanged.
- AC-4: After `aggregate()`, every directory node has `size = Σ children.size`, `alloc = Σ children.alloc`, `entries = 1 + Σ children.entries` (saturating).
- AC-5: After `aggregate()`, the root reflects the full subtree totals.
- AC-6: Aggregation is iterative — the data structure tolerates depths ≥ 100 000 without call-stack overflow.
- AC-7: Saturating `u64` add: aggregating values that overflow caps at `u64::MAX` and sets `size_overflow() == true`; no panic.
- AC-8: `sort(parent, key)` is stable; ties broken by name ascending. Verified for all six `SortKey` variants.
- AC-9: `children(parent)` yields children in the **current** sibling-link order; calling `sort` then `children` reflects the new order.
- AC-10: `path(id)` returns the breadcrumb from root to `id` (inclusive) in `O(depth)`.
- AC-11: `mark_error(id)` flags the node; `aggregate` propagates the flag upward (any ancestor of an errored node has `error = true`).

## Security ACs

- SEC-1: No `unsafe` code in this module.
- SEC-2: `add_child` rejects when arena would exceed `u32::MAX` capacity (no panic, explicit error).
- SEC-3: Invalid `NodeId` access via `get` returns `Option::None`; never panics.
- SEC-4: Aggregation is allocation-bounded (single `Vec<(NodeId, bool)>` stack used; size proportional to depth, not total node count).
- SEC-5: No I/O; no network; no `std::process`.

## Failure Modes

- `OutOfCapacity` — surfaced to caller; subsequent `add_child` calls also fail.
- `InvalidParent` — surfaced; tree unchanged.
- Saturated sums — silent latch via `size_overflow()` flag the caller may inspect.

## Test Matrix

| AC    | Unit | Integration | Notes                                        |
|-------|------|-------------|----------------------------------------------|
| AC-1  | ✅   | —           |                                              |
| AC-2  | partial | —        | Cannot allocate 4G nodes in unit test; cap test verified by mocking the threshold via private helper or reviewed by code inspection. |
| AC-3  | ✅   | —           |                                              |
| AC-4  | ✅   | —           | Multi-level fixture                          |
| AC-5  | ✅   | —           |                                              |
| AC-6  | ✅   | —           | Build a 100k-deep linear chain, aggregate    |
| AC-7  | ✅   | —           | Two children with size = u64::MAX            |
| AC-8  | ✅   | —           | One test per SortKey + tie test              |
| AC-9  | ✅   | —           |                                              |
| AC-10 | ✅   | —           |                                              |
| AC-11 | ✅   | —           |                                              |
| SEC-1 | static (grep) | —     |                                              |
| SEC-2 | review| —          | AC-2 partial — safety logic inspected         |
| SEC-3 | ✅   | —           |                                              |
| SEC-4 | review| —          | Inspected: stack push count ≤ 2N             |
| SEC-5 | static (grep) | —     |                                              |

## Forbidden during this leaf

- New crate dependencies (per ADR §7).
- `unsafe` blocks.
- Recursive aggregation (stack-overflow risk on deep trees).
- I/O of any kind.
