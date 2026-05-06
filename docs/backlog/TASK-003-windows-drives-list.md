# TASK-003 — Windows drive-list view at filesystem root

**Status:** READY  **Type:** TASK
**User Mode:** GUIDED  **Guided Flow Stage:** PLAN_READY
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

(Same bundled message as TASK-002.) Specifically:
> so under win user should be able (if has permissions) to navigate to list of disks.

## Refined Description

**Scope:** when the user ascends from a Windows drive root (e.g. `C:\`), enter a synthetic "Computer" view listing the available drive letters as navigable entries. Probing approach (no Win32 FFI required): try `std::fs::metadata("A:\\")` … `Z:\\`; drives that respond are listed. Selecting a drive (Right/Enter) invokes `scan()` on it as a fresh root.

**Out of scope:** drive-size totals (probing free/used per drive needs Win32 FFI — out of TASK-003); per-drive labels/volume names; UNC-share enumeration; removable-media notification.

**Impacted UCs:** UC-002 (navigation — new "drive list" mode).
**Impacted BR/WF:** BR-001 preserved (read-only).

## Estimation

**Complexity:** ABOVE_EASY  **Justification:** introduces a "drive list" mode in `App`, requires a small Windows-only probe, touches `scan_filesystem_parent` flow, adds Windows-only tests (skipped on Unix CI).

## Status

**Deferred** to a follow-up turn after TASK-002 lands. Recorded in `docs/spec.md` §9 so the gap is visible.

## Speck Reference
`docs/specks/SDD-TASK-003-windows-drives.md` — to be authored when the leaf is picked up.
