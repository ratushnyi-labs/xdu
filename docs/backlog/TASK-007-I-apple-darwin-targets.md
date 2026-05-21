# TASK-007-I — add x86_64-apple-darwin + aarch64-apple-darwin via cargo-zigbuild

**Status:** DONE (Dockerfile + scripts wired; full cross-build verification deferred to TASK-007-O)  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
Per D1.1/D1.3 add macOS targets cross-compiled from Linux Docker via
[`cargo-zigbuild`](https://github.com/rust-cross/cargo-zigbuild). Zig bundles a
minimal macOS sysroot (libSystem stubs) that's sufficient for a pure-std + libc
Rust CLI like xdu. No Apple SDK extraction required.

Changes in `Dockerfile`:
- Install `zig` (download a versioned tarball from `https://ziglang.org/download/`).
  Pin the version (e.g. `0.13.0`) for reproducibility.
- `cargo install --locked cargo-zigbuild`.
- `rustup target add x86_64-apple-darwin aarch64-apple-darwin`.
- `build` stage detects `*-apple-darwin` targets and routes through `cargo zigbuild`
  instead of `cargo build`.

Changes in `scripts/build.sh`:
- Default `TARGETS` list gains both apple-darwin triples.

**Risk:** zigbuild's macOS sysroot may not link against frameworks like
`Cocoa` / `AppKit`. xdu only uses std + libc (via crossterm's `termios` calls); no
Apple framework usage is in the dependency tree (`crossterm`, `ratatui` are
framework-free). So linking should succeed.

If linking against macOS frameworks ever becomes necessary, the user can opt to
extract a full SDK from Xcode via `xcode-sdk-extractor` and bind-mount it into the
container. That's recorded as a future option.

## Estimation
**Complexity:** ABOVE_EASY. New zig install + cargo-zigbuild + 2 new rustup targets.

## Speck Reference
[`docs/specks/SDD-TASK-007-I-apple-darwin-targets.md`](../specks/SDD-TASK-007-I-apple-darwin-targets.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
