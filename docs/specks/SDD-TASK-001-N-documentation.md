# SDD-TASK-001-N — Documentation (manuals + final README)

**Backlog item:** TASK-001-N
**Impacted UCs:** all (documentation surface)

## Scope
Author the lawbook §2.1 required documents:
- `docs/development-manual.md` — local dev workflow (Docker only).
- `docs/installation-manual.md` — how end-users install / use the binary.
- `docs/configuration-manual.md` — every CLI flag, env var, color rule, size budget.
- Final `README.md` replacing the scaffold skeleton from TASK-001-C.

## ACs
- AC-1: All four files exist and are coherent with the implemented v0.1.0 feature set.
- AC-2: Manuals reference applicable UC IDs from `docs/spec.md`.
- AC-3: `configuration-manual.md` enumerates every flag, every env var (`NO_COLOR`, `LANG`, `LC_ALL`, `LC_CTYPE`), color-decision tree, and size budgets per ADR-001 §5.
- AC-4: README documents Docker-based build flow + extraction commands per platform.
- AC-5: All four documents preserve stealth-identity policy (§5.4): no agent / model / vendor attribution.
