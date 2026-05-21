# TASK-007-N2 — dotfiles align to org canonical form

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT

Verification: dotfile-only change; build pipeline unaffected (validated in N1 smoke). Pre-merge baseline security gate: PASS — no code; `.env` comment-only; §5.4 stealth-identity preserved. §9.5 skipped per session-wide rule.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
- `.gitignore` — replace verbose form with canonical 8-line form per TASK-007-A findings.
- `.gitattributes` — replace per-extension EOL list with canonical 11-line form (binary file types).
- `.dockerignore` — replace with canonical 9-line form.
- `.env` — add a comment-only file (xdu currently has only `.env.example`).

**Impacted UCs:** none.
**Impacted BR/WF:** none.

## Estimation
**Complexity:** EASY. Four small files.

## Speck Reference
[`docs/specks/SDD-TASK-007-N2-dotfiles-align.md`](../specks/SDD-TASK-007-N2-dotfiles-align.md)

## Compliance Notes — §9.5
Skipped per session-wide rule. See TASK-007.
