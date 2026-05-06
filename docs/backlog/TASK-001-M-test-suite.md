# TASK-001-M — Test suite (unit + integration + TUI snapshot)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
Pre-merge baseline security gate: PASS. v0.1.0 simplifications (deferred to v0.2): TUI snapshot harness; cargo-llvm-cov coverage measurement (both recorded in spec.md §9). 67 tests pass total: 61 unit + 6 integration. Speck: `docs/specks/SDD-TASK-001-M-test-suite.md`. Implementation: `tests/integration_test.rs` plus per-module `#[cfg(test)] mod tests`.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
**Scope:** lawbook §6 + §7 compliance — unit, integration, and TUI snapshot tests, all running container-first per §8.2. 100% coverage of the lib crate, positive + negative + security at each layer (`[SEC]` labelled).
**Non-goals:** Playwright (web UI tool — N/A here per §0.3 "apply only what exists").
**Impacted UCs:** all.
**Depends on:** TASK-001-E, F, G, H, I.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE | ESTIMATION | DONE | DECOMPOSITION | N/A | DESIGN | DONE (ADR §6) | TESTING | PENDING | POST_AUDIT | PENDING |

## Estimation
**Complexity:** ABOVE_EASY  **Justification:** TUI snapshot harness + ephemeral filesystem fixtures + cross-platform path quirks.

## Acceptance (pointers)
- AC-1: 100% lines/branches/functions on lib (excluding `main.rs` glue).
- AC-2: integration tests build a fixture filesystem under `./.dev-temp/` and assert scan results.
- AC-3: TUI snapshot tests render each screen state to a virtual terminal buffer; baselines under `docs/ui-baselines/UC-###/<step>.txt` with matching color metadata.
- AC-4: `[SEC]` cases for null-byte filenames, very deep trees (50 000+ depth), symlink loops, hardlink chains, oversize CLI args, path traversal in `--exclude` patterns.
- AC-5: all stages run via `scripts/ci.sh` in fail-fast order.
- SEC-1: no test relies on files outside `./.dev-temp/`.
- SEC-2: no test makes network calls.

## Speck Reference
`docs/specks/SDD-TASK-001-M-test-suite.md` (to be authored).
