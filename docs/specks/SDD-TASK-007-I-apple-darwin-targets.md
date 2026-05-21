# SDD-TASK-007-I — Add x86_64 + aarch64 apple-darwin via cargo-zigbuild

**Backlog item:** TASK-007-I
**Impacted UCs:** none directly (build pipeline only)

## Scope
- `Dockerfile`:
  - Install `zig` from a pinned versioned tarball published at https://ziglang.org/download/. Pin version `0.13.0` (record the SHA256 in a Dockerfile comment for reproducibility).
  - `cargo install --locked cargo-zigbuild`.
  - `rustup target add x86_64-apple-darwin aarch64-apple-darwin`.
  - Build stage routes `*-apple-darwin` targets through `cargo zigbuild --release --target …` instead of `cargo build`.
- `scripts/build.sh`: default `TARGETS` list gains `x86_64-apple-darwin` and `aarch64-apple-darwin`.
- `.cargo/config.toml`: no changes — cargo-zigbuild discovers its linker config automatically via `CARGO_TARGET_*_LINKER` env it sets.

## ACs
- AC-1: `TARGETS="x86_64-apple-darwin" scripts/build.sh` produces `dist/x86_64-apple-darwin/xdu` linked as a Mach-O 64-bit executable (verifiable via `file` reporting "Mach-O 64-bit x86_64 executable").
- AC-2: Same as AC-1 for `aarch64-apple-darwin` (`file` reports "Mach-O 64-bit arm64 executable").
- AC-3: Both binaries are within the 1.5 MiB size ceiling.
- AC-4: No Apple SDK is required on the host or in the image (zig bundles its own libSystem stubs).

## SEC ACs
- SEC-1: No new run-time `unsafe`; pure build-time tooling addition.
- SEC-2: `zig` tarball pin recorded with SHA256; `cargo-zigbuild` installed with `--locked`.
- SEC-3: §8.2 host isolation preserved.
- SEC-4: No proprietary Apple SDK redistributed; uses zig's libSystem stubs which are header-only re-implementations.

## §9.5 cross-agent attack review
Skipped per session-wide rule (no second-agent vendor reachable). See parent TASK-007.
