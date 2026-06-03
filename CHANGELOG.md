# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this
project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

### Changed

### Fixed

## [0.1.0] - 2026-06-03

Initial release. Single static binary per platform, read-only disk-usage explorer
with an interactive terminal UI.

### Added
- `--trash` flag — move deleted items to the OS-native trash instead of
  permanently deleting them. Linux follows the freedesktop.org Trash spec v1.0
  (`$XDG_DATA_HOME/Trash` with `.trashinfo` sidecars). Windows uses
  `SHFileOperationW(FO_DELETE | FOF_ALLOWUNDO | FOF_NO_UI)`. macOS / BSD return
  `Unsupported` (v0.2 follow-up). (TASK-007-J / LATER-007.)
- Windows drive picker — when `xdu` is launched with no path argument on
  Windows, it prints a numbered drive list (with type + label + free / total
  space) and reads stdin for a selection. Other platforms continue to default
  to the current directory. (TASK-007-H / TASK-003.)
- macOS free-space indicator — Darwin's `statfs(2)` FFI lights up the footer
  free-space figure on x86_64 and aarch64 Apple Silicon. (TASK-007-L / LATER-009.)
- Linux lock-holder enumeration — walks `/proc/*/fd/` to identify which
  processes hold a file open, mirroring the Windows Restart-Manager modal from
  BUG-004 (informational on Linux since unlink succeeds with open holders).
  (TASK-007-K / LATER-008b.)
- TUI snapshot test harness — `tests/snapshot_test.rs` renders the App into
  ratatui's `TestBackend` and diffs against committed text baselines under
  `docs/ui-baselines/snapshots/`. Regenerate with `XDU_BLESS_SNAPSHOTS=1`.
  (TASK-007-F / LATER-004.)
- `cargo-llvm-cov` coverage tooling — installed in the test image; new
  `scripts/coverage.sh` writes `coverage.lcov` and prints a summary. Test
  functions named `sec_uc*` renamed to `test_sec_uc*` for naming consistency.
  (TASK-007-G / LATER-006.)
- Scaffold: Rust workspace, MIT LICENSE, `.gitignore`/`.dockerignore`/`.editorconfig`,
  multi-stage `Dockerfile`, `docker-compose.yml`, `scripts/build.sh`,
  `scripts/ci.sh`, `Cargo.lock`. (`TASK-001-C`)
- Multi-target Docker build pipeline (6 targets): `x86_64-unknown-linux-musl`,
  `aarch64-unknown-linux-musl`, `x86_64-pc-windows-msvc`,
  `aarch64-pc-windows-msvc`, `x86_64-apple-darwin`, `aarch64-apple-darwin`.
  MSVC targets cross-compile via `cargo-xwin`; Apple targets via
  `cargo-zigbuild`. Size budget 1.5 MiB enforced.
  (`TASK-001-D`, `TASK-007-N8a`, `TASK-007-N8b`, `TASK-007-I`)
- Size formatter (`format_size`) and proportion-bar generator (`bar`) with
  IEC / SI modes and UTF-8 / ASCII fallback. (`TASK-001-G`)
- Tree model: arena-backed `Vec<Node>` indexed by `NodeId(u32)`, stable sort
  with name-asc tiebreak, iterative post-order aggregation safe at deep nesting,
  saturating overflow guard. (`TASK-001-F`)
- Filesystem scanner: iterative DFS, hardlink dedup `(dev, inode)` on Unix,
  mount-boundary detection on Unix, symlinks not followed, per-entry I/O errors
  recorded without aborting the scan. (`TASK-001-E`)
- TUI suite: terminal init/teardown with panic-safe restoration, tree-view widget
  with selection + scroll, header/footer/help/errors overlays, synchronous
  rescan. (`TASK-001-H/I/J/K`)
- JSON export of the current subtree (`xdu-export-1` schema), hand-rolled
  streaming encoder, deterministic child ordering, iterative walk safe at any
  depth, path-traversal rejection. (`TASK-001-L`)
- Integration tests against an ephemeral temp-dir fixture, multilingual /
  spec / install / dev / config manuals, final README skeleton.
  (`TASK-001-M/N`)
- TUI 3-column layout (name | bar | size | date) with last-access date column;
  synthetic `..` row when ascend would have an effect; named after `Cargo.toml`
  `description` to match the org pattern. (`TASK-002`)
- Free-space indicator on the footer's right edge (Linux `statvfs` + Windows
  `GetDiskFreeSpaceExW` raw FFI). (`TASK-004`)
- Delete file/directory via `d` key with a `[ No ]`-default confirmation modal.
  Refuses to delete the scan root. Windows in-use error codes 5 / 32 / 33 produce
  an actionable status hint pointing at Sysinternals Process Explorer / Resource
  Monitor. (`TASK-005`)
- Lock-holder detection on Windows via the Restart Manager API
  (`rstrtmgr.dll`) — after a delete fails because the file is in use, xdu lists
  the holding processes (PID + app name) in a modal. (`BUG-004`)
- Force-close foreign file handles via the OS-sanctioned
  `DuplicateHandle(DUPLICATE_CLOSE_SOURCE)` form (Windows, `ntdll` +
  `kernel32` FFI). Triggered with `f` from the lock-prompt, gated by a
  separate `[ No ]`-default confirmation modal that warns about possible
  foreign-app data loss. No process termination; no direct `NtClose` on a
  foreign handle. (`TASK-006`)

### Changed
- Repo conventions aligned with the `ratushnyi-labs` org: LICENSE copyright holder,
  `Cargo.toml` metadata (`repository`, `homepage`, `authors`), standard
  `[profile.release]` replacing the custom `release-min` profile, simpler
  `.gitignore` / `.gitattributes` / `.dockerignore`, `.env` for §8.1 compliance.
- Added `.github/` tree: `CODEOWNERS`, `SECURITY.md`, `PULL_REQUEST_TEMPLATE.md`,
  three `ISSUE_TEMPLATE/*.yml`. (TASK-007 / wave 2.)
- Added `CONTRIBUTING.md` describing dev setup, workflow, code style, safety rules,
  testing requirements, commit-message convention, and PR checklist.
- README restructured to the org pattern: centered hero, three-badge row, supported
  targets table, documentation table, multilingual user-manual stub.
- Scanner moved to an asynchronous worker thread; the TUI opens immediately and
  renders an animated spinner in the size column for non-finalized rows. Eager
  bubble-up aggregation surfaces partial sums mid-scan. (`BUG-002`)
- Ascend from the scan root with an empty back-stack now walks the filesystem
  parent and re-scans, rather than being a silent no-op. (`BUG-001`)

### Fixed
- Delete now reads back `Path::exists()` after the syscall returns `Ok` and
  surfaces a warning if the entry is still on disk (catches the Windows
  delete-on-close case and any path-resolution mismatch). The status message
  shows the canonical absolute path so resolution surprises are visible.
  (`BUG-003`)

### Security
- Pre-merge baseline security gate (`SECURITY_AUDIT`,
  `DEPENDENCY_TREE_SECURITY_AUDIT`, `MEMORY_LEAK_AUDIT`) recorded `PASS` for
  every implementable item from scaffold to `TASK-006`.
- Zero `unsafe` outside the documented Win32 / libc FFI blocks in
  `src/disk_space.rs` and `src/lock_detect.rs`; each block carries a
  `// SAFETY:` comment per ADR §7.
