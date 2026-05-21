# TASK-007-N7 — .github/workflows/ci.yml (native-runner matrix, 6 targets)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
Add `.github/workflows/ci.yml`, modelled on the org's xsfx CI but adapted for xdu's
6-target matrix and Docker-based local test gate.

Structure:
- `on: push` (branches main, feat/**, fix/**, task/**) + `on: pull_request`.
- `test` job — `ubuntu-latest`, runs `docker compose run --rm dev sh -c "fmt + clippy + test"`. Mirrors the local dev gate.
- `build` job — matrix of 6 targets across `ubuntu-latest` / `macos-latest` / `macos-14` / `windows-latest`. Each runner builds its NATIVE target via `cargo build --release --target …`. Output binaries uploaded as build artifacts (retention 7 days) for the per-target build log.

Per org pattern: uses `dtolnay/rust-toolchain@stable`, `actions/checkout@v4`,
`actions/upload-artifact@v4`.

## Estimation
**Complexity:** ABOVE_EASY. Workflow YAML + matrix design.

## Speck Reference
[`docs/specks/SDD-TASK-007-N7-github-actions-ci.md`](../specks/SDD-TASK-007-N7-github-actions-ci.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
