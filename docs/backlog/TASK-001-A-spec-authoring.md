# TASK-001-A — Author `docs/spec.md` v0.1.0

**Status:** DONE  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request
> Inherited from TASK-001.

## Refined Description
**Scope:** author the v0.1.0 specification (UCs, BR/WF, diagrams, design-token preliminary table) describing the read-only disk-usage explorer.
**Non-goals:** schematics & baseline screenshots — deferred to leaf sub-tasks (DESIGN inside FE leaves; baselines inside TASK-001-M).
**Impacted UCs:** UC-001 … UC-007 (new).
**Impacted BR/WF:** BR-001 … BR-006, WF-001, WF-002 (new).
**Depends on:** —

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE (parent) |
| ESTIMATION | DONE |
| DECOMPOSITION | N/A (leaf) |
| DESIGN | DONE |
| FRONTEND | N/A (docs-only) |
| BACKEND | N/A (docs-only) |
| TESTING | N/A (docs-only) |
| POST_AUDIT | DONE |

## Estimation
**Complexity:** EASY  **Justification:** single-file authored from clarification answers + ADR.

## Acceptance
- AC-1: `docs/spec.md` exists with sections 1–8.
- AC-2: every UC in scope (UC-001…UC-007) carries Functional Requirements, BE Tech, FE Tech (where applicable), Mermaid flow, and references to schematic + baseline paths.
- AC-3: every BR-### / WF-### is listed with WF sequence diagrams (BR-001…BR-006, WF-001, WF-002).

## Speck Reference
N/A — this leaf's deliverable IS the spec document; treating `docs/spec.md` as its own design-record per §3.4 ("Speck before any code change") since this leaf produces no code.

## Deliverables
- [docs/spec.md](../spec.md)
