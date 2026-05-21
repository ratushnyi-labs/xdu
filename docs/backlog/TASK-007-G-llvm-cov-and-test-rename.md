# TASK-007-G — cargo-llvm-cov coverage gate + `sec_uc*` → `test_sec_uc*` rename

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-21  **Updated:** 2026-05-21

## Refined Description
Resolves `LATER-006` from `docs/spec.md` §9.

Changes:
- Add `cargo-llvm-cov` to the Dockerfile `base` stage so `docker compose run --rm test cargo llvm-cov ...` works inside the container.
- Add `scripts/coverage.sh` that runs `cargo llvm-cov --workspace --locked --lcov --output-path coverage.lcov` and prints a human-readable summary.
- Add `coverage.lcov` + `lcov-report/` to `.gitignore`.
- Rename test functions whose names start with `sec_uc<digits>_` to `test_sec_uc<digits>_` for consistency with the wider test-naming convention (Rust does not require the `test_` prefix, but the lawbook security-test labelling is clearer with it).
- Update any references (e.g. test count assertions) accordingly.

**Behaviour change:** none in shipping code; test infrastructure only.

## Estimation
**Complexity:** EASY. Tool install + a few test renames + one new shell script.

## Speck Reference
[`docs/specks/SDD-TASK-007-G-llvm-cov-and-test-rename.md`](../specks/SDD-TASK-007-G-llvm-cov-and-test-rename.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
