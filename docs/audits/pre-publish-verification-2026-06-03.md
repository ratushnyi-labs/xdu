# Pre-publish verification gate — 2026-06-03

**Date:** 2026-06-03 (UTC)
**Command run:** `docker compose run --rm -T test sh -c '...'` chaining
   `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked --
   -D warnings`, `cargo test --all --locked`, and `cargo audit` (single
   `set -eu` shell, fail-fast).
**Container toolchain (per `Dockerfile` `base` stage):** `rust:1.95-bookworm`
   (`cargo 1.95.0 (f2d3ce0bd 2026-03-21)` / `rustc 1.95.0 (59807616e
   2026-04-14)`) + `cargo-llvm-cov` + `cargo-audit 0.22.1` (advisory-db
   1105 advisories loaded).
**Host isolation:** all commands executed inside the `xdu-test` container
   per lawbook §8.2; no host-side `cargo`/`rustc` invocation.
**Backlog:** [TASK-007-O](../backlog/TASK-007-O-final-publish.md)
**Speck:** [SDD-TASK-007-O](../specks/SDD-TASK-007-O-final-publish.md)
   (Step 1 of 7; this transcript covers AC-1 only — AC-2..AC-8 are
   downstream)

## Gate

### `cargo fmt --all -- --check` — PASS

Verdict: **PASS** (clean — exit 0, no diff emitted, no output beyond the
echoed header).

```
=== cargo fmt --check ===
```

### `cargo clippy --all-targets --locked -- -D warnings` — PASS

Verdict: **PASS** (clean — exit 0, no warnings, no diagnostics).

```
=== cargo clippy ===
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.33s
```

(Build artefacts were already warm from prior wave verifications; clippy
short-circuits to a `Finished` line when the cached check graph is up to
date.)

### `cargo test --all --locked` — PASS (106/106)

Verdict: **PASS** — 97 unit + 6 integration + 3 snapshot = **106 passed,
0 failed**, exit 0.

```
test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
   (unittests src/lib.rs)

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   (unittests src/main.rs — bin entrypoint, no #[test])

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   (tests/integration_test.rs)

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
   (tests/snapshot_test.rs)

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   (Doc-tests xdu — none defined)
```

Matches the steady-state count quoted in `SDD-TASK-007-O` AC-1 and the
prior wave-5/wave-6 verification transcripts.

### `cargo audit` — PASS (2 known informational warnings, exit 0)

Verdict: **PASS** — exit 0, 2 surviving informational warnings (no
vulnerability-class advisories). Advisory-db count rose from 1103
(P's run) to **1105**; the 2 net-new advisories did not match any crate
in `Cargo.lock`. The 2 surviving warnings are byte-identical to P's
`dependency-audit-2026-06-03.md` capture.

```
    Scanning Cargo.lock for vulnerabilities (70 crate dependencies)
Crate:     paste
Version:   1.0.15
Warning:   unmaintained
Title:     paste - no longer maintained
Date:      2024-10-07
ID:        RUSTSEC-2024-0436
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0436
Dependency tree:
paste 1.0.15
└── ratatui 0.29.0
    └── xdu 0.1.0

Crate:     lru
Version:   0.12.5
Warning:   unsound
Title:     `IterMut` violates Stacked Borrows by invalidating internal pointer
Date:      2026-01-07
ID:        RUSTSEC-2026-0002
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0002
Dependency tree:
lru 0.12.5
└── ratatui 0.29.0
    └── xdu 0.1.0

warning: 2 allowed warnings found
```

## Advisory verdict

Two surviving informational advisories — **identical IDs, versions, and
dependency paths** to P's audit:

| ID                 | Crate         | Kind          | Reaches xdu via         | Recommendation (per P)             |
|--------------------|---------------|---------------|-------------------------|------------------------------------|
| RUSTSEC-2024-0436  | paste 1.0.15  | unmaintained  | ratatui 0.29.0 → paste  | ACCEPT for v0.1.0 (transitive)     |
| RUSTSEC-2026-0002  | lru 0.12.5    | unsound       | ratatui 0.29.0 → lru    | ACCEPT for v0.1.0 (transitive)     |

Full per-advisory rationale (reachability, exploitability, mitigation
path) lives in [`docs/audits/dependency-audit-2026-06-03.md`](dependency-audit-2026-06-03.md).
**These are NOT marked as approved in this transcript** — per Speck AC-2,
explicit user approval (via supervisor `AskUserQuestion`) is required
before Step 2 of TASK-007-O fires. This DEV-leaf only confirms that the
advisory state has not regressed since P landed.

No vulnerability-class (`cargo audit` non-zero exit) findings. No new
RUSTSEC IDs against xdu's `Cargo.lock`.

## Cross-build smoke

The full 6-target cross-build (`aarch64-apple-darwin`,
`aarch64-unknown-linux-gnu`, `aarch64-pc-windows-msvc`,
`x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`,
`x86_64-pc-windows-msvc`) was verified end-to-end on 2026-06-03 in
[`docs/audits/cross-build-2026-06-03-after-bug005-fix.md`](cross-build-2026-06-03-after-bug005-fix.md)
as part of the BUG-005 fix verification.

`git status` at this transcript's run-time shows no modifications to
`Dockerfile`, `scripts/`, `src/`, `Cargo.toml`, or `Cargo.lock` since
that audit landed, so the cross-build smoke remains valid. A re-run is
**OPTIONAL** per Speck AC-1; AC-1 passes on the strength of the 4 gates
above alone.

## Overall

**READY-FOR-USER-APPROVAL.**

Speck AC-1 satisfied — all 4 pre-publish DEV gates green inside the
Docker test container; advisory state byte-identical to P's recorded
baseline; cross-build smoke unchanged since the BUG-005 verification.

Next step per Speck §Scope is Step 2 (user-approval gates / governance):
supervisor surfaces the 2 informational advisories + the CHANGELOG flip
for explicit user approval. That is a supervisor-inline action, not a
DEV subagent action.
