# TASK-007-L — macOS / *BSD free-space via statvfs(3)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-21  **Updated:** 2026-05-21

## Refined Description
Resolves `LATER-009` from `docs/spec.md` §9 and finishes the cross-platform
free-space story alongside TASK-004 (Linux) and `GetDiskFreeSpaceExW` (Windows).

`src/disk_space.rs` currently gates the `statvfs(3)` FFI behind
`#[cfg(target_os = "linux")]`. macOS and the BSDs all expose the POSIX `statvfs`
call with the same minimum-required field set (`f_frsize`, `f_bavail`,
`f_blocks`). Widen the cfg-gate to `unix` (sans Linux-specific extensions) so
the function returns real values on macOS Intel + Apple Silicon and on FreeBSD
when those platforms are exercised.

**Approach:** Use a single `extern "C"` declaration that matches the POSIX-portable
subset of the struct (we only need `f_bsize`/`f_frsize`, `f_blocks`, `f_bavail`).
The struct layout varies by platform but the relevant prefix is stable across
glibc / musl / Darwin / FreeBSD per POSIX.1-2008.

**Behaviour change:** the free-space footer goes from showing "—" on macOS/BSD to
showing the real free-space figure.

## Estimation
**Complexity:** EASY. cfg widening + struct field audit on the existing FFI.

## Speck Reference
[`docs/specks/SDD-TASK-007-L-macos-bsd-free-space.md`](../specks/SDD-TASK-007-L-macos-bsd-free-space.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
