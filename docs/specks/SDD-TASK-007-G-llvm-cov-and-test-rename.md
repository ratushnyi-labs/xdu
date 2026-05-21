# SDD-TASK-007-G — cargo-llvm-cov coverage + `test_sec_uc*` rename

**Backlog item:** TASK-007-G
**Impacted UCs:** none (test infrastructure only)

## Scope
- `Dockerfile` `base` stage: install `cargo-llvm-cov` via
  `cargo install --locked cargo-llvm-cov` AND `rustup component add llvm-tools-preview`.
- New file `scripts/coverage.sh`: runs `cargo llvm-cov --workspace --locked
  --lcov --output-path coverage.lcov` (inside the `test` service) and prints
  the human-readable summary table.
- `.gitignore`: add `coverage.lcov` and `lcov-report/`.
- Rename test functions `sec_uc<digits>_<rest>` → `test_sec_uc<digits>_<rest>`
  in `src/**` and `tests/**`.

## ACs
- AC-1: `docker compose run --rm test sh -c "cargo llvm-cov --version"` succeeds.
- AC-2: `sh scripts/coverage.sh` emits a coverage summary table and writes
  `coverage.lcov`.
- AC-3: `cargo test --all --locked` still passes after the renames; no test
  is silently skipped (count unchanged or higher).
- AC-4: `grep -rn "fn sec_uc[0-9]" src/ tests/` returns no matches (all
  renamed).

## SEC ACs
- SEC-1: No new shipping crate dependency (cargo-llvm-cov is dev tooling only).
- SEC-2: §8.2 host isolation preserved (tool runs inside the `test` container).

## §9.5 cross-agent attack review
Skipped per session-wide rule.
