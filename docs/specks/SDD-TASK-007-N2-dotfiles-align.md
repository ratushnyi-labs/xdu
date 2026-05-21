# SDD-TASK-007-N2 — dotfiles align

**Backlog item:** TASK-007-N2
**Impacted UCs:** none

## Scope
Replace `.gitignore`, `.gitattributes`, `.dockerignore` with the org-canonical contents documented in TASK-007-A `## Findings`. Create a comment-only `.env` for §8.1 compliance (org pattern).

## ACs
- AC-1: `.gitignore` is exactly the 8-line canonical form (`.idea`, `target`, `lcov.info`, `*.profraw`, `dist/`, `.build/`, `.dev-data/`, `.dev-temp/`).
- AC-2: `.gitattributes` is the 11-line binary-type list with `* text=auto eol=lf` first.
- AC-3: `.dockerignore` is the 9-line form.
- AC-4: `.env` exists as a comment-only file.
- AC-5: `git status` after the change shows no surprising files becoming tracked or untracked outside the dotfile changes.

## SEC ACs
- SEC-1: No secrets introduced; `.env` is comment-only.
- SEC-2: §5.4 stealth identity preserved.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
