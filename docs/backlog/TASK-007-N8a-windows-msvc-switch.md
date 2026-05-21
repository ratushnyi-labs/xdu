# TASK-007-N8a — switch Windows target gnu → msvc via cargo-xwin

**Status:** DONE (Dockerfile + scripts wired; full cross-build verification deferred to TASK-007-O)  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
Per D1.1/D1.3 the Windows target switches from `x86_64-pc-windows-gnu` to
`x86_64-pc-windows-msvc`, cross-compiled from Linux Docker via
[`cargo-xwin`](https://github.com/rust-cross/cargo-xwin). xwin downloads the MSVC
headers/libs from Microsoft's installer at image build time and stores them under
`XWIN_CACHE_DIR=/xwin`. Then `cargo xwin build --release --target
x86_64-pc-windows-msvc` produces a Windows binary linked against the MSVC C runtime.

Changes in `Dockerfile`:
- `apt-get install` adds `llvm` + `clang` (xwin uses lld/llvm-dlltool).
- `rustup target add` swaps `x86_64-pc-windows-gnu` for `x86_64-pc-windows-msvc`.
- `cargo install --locked cargo-xwin`.
- New `RUN cargo xwin --accept-license -- download --arch x86_64` (or equivalent: `xwin --accept-license splat ...`) to populate the SDK cache; the Microsoft EULA acceptance is recorded in the Dockerfile comment.
- The `build` stage detects `*windows-msvc*` targets and routes through `cargo xwin build` instead of `cargo build`.
- `.cargo/config.toml` removes the `x86_64-pc-windows-gnu` linker entry.

Changes in `scripts/build.sh`:
- Default `TARGETS` list updated: `x86_64-pc-windows-gnu` → `x86_64-pc-windows-msvc`.

## Risk
xwin downloads ~1–2 GB of MSVC headers on the first image build. This is acceptable
for a docker layer that caches. License: Microsoft's MSVC EULA. We accept it
programmatically via `--accept-license` per xwin documentation.

## Estimation
**Complexity:** ABOVE_EASY. Multi-file change with new tooling dependency.

## Speck Reference
[`docs/specks/SDD-TASK-007-N8a-windows-msvc-switch.md`](../specks/SDD-TASK-007-N8a-windows-msvc-switch.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
