# TASK-001-E — Scanner module

**Status:** READY  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** PLAN_READY
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request
> Inherited from TASK-001.

## Refined Description
**Scope:** filesystem walker producing entries for the tree model. stdlib-only `read_dir` + `Metadata`, single-threaded scanner, hardlink dedup `(dev, ino)` Unix / `(volume, file_index)` Windows, mount/volume boundary detection, error capture per BR-005.
**Non-goals:** parallelism (deferred per ADR §4); symlink following (BR-004); content reading.
**Impacted UCs:** UC-001 (scan), UC-005 (rescan).
**Impacted BR/WF:** BR-002, BR-003, BR-004, BR-005, BR-006; WF-001.
**Depends on:** TASK-001-C.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE |
| ESTIMATION | DONE |
| DECOMPOSITION | N/A |
| DESIGN | DONE (ADR §2/§4) |
| BACKEND | PENDING |
| TESTING | PENDING |
| POST_AUDIT | PENDING |

## Estimation
**Complexity:** ABOVE_EASY  **Justification:** cross-platform metadata + hardlink set + mount detection + error model.

## Acceptance (pointers)
- AC-1: `scan(root, opts)` returns a handle yielding progress + final result.
- AC-2: BR-002 hardlinks counted once per `(dev,ino)` Unix / `(volume, file_index)` Windows.
- AC-3: BR-003 default does not cross mount boundary; `--cross-fs` opts in.
- AC-4: BR-004 symlinks recorded as their own size; not followed.
- AC-5: BR-005 per-entry errors collected; scan continues.
- SEC-1: refuse paths containing null bytes.
- SEC-2: no subprocess spawning.
- SEC-3: bounded error vector (cap 10 000) — no unbounded memory growth.
- SEC-4: stack-safe traversal — iterative, not recursive call-stack — handles trees ≥ 50 000 deep.

## Speck Reference
`docs/specks/SDD-TASK-001-E-scanner.md` (to be authored).
