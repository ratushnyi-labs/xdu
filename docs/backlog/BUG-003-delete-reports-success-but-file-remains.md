# BUG-003 — delete reports success but file is not actually removed

**Status:** DONE  **Type:** BUG
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
Pre-merge baseline security gate: PASS — only `Path::exists()` and `canonicalize` added (metadata-only); BR-001 read-only invariant preserved. 89 tests pass (83 unit + 6 integration). Multi-target rebuild within budget (657 / 657 / 515 KiB).

## Compliance Notes (added 2026-05-06 after lawbook v0.7.0 → v0.8.0 upgrade)

**Cross-agent attack review skip (§9.5.2 — recorded retroactively):**

- Both LIGHT and FULL §9.5 passes are **SKIPPED** for this item.
- Concrete reason: no second-agent vendor reachable from this laptop. User statement
  (2026-05-06, verbatim): _"this laptop has access to claude only, the gemini/gpt
  are cut-off"_. Per §9.5.1, a human attacker MAY substitute for the second agent;
  the user has not elected to perform that substitution for this item.
- Retroactive context: this item was authored and committed (`ae66380`) under
  lawbook **v0.7.0**, which did not include §9.5. The session's lawbook reload
  picked up v0.8.0, which introduces the rule.
- Continuity surfacing per §9.5.2: skip is acknowledged here so it does not silently
  omit the gate. Pre-DONE check 9 (§12.8.1) treats this as an active-Compliance-Notes
  skip rather than a `MAJOR` continuity blocker.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> i tried to delete the file qnd tool reports it is deleted, but actually not.

## Reproduction

User pressed `d` on a file in the TUI, navigated to `[ Yes ]`, pressed Enter (or pressed
`y`), saw the status `deleted <path>` in the footer, but the file was still on disk
afterwards.

## Hypotheses

There are two paths that can produce this report:

1. **Path resolution mismatch.** `App::absolute_path_for` builds the target path by
   joining `root_path` with the breadcrumb names returned by `Tree::path(node)` (skipping
   the first part, which is the scan root's basename). If a path component contains
   anything that `Path::push` reinterprets, or if `root_path` was relative (`.`) and the
   process CWD changed between commands, the resolved path could point at a different
   entry — `remove_file` succeeds, but it is not the file the user expected.
2. **Windows delete-on-close.** On Windows, when a handle is open with
   `FILE_SHARE_DELETE`, `DeleteFileW` can mark the file for deletion (the syscall returns
   `TRUE`) yet the entry persists in the directory listing until the last handle closes.
   `std::fs::remove_file` is a thin wrapper over `DeleteFileW` and forwards the success
   status verbatim.

Both classes manifest the same way to the user: the footer says "deleted" but the file
is still there on the next listing.

## Refined Description

**Fix:** after the delete syscall returns `Ok`, **read the path back via `Path::exists`**.
If the entry is still on disk, replace the success status with an explicit
"delete reported success but `<path>` is still on disk — try `r` to refresh" warning
that names the *canonical* absolute path. The canonical path is captured **before** the
delete (the path no longer exists post-delete and `canonicalize` would fail).

This fix makes both hypothesis-1 and hypothesis-2 visible to the user. In hypothesis 1
the user sees a path that doesn't match what they expected and can report it. In
hypothesis 2 the user knows to wait / close the holding app / press `r` to refresh.

**Impacted UCs:** UC-008 (delete).
**Impacted BR/WF:** BR-001 (read-only invariant: still preserved; the read-back is a
metadata-only `exists()` call).

## Estimation

**Complexity:** EASY  **Justification:** four-line change in `perform_delete` plus one
unit test. No new module, no new dependency.

## Speck Reference
[`docs/specks/SDD-BUG-003-verify-delete.md`](../specks/SDD-BUG-003-verify-delete.md)
