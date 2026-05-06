# SDD-TASK-002 — TUI 3-column layout, ".." parent row, last-access date column

**Backlog item:** TASK-002
**Impacted UCs:** UC-001, UC-002, UC-003

## Scope

### Format helpers (`src/format.rs`)
- Add `pub fn format_ymd(secs: u64) -> String` returning `YYYY-MM-DD` for the given Unix-epoch seconds, or 10 spaces when `secs == 0`.
- Add `pub fn format_iso8601_utc(secs: u64) -> String` returning `YYYY-MM-DDTHH:MM:SSZ`.
- Promote `days_to_ymd` to a private module-level helper (single canonical impl).

### Export (`src/export.rs`)
- Replace local `days_to_ymd` and `epoch_to_iso8601` with calls to the new `crate::format` helpers.

### Tree (`src/tree.rs`)
- Add `pub accessed_unix: u64` field to `Node` (default `0` = unknown).
- Add `pub fn set_accessed(&mut self, id: NodeId, secs: u64)`.
- Aggregation does NOT propagate `accessed_unix` — it's a per-entry attribute, displayed only on rows.

### Scanner (`src/scanner.rs`)
- For each entry, derive `accessed_secs = meta.accessed().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0)`.
- Call `tree.set_accessed(id, accessed_secs)` after `add_child`.

### TUI (`src/tui.rs`)
- New row format: `{name:left-padded}  {bar:10}  {size:>9}  {date:10}`.
- Insert a synthetic `..` row at index 0 when `App::can_ascend()` is true. The row shows just `..` in the name column; bar/size/date are blank spaces. Pressing `Right` / `l` / Enter on selection 0 invokes `ascend()`.
- `move_selection` and `End` clamp against `total_rows = children + (1 if dotdot else 0)`.
- `descend()` checks `if dotdot && selection == 0 → ascend()`.
- Color rules apply only to real children (not `..`).
- Help overlay updated.

## ACs

- AC-1: Renderer emits a row whose left column is the entry name + type suffix, then a 10-cell bar, then a 9-char size, then a 10-char date column.
- AC-2: When `App::can_ascend()` returns true, the children area shows a `..` synthetic row at the top; pressing Enter / Right / l on it invokes `ascend()`.
- AC-3: When `can_ascend()` returns false (truly at FS root with no back-stack), no `..` row is shown.
- AC-4: `format_ymd(0)` returns 10 spaces; `format_ymd(1735689600)` returns `2025-01-01`.
- AC-5: `format_iso8601_utc(0)` returns `1970-01-01T00:00:00Z`.
- AC-6: After scan, every node's `accessed_unix` matches `meta.accessed()` ± a few seconds (filesystem precision varies).
- AC-7: Existing column-aware tests (truncate, sanitize, sort labels) continue to pass.

## SEC ACs
- SEC-1: BR-001 preserved (only `meta.accessed()` is consulted; no file content read).
- SEC-2: No `unsafe`. No new crate dependency.
- SEC-3: Date formatting is total — `format_ymd(u64::MAX)` does not panic.
- SEC-4: Synthetic `..` row name is constant and not derived from a filesystem name → no terminal-injection surface.

## Test
- New unit tests in `format`, `tree`, `tui`.
- Existing 68 tests must still pass.
