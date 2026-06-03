# TASK-007-P — cargo-audit install + clean pass; llvm-cov baseline

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Priority:** MEDIUM
**Created:** 2026-06-03  **Updated:** 2026-06-03

## Raw Request

Supervisor-derived pre-publish hardening leaf identified in the 2026-06-03
audit. The org's Dockerfile CMD (per TASK-007-A finding line 226) chains
`cargo fmt --check && clippy && llvm-cov && audit`; xdu ships llvm-cov
(installed via TASK-007-G) but never installed cargo-audit and never RAN
either tool. Pre-publish should:
1. install `cargo-audit` in the Dockerfile `base` stage;
2. run `cargo audit` once and confirm a clean advisories pass (record any
   advisories that survive as `## Compliance Notes` entries with rationale);
3. run `cargo llvm-cov --workspace --locked --summary-only` once and record
   the baseline coverage percentage; decide whether to ship
   `--fail-under-lines N` in `scripts/coverage.sh` for the CI gate.

## Refined Description

Two related tool installs + recorded baselines, no shipping-code changes.

Changes:
- `Dockerfile` `base` stage: `cargo install --locked cargo-audit`. Pinned
  version recorded in the Dockerfile comment.
- `scripts/audit.sh`: thin wrapper that runs `docker compose run --rm test
  sh -c 'cargo audit'`. Mirrors the shape of `scripts/coverage.sh`.
- Run both `scripts/audit.sh` and `scripts/coverage.sh` once. Record the
  outputs as transcripts under `docs/audits/` (filenames stamped
  `dependency-audit-2026-06-03.md` and `coverage-baseline-2026-06-03.md`).
- If the coverage baseline is ≥95%, update `scripts/coverage.sh` to pass
  `--fail-under-lines 95` (or the actual baseline rounded down) so the
  CI gate is honest. If baseline is lower, ship `scripts/coverage.sh`
  as-is and record the gap as a v0.2 follow-up — do NOT block v0.1.0 on a
  coverage gate the codebase can't yet meet.

**Out of scope:** writing new tests to lift coverage. P just records the
truth; lifting it is a separate leaf.

## Estimation

**Complexity:** EASY. Tool install + script + two transcript writes.

## Speck Reference

[`docs/specks/SDD-TASK-007-P-audit-and-coverage-baseline.md`](../specks/SDD-TASK-007-P-audit-and-coverage-baseline.md)

## Subagent Spawn Log

| Time (UTC)         | Role | Purpose                                  | Status   | Notes |
|--------------------|------|------------------------------------------|----------|-------|
| 2026-06-03 | BA   | Refine + author Speck                    | SUCCESS  | Speck written; 4 ACs + 3 SEC ACs; BA scope respected (docs/ only) |
| 2026-06-03 | DEV  | Implement P per Speck                    | SUCCESS  | cargo-audit 0.22.1 installed in base stage; audit verdict = 2 informational warnings (RUSTSEC-2024-0436 paste unmaintained, RUSTSEC-2026-0002 lru unsound — both transitive via ratatui 0.29.0, flagged in Compliance Notes for TASK-007-O gate); coverage baseline = 74.06% lines (rounded down: 74%); below 95% threshold so no `--fail-under-lines` gate added, gap logged as v0.2 follow-up; pre-DONE gate fmt/clippy/test all green (97+6+3 = 106 tests pass) |

## Compliance Notes — §9.5 cross-agent attack review

Skipped per session-wide rule (no second-agent vendor reachable; gemini/gpt
cut off per user 2026-05-06). LIGHT pass would normally fire after the Speck;
FULL pass after impl. Both recorded as skipped here.
