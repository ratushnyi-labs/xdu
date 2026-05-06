# TASK-005 — delete file/dir with confirmation popup

**Status:** IN PROGRESS  **Type:** TASK
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> 2. add ability to delete file / dir. add popup that asks and keep default focus on No instead of Yes. Note, file may be opened by some app in that case you can ask user to unlock it (like in sysinternals utilities). some tools keep very large logs locked for simple deleting

## Spec impact (handled before code, per §3.5)

This task **reverses BR-001** (the original "all file-system operations MUST be read-only"
invariant the user requested at intake). The amended BR-001 reads: **xdu defaults to
read-only; file/directory deletion is exposed only behind the `d` key plus a confirmation
prompt whose default focus is "No"**. There is no `--read-only` lockout flag in v0.1.x —
users who never press `d` retain the original read-only behavior.

## Refined Description

- New keybinding `d` on a real entry (not on `..`): opens a centered confirmation modal.
  Modal shows the absolute path, the entry kind (file vs directory — directories are
  removed recursively via `std::fs::remove_dir_all`), and `[ Yes ]  [ No ]` buttons with
  **default focus on `[ No ]`**.
- `Tab`, `Left`/`Right`, `h`/`l`: toggle focus between Yes and No.
- `Enter`: activate the focused button. Default-focused No → cancel. Yes → perform delete.
- `y`/`Y`: quick-confirm shortcut (still requires user to press a separate key, distinct
  from the default Enter-on-No path).
- `n`/`N`/`Esc`/`q`: cancel the prompt. (`q` while a prompt is open cancels the prompt
  rather than quitting the app — it would be surprising to lose the prompt and the app.)
- The prompt refuses to delete the scan root itself.
- After a successful delete, status message + asynchronous rescan of the current root
  (so totals refresh).

**"Locked file" handling:** when `remove_file` / `remove_dir_all` returns an OS error that
indicates the file is in use (Windows `ERROR_SHARING_VIOLATION = 32`,
`ERROR_LOCK_VIOLATION = 33`, `ERROR_ACCESS_DENIED = 5` on a delete syscall), the status
message tells the user the file is in use and suggests they identify and close the
holding handle using **Sysinternals Process Explorer** or **Resource Monitor**, then try
again. **Forcible unlock** (calling RestartManager via FFI to close foreign handles)
is **not** implemented in v0.1 — recorded as a v0.2 follow-up. Justification: it requires
elevated privileges, can corrupt application state, and is rarely the right call.

**Impacted UCs:** UC-002 (new keybinding), new UC-008 (delete flow).
**Impacted BR/WF:** BR-001 amended.

## Estimation

**Complexity:** ABOVE_EASY  **Justification:** spec invariant change + popup state machine + actionable error messages + small render block. No new crate dep.

## Speck Reference
[`docs/specks/SDD-TASK-005-delete-with-confirmation.md`](../specks/SDD-TASK-005-delete-with-confirmation.md)
