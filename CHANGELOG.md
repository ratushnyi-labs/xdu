# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this
project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

## [0.1.0] - 2026-05-06

Initial release. Single static binary per platform, read-only disk-usage explorer
with an interactive terminal UI.

### Added
- Scaffold: Rust workspace, MIT LICENSE, `.gitignore`/`.dockerignore`/`.editorconfig`,
  multi-stage `Dockerfile.build`, `docker-compose.yml`, `scripts/build.sh`,
  `scripts/ci.sh`, `Cargo.lock`. (`TASK-001-C`)
- Multi-target Docker build pipeline: `x86_64-unknown-linux-musl`,
  `aarch64-unknown-linux-musl`, `x86_64-pc-windows-gnu`. Size budget 1.5 MiB
  enforced. (`TASK-001-D`)
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
