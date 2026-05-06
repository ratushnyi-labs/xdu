# SDD-TASK-001-M — Integration test suite

**Backlog item:** TASK-001-M
**Impacted UCs:** all (smoke coverage)

## Scope
Add `tests/integration_test.rs` exercising the public API of every module
end-to-end on a real ephemeral filesystem in `std::env::temp_dir()`:
scanner → tree → format → export round trip.

## v0.1.0 simplifications (recorded in `docs/spec.md` §9 known limitations)
- **TUI snapshot harness** (`docs/ui-baselines/UC-###/<step>.txt`) is **deferred** to v0.2 (LATER-004). v0.1.0 verifies the TUI by:
  1. Compilation and clippy on every wave.
  2. Manual visual check of `xdu` against this repo at release.
  3. Per-module unit tests for non-rendering pieces (`truncate_left`, `sanitize`, `display_name`, `sort_label`).
- **Coverage measurement via `cargo-llvm-cov`** is deferred (LATER-006). v0.1.0 reaches "100% functions" by inspection — every public function has at least one test in either its module's unit tests or this integration suite.

## ACs
- AC-1: `cargo test --locked` (inside the dev container) compiles and runs all unit + integration tests with zero failures.
- AC-2: end-to-end scan → format → export round trip on a constructed fixture verifies sums, schema, and exported entry names.
- AC-3: scan returns `Err` (not panic) on invalid roots.
- AC-4: empty directory scans cleanly to a single root node.

## SEC ACs
- SEC-1: tests use a private `TempDir` rooted in `std::env::temp_dir()` — no `./.dev-data` pollution.
- SEC-2: no network calls.
- SEC-3: no `Command::spawn`.
- SEC-4: temp dirs cleaned via `Drop`.
