# TASK-001-D — Docker build pipeline (multi-stage, multi-target)

**Status:** READY  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** PLAN_READY
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
`docs/specks/SDD-TASK-001-D-build-pipeline.md` (to be authored at start of D).
