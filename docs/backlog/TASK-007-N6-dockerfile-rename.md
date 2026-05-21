# TASK-007-N6 — Dockerfile rename + output dir → dist/

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
- Rename `Dockerfile.build` → `Dockerfile`.
- Output directory `out/` → `dist/` everywhere (Dockerfile, `docker-compose.yml`, `scripts/build.sh`, `.env.example`, README references).
- The org-canonical `.gitignore` (committed in N2) already lists `dist/`, so the new artefacts are correctly ignored without further changes.

**No behavior change.** The build is identical; only path and filename move.

## Estimation
**Complexity:** EASY. Pure rename across ~5 files.

## Speck Reference
[`docs/specks/SDD-TASK-007-N6-dockerfile-rename.md`](../specks/SDD-TASK-007-N6-dockerfile-rename.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
