# SDD-BUG-003 — verify delete by reading back path.exists()

**Backlog item:** BUG-003
**Impacted UCs:** UC-008

## Scope

Tighten `App::perform_delete` in `src/tui.rs`:

1. Capture `canonical_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone())`
   **before** the delete so the status message shows the absolute path the OS actually
   resolved (this catches relative-path / `.`-rooted resolution surprises).
2. After `remove_file` / `remove_dir_all` returns `Ok`, check `path.exists()`. If the
   entry is still present, emit `"delete reported success but {canonical_path} is still
   on disk — try \`r\` to refresh"` instead of `deleted …` — and **skip** the rescan, so
   the user can inspect what happened on the same tree state.

## ACs

- AC-1: When the delete genuinely succeeds, the success status reads
  `deleted <canonical absolute path>`.
- AC-2: When the syscall returns `Ok` but the entry persists, the status reads the
  defensive warning and includes the canonical path.
- AC-3: When the syscall returns `Err`, the status is unchanged from TASK-005.
- AC-4: BR-001 read-only invariant preserved (only `exists()` is added — metadata only).

## SEC ACs

- SEC-1: No new `unsafe`. No new crate dep.
- SEC-2: Canonicalize-before-delete cannot raise — failure falls back to the original
  `path`, so the status always renders some path.

## Test

- New `tui::tests::bug003_status_reports_canonical_path_on_success` — create a temp
  file, drive the App to delete it, assert the status string contains `"deleted"` AND
  matches the canonical path the OS reports for that file.
- The "Ok but path still exists" branch is hard to exercise reliably on Linux/Windows in
  unit tests — we cover it by static review (the new `if path.exists()` is one branch
  in `perform_delete`) and by the AC-2 wording in the status message which the user can
  recognise the next time they run into the original symptom.
