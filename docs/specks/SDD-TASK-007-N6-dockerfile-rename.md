# SDD-TASK-007-N6 — Dockerfile rename + output dir → dist/

**Backlog item:** TASK-007-N6
**Impacted UCs:** none (build pipeline only)

## Scope
- Rename file `Dockerfile.build` → `Dockerfile`.
- Update `docker-compose.yml` `dockerfile:` references (2 occurrences: `build` service + `dev` service).
- Update `scripts/build.sh`: change `mkdir -p out` → `mkdir -p dist`; change `bin="out/$t/xdu$ext"` → `bin="dist/$t/xdu$ext"`.
- Update `docker-compose.yml` volume mount: `./out:/out_host` → `./dist:/out_host` (keep container-side mount name `/out_host` since it is internal).
- Update README references from `out/` → `dist/`.
- Remove (or migrate any local user-side) `out/` directory contents — `dist/` already in `.gitignore` per N2.

## ACs
- AC-1: `Dockerfile.build` no longer exists; `Dockerfile` exists with identical content (modulo the rename itself).
- AC-2: `docker compose build dev` still succeeds.
- AC-3: `TARGETS="x86_64-unknown-linux-musl" scripts/build.sh` produces `dist/x86_64-unknown-linux-musl/xdu` (not `out/...`).
- AC-4: README contains no `out/` references; `dist/` is the documented artefact path.
- AC-5: `git status` shows `dist/` untracked (gitignored) — never committed.

## SEC ACs
- SEC-1: No code path changes; behaviour identical.
- SEC-2: No new tooling.
- SEC-3: §8.2 host isolation preserved — all build still runs in containers.

## §9.5 cross-agent attack review
Skipped per session-wide rule (no second-agent vendor reachable). See parent TASK-007.
