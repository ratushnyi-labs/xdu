# TASK-007-J — Trash mode for delete (LATER-007)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-21  **Updated:** 2026-05-21

## Refined Description
Resolves `LATER-007` from `docs/spec.md` §9. Adds an opt-in "send to trash"
mode for the `d` delete action.

UX:
- New CLI flag `--trash` (and runtime toggle keybinding `t` to swap modes mid-session).
- Delete-confirmation modal label changes from `[ Delete ]` to `[ Move to trash ]`
  when trash mode is active.
- Footer shows current mode (`mode: trash` vs default).

Implementation (per platform; zero new crate deps):
- **Linux:** Move target into `${XDG_DATA_HOME:-$HOME/.local/share}/Trash/files/`,
  writing a sibling `${name}.trashinfo` under `Trash/info/` per the freedesktop.org
  Trash spec v1.0. Use atomic `rename(2)` on same-fs; fall back to copy+unlink
  across filesystem boundaries.
- **Windows:** `SHFileOperationW` from `shell32.dll` with `FO_DELETE` +
  `FOF_ALLOWUNDO` + `FOF_NO_UI`. (`IFileOperation` COM is the modern path but
  requires more boilerplate; SHFileOperationW remains supported.)
- **macOS / others:** Out of scope for v0.1.0 — trash mode disabled, with a
  startup warning if `--trash` is passed on an unsupported platform.

ACs include:
- Trash-mode delete moves the item; original is gone, trash entry present.
- Non-trash-mode delete continues to permanently delete (existing behaviour).
- Mode is reflected in the modal and in the footer.

## Estimation
**Complexity:** ABOVE_EASY (single platform), HARD (full cross-platform). For
v0.1.0: ship Linux + Windows; macOS deferred.

## Speck Reference
[`docs/specks/SDD-TASK-007-J-trash-mode.md`](../specks/SDD-TASK-007-J-trash-mode.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
