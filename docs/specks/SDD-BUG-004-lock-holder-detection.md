# SDD-BUG-004 — lock-holder detection on locked-file delete

**Backlog item:** BUG-004
**Impacted UCs:** UC-008

## Scope

### `src/lock_detect.rs` (new)

```rust
pub struct LockHolder {
    pub pid: u32,
    pub name: String,
}

pub fn holders(path: &Path) -> Vec<LockHolder>;
```

- Windows: FFI to `rstrtmgr.dll`:
  - `RmStartSession`, `RmEndSession`
  - `RmRegisterResources(session, n_files, *files)`
  - `RmGetList(session, *needed, *count, *array, *reasons)` — call twice
    (probe-needed pattern: first call returns `ERROR_MORE_DATA` and the size,
    second call gets the data into a Vec-backed array).
  - `#[repr(C)] struct RmUniqueProcess { dwProcessId: u32, ProcessStartTime: FILETIME }`
  - `#[repr(C)] struct RmProcessInfo { Process: RmUniqueProcess, strAppName: [u16; 256], strServiceShortName: [u16; 64], ApplicationType: u32, AppStatus: u32, TSSessionId: u32, bRestartable: i32 }`
- Linux: returns empty `Vec` for v0.1 — recorded as `LATER-008b`.
- macOS / BSD: same — empty `Vec`.

Each FFI block carries a `// SAFETY:` comment per ADR §7.

### `src/tui.rs`

- `LockPrompt { target: NodeId, path: PathBuf, holders: Vec<LockHolder>, retry_focused: bool }`.
- `App::lock_prompt: Option<LockPrompt>`.
- After `perform_delete` either gets `Err` with code `5/32/33` OR gets `Ok` but
  `path.exists()`, call `lock_detect::holders(&path)`. If non-empty, open a `LockPrompt`
  modal; otherwise keep the existing BUG-003 / TASK-005 behavior.
- `handle_prompt_key` is generalised to dispatch between the existing `ConfirmPrompt`
  and the new `LockPrompt`. While `LockPrompt` is open:
  - `Tab` / `Left` / `Right` / `h` / `l`: toggle `retry_focused`.
  - `Enter`: activate focused button. Retry → `perform_delete` again on the same target.
    Cancel → close prompt with a status message.
  - `r` / `R`: quick retry shortcut.
  - `Esc` / `q`: cancel.
- `render_lock_prompt`: centered modal, 90 cols wide, lists up to 8 holders as
  `PID — name`, two buttons with default focus on `[ Retry ]`.

## ACs

- AC-1: When delete fails with Windows `ERROR_SHARING_VIOLATION` (32) and `holders`
  returns at least one entry, the lock-prompt modal opens.
- AC-2: When delete returns `Ok` but `path.exists()` AND `holders` returns at least one
  entry, the lock-prompt modal opens.
- AC-3: When `holders` returns empty (or platform unsupported), the existing
  TASK-005 / BUG-003 status messages are used.
- AC-4: The modal lists the holder PIDs and names, capped at 8 entries.
- AC-5: Default focus is `[ Retry ]`.
- AC-6: Pressing Retry invokes `perform_delete` again on the same target.
- AC-7: Pressing Cancel closes the modal and sets a status message.
- AC-8: Linux / macOS / BSD: `lock_detect::holders` returns empty; existing flow
  unchanged.

## SEC ACs

- SEC-1: BR-001 read-only invariant preserved — Restart Manager calls are
  enumeration-only; we never call `TerminateProcess` or `NtClose`.
- SEC-2: All FFI blocks carry `// SAFETY:` comments. No new crate dep.
- SEC-3: `RmStartSession` is paired with `RmEndSession` in every code path
  (use a small RAII guard).
- SEC-4: Buffer for `RmGetList` is sized via the probe-needed pattern; `unsafe`
  reads only bytes the API claims it wrote (`count <= capacity`).
- SEC-5: PID + UTF-16 app names are converted to safe `String` via
  `String::from_utf16_lossy` and trimmed at the first NUL.

## Test

- `lock_detect::tests::holders_for_unknown_path_returns_empty` — call with a path
  that does not exist; expect empty.
- `lock_detect::tests::holders_for_temp_file_returns_empty_if_not_held` — create
  a temp file, call without holding it open; expect empty.
- The "non-empty holder list" path is hard to exercise reliably in unit tests
  (requires a Windows-only background process holding the file). Covered by AC-1
  via static review; the user can verify on their actual locked file.
