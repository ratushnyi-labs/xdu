# SDD-TASK-006 — force-close foreign handles via DUPLICATE_CLOSE_SOURCE

**Backlog item:** TASK-006
**Impacted UCs:** UC-008

## Spec impact (per §3.5, handled before code)

`docs/spec.md` BR-001 amended: the second mutation path is the explicit `f` key
inside the lock-prompt (BUG-004), gated behind a force-confirm modal whose default
focus is `[ No ]`. Forcibly closes only specific file handles in foreign processes
via Windows `DuplicateHandle(... DUPLICATE_CLOSE_SOURCE)`. No process termination.

## Scope

### `src/lock_detect.rs`

- New public function:

  ```rust
  pub fn force_close_handles_for_path(path: &Path, pids: &[u32]) -> usize;
  ```

  Returns the number of handles forcibly closed. `pids` filter limits the scan to
  the holders identified by `holders()` — keeps the system-handle enumeration
  cheap and avoids touching unrelated processes.

- Windows implementation:
  - `NtQuerySystemInformation(SystemExtendedHandleInformation = 64)` with the
    grow-buffer-on-`STATUS_INFO_LENGTH_MISMATCH` pattern.
  - `QueryDosDeviceW` to convert the canonical Win32 path's drive letter to its
    NT device prefix.
  - Per-candidate handle: `OpenProcess(PROCESS_DUP_HANDLE)` →
    `DuplicateHandle(DUPLICATE_SAME_ACCESS)` for inspection →
    `NtQueryObject(ObjectNameInformation)` in a 100 ms-timeout worker thread →
    if name matches: `DuplicateHandle(DUPLICATE_CLOSE_SOURCE)`.
  - Skips `pid == GetCurrentProcessId()` to avoid closing our own handles.
  - Skips kernel/protected processes (their `OpenProcess` simply fails).
  - All FFI blocks carry `// SAFETY:` comments per ADR §7.

- Non-Windows: returns `0`.

### `src/tui.rs`

- New struct `ForceConfirmPrompt { target: NodeId, file_path: PathBuf, holders: Vec<LockHolder>, yes_focused: bool }` with default `yes_focused = false`.
- New `App::force_prompt: Option<ForceConfirmPrompt>`.
- New keybinding `f` / `F` while the lock-prompt is open: closes the lock-prompt
  and opens the force-confirm modal pre-populated with the same holders.
- `render_force_prompt`: red-tinted modal explaining what will happen, listing
  the holders, and showing `[ Yes ]` `[ No ]` buttons (`No` highlighted).
- `handle_force_prompt_key`:
  - `y` / `Y`: invoke `lock_detect::force_close_handles_for_path` then re-invoke
    `perform_delete` on the target.
  - `n` / `N` / `Esc` / `q`: cancel.
  - `Tab` / arrows / `h` / `l`: toggle focus.
  - `Enter`: activate focused button.
- After force-close runs, the resulting status reports how many handles were
  closed AND whether the subsequent delete succeeded.

## ACs

- AC-1: When the lock-prompt is open and `f` / `F` is pressed, the force-confirm
  modal opens with default focus `[ No ]`.
- AC-2: Choosing No / Esc / q from the force-confirm cancels and returns to a
  status message; no FFI is invoked.
- AC-3: Choosing Yes invokes `force_close_handles_for_path` with the holder PIDs,
  then re-invokes `perform_delete` on the target.
- AC-4: The post-action status reports the number of handles closed.
- AC-5: On Linux / macOS / BSD, `force_close_handles_for_path` returns 0 and the
  delete still fails — the existing flow is unchanged.
- AC-6: BR-001 amendment recorded in spec before code.
- AC-7: `OpenProcess` failures for protected processes are silently skipped — the
  function does not panic and reports the count of *successfully* closed handles.

## SEC ACs

- SEC-1: All FFI blocks carry `// SAFETY:` comments per ADR §7.
- SEC-2: `RmStartSession`/`RmEndSession` (BUG-004) and the new force-close path
  share the same RAII discipline. Force-close uses RAII for `OpenProcess` /
  `DuplicateHandle` / inspection thread join.
- SEC-3: `NtQueryObject` is run in a worker thread with a 100 ms join timeout
  (`thread::Builder::spawn` + `JoinHandle::join` is unstable for timeout; we use
  a `mpsc::channel` with `recv_timeout`). On timeout we drop the channel,
  abandon the thread (it remains blocked until the kernel returns), and skip
  the handle. The leaked thread is OS-bounded by the count of stuck handles.
- SEC-4: Skip our own process (`pid == GetCurrentProcessId()`) to avoid
  closing our own file handles.
- SEC-5: No new crate dependency.
- SEC-6: BR-001 still preserves "no `rename` / `chmod` / `truncate` / process
  termination". Adding only the explicit `DUPLICATE_CLOSE_SOURCE` path.

## Test

- `lock_detect::force_close_handles_for_path` is hard to unit-test reliably
  across platforms. We cover:
  - `force_close_for_unknown_path_returns_zero` — call on a path that does not
    exist; expect `0`.
  - `force_close_with_empty_pids_returns_zero` — call with empty PID filter;
    expect `0`.
- Integration / smoke verification: on Windows, hold a file open in Notepad,
  invoke `xdu`, navigate to the file, `d` → Yes → lock-prompt opens → `f` →
  force-confirm → Yes → handle is closed in Notepad → delete succeeds.
