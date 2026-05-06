# SDD-TASK-004 — footer free-space indicator

**Backlog item:** TASK-004
**Impacted UCs:** UC-002, UC-007 (footer)

## Scope

### `src/disk_space.rs` (new)
- `pub fn free_and_total(path: &Path) -> Option<(u64, u64)>` returning
  `(available_bytes, total_bytes)` for the volume containing `path`. `None`
  when the platform is unsupported or the syscall fails.
- Linux: `extern "C" { fn statvfs(path: *const c_char, buf: *mut c_void) -> c_int; }`
  with a `#[repr(C)]` Linux-musl-shaped `Statvfs` struct. Single small `unsafe`
  block; documented per ADR §7.
- Windows: `extern "system" { fn GetDiskFreeSpaceExW(...) -> i32; }`. Path
  encoded as UTF-16 with NUL terminator via `OsStrExt::encode_wide`. Single
  small `unsafe` block.
- Other Unix (macOS, BSDs): `None`. v0.2 follow-up.
- A `pub fn format_free(path: &Path, mode: SizeMode) -> String` helper returns
  ` free: 12.3 GiB (45%) ` (or empty when unavailable). Calls `format_size`.

### `src/tui.rs` footer
- Footer becomes two-segment: `left` (existing status / scanning / hints) and
  `right` (the free-space block from `format_free`). Helper `compose_footer(left, right, width)` pads or truncates so both fit; on extreme narrow terminals, the right segment wins.

## ACs

- AC-1: Footer right edge shows `free: <size> (<pct>%)` whenever
  `disk_space::free_and_total(scan_root)` returns `Some`.
- AC-2: Free segment is empty / hidden when the platform is unsupported or
  the syscall fails.
- AC-3: Percent is rounded to the nearest whole number; size respects the
  active `SizeMode` (IEC/SI).
- AC-4: When terminal width < `len(left) + len(right)`, the right segment is
  preserved and the left is truncated from the left.
- AC-5: When the scan is running, both the `scanning … <spinner>` indicator
  AND the free-space block render simultaneously (left & right respectively).

## SEC ACs

- SEC-1: BR-001 read-only preserved — `statvfs` / `GetDiskFreeSpaceExW` are
  metadata-only; no write occurs.
- SEC-2: Path passed to FFI is bounded — UTF-8 conversion via `to_str()` /
  UTF-16 via `encode_wide`; both produce a NUL-terminated buffer with known
  length.
- SEC-3: `Vec<u16>` buffer is built locally; no aliasing; `unsafe` blocks
  contain only the syscall and read pre-zeroed locals afterwards.
- SEC-4: Each `unsafe` block annotated with `// SAFETY:` comment.
- SEC-5: No new crate dependency.

## Test
- New unit tests in `disk_space::tests`: query for the temp directory's
  filesystem; assert `total > 0` and `free <= total`. Skipped quietly on
  unsupported platforms.
