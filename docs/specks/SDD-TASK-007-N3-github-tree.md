# SDD-TASK-007-N3 — `.github/` tree

**Backlog item:** TASK-007-N3
**Impacted UCs:** none

## Scope
Add the `.github/` directory with CODEOWNERS, SECURITY.md, PULL_REQUEST_TEMPLATE.md, and three ISSUE_TEMPLATE files. Wording adapted from `xsfx` / `spad` patterns; substitute xdu-specific elements (project name, OS dropdown, scope language matching BR-001 mutation surface).

## ACs
- AC-1: `.github/CODEOWNERS` = `* @paveldhq` (matches spad/trim convention).
- AC-2: `.github/SECURITY.md` has Supported Versions, Reporting (advisory link), Response Timeline, Scope sections.
- AC-3: `.github/PULL_REQUEST_TEMPLATE.md` has Summary, Checklist, Test Plan.
- AC-4: `.github/ISSUE_TEMPLATE/bug_report.yml` requires xdu version + OS dropdown + description + repro + logs (org-pattern shape).
- AC-5: `.github/ISSUE_TEMPLATE/feature_request.yml` has problem + solution + alternatives.
- AC-6: `.github/ISSUE_TEMPLATE/config.yml` sets `blank_issues_enabled: false` and a contact link to `docs/`.

## SEC ACs
- SEC-1: No secrets; no executable shell in templates.
- SEC-2: §5.4 stealth identity preserved.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
