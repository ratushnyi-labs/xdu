# SDD-TASK-001-E — Scanner

**Backlog item:** TASK-001-E
**Impacted UCs:** UC-001, UC-005
**Impacted BR/WF:** BR-002 (Unix only in v0.1), BR-003 (Unix only in v0.1), BR-004, BR-005, BR-006; WF-001

## Scope / Non-goals

**In scope** (`src/scanner.rs`):
- `ScanOpts { cross_fs, max_errors }` (default `cross_fs=false`, `max_errors=10_000`).
- `ScanError { path, message }` with `Display` + `std::error::Error`.
- `ScanResult { tree, errors, overflow_errors }`.
- `pub fn scan(root: &Path, opts: &ScanOpts) -> Result<ScanResult, ScanError>`.
- Iterative DFS using a `Vec<(NodeId, PathBuf)>` stack — call-stack-safe at any depth.
- Per-entry I/O errors recorded (capped at `max_errors`); the scan continues (BR-005).
- Hardlink dedup on Unix via `(dev, inode)` `HashSet` (BR-002).
- Mount-boundary check on Unix via `Metadata::dev()` vs root's dev (BR-003).
- Symlinks not followed (BR-004): `DirEntry::metadata()` is `lstat`-equivalent.
- Apparent vs allocated size: Unix uses `MetadataExt::blocks() * 512`; Windows approximates `alloc = len`.
- After traversal, `tree.aggregate()` is called.
- Unit tests using a private `TempDir` helper (no `tempfile` crate).

**Out of scope:**
- Parallelism (ADR §4 — Phase-3 candidate).
- File-content reading.
- Glob excludes (`--exclude`) — deferred per spec §9 known limitation.
- Windows hardlink dedup / cross-fs detection — deferred per spec §9.
- Async progress callback — wave-5 integration adds it via channel wrapping.

## Acceptance Criteria

- AC-1: `scan(root, opts)` returns `Err(ScanError)` if `root` does not exist, is not a directory, or its path contains a NUL byte. The TUI / caller maps these to a non-zero exit (UC-001 §Failure modes).
- AC-2: A normal directory tree scan returns `ScanResult` with a populated `Tree` whose root totals match the expected sum across all reachable entries.
- AC-3: Per-entry I/O errors (e.g., `read_dir` permission denied on a subdir) are captured into `errors` and the affected node is `mark_error`'d; the scan continues and finishes (BR-005).
- AC-4: Errors are capped at `opts.max_errors`; when exceeded, `overflow_errors == true` and additional errors are silently dropped (no panic, no allocation growth).
- AC-5: Symlinks are not followed: a symlink entry is recorded as `NodeKind::Symlink` with its own `len()` only; the target is never traversed.
- AC-6 (Unix): with `cross_fs == false`, entries on a different `dev` than the root are skipped.
- AC-7 (Unix): hardlinks are deduplicated — for the same `(dev, ino)` after the first encounter, subsequent file entries record `alloc = 0` (logical size still recorded).
- AC-8: After scan, `result.tree.aggregate()` has been applied; root has full sums (delegates to `Tree`).
- AC-9: Iterative DFS — scanner does NOT recurse on the call stack (verified by review + a deep-tree integration smoke test in M).

## Security ACs

- SEC-1: Reject root paths containing `\0` before any FS call.
- SEC-2: No subprocess spawning anywhere in this module.
- SEC-3: No file content reads — metadata only.
- SEC-4: Bounded error vector with documented overflow behavior.
- SEC-5: Path stack is depth-bounded by the actual tree; no synthetic path inflation. (No path concatenation longer than the OS allows since we use `entry.path()` — derived from a real `read_dir`.)
- SEC-6 (Unix): hardlink set is a `HashSet<(u64, u64)>` — bounded by file count; no unbounded growth beyond inodes scanned.

## Failure Modes

| Failure                                | Behavior                                                                  |
|----------------------------------------|---------------------------------------------------------------------------|
| Root missing / not dir / NUL in path   | `Err(ScanError)` returned to caller                                       |
| `read_dir` on subdir denied            | Push `ScanError`; mark parent dir `error=true`; continue siblings          |
| `entry.metadata()` fails               | Push `ScanError`; mark parent dir `error=true`; skip entry; continue       |
| `Tree::add_child` returns `OutOfCapacity` | Stop adding further entries; `tree.overflow() == true` reflects state    |
| `errors.len() == max_errors`            | Set `overflow_errors = true`; drop subsequent errors silently             |

## Test Matrix

| AC    | Unit | Integration (M) | Notes                                                |
|-------|------|-----------------|------------------------------------------------------|
| AC-1  | ✅   | —               | NUL-path + missing + non-dir                          |
| AC-2  | ✅   | ✅              | small fixture in unit; deeper in M                    |
| AC-3  | ✅ (Unix)| ✅           | chmod-000 unit on Unix; skipped on Windows           |
| AC-4  | ✅   | —               | inject `max_errors = 1`                               |
| AC-5  | ✅ (Unix)| ✅           | symlink fixture                                       |
| AC-6  | ✅ (Unix)| —            | mock by injecting different `dev` is hard; verified manually + via M cross-fs scan |
| AC-7  | ✅ (Unix)| —            | hardlink fixture                                      |
| AC-8  | ✅   | —               | result.tree.get(root).size > 0                        |
| AC-9  | review + ✅ (M deep tree) | ✅ |                                                |
| SEC-1 | ✅   | —               | NUL byte test                                         |
| SEC-2 | static| —              |                                                      |
| SEC-3 | static| —              |                                                      |
| SEC-4 | ✅   | —               | overflow_errors latch                                 |
| SEC-5 | review| —              |                                                      |
| SEC-6 | review| —              |                                                      |

## Forbidden during this leaf

- New crate dependencies.
- Reading file contents.
- Spawning subprocesses or threads (sync only in v0.1; threading wrapping in wave 5).
- Recursive call-stack DFS.
- `unsafe` blocks (none required — Unix uses `MetadataExt`, Windows uses `len()` only).
