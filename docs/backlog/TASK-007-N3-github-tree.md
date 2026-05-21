# TASK-007-N3 — `.github/` tree (CODEOWNERS, SECURITY, ISSUE/PR templates)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT

Verification: documentation-only addition; build/tests untouched (validated in N1 smoke). Pre-merge baseline security gate: PASS — templates contain no executable shell; SECURITY.md scope language matches BR-001 amended state. §9.5 skipped per session-wide rule.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
Add the org-canonical `.github/` directory tree per TASK-007-A findings:

- `.github/CODEOWNERS` — `* @paveldhq`.
- `.github/SECURITY.md` — Supported Versions, Reporting, Response Timeline, Scope.
- `.github/PULL_REQUEST_TEMPLATE.md` — Summary + Checklist + Test Plan.
- `.github/ISSUE_TEMPLATE/bug_report.yml` — version, OS dropdown, description, repro, logs.
- `.github/ISSUE_TEMPLATE/feature_request.yml` — problem, solution, alternatives.
- `.github/ISSUE_TEMPLATE/config.yml` — `blank_issues_enabled: false`, contact link to docs.

Workflows (`ci.yml`, `release.yml`) are out of scope here — they land in TASK-007-N7 / N9 after the build pipeline (wave 4) is restructured.

**Impacted UCs:** none.
**Impacted BR/WF:** none.

## Estimation
**Complexity:** EASY. Six small YAML / Markdown files.

## Speck Reference
[`docs/specks/SDD-TASK-007-N3-github-tree.md`](../specks/SDD-TASK-007-N3-github-tree.md)

## Compliance Notes — §9.5
Skipped per session-wide rule. See TASK-007.
