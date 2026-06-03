# SDD-TASK-007-P — cargo-audit install + clean pass; llvm-cov baseline

**Backlog item:** TASK-007-P
**Impacted UCs:** none (pre-publish hardening; tooling + baselines only)

## Scope
- `Dockerfile` `base` stage: install `cargo-audit` via
  `cargo install --locked cargo-audit`. Pinned version recorded in a comment
  adjacent to the install line (mirrors the cargo-llvm-cov pattern from N6/G).
- New file `scripts/audit.sh`: thin wrapper that runs
  `docker compose run --rm test sh -c 'cargo audit'`. Shape mirrors
  `scripts/coverage.sh`.
- Run `scripts/audit.sh` once. Capture its output as a transcript at
  `docs/audits/dependency-audit-2026-06-03.md`.
- Run `scripts/coverage.sh` once. Capture its summary as a transcript at
  `docs/audits/coverage-baseline-2026-06-03.md`, recording the workspace
  line-coverage percentage.
- Coverage gate decision:
  - If baseline ≥95%: update `scripts/coverage.sh` to pass
    `--fail-under-lines <baseline rounded down>` to `cargo llvm-cov`.
  - If baseline <95%: leave `scripts/coverage.sh` un-gated and record the
    gap as a v0.2 follow-up note in the coverage-baseline transcript.

**Out of scope:** writing new tests to lift coverage. P records the truth;
lifting it is a separate leaf.

## ACs
- AC-1: `docker compose run --rm test sh -c 'cargo audit --version'` succeeds.
- AC-2: `cargo audit` exits clean with zero advisories, OR every surviving
  advisory is listed in `docs/audits/dependency-audit-2026-06-03.md` under a
  `## Compliance Notes` section with per-advisory rationale and explicit
  user approval.
- AC-3: Both `docs/audits/dependency-audit-2026-06-03.md` and
  `docs/audits/coverage-baseline-2026-06-03.md` exist with non-empty content
  (real captured output, not placeholders).
- AC-4: `scripts/coverage.sh` either carries `--fail-under-lines <N>` where
  `<N>` matches the baseline recorded in the coverage transcript (rounded
  down), or carries no `--fail-under-lines` flag AND the coverage transcript
  records the gap as a v0.2 follow-up.

## SEC ACs
- SEC-1: No new shipping crate dependency added. cargo-audit is dev tooling
  installed only in the Dockerfile `base` stage.
- SEC-2: §8.2 host isolation preserved — `cargo audit` runs inside the
  `test` container via `docker compose run --rm test`; never on the host.
- SEC-3: Audit transcripts contain no secrets, credentials, tokens, or
  absolute host paths beyond the project root (`/work` inside container,
  repo-relative paths only on host capture).

## §9.5 cross-agent attack review
Skipped per session-wide rule.
