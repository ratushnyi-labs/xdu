# TASK-007-N5 — README rewrite to org style

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT

Verification: README-only rewrite. Centered hero, three-badge row, supported-targets table (6 targets), documentation table, multilingual user-manual row stubbed (links resolve once N10 lands), Contributing/Security/License sections present, no agent attribution. §9.5 skipped per session-wide rule.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
Rewrite `README.md` to match the org pattern observed in `xsfx` / `spad` / `trim`:

- Centered title (`<p align="center">`) with one-line tagline.
- Three badges in a row: CI, License: MIT, Release.
- Section order: hero → Supported Targets table (6 targets confirmed in wave 4) → Quick Start / Usage → Install (Pre-built binaries via curl + Build from Source) → Testing → Documentation table (links to `docs/spec.md`, etc.) → Contributing → Security → License.
- Documentation table includes a multilingual user-manual row stub (filled in by TASK-007-N10).

**Impacted UCs:** none (documentation only).
**Impacted BR/WF:** none.

## Estimation
**Complexity:** EASY. Single-file rewrite.

## Speck Reference
[`docs/specks/SDD-TASK-007-N5-readme-rewrite.md`](../specks/SDD-TASK-007-N5-readme-rewrite.md)

## Compliance Notes — §9.5
Skipped per session-wide rule. See TASK-007.
