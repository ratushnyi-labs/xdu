# TASK-001-N — Documentation (manuals + final README)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
Pre-merge baseline security gate: PASS (no executable code; Markdown only; stealth identity policy preserved — no agent / model / vendor names in any document). Speck: `docs/specks/SDD-TASK-001-N-documentation.md`. Deliverables: `docs/development-manual.md`, `docs/installation-manual.md`, `docs/configuration-manual.md`, final `README.md`.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
**Scope:** lawbook §2.1 required docs:
- `docs/development-manual.md` — local dev workflow (Docker only)
- `docs/installation-manual.md` — how end-users install / use the binary on each platform
- `docs/configuration-manual.md` — all CLI flags, env vars, color rules, build budgets
- final `README.md` (replacing scaffold skeleton from TASK-001-C) with usage, screenshots placeholder, license, build instructions.
**Non-goals:** marketing site, blog posts.
**Depends on:** all of E…L.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE | ESTIMATION | DONE | DECOMPOSITION | N/A | DESIGN | DONE | DOC | PENDING | POST_AUDIT | PENDING |

## Estimation
**Complexity:** EASY

## Acceptance (pointers)
- AC-1: all four files exist and are coherent with the implemented feature set.
- AC-2: manuals reference relevant UC IDs from `docs/spec.md`.
- AC-3: `configuration-manual.md` enumerates: every CLI flag, every env var, color decision tree, size budgets per ADR-001 §5.

## Speck Reference
`docs/specks/SDD-TASK-001-N-documentation.md` (to be authored).
