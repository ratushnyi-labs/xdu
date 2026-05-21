# SDD-TASK-007-N7 — .github/workflows/ci.yml (native-runner matrix, 6 targets)

**Backlog item:** TASK-007-N7
**Impacted UCs:** none directly (CI scaffolding)

## Scope
- New file `.github/workflows/ci.yml`. Triggers:
  - `on: push:` branches `main`, `feat/**`, `fix/**`, `task/**`.
  - `on: pull_request:` (default branches).
- Job `test`:
  - `runs-on: ubuntu-latest`.
  - Steps: `actions/checkout@v4` → `docker compose build dev` → `docker compose run --rm dev sh -c "cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test --all --locked"`.
  - This mirrors the local dev gate verbatim.
- Job `build`:
  - `needs: test`.
  - `strategy.matrix.include:` 6 entries pairing each target with its native runner:
    - `x86_64-unknown-linux-musl` → `ubuntu-latest`
    - `aarch64-unknown-linux-musl` → `ubuntu-latest` (uses `actions/setup-rust` cross via `target add` + `apt-get install gcc-aarch64-linux-gnu` linker)
    - `x86_64-apple-darwin` → `macos-latest` (Intel; x86_64 runners are still available on macOS-13/14 hosted runners — pin `macos-13` if necessary)
    - `aarch64-apple-darwin` → `macos-14` (Apple Silicon)
    - `x86_64-pc-windows-msvc` → `windows-latest`
    - `aarch64-pc-windows-msvc` → `windows-latest` (cross-compile with `rustup target add` + native MSVC toolchain).
  - Steps: `actions/checkout@v4` → `dtolnay/rust-toolchain@stable` (with `targets: ${{ matrix.target }}`) → `cargo build --release --target ${{ matrix.target }} --locked` → `actions/upload-artifact@v4` with `name: xdu-${{ matrix.target }}`, `path: target/${{ matrix.target }}/release/xdu*`, `retention-days: 7`.
- Workflow `name: ci`. Concurrency group `ci-${{ github.ref }}` with `cancel-in-progress: true` (per org convention).

## ACs
- AC-1: Push to a `feat/*` branch fires `ci`; both `test` and all 6 `build` jobs run.
- AC-2: `test` job runs the same gate as `docker compose run --rm dev sh -c "fmt + clippy + test"` and fails the PR on any violation.
- AC-3: Each `build` job uploads its native binary as an artifact under name `xdu-<TARGET>`.
- AC-4: A red `test` short-circuits all `build` jobs (via `needs: test`).
- AC-5: Workflow YAML passes `actionlint` (no syntax errors).

## SEC ACs
- SEC-1: All actions pinned to a major version (`@v4`, `@stable`). No third-party action outside `actions/*` and `dtolnay/rust-toolchain`.
- SEC-2: No secrets accessed in `ci.yml` (no push to remote registries, no release uploads).
- SEC-3: §8.2 — Docker test path used for `test` job (mirrors local). Native runners used only for matrix builds, which produce binaries (no test execution outside container).
- SEC-4: `permissions:` block set to `contents: read` only.

## §9.5 cross-agent attack review
Skipped per session-wide rule (no second-agent vendor reachable). See parent TASK-007.
