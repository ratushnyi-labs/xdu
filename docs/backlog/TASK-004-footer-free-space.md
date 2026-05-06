# TASK-004 — footer free-space indicator

**Status:** DONE  **Type:** TASK
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
Pre-merge baseline security gate: PASS — read-only metadata FFI; small `unsafe` blocks per ADR §7 with `// SAFETY:` comments; no new crate dep. 80 tests pass (74 unit + 6 integration); 3 new disk_space tests. Multi-target rebuild within budget (~648/648/504 KiB).
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> 1. in the status bar in the bottom near the right corner add information about available space in the fs and %

## Refined Description

Right-aligned segment at the right edge of the footer showing the **available** filesystem
space (and percent of total) for the volume containing the scan root. Format example:
` free: 12.3 GiB (45%) `.

When the scan is running, the existing `scanning (… entries, …) <spinner>` string sits on
the left and the free-space block sits on the right; both render simultaneously. When the
window is too narrow to fit both, the free-space segment is preserved and the left-side
hints are truncated.

**Implementation:** new module `src/disk_space.rs` with one public function
`pub fn free_and_total(path: &Path) -> Option<(u64, u64)>`.
- Linux (`#[cfg(target_os = "linux")]`): raw FFI to `statvfs(3)` — small, justified
  `unsafe` block per ADR §7. Computes
  `free = f_bavail × f_frsize`, `total = f_blocks × f_frsize`.
- Windows (`#[cfg(windows)]`): raw FFI to `GetDiskFreeSpaceExW` (kernel32) returning
  `(free_bytes_available_to_caller, total_number_of_bytes)`.
- Other Unix (macOS, BSDs): returns `None` for v0.1.x; documented as a v0.2 follow-up.

**Impacted UCs:** UC-002 / UC-007 (footer rendering).
**Impacted BR/WF:** BR-001 read-only invariant preserved (statvfs / GetDiskFreeSpaceExW are read-only metadata calls).

## Estimation

**Complexity:** EASY  **Justification:** small FFI module per platform + one footer layout helper. No new crate dep.

## Speck Reference
[`docs/specks/SDD-TASK-004-footer-free-space.md`](../specks/SDD-TASK-004-footer-free-space.md)
