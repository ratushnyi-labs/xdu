# SDD-TASK-007-L — macOS / *BSD free-space via statvfs(3)

**Backlog item:** TASK-007-L
**Impacted UCs:** UC-007 (status / footer free-space indicator, originally from TASK-004)

## Scope
- `src/disk_space.rs`: add a new `#[cfg(target_os = "macos")]` `platform_impl`
  that calls Darwin's `statfs(2)` (from `<sys/mount.h>`). Reason for choosing
  `statfs` over `statvfs` on Darwin: Darwin's `statvfs` declares
  `fsblkcnt_t = unsigned int` (32-bit), which silently truncates block counts
  on volumes ≥ 16 TiB. Darwin's `statfs` keeps `f_blocks`/`f_bfree`/`f_bavail`
  as `uint64_t` directly. We use the layout published in `<sys/mount.h>` for
  modern macOS and read only the prefix fields up to and including `f_bavail`.
- The catch-all `#[cfg(all(unix, not(target_os = "linux")))]` arm narrows to
  `not(any(target_os = "linux", target_os = "macos"))` so non-macOS BSDs keep
  returning `None` (recorded as a v0.2 follow-up).
- No changes to `format::format_size` or to the footer composition.
- No new crate deps.

## ACs
- AC-1: `cargo check --target x86_64-apple-darwin` succeeds (build-time
  validation; no runtime needed).
- AC-2: On a Linux container the existing test
  `disk_space::tests::*` still passes (no regression).
- AC-3: Footer on macOS/BSD shows a real free-space figure instead of "—".
  (Manual verification deferred — Linux test container does not exercise
  the macOS code path.)

## SEC ACs
- SEC-1: `unsafe` is limited to the `extern "C"` declaration plus the
  single `statvfs(c_path, &mut buf)` call, both already audited by the
  Linux variant. A new `// SAFETY:` block is added when widening the cfg
  if comments diverge per-platform.
- SEC-2: No new code path takes input from the user beyond the existing
  `Path` argument; canonicalisation happens upstream.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
