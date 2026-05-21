# SDD-TASK-007-N8a — Switch Windows target gnu → msvc via cargo-xwin

**Backlog item:** TASK-007-N8a
**Impacted UCs:** none directly (build pipeline + runtime ABI choice)

## Scope
- `Dockerfile`:
  - `apt-get install` adds `clang lld llvm` (xwin uses lld + llvm tools).
  - `rustup target add` drops `x86_64-pc-windows-gnu`, adds `x86_64-pc-windows-msvc`.
  - `cargo install --locked cargo-xwin`.
  - Populate MSVC SDK cache: `RUN xwin --accept-license splat --output /xwin --arch x86_64`. Set `XWIN_CACHE_DIR=/xwin` and `CC_x86_64_pc_windows_msvc=clang-cl`, `CXX_x86_64_pc_windows_msvc=clang-cl`, `AR_x86_64_pc_windows_msvc=llvm-lib` as env where required.
  - Build stage detects `*windows-msvc*` and uses `cargo xwin build --release --target "${TARGET}"` (xwin builds wrap cargo and inject xwin's `XWIN_*` config).
  - Drop `gcc-mingw-w64-x86-64` from apt list (no longer needed; gnu toolchain retired).
- `.cargo/config.toml`: remove `[target.x86_64-pc-windows-gnu]` linker entry.
- `scripts/build.sh`: default `TARGETS` list swaps `x86_64-pc-windows-gnu` → `x86_64-pc-windows-msvc`.
- README & docs: update target name everywhere (`README.md`, `docs/INDEX.md`, ADR-001 references).
- License acceptance: `--accept-license` flag is the documented programmatic acceptance of the Microsoft MSVC EULA per xwin docs. A comment in the Dockerfile records this acceptance + the SHA / version of xwin used.

## ACs
- AC-1: `docker compose build dev` succeeds with the new image (xwin downloads + splats SDK).
- AC-2: `TARGETS="x86_64-pc-windows-msvc" scripts/build.sh` produces `dist/x86_64-pc-windows-msvc/xdu.exe` linked against the MSVC C runtime (verifiable via `file` and presence of `MSVCR*.dll`/UCRT imports).
- AC-3: `x86_64-pc-windows-gnu` produces no entry in the new toolchain; CI's old triple name is removed everywhere.
- AC-4: Binary size for `xdu.exe` remains under the 1.5 MiB ceiling (or, if breached, recorded as a §11 EXC with rationale — MSVC import table is slightly larger than mingw).
- AC-5: Dockerfile has a comment block citing the MSVC EULA URL + the `--accept-license` rationale.

## SEC ACs
- SEC-1: No new run-time `unsafe`; ABI switch does not introduce new code.
- SEC-2: `cargo-xwin` and `xwin` are both vendored from public registries; pinned versions are recorded in the Dockerfile via `cargo install --locked` and explicit xwin version (`--version X.Y.Z`).
- SEC-3: §8.2 host isolation preserved — xwin runs only inside the image.
- SEC-4: MSVC SDK is cached inside `/xwin` in the image layer; never extracted to host.

## §9.5 cross-agent attack review
Skipped per session-wide rule (no second-agent vendor reachable). See parent TASK-007.
