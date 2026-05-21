# TASK-007-K — Linux lock-holder enumeration (LATER-008b)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-21  **Updated:** 2026-05-21

## Refined Description
Resolves `LATER-008b` from `docs/spec.md` §9. Mirrors `BUG-004` (Windows
Restart-Manager lock-holder lookup) for Linux by walking `/proc/*/fd/`.

**Semantic difference from Windows:** Linux does not block `unlink()` for files
currently open in another process — the file is unlinked from the directory and
its inode remains live until the last fd closes. So the lock-holder modal on
Linux is purely informational ("file is open in N processes — proceed?") rather
than gating, mirroring the Windows experience without changing the underlying
unlink semantics.

**Implementation in `src/lock_detect.rs`:**
- New `#[cfg(target_os = "linux")] holders_linux(path: &Path) -> Vec<Holder>`:
  - `read_dir("/proc")`, filter numeric pid dirs.
  - For each pid: read `/proc/<pid>/comm` for the command name; iterate
    `/proc/<pid>/fd/`, `readlink` each entry; collect entries whose target ==
    the canonicalised input path.
  - Skip pids we can't `readlink` (permission-denied is normal for other users'
    processes — record nothing rather than erroring).
- Wire into the existing `holders(path)` cfg-gated dispatch.

**Force-close path:** `force_close_handles_for_path` stays Windows-only (Linux
has no API equivalent without `ptrace` + huge complexity). On Linux the modal
gives the user the choice to proceed with the unlink (which will succeed) or
cancel.

## Estimation
**Complexity:** ABOVE_EASY. /proc scan + path-canonicalisation match + cfg-gating.

## Speck Reference
[`docs/specks/SDD-TASK-007-K-linux-lock-holder.md`](../specks/SDD-TASK-007-K-linux-lock-holder.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
