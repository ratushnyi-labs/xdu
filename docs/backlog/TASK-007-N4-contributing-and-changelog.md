# TASK-007-N4 — CONTRIBUTING.md + CHANGELOG.md

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT

Verification: Markdown-only. CHANGELOG `[0.1.0]` entry cross-checked against `git log --oneline` and covers every committed leaf since scaffold. CONTRIBUTING safety rules align with BR-001 amended state. §9.5 skipped per session-wide rule.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
- `CONTRIBUTING.md` — sections: Development Setup → Workflow → Code Style → Safety Rules → Testing Requirements → Conventional Commits → PR Checklist. Mirror xsfx structure; substitute xdu-specific safety rules (read-only invariant, force-close handles, mutex contention).
- `CHANGELOG.md` — Keep-a-Changelog format. `[Unreleased]` and `[0.1.0] - 2026-05-06` entry covering every commit so far (scaffold → BUG-004 / TASK-006).

**Impacted UCs:** none.
**Impacted BR/WF:** none.

## Estimation
**Complexity:** EASY. Two Markdown files; CHANGELOG content compiled from `git log`.

## Speck Reference
[`docs/specks/SDD-TASK-007-N4-contributing-and-changelog.md`](../specks/SDD-TASK-007-N4-contributing-and-changelog.md)

## Compliance Notes — §9.5
Skipped per session-wide rule. See TASK-007.
