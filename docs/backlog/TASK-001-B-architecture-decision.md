# TASK-001-B — ADR-001 baseline architecture

**Status:** DONE  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request
> Inherited from TASK-001.

## Refined Description
**Scope:** record the option analysis required by §3.1.3 and select v0.1.0 baseline architecture (TUI library, walker strategy, tree storage, concurrency boundary, build pipeline).
**Non-goals:** detailed implementation; that lives in implementation Specks (D…N).
**Impacted UCs:** all (architectural).
**Impacted BR/WF:** —
**Depends on:** TASK-001-A (so spec UCs are visible to architecture choice).

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE (parent) |
| ESTIMATION | DONE |
| DECOMPOSITION | N/A (leaf) |
| DESIGN | DONE |
| FRONTEND | N/A |
| BACKEND | N/A |
| TESTING | N/A |
| POST_AUDIT | DONE |

## Estimation
**Complexity:** EASY  **Justification:** option analysis + decision record, no code.

## Acceptance
- AC-1: ADR file exists at `docs/architecture/ADR-001-baseline-architecture.md`.
- AC-2: at least three options analyzed (A, B, C), with tradeoff matrix and decision rationale.
- AC-3: build pipeline targets and profile flags recorded.
- AC-4: forbidden-during-implementation list recorded.
- AC-5: evolution triggers documented.

## Speck Reference
N/A — deliverable IS the ADR document.

## Deliverables
- [docs/architecture/ADR-001-baseline-architecture.md](../architecture/ADR-001-baseline-architecture.md)
