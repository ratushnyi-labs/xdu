# SDD-TASK-007-N8b — Add aarch64-pc-windows-msvc to the matrix

**Backlog item:** TASK-007-N8b
**Impacted UCs:** none (build pipeline only)

## Scope
- `Dockerfile`:
  - `rustup target add aarch64-pc-windows-msvc` (alongside the x86_64 one from N8a).
  - Populate aarch64 SDK via `xwin --accept-license splat --output /xwin --arch x86_64 --arch aarch64` (single splat with both arches) OR a second `xwin … --arch aarch64` invocation — whichever the xwin CLI supports cleanly. Document the chosen form in a Dockerfile comment.
- `scripts/build.sh`: default `TARGETS` list gains `aarch64-pc-windows-msvc`.

## ACs
- AC-1: Image rebuild succeeds with both arches splatted; `/xwin/sdk/lib/{x86_64,aarch64}` both present.
- AC-2: `TARGETS="aarch64-pc-windows-msvc" scripts/build.sh` produces `dist/aarch64-pc-windows-msvc/xdu.exe` linked against ARM64 MSVC CRT (verifiable via `file` reporting "PE32+ executable for MS Windows, ARM64").
- AC-3: Binary size within ceiling (same threshold as x86_64-msvc).

## SEC ACs
- SEC-1: No new tooling beyond what N8a introduced.
- SEC-2: Identical license-acceptance posture as N8a (xwin `--accept-license`).
- SEC-3: §8.2 host isolation preserved.

## §9.5 cross-agent attack review
Skipped per session-wide rule (no second-agent vendor reachable). See parent TASK-007.
