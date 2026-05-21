# TASK-007-N8b — add aarch64-pc-windows-msvc

**Status:** DONE (Dockerfile + scripts wired; full cross-build verification deferred to TASK-007-O)  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
After N8a lands, add `aarch64-pc-windows-msvc` to the build matrix. Trivially additive:
- `Dockerfile` `rustup target add aarch64-pc-windows-msvc`.
- `xwin --accept-license splat --arch x86_64 --arch aarch64` (or repeated invocation per arch).
- `scripts/build.sh` default `TARGETS` gains the new triple.

## Estimation
**Complexity:** EASY. One-line additions to Dockerfile + scripts.

## Speck Reference
[`docs/specks/SDD-TASK-007-N8b-aarch64-windows-msvc.md`](../specks/SDD-TASK-007-N8b-aarch64-windows-msvc.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
