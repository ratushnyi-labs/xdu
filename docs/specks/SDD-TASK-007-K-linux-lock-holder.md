# SDD-TASK-007-K — Linux lock-holder enumeration via /proc/*/fd

**Backlog item:** TASK-007-K
**Impacted UCs:** UC-008 (delete with confirmation modal), specifically the
`is-locked` follow-on path that was previously Windows-only via BUG-004.

## Scope
- `src/lock_detect.rs`:
  - Replace the catch-all `#[cfg(not(windows))]` arm with two arms:
    `#[cfg(target_os = "linux")]` (real implementation) and
    `#[cfg(all(unix, not(target_os = "linux")))]` (stub returning empty).
  - Linux implementation:
    - Canonicalize the input path.
    - Iterate `/proc/<digits>/fd/`, readlink each entry, compare to the
      canonical target.
    - For matched pids, read `/proc/<pid>/comm` for the command name.
    - Permission-denied on other users' processes is normal — treat
      unreadable `fd/` as "no match" rather than erroring.
  - `force_close_handles_for_path` stays Windows-only; the Linux variant
    returns 0 since Linux's POSIX unlink succeeds even with open holders.

## ACs
- AC-1: `cargo test --lib lock_detect` includes a Linux-only test
  (`holders_finds_self_when_file_is_kept_open`) that opens a temp file,
  calls `holders(&path)`, asserts the current pid is in the result.
- AC-2: `holders(&path_that_does_not_exist)` returns an empty `Vec` on
  Linux (existing cross-platform test).
- AC-3: `force_close_handles_for_path(&path, &[])` returns 0 on Linux
  (it is intentionally inert there).

## SEC ACs
- SEC-1: No new `unsafe` blocks. The Linux impl uses safe `std::fs` only.
- SEC-2: Reads are restricted to `/proc/*` — well-defined kernel surface;
  no PID-spoofing risk because we use the directory name verbatim.
- SEC-3: Permission-denied handling is silent-skip; no information leak
  from one user to another beyond what is already visible via `ps`.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
