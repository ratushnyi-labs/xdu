# TASK-007-H — Windows drive list (TASK-003)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-21  **Updated:** 2026-05-21

## Refined Description
Resolves `TASK-003` from `docs/spec.md` §9. When `xdu` is launched on Windows
without a path argument, present a drive-picker UI listing available drives
with their type, optional volume label, and free / total space. Selecting a
drive opens it as the scan root.

Linux/macOS retain the existing default of "current working directory".

**FFI (Windows-only) in a new `src/drives_win.rs`:**
- `GetLogicalDriveStringsW(buf_len, buf_ptr) -> DWORD` from kernel32 —
  populates a buffer with `"C:\\\0D:\\\0...\0"`.
- `GetDriveTypeW(root) -> DWORD` from kernel32 — for filtering / labelling
  (`DRIVE_FIXED`, `DRIVE_REMOVABLE`, `DRIVE_REMOTE`, `DRIVE_CDROM`, `DRIVE_RAMDISK`).
- `GetVolumeInformationW(root, vol_name_buf, vol_name_len, ...)` from kernel32
  — fetches the volume label.
- Free/total: reuses `src/disk_space.rs::free_space_for(&Path)`.

**TUI (drive picker mode):**
- New `App::Mode::DrivePicker` state, entered when `args.path.is_none()` on Windows.
- List rows: `[type] X:  <label>  <free> / <total>`.
- Arrow keys move selection; Enter sets `App.root = Path::new("X:\\")` and
  transitions to normal `ScanInProgress` mode; Esc/q quits.
- Read-only invariant preserved — the picker only reads metadata.

## Estimation
**Complexity:** ABOVE_EASY. New module + new App mode + 3 FFI wrappers.

## Speck Reference
[`docs/specks/SDD-TASK-007-H-windows-drive-list.md`](../specks/SDD-TASK-007-H-windows-drive-list.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
