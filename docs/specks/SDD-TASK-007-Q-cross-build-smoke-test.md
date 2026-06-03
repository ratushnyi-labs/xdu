# SDD-TASK-007-Q — 6-target cross-build smoke-test

**Backlog item:** TASK-007-Q
**Impacted UCs:** none (pre-publish hardening; observational only)

## Scope
- Run `TARGETS="x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-msvc aarch64-pc-windows-msvc x86_64-apple-darwin aarch64-apple-darwin" scripts/build.sh` end-to-end inside Docker (per §8.2 host isolation). Capture stdout/stderr verbatim.
- New transcript `docs/audits/cross-build-2026-06-03.md` capturing, per target:
  - exact command invoked
  - `file(1)` output on the produced binary
  - `wc -c` byte size
  - pass/fail vs the 1.5 MiB ceiling (`SIZE_CEILING_BYTES=1572864`)
  - a final summary table (target, status, binary path, byte size, ceiling check)
- On per-target failure, the DEV agent creates a NEW INTAKE bug file under `docs/backlog/BUG-###-<short>.md` (per §1.2.13.5 mid-task-discovery), appends a one-line follow-up entry to TASK-007-Q's Subagent Spawn Log, and CONTINUES the smoke-test for the remaining targets. Do not abort the entire matrix on first failure.
- No source / Dockerfile / script / workflow edits expected. Q is observational; it records what the existing pipeline produces.

**Out of scope:** fixing whatever cross-build problem surfaces. That is a separate leaf, refined and estimated on its own merits.

## ACs
- AC-1: `scripts/build.sh` runs to completion for all 6 targets — each target is either green or recorded as failed in the transcript. No silent skips; the matrix is not aborted on first failure.
- AC-2: `docs/audits/cross-build-2026-06-03.md` exists with one section per target (command + `file(1)` + `wc -c` + ceiling check) and a final summary table covering all 6 targets (target, status, binary path, byte size, ceiling check).
- AC-3: Every per-target failure is mirrored by a NEW INTAKE bug file `docs/backlog/BUG-###-<short>.md` (per §1.2.13.5), with a matching one-line entry appended to TASK-007-Q's Subagent Spawn Log.
- AC-4: No edits made under `src/`, `Cargo.toml`, `Dockerfile`, `scripts/build.sh`, or `.github/workflows/*` as part of Q. Q is observational; any required fix is a separate leaf.

## SEC ACs
- SEC-1: All 6 builds run inside Docker per §8.2 host isolation; no `cargo build` invoked on the host.
- SEC-2: Transcript redacts absolute host paths beyond the project root — record container-side paths as `/src/...` and host-side artefacts as `dist/.../xdu` (or `xdu.exe`) repo-relative only.
- SEC-3: No advisory bypass on the size ceiling. If a build accidentally inflates a binary above the 1.5 MiB ceiling, AC-2 records the breach explicitly in the per-target section AND in the summary table. No silent ceiling waiver; any waiver requires a separate user-approved leaf.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
