# xdu — Development Manual

Local dev runs **inside Docker**. The host machine never installs Rust,
target toolchains, or test fixtures (project rule §8.2 — host isolation).

## Prerequisites on the host

- Docker 24+ with Compose v2.
- `git` for repository operations.
- A POSIX-ish shell to run `scripts/*.sh`. Git Bash works on Windows.

That is the entire host footprint.

## One-shot dev container

```sh
docker compose run --rm test sh
```

Drops you into a shell at `/src` with the pinned Rust toolchain
(`rust:1.95-bookworm`), all six rustup targets, `cargo`, `clippy`,
`rustfmt`, `cargo-xwin`, `cargo-zigbuild`, and a splatted MSVC SDK
under `/xwin` ready to go. Cargo registry and `target/` are kept in
named volumes so subsequent builds reuse cached crates.

## Common commands inside the dev shell

| Goal                        | Command                                                           |
|-----------------------------|-------------------------------------------------------------------|
| Format code                 | `cargo fmt --all`                                                 |
| Format check                | `cargo fmt --all -- --check`                                      |
| Lint                        | `cargo clippy --all-targets --all-features --locked -- -D warnings` |
| Type-check                  | `cargo check --all-targets --locked`                              |
| Unit + integration tests    | `cargo test --locked`                                             |
| Single test                 | `cargo test --locked some_substring`                              |

## Multi-target release builds

Run from the host (these scripts wrap `docker compose run`):

```sh
sh scripts/ci.sh        # full pipeline: fmt -> clippy -> check -> test -> all-target build
sh scripts/build.sh     # release-min build for every supported target
TARGET=x86_64-unknown-linux-musl docker compose run --rm build  # single target
```

Outputs land in `dist/<TARGET>/xdu(.exe)`. Each binary is checked against
the **1.5 MiB hard ceiling** (configurable via `SIZE_CEILING_BYTES`,
see [`.env.example`](../.env.example)). Build fails if any target exceeds it.

## Directory conventions

| Path             | Purpose                                                       |
|------------------|---------------------------------------------------------------|
| `.dev-data/`     | Reserved for dev-only data volumes (currently unused by xdu). |
| `.dev-temp/`     | Reserved for build-time scratch (currently unused by xdu).    |
| `dist/`          | Released binaries (per-target). Git-ignored.                  |
| `target/`        | Cargo's build cache. Git-ignored. Lives in a Docker volume.   |

## Workflow

1. Pick a leaf from `docs/backlog/TASK-001-*.md`. Read its Speck under
   `docs/specks/SDD-TASK-001-*.md`.
2. Update the leaf's status to `IN PROGRESS`; bump its lifecycle map.
3. Implement code + tests inside the dev container.
4. Run the verifier:
   ```sh
   docker compose run --rm test sh -c \
     "cargo fmt --all && cargo fmt --all -- --check \
      && cargo clippy --all-targets --all-features --locked -- -D warnings \
      && cargo test --locked"
   ```
5. Run a representative release build:
   ```sh
   TARGET=x86_64-unknown-linux-musl docker compose run --rm build
   ```
6. Run the pre-merge baseline security gate (`SECURITY_AUDIT`,
   `DEPENDENCY_TREE_SECURITY_AUDIT`, `MEMORY_LEAK_AUDIT`) per project rule §9.4.
   Record the disposition in the leaf's backlog file.
7. Commit. Commit messages reference the backlog item ID and **MUST NOT**
   include agent attribution or `Co-authored-by:` trailers.
8. After merge, advance the leaf's status to `MERGED → DONE` once any
   end-to-end verification (e.g. multi-target build) confirms its ACs.

## Rust constraints (project rule §5)

- No `unsafe` outside small clearly-justified blocks (e.g. Win32 FFI).
- `cargo clippy` runs with `-D warnings` — treat any warning as an error.
- Max function 30 non-empty / non-comment lines; max class/type 500.
- Max line length 120 chars.
- No `unwrap()` on user-facing failure paths.
- New crate dependencies require an ADR amendment.

## Adding a new module

1. Add a leaf backlog file under `docs/backlog/`.
2. Author its Speck under `docs/specks/`.
3. Declare the module in `src/lib.rs`.
4. Implement in `src/<module>.rs` with `#[cfg(test)] mod tests` colocated.
5. Run the verifier above.
