# TASK-007-R1 — Author parent SDD-TASK-007 Speck

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Priority:** MEDIUM
**Atomic Check:** ATOMIC (one deliverable: the umbrella Speck for TASK-007)
**Conflict Resolution Reference:** sibling of TASK-007-R2 (peer split from
  the now-retired umbrella R per §12.4.1; user confirmed split 2026-06-03)
**Created:** 2026-06-03  **Updated:** 2026-06-03

## Raw Request

Supervisor-derived, peer-confirmed via §12.4.1 split (user "Split into
R-1 + R-2" 2026-06-03). The parent TASK-007 backlog promises a
`docs/specks/SDD-TASK-007-prepare-for-publishing.md` (parent line 213:
"minimal parent-level Speck after TASK-007-A reports its findings") that
has never been authored. We shipped 17 child Specks but no umbrella one
documenting the cross-cutting ACs that v0.1.0 collectively satisfies.

## Refined Description

Author **`docs/specks/SDD-TASK-007-prepare-for-publishing.md`** as a
minimal parent-level Speck. It does NOT duplicate the child Specks;
instead it captures only what the children cannot express individually:

- Cross-cutting ACs that span multiple leaves (e.g. "all 6 release-matrix
  targets build under 1.5 MiB", "the .github/ tree matches the org
  convention", "the README + CHANGELOG + CONTRIBUTING set is internally
  consistent").
- The traceability map from each child Speck back to the umbrella —
  one-line per child, "TASK-007-X → satisfies AC-N of parent".
- SEC ACs that span multiple leaves (e.g. "stealth identity preserved
  across every wave-2..wave-7 commit", "no agent attribution leaked
  anywhere in shipping artefacts").
- An explicit §9.5 compliance-notes pointer to the session-wide skip
  rationale.

**Out of scope:** restating any child Speck content verbatim; making
any code / Dockerfile / script change; widening the parent's ACs beyond
what is already implied by the 17 children.

## Estimation

**Complexity:** EASY. One file under 150 lines; pure document compose.

## Speck Reference

The deliverable IS the Speck. No second Speck about authoring the
Speck — that would be infinite regress.

## Subagent Spawn Log

| Time (UTC) | Role | Purpose | Status | Notes |
|------------|------|---------|--------|-------|
| 2026-06-03 | BA   | Refine R1 + R2 and produce both deliverables (single BA spawn handling both atomic peers sequentially in its own context) | SUCCESS | Parent Speck authored at `docs/specks/SDD-TASK-007-prepare-for-publishing.md` — 8 cross-cutting ACs + 3 SEC ACs + 28-row traceability table (all 22 child leaves + this leaf + R2 + O) + §9.5 skip pointer; 118 lines. Spawn continues with R2 next. |

## Compliance Notes — §9.5

Skipped per session-wide rule.
