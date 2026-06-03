# Coverage baseline — 2026-06-03

**Date:** 2026-06-03 (UTC)
**Tool:** `cargo-llvm-cov 0.8.7`
**Rust toolchain:** `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` / `rustc 1.95.0 (59807616e 2026-04-14)`
**Command:** `scripts/coverage.sh` →
`docker compose run --rm -T test sh -c 'cargo llvm-cov --workspace --locked --lcov --output-path coverage.lcov && cargo llvm-cov --workspace --locked --summary-only'`
**Container:** `xdu-test` (Dockerfile `base` stage; lawbook §8.2 host isolation preserved)
**Backlog:** [TASK-007-P](../backlog/TASK-007-P-audit-and-coverage-baseline.md)
**Speck:** [SDD-TASK-007-P](../specks/SDD-TASK-007-P-audit-and-coverage-baseline.md)
**Test result:** 97 unit + 6 integration + 3 snapshot = **106 tests, 0 failed**.
**lcov artifact:** `coverage.lcov`, 90,726 bytes.

## Captured `--summary-only` table

```
Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
disk_space.rs                     108                 5    95.37%           7                 0   100.00%          56                 4    92.86%           0                 0         -
drives_win.rs                      42                 1    97.62%           5                 0   100.00%          28                 1    96.43%           0                 0         -
export.rs                         395                38    90.38%          21                 1    95.24%         192                 9    95.31%           0                 0         -
format.rs                         315                 1    99.68%          30                 0   100.00%         190                 1    99.47%           0                 0         -
lock_detect.rs                    169                 8    95.27%          14                 1    92.86%         110                 7    93.64%           0                 0         -
main.rs                           199               199     0.00%           4                 4     0.00%         129               129     0.00%           0                 0         -
scanner.rs                        755               103    86.36%          38                 4    89.47%         425                70    83.53%           0                 0         -
trash.rs                          214                36    83.18%          12                 2    83.33%         132                20    84.85%           0                 0         -
tree.rs                           943                20    97.88%          46                 2    95.65%         405                12    97.04%           0                 0         -
tui.rs                           2182               866    60.31%         100                30    70.00%        1190               488    58.99%           0                 0         -
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                            5322              1277    76.01%         277                44    84.12%        2857               741    74.06%           0                 0         -
```

## Baseline

**Workspace line coverage: 74.06% (1 line-coverage figure of merit)**

Rounded-down integer baseline: **74%**.

Per-file breakdown (descending):
- `format.rs` 99.47% — near-perfect; 1 line missed
- `tree.rs` 97.04%
- `drives_win.rs` 96.43%
- `export.rs` 95.31%
- `disk_space.rs` 92.86%
- `lock_detect.rs` 93.64%
- `trash.rs` 84.85%
- `scanner.rs` 83.53%
- `tui.rs` 58.99% — drags the workspace; large render/event-loop surface
  not yet covered by snapshot tests
- `main.rs` 0.00% — the CLI binary entry point. `cargo llvm-cov` runs
  unit/integration tests against the library; `main.rs` is exercised only
  by external invocations, not by the unit harness. This is the canonical
  "binary entry point gap" — addressing it requires either a thin
  `lib_main()` extraction or explicit binary-level integration tests.

`main.rs` accounts for 129/741 missed lines = 17.4% of the gap. `tui.rs`
accounts for 488/741 = 65.9% of the gap. Together those two files explain
~83% of the coverage shortfall.

## Gate decision

**Decision: NO `--fail-under-lines` gate added; recording the gap as a v0.2 follow-up.**

Speck AC-4 specifies the gate is added only when baseline ≥95%. Baseline is
74.06%, which is well below the threshold; per the Speck and the backlog
"Refined Description" line 38, the v0.1.0 release ships `scripts/coverage.sh`
without a fail-under flag rather than blocking publication on a gate the
codebase cannot currently meet.

`scripts/coverage.sh` is therefore unchanged by this leaf.

## v0.2 follow-up — close the coverage gap to ≥95%

Logged for future work (not part of this leaf per Speck "Out of scope"):

1. **Cover `tui.rs` event-loop branches** (≈58.99% → ≥90%). Expand the
   snapshot/golden-frame harness in `tests/snapshot_test.rs` to drive
   keymap + redraw paths under simulated terminal events. Estimated 25-30
   new test cases.
2. **Cover `main.rs` CLI entry** (0% → ≥80%). Extract a `pub fn run(...)
   -> Result<i32>` wrapper from `main()` and call it from an integration
   test, leaving `fn main()` as a 3-line shim. The shim itself can stay
   uncovered if llvm-cov's `--ignore-filename-regex` is pointed at it.
3. **Lift `scanner.rs` and `trash.rs`** (~84%) by covering the cross-platform
   error paths that only fire on the non-host OS — those branches are
   currently dead code at coverage time on a Linux container.
4. Once total ≥95%, add `--fail-under-lines 95` to `scripts/coverage.sh`
   and to the CI workflow in `.github/workflows/`.

This is a separate v0.2 leaf, not blocking v0.1.0.
