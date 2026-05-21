# SDD-TASK-007-N4 — CONTRIBUTING.md + CHANGELOG.md

**Backlog item:** TASK-007-N4
**Impacted UCs:** none

## Scope
- `CONTRIBUTING.md` — sections: Development Setup, Workflow, Code Style, Safety Rules, Testing Requirements, Conventional Commits, PR Checklist. Safety section names xdu-specific invariants (BR-001 read-only default; force-close path restricted to `DUPLICATE_CLOSE_SOURCE`; no `TerminateProcess`/`NtClose` on foreign handles).
- `CHANGELOG.md` — Keep-a-Changelog format. `## [Unreleased]` placeholder. `## [0.1.0] - 2026-05-06` with one consolidated entry per top-level lawbook item (`TASK-001`, `BUG-001`, `TASK-002`/`003` filed-deferred, `TASK-004`, `TASK-005`, `BUG-002`, `BUG-003`, `BUG-004`, `TASK-006`).

## ACs
- AC-1: `CONTRIBUTING.md` exists with the 7 sections above.
- AC-2: `CHANGELOG.md` exists with Keep-a-Changelog header, `[Unreleased]`, and `[0.1.0] - 2026-05-06`.
- AC-3: 0.1.0 entry lists Added / Changed / Fixed / Security sub-sections (only those used; empty ones omitted).
- AC-4: Conventional Commit prefixes from `CONTRIBUTING.md` match the commit-message style xdu already used (`TASK-001-C:`, `BUG-001:`, etc. — these stay as the project's documented convention rather than strict `feat:` / `fix:`; document this divergence explicitly in CONTRIBUTING.md).

## SEC ACs
- SEC-1: §5.4 stealth identity preserved — no agent attribution; CHANGELOG entries are authored in the project's voice.
- SEC-2: Safety rules in CONTRIBUTING.md accurately reflect BR-001 amended state (delete + force-close are the only mutation paths; everything else read-only).

## §9.5 cross-agent attack review
Skipped per session-wide rule.
