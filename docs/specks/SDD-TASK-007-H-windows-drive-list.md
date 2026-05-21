# SDD-TASK-007-H — Windows drive list

**Backlog item:** TASK-007-H
**Impacted UCs:** UC-001 (initial scan entry), specifically the "launched
with no path on Windows" sub-flow that was previously undefined.

## Scope
- New module `src/drives_win.rs`:
  - `pub fn list_drives() -> Vec<Drive>` — enumerates logical drives via the
    Win32 trio: `GetLogicalDriveStringsW`, `GetDriveTypeW`,
    `GetVolumeInformationW`.
  - Free / total space reuses `disk_space::free_and_total`.
  - Filters out drives whose total size cannot be probed (e.g. empty CD-ROM
    bays).
  - Non-Windows targets compile to a no-op stub returning an empty `Vec`.
- `src/lib.rs`: export the new module.
- `src/main.rs`:
  - Change `args.path` to `Option<PathBuf>`.
  - When `args.path` is `None`:
    - If `list_drives()` returns a non-empty list (Windows): print a
      numbered menu, read stdin, select a drive.
    - Otherwise (non-Windows): default to `"."` (existing behaviour).
  - Update `--help` to mention the drive picker on Windows.
- This is the minimum-viable shell-style picker. A fully-interactive TUI
  drive picker mode is recorded for v0.2.

## ACs
- AC-1: On non-Windows, `cargo run` with no args still scans `.`.
- AC-2: On Windows, `cargo run` with no args prints a numbered drive
  list and reads stdin; entering a valid number proceeds to TUI; pressing
  Enter with empty input selects #1.
- AC-3: `drives_win::list_drives()` reports at least the system drive
  (`C:`) on a Windows host with no removable media (manual verification
  deferred to TASK-007-O).
- AC-4: `DriveType::short()` returns stable lowercase tags for the menu.
- AC-5: Tests confirm: (a) non-Windows returns empty; (b) short-tag
  strings are stable; (c) Windows reports at least one drive (Windows-only
  test cfg-gated).

## SEC ACs
- SEC-1: `unsafe` is limited to the three FFI sites + small buffer manipulation,
  all bounded by passed-in buffer sizes; identical pattern to
  `disk_space.rs` / `lock_detect.rs`.
- SEC-2: No write operations on any drive; only read enumeration + label /
  size queries.
- SEC-3: Stdin prompt validates the entered number against the list length
  before indexing; out-of-range input returns `None` and the binary exits
  with a clear error rather than crashing.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
