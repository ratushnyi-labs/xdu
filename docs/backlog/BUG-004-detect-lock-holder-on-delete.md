# BUG-004 — detect lock holders when delete fails on a locked file

**Status:** DONE  **Type:** BUG
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
Pre-merge baseline security gate: PASS — Restart Manager calls are enumeration-only (no `TerminateProcess`, no `NtClose`); each `unsafe` block carries a `// SAFETY:` comment per ADR §7; `#[link(name = "rstrtmgr")]` adds the kernel32-adjacent system library link only on Windows; `RmStartSession` paired with `RmEndSession` via RAII guard on every exit path. 91 tests pass (85 unit + 6 integration). Multi-target rebuild within budget (~660 / 660 / 519 KiB).
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> actually the file i tried to delete is blocked by another process. tool shpould find out it and ask me for unblocking

## Refined Description

When a delete cannot complete because another process holds the file open, xdu should
identify the holding processes (by name and PID) and show them in a modal so the user
can close them, then retry the delete.

**Detection trigger:** the lock-holder probe runs whenever `perform_delete` either
- returns `Err` with a Windows in-use code (`5` `ACCESS_DENIED`,
  `32` `SHARING_VIOLATION`, `33` `LOCK_VIOLATION`), or
- returns `Ok` but `path.exists()` after the syscall (the BUG-003 read-back path —
  Windows can queue a delete while a handle remains open).

**Holder enumeration (Windows):** raw FFI to **Windows Restart Manager**
(`rstrtmgr.dll`):
1. `RmStartSession` opens a session.
2. `RmRegisterResources` registers the locked file path.
3. `RmGetList` returns an array of `RM_PROCESS_INFO` records — each carries the holding
   process's PID, app name, and start time.
4. `RmEndSession` closes the session.
This is the same API Windows installers use to list "the following programs are using
files that need to be updated". No elevated privileges required.

**Modal UI:** new `LockPrompt` rendered when the probe finds at least one holder. Shows:
- The file path (canonical).
- A short list of holders (`PID — name`), capped at the most relevant 8.
- Two buttons: `[ Retry ]` (default focus) and `[ Cancel ]`.
- Instructions: "Close the listed apps in your task manager, then press Retry.
  xdu does not terminate processes for you."

**Retry flow:** pressing `[ Retry ]` calls `perform_delete` again on the same `NodeId`.
The flow is idempotent — if the file is now free, it deletes; if still locked, the
modal re-opens with the (possibly updated) holder list.

**Linux / macOS / BSD** (`LATER-008b`): no lock-holder enumeration in v0.1.x. Falls back
to the existing format_delete_error message ("close the holding app and try again"). The
`/proc/*/fd` walk on Linux and `lsof`-equivalent on macOS are recorded for v0.2.

**Out of scope (recorded as `LATER-008c`):** automatic process termination
(`OpenProcess` + `TerminateProcess`) and forcible handle closure (`NtClose` on a
duplicated foreign handle). Both are dangerous (foreign-app data loss) and are NOT in
v0.1 — Sysinternals' `handle.exe` shows the same restraint with strong warnings.

**Impacted UCs:** UC-008 (delete).
**Impacted BR/WF:** BR-001 read-only invariant preserved (Restart Manager calls only
read state — no write or termination).

## Estimation

**Complexity:** ABOVE_EASY  **Justification:** new platform module
(`src/lock_detect.rs`) with Windows FFI + struct definitions, new `LockPrompt` modal,
detection-and-retry plumbing in `perform_delete`. No new crate dependency.

## Speck Reference
[`docs/specks/SDD-BUG-004-lock-holder-detection.md`](../specks/SDD-BUG-004-lock-holder-detection.md)
