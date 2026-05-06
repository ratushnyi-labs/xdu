# SDD-TASK-001-D — Docker build pipeline

**Backlog item:** TASK-001-D
**Impacted UCs:** all (build/test infrastructure for every UC)
**Impacted BR/WF:** —

## Scope / Non-goals

**In scope:**
- `Dockerfile.build` — multi-stage:
  - `base` — Rust toolchain + cross-compile toolchains (mingw-w64) + rustup targets added
  - `build` — copies source, runs `cargo build --profile release-min --target $TARGET`, places artifact under `/out/$TARGET/xdu(.exe)`
  - `artifact` — `FROM scratch` final stage with just the binary at root
- `docker-compose.yml` — services:
  - `build` — invokes `Dockerfile.build` `target: build`, mounts `./out:/out_host`, copies binary out
  - `dev` — `target: base`, mounts source tree, runs `cargo` commands interactively (used by CI script)
- `.cargo/config.toml` — per-target linker selection (rust-lld for musl, mingw gcc for windows-gnu)
- `scripts/ci.sh` — full CI simulation per lawbook §8.7: fmt → clippy → check → test → multi-target release builds → size report
- `scripts/build.sh` — multi-target release build loop
- `.gitattributes` — enforce LF for shell scripts (Windows-host safety)
- `Cargo.lock` — committed (binary crate convention)
- `.env.example` — declares the override-able build vars (`TARGET`)

**Out of scope:**
- `scripts/ci.sh` runs `cargo test` but no test source exists yet → `cargo test` will compile zero tests and pass trivially. Real tests land in TASK-001-M.
- macOS targets (osxcross) — recorded as a follow-up; the pipeline supports `TARGET=*-apple-darwin` but the Docker image does not bundle the SDK by default. AC-4 documents this limitation.
- CI/CD provider integration (GitHub Actions / GitLab CI / etc.).
- Release publishing automation.
- mold linker — deferred (build-speed optimization; not a binary-size optimization per ADR §4).

## Acceptance Criteria

- AC-1: `TARGET=x86_64-unknown-linux-musl docker compose run --rm build` produces `./out/x86_64-unknown-linux-musl/xdu` ≤ **1.5 MB** stripped (size budget per ADR-001 §4).
- AC-2: `TARGET=aarch64-unknown-linux-musl docker compose run --rm build` produces `./out/aarch64-unknown-linux-musl/xdu`.
- AC-3: `TARGET=x86_64-pc-windows-gnu docker compose run --rm build` produces `./out/x86_64-pc-windows-gnu/xdu.exe`.
- AC-4: macOS targets either build successfully (if osxcross SDK is mounted) or fail with a clear, documented error message; the failure is non-blocking for v0.1.0.
- AC-5: `scripts/ci.sh` exits 0 on a clean repo and runs in fail-fast order: fmt → clippy → check → test → multi-target build.
- AC-6: All container images base on a pinned digest or version tag — no `:latest`.
- AC-7: Final binary in `out/` is fully static for Linux musl targets (verifiable via `file out/x86_64-unknown-linux-musl/xdu` showing `statically linked`).
- AC-8: First successful build also retires TASK-001-C AC-1 (scaffold compiles) and AC-5 (binary prints `xdu v0.1.0`).
- AC-9: Build is reproducible: identical source + identical Docker image yield byte-identical binaries (best-effort; fully reproducible Rust builds may require Phase-3 work — record any gap in a follow-up).

## Security ACs

- SEC-1: No secrets in any image layer (image scanned by `docker history`; no env/file references to credentials).
- SEC-2: Base images pinned to specific tags (`rust:1.85-bookworm`, not `rust:latest`).
- SEC-3: `apt-get` runs use `--no-install-recommends` and clean apt lists in same RUN to minimize image size + attack surface.
- SEC-4: `.dockerignore` excludes `.git/`, secrets, `.env*` (except `.env.example`), and dev volumes.
- SEC-5: Build does not require `--privileged` or extra capabilities.
- SEC-6: No network calls during build other than crates.io (no deps yet) and apt repository — both required for build itself.

## Failure Modes / Error Mapping

- Cross-toolchain missing for a target → user receives the cargo error pointing at the linker; `scripts/build.sh` continues to the next target with a non-zero summary at the end.
- Insufficient disk space for build cache → docker build fails with clear apt/cargo error; not masked.
- Binary > 1.5 MB ceiling → `scripts/build.sh` reports size and **exits non-zero** to enforce ADR-001 §4 budget.
- macOS target requested without osxcross → pipeline returns a clearly-prefixed `[xdu] macOS SDK not present in image; see docs/development-manual.md` error.

## Test Matrix

| AC    | Unit | Integration | Build (compose) | Smoke | Production |
|-------|------|-------------|-----------------|-------|------------|
| AC-1  | —    | —           | ✅              | ✅    | —          |
| AC-2  | —    | —           | ✅              | —     | —          |
| AC-3  | —    | —           | ✅              | —     | —          |
| AC-4  | —    | —           | manual          | —     | —          |
| AC-5  | —    | —           | ✅              | —     | —          |
| AC-6  | —    | —           | static-check    | —     | —          |
| AC-7  | —    | —           | ✅ (`file` check)| —    | —          |
| AC-8  | —    | ✅          | ✅              | ✅    | —          |
| AC-9  | —    | —           | manual / Phase-3| —     | —          |
| SEC-1 | —    | —           | static-check    | —     | —          |
| SEC-2 | —    | —           | static-check    | —     | —          |
| SEC-3 | —    | —           | static-check    | —     | —          |
| SEC-4 | —    | —           | static-check    | —     | —          |
| SEC-5 | —    | —           | manual          | —     | —          |
| SEC-6 | —    | —           | manual          | —     | —          |

"Smoke" = `docker compose run --rm build` followed by executing the produced binary inside a `scratch`/`alpine` test container to confirm it prints `xdu v0.1.0`.

## Pinning

- Base image: `rust:1.85-bookworm`. The toolchain pin lives **in the Docker image tag**, not in a `rust-toolchain.toml` file. Rationale: `rust-toolchain.toml` with `channel = "stable"` causes rustup to fetch whatever the latest stable is when the build runs, defeating the image-tag pin and discarding pre-installed cross-targets. With `rust-toolchain.toml` deleted, builds use the toolchain bundled with the image, deterministically.
- `Cargo.lock` committed — single line of metadata + `xdu` package; deterministic.
