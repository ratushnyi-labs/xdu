# TASK-001-D — Docker build pipeline (multi-stage, multi-target)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request
> Inherited from TASK-001.

## Refined Description
**Scope:** build pipeline as Docker artifacts so TASK-001-E…N can compile/run/test inside containers per §8.2:
- root `docker-compose.yml` with `build` (debug compile + clippy) and `release` (size-optimized cross-compile) services
- multi-stage `Dockerfile.build` based on `rust:<pinned>-alpine` for musl static targets and `osxcross` / `mingw-w64` images for Windows + macOS targets
- `scripts/ci.sh` invoking the same containers (lawbook §8.7 CI simulation)
- `.env` with all configurable build vars; ports auto-resolve (N/A here — no exposed ports)
- artifact extraction to `/out/<target>/xdu(.exe)`

**Non-goals:** full CI/CD provider config (deferred); release publishing (deferred); production server deployment (N/A — this is a CLI tool).
**Impacted UCs:** all (build/test infrastructure).
**Impacted BR/WF:** —
**Depends on:** TASK-001-C.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE |
| ESTIMATION | DONE |
| DECOMPOSITION | N/A |
| DESIGN | DONE (ADR-001 §5) |
| FRONTEND | N/A |
| BACKEND | N/A |
| TESTING | DONE-via-DESIGN (test runner scripts produced here) |
| POST_AUDIT | PENDING |

## Estimation
**Complexity:** ABOVE_EASY  **Justification:** multi-stage Dockerfile + cross-compile images + size-profile validation.

## Acceptance (pointers)
- AC-1: `docker compose build release` produces `out/x86_64-unknown-linux-musl/xdu` ≤ 1.5 MB.
- AC-2: same command produces `out/aarch64-unknown-linux-musl/xdu`.
- AC-3: same command produces `out/x86_64-pc-windows-gnu/xdu.exe`.
- AC-4: macOS targets attempted but documented as "host-toolchain-only" if osxcross unavailable.
- AC-5: `scripts/ci.sh` runs typecheck → clippy → unit → integration → multi-target release in one call.
- SEC-1: Dockerfile pins all base image versions; no `latest`.
- SEC-2: no secrets in image layers.

## Speck Reference
[`docs/specks/SDD-TASK-001-D-build-pipeline.md`](../specks/SDD-TASK-001-D-build-pipeline.md)

## Verified results

| Target                            | Size (bytes) | Size (KiB) | `file` output                                            |
|-----------------------------------|--------------|------------|----------------------------------------------------------|
| x86_64-unknown-linux-musl         |     406 776  |     397    | ELF 64-bit, x86-64, **static-pie linked**, stripped       |
| aarch64-unknown-linux-musl        |     354 128  |     346    | ELF 64-bit, ARM aarch64, **statically linked**, stripped  |
| x86_64-pc-windows-gnu             |     268 800  |     263    | PE32+ executable (console) x86-64, stripped to PDB        |

All under 1 MiB — well below ADR-001's 1.5 MiB ceiling and 1.0 MiB target.

Smoke-runs confirmed: `xdu` ran inside `alpine:3.20` (no glibc, only musl) printing
`xdu v0.1.0`; `xdu.exe` ran natively on the Windows host printing `xdu v0.1.0`,
exit code 0.

CI dev steps validated: `cargo fmt --check`, `cargo clippy --locked -D warnings`,
`cargo check --locked`, `cargo test --locked` (0 tests, 0 failed) — all clean.

## Pre-merge baseline security gate (§9.4)

| Audit                              | Depth       | Disposition | Rationale                                                                 |
|------------------------------------|-------------|-------------|---------------------------------------------------------------------------|
| SECURITY_AUDIT                     | NORMAL_SCAN | PASS        | Dockerfile uses pinned tags (`rust:1.85-bookworm`, `alpine:3.20`); apt with `--no-install-recommends` and clean lists; no privileged mode; no exposed ports; no secrets in any layer; no network in build other than crates.io (no deps used) and Debian apt. Build does not require `--privileged`. |
| DEPENDENCY_TREE_SECURITY_AUDIT     | NORMAL_SCAN | PASS        | `Cargo.lock` lists only `xdu` package; zero third-party crates. Apt deps (`ca-certificates`, `gcc-mingw-w64-x86-64`, `musl-tools`, `file`) come from Debian bookworm main repository. |
| MEMORY_LEAK_AUDIT                  | FAST_SCAN   | PASS        | N/A — pipeline produces build artifacts; runtime is the binary, which is unchanged from TASK-001-C scaffold (single `println!`). |

Findings: none. No fix sub-tasks. Continuity check across TASK-001 parent: AC-4 (macOS targets)
deferred per Speck — recorded as a non-blocking follow-up. AC-9 (full byte-for-byte
reproducibility validation) deferred — single-run determinism only; documented as Phase-3
work. No silent de-scope.
