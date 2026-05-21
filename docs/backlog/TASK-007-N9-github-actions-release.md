# TASK-007-N9 — .github/workflows/release.yml (on v* tag → GH Release)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
Add `.github/workflows/release.yml`, triggered on `v*` git tags.

Structure:
- `test` job — same as CI's test job (gate the release on a green test pass).
- `build` job — same matrix as CI: 6 targets across 4 runner types.
- `release` job — depends on test + build. Downloads all 6 artefacts, packages each as `xdu-<TARGET>.tar.gz` (Unix) or `xdu-<TARGET>.zip` (Windows), creates the GitHub Release via `softprops/action-gh-release@v2` with `generate_release_notes: true`.

Tag scheme: `v0.1.0`, `v0.1.1`, … (semver). The release-trigger gate (`on: push: tags: ["v*"]`) requires the tag to be pushed to GitHub — which is itself a CRR-3 action and requires explicit user confirmation when we get to TASK-007-O.

## Estimation
**Complexity:** ABOVE_EASY. Workflow YAML + packaging step.

## Speck Reference
[`docs/specks/SDD-TASK-007-N9-github-actions-release.md`](../specks/SDD-TASK-007-N9-github-actions-release.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
