# TASK-006 — force-close foreign handles to delete a locked file

**Status:** DONE  **Type:** TASK
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
Pre-merge baseline security gate: PASS — all FFI in small `unsafe` blocks with `// SAFETY:` comments per ADR §7; `RmStartSession`/`RmEndSession` paired via RAII guard; `OpenProcess` results cached and unconditionally closed at end via the cache map; `NtQueryObject` runs in a worker thread with a 100 ms timeout to mitigate the known sync-I/O hang; raw kernel handle transported across threads as `usize` (Send + Copy). Windows toolchain links `rstrtmgr.dll`, `ntdll.dll`, `kernel32.dll`. Linux/macOS path returns `0`. 93 tests pass (87 unit + 6 integration). Multi-target rebuild within budget (663 / 663 / 548 KiB).
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> xdu does NOT terminate processes - no tet=rmination is needed. just unblock the handler of the file and close it on OS level

## Refined Description

When the lock-prompt (BUG-004) lists processes holding a file open, xdu should be able
to **forcibly close just the offending handle in the foreign process** — without
terminating the process — and then retry the delete. This is exactly what
`Sysinternals handle.exe -c <handle>` does.

**Mechanism (Windows):**
1. Enumerate every handle in the system via `NtQuerySystemInformation(SystemExtendedHandleInformation)`.
2. Filter to the PIDs we already know hold the file (from the Restart Manager pass in
   `lock_detect::holders`).
3. For each candidate handle:
   a. `OpenProcess(PROCESS_DUP_HANDLE, false, pid)` to get a process handle.
   b. `DuplicateHandle(... DUPLICATE_SAME_ACCESS)` for read-only inspection.
   c. `NtQueryObject(ObjectNameInformation)` to read the handle's NT path.
      Run this in a worker thread with a 100 ms timeout — synchronous I/O
      handles can hang `NtQueryObject`. Drop & move on if timeout fires.
   d. Compare the NT path against the canonical NT-path form of our target
      file (drive letter resolved via `QueryDosDeviceW`).
   e. If it matches: `DuplicateHandle(... DUPLICATE_CLOSE_SOURCE)` — closes
      the handle in the foreign process while duplicating it locally; then
      `CloseHandle` the local copy.
4. After force-close, retry `perform_delete` automatically.

**No process termination, no `NtClose` on a foreign-process handle directly** — the
`DUPLICATE_CLOSE_SOURCE` form is the OS-sanctioned way to close a foreign handle.

**Linux / macOS / BSD (`LATER-008b`):** not implemented in v0.1.x. The lock prompt's
"Force close" button is hidden / no-op.

## Spec impact (handled before code per §3.5)

`docs/spec.md` §4 BR-001 — the "default read-only" wording stays; the only mutation paths
remain `d` + confirmation (file delete) and now `f` + confirmation (force-close
foreign handles, Windows only). New UC-008 sub-section documents the force-close flow.

## Risks (recorded explicitly)

- **Foreign apps may lose unsaved data.** When xdu force-closes a file handle in
  Notepad, Word, etc., the next time that app writes through that (now-invalid)
  handle, the write fails and any in-memory edits since the last save are at risk.
- **Privileges.** `OpenProcess(PROCESS_DUP_HANDLE)` on a foreign process succeeds when
  the caller has the same user identity OR `SeDebugPrivilege`. Standard non-admin
  users can force-close handles in their *own* processes; cross-user / system
  processes fall through silently (the handle is skipped).
- **Race.** Between holder enumeration and force-close, the handle may be closed
  legitimately or duplicated elsewhere. Idempotent re-probe after the action handles
  this.

These risks are surfaced in the force-confirm modal (default focus `[ No ]`,
explicit "unsaved data may be lost" warning). Per the user's request, no automatic
process termination path is added.

**Impacted UCs:** UC-008.
**Impacted BR/WF:** BR-001 amended (force-close added as second mutation path).

## Estimation

**Complexity:** ABOVE_EASY  **Justification:** ~200 LOC of `ntdll` + `kernel32` FFI,
worker-thread timeout for `NtQueryObject`, NT-path normalization, force-confirm sub-modal,
spec amendment. No new crate dep.

## Speck Reference
[`docs/specks/SDD-TASK-006-force-close-handles.md`](../specks/SDD-TASK-006-force-close-handles.md)
