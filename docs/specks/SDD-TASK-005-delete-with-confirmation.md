# SDD-TASK-005 — delete file/dir with confirmation popup

**Backlog item:** TASK-005
**Impacted UCs:** UC-002, new UC-008

## Spec impact (handled before code per §3.5)

`docs/spec.md` §4 BR-001 amended: from "all file-system operations MUST be read-only"
to "default read-only; deletion is exposed only via the explicit `d` key plus a
confirmation modal whose default focus is `[ No ]`". A new UC-008 documents the
delete flow.

## Scope

### `src/tui.rs`
- `App` gains `confirm_prompt: Option<ConfirmPrompt>`.
- `ConfirmPrompt { target: NodeId, message: String, yes_focused: bool }`.
- New keybinding `d` on a real entry (not on `..`) opens the prompt with `yes_focused = false`.
- While the prompt is open:
  - `Tab` / `Left` / `Right` / `h` / `l`: toggle `yes_focused`.
  - `Enter`: dispatch on the focused button.
  - `y` / `Y`: quick-confirm.
  - `n` / `N` / `Esc` / `q`: cancel (in the prompt scope, `q` cancels the prompt rather
    than quitting xdu, to avoid the surprising sequence "user opened prompt, hit q, lost
    both the prompt and the app").
- Selecting `[ Yes ]` or pressing `y`:
  - Resolve the target's absolute path via `tree.path(node_id)` joined under
    `App::root_path`.
  - Refuse to delete the scan root (canonical-path equality check).
  - Files: `std::fs::remove_file`. Directories: `std::fs::remove_dir_all` (recursive).
  - On success: status message + asynchronous rescan of `App::root_path` so totals refresh.
  - On error: actionable status. Specific Windows error codes
    (`ERROR_SHARING_VIOLATION = 32`, `ERROR_LOCK_VIOLATION = 33`,
    `ERROR_ACCESS_DENIED = 5`) trigger a "file is in use — close it with Process Explorer
    / Resource Monitor and try again" message.

### Render
- New `render_confirm` modal, centered, ~70 cols wide. Yellow on selected button border.

## ACs

- AC-1: Pressing `d` on a real entry opens a confirmation modal with default focus on
  `[ No ]`.
- AC-2: Pressing `d` on the `..` row sets a status message and does NOT open the modal.
- AC-3: With the modal open, `Enter` activates the focused button. Default-focus No →
  cancel.
- AC-4: `Tab` / `Left` / `Right` / `h` / `l` toggles focus.
- AC-5: `y` is a quick-confirm; `n` / `Esc` / `q` cancel (without quitting xdu).
- AC-6: Confirming Yes deletes the entry and starts an async rescan of the current root.
- AC-7: Refuses to delete the scan root.
- AC-8: Windows in-use error → status mentions Process Explorer / Resource Monitor.
- AC-9: Other delete errors → status reports the OS error.

## SEC ACs

- SEC-1: Modal default focus is `[ No ]` — verified by a render test that builds a
  prompt, captures `confirm_buttons_line(false)`, asserts the No button carries the
  reverse-video modifier and Yes does not.
- SEC-2: Scan root cannot be deleted (refused before any FS call).
- SEC-3: No `unsafe`. No new crate dep.
- SEC-4: BR-001 amended in spec — recorded explicitly as a behavior change.

## v0.1.x non-goals (recorded; v0.2 follow-up `LATER-007`)

- **Forcible unlock** of locked Windows files via RestartManager / NTQuerySystemInformation
  / NtClose. Requires elevated privileges, can corrupt foreign apps' state. Not implemented.
- **Trash / Recycle Bin** — delete is permanent. v0.2 may offer a `--trash` flag that uses
  `IFileOperation` (Windows) / `gio trash` style helpers (Linux).

## Test
- Tests in `src/tui.rs::tests`:
  - `task005_confirm_buttons_default_focus_no` — buttons line for `yes_focused=false`
    paints No reversed and Yes plain.
  - `task005_open_prompt_then_cancel` — open the prompt, press Esc, prompt cleared, no
    deletion.
  - `task005_yes_actually_deletes_file` — happy-path: build temp file, open prompt,
    `yes_focused = true`, `Enter` → file gone.
- Existing 77 tests must still pass.
