# TASK-001-C — Repo scaffold

**Status:** IN PROGRESS  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request
> Inherited from TASK-001.

## Refined Description
**Scope:** create the minimal repository scaffold required for TASK-001-D…N to work against:
LICENSE (MIT), `.gitignore`, `.dockerignore`, `Cargo.toml` with `release-min` profile,
`rust-toolchain.toml`, `src/main.rs` stub that compiles and prints version, `README.md`
skeleton, `.editorconfig`, `git init` + initial commit.

**Non-goals:** Dockerfile (TASK-001-D), source modules (TASK-001-E…L), full README (TASK-001-N), tests (TASK-001-M).

**Impacted UCs:** none directly (infrastructure).
**Impacted BR/WF:** —
**Depends on:** TASK-001-A, TASK-001-B (so Cargo profile + module shape match ADR).

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE (parent) |
| ESTIMATION | DONE |
| DECOMPOSITION | N/A (leaf) |
| DESIGN | DONE |
| FRONTEND | N/A (no UI in scaffold) |
| BACKEND | N/A (stub only) |
| TESTING | N/A (no tests yet — added in TASK-001-M) |
| POST_AUDIT | PENDING |

## Estimation
**Complexity:** EASY  **Justification:** ~9 small static files + git init.

## Acceptance (pointers; full ACs in Speck)
- AC-1: scaffold compiles (target: builds successfully when invoked under TASK-001-D's Docker pipeline).
- AC-2: MIT LICENSE present; year 2026; no agent attribution.
- AC-3: `Cargo.toml` defines `release-min` profile per ADR-001.
- AC-4: `git init` performed; first commit references `TASK-001-C`.
- AC-5: no source modules / no Dockerfile / no tests written (out of scope).

## Speck Reference
[`docs/specks/SDD-TASK-001-C-repo-scaffold.md`](../specks/SDD-TASK-001-C-repo-scaffold.md)
