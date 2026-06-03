# Cross-build smoke-test — 2026-06-03 (post-BUG-005 fix)

**Date:** 2026-06-03 (UTC)
**Wall-clock window:** ~10:33 → ~10:43 UTC (~10 min wall-clock for
   the full 6-target run; the `cross-tools` layer was rebuilt once
   with the corrected `xwin` invocation, then cached for the
   remaining targets).
**Host tools:**
  - Docker Engine: `29.5.2` (build `79eb04c`)
  - Docker Compose: `v5.1.4`
**Container toolchain (per `Dockerfile` `base` stage):** Rust
   `1.95.0` (`rustc 1.95.0 (59807616e 2026-04-14)` / `cargo 1.95.0
   (f2d3ce0bd 2026-03-21)`) with the 6-target `rustup target add`
   set.
**`cross-tools` stage installed:** Zig `0.13.0` (pinned + SHA256
   verified), `cargo-xwin v0.22.0` (locked), `cargo-zigbuild v0.22.3`
   (locked), `xwin v0.9.0` (locked) — same as `cross-build-2026-06-03.md`,
   but with the BUG-005 Dockerfile fix applied to the splat invocation.
**BUG-005 fix summary:** `Dockerfile` lines 88-104 — `xwin v0.9.0`
   moved `--arch` from the `splat` subcommand to a global option
   placed BEFORE the subcommand, and accepts a comma-separated list
   rather than repeated flags. The invocation now reads:

   ```
   ENV XWIN_CACHE_DIR=/xwin
   RUN xwin --accept-license \
           --arch x86_64,aarch64 \
           --timeout 300s \
           --http-retry 3 \
           splat \
           --output /xwin
   ```

   The syntax was verified inside a throwaway `rust:1.95-bookworm`
   container (`xwin --help` / `xwin splat --help`) before the
   Dockerfile edit, per the CRR-1 gate baked into the BUG-005 Speck.

**Backlog:** [BUG-005](../backlog/BUG-005-xwin-splat-arch-flag-removed.md)
**Speck:** [SDD-BUG-005](../specks/SDD-BUG-005-xwin-splat-arch-flag-removed.md)
**Supersedes:** [`cross-build-2026-06-03.md`](./cross-build-2026-06-03.md)
   (the original TASK-007-Q transcript that recorded the 6/6
   foundational failure). That file is preserved as the historical
   record per §1.2.13.5 and is NOT edited by this wrap-up.

## Command run (verbatim)

```sh
TARGETS="x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-msvc aarch64-pc-windows-msvc x86_64-apple-darwin aarch64-apple-darwin" sh scripts/build.sh
```

Stdout/stderr captured to `dist/.smoke-test-after-bug005-fix.log`.
Final line of that log: `all targets built within size budget.`

**Exit code:** `0` (per `scripts/build.sh` — every target produced a
binary that fit the 1.5 MiB ceiling).

**Size ceiling:** `SIZE_CEILING_BYTES=1572864` (1.5 MiB) per
`scripts/build.sh` and ADR-001 §4.

## Per-target results

Each target's `file(1)` and `wc -c` measurements were captured on the
host with the `dist/<triple>/xdu[.exe]` artefact produced by the
in-container build (per §8.2 the build itself runs inside Docker; the
post-build measurement on the host filesystem is observational and
does not touch the toolchain).

### `x86_64-unknown-linux-musl`

- **Status:** PASS
- **Binary path:** `dist/x86_64-unknown-linux-musl/xdu`
- **`file(1)` output:**
  ```
  dist/x86_64-unknown-linux-musl/xdu: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), static-pie linked, stripped
  ```
- **`wc -c` byte size:** `766632`
- **Ceiling check (vs 1.5 MiB / 1572864 B):** PASS (766632 / 1572864
  = ~48.7 % of budget).

### `aarch64-unknown-linux-musl`

- **Status:** PASS
- **Binary path:** `dist/aarch64-unknown-linux-musl/xdu`
- **`file(1)` output:**
  ```
  dist/aarch64-unknown-linux-musl/xdu: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), statically linked, stripped
  ```
- **`wc -c` byte size:** `690840`
- **Ceiling check (vs 1.5 MiB / 1572864 B):** PASS (690840 / 1572864
  = ~43.9 % of budget).

### `x86_64-pc-windows-msvc`

- **Status:** PASS
- **Binary path:** `dist/x86_64-pc-windows-msvc/xdu.exe`
- **`file(1)` output:**
  ```
  dist/x86_64-pc-windows-msvc/xdu.exe: PE32+ executable for MS Windows 6.00 (console), x86-64, 7 sections
  ```
- **`wc -c` byte size:** `441856`
- **Ceiling check (vs 1.5 MiB / 1572864 B):** PASS (441856 / 1572864
  = ~28.1 % of budget).

### `aarch64-pc-windows-msvc`

- **Status:** PASS
- **Binary path:** `dist/aarch64-pc-windows-msvc/xdu.exe`
- **`file(1)` output:**
  ```
  dist/aarch64-pc-windows-msvc/xdu.exe: PE32+ executable for MS Windows 6.00 (console), ARM64, 7 sections
  ```
- **`wc -c` byte size:** `417792`
- **Ceiling check (vs 1.5 MiB / 1572864 B):** PASS (417792 / 1572864
  = ~26.6 % of budget).

### `x86_64-apple-darwin`

- **Status:** PASS
- **Binary path:** `dist/x86_64-apple-darwin/xdu`
- **`file(1)` output:**
  ```
  dist/x86_64-apple-darwin/xdu: Mach-O 64-bit x86_64 executable, flags:<NOUNDEFS|DYLDLINK|TWOLEVEL|NO_REEXPORTED_DYLIBS|PIE|HAS_TLV_DESCRIPTORS>
  ```
- **`wc -c` byte size:** `578312`
- **Ceiling check (vs 1.5 MiB / 1572864 B):** PASS (578312 / 1572864
  = ~36.8 % of budget).

### `aarch64-apple-darwin`

- **Status:** PASS
- **Binary path:** `dist/aarch64-apple-darwin/xdu`
- **`file(1)` output:**
  ```
  dist/aarch64-apple-darwin/xdu: Mach-O 64-bit arm64 executable, flags:<NOUNDEFS|DYLDLINK|TWOLEVEL|NO_REEXPORTED_DYLIBS|PIE|HAS_TLV_DESCRIPTORS>
  ```
- **`wc -c` byte size:** `569984`
- **Ceiling check (vs 1.5 MiB / 1572864 B):** PASS (569984 / 1572864
  = ~36.2 % of budget).

## `/xwin` contents (AC-2 evidence)

The BUG-005 Speck AC-2 requires the `/xwin` cache to contain BOTH
`x86_64` AND `aarch64` arch trees after the corrected `xwin splat`
invocation. Verified by running an interactive shell against the
`build` service (which depends on `cross-tools`):

```
$ docker compose run --rm -e TARGET=x86_64-pc-windows-msvc \
    --entrypoint sh build -c \
    'ls /xwin && echo --- SDK --- && ls /xwin/sdk && echo --- CRT --- && ls /xwin/crt'
crt
sdk
--- SDK ---
Include
Lib
include
lib
--- CRT ---
include
lib
```

Drilling into the arch-partitioned `lib` directories confirms both
target architectures were splatted:

```
$ docker compose run --rm -e TARGET=x86_64-pc-windows-msvc \
    --entrypoint sh build -c \
    'echo === CRT lib === && ls /xwin/crt/lib && echo === SDK lib um === && ls /xwin/sdk/lib/um'
=== CRT lib ===
aarch64
x86_64
=== SDK lib um ===
aarch64
x86_64
```

Both `aarch64` and `x86_64` subtrees are present under
`/xwin/crt/lib` and `/xwin/sdk/lib/um`. AC-2 satisfied. The
successful build of both `x86_64-pc-windows-msvc` and
`aarch64-pc-windows-msvc` (sections above) provides downstream
confirmation that the splatted SDK is consumable by `cargo xwin
build` for each arch.

## Summary

| Target                         | Status | Binary path                                    | Byte size | Ceiling check (≤ 1.5 MiB) |
|--------------------------------|--------|------------------------------------------------|-----------|---------------------------|
| `x86_64-unknown-linux-musl`    | PASS   | `dist/x86_64-unknown-linux-musl/xdu`           | 766632    | PASS                      |
| `aarch64-unknown-linux-musl`   | PASS   | `dist/aarch64-unknown-linux-musl/xdu`          | 690840    | PASS                      |
| `x86_64-pc-windows-msvc`       | PASS   | `dist/x86_64-pc-windows-msvc/xdu.exe`          | 441856    | PASS                      |
| `aarch64-pc-windows-msvc`      | PASS   | `dist/aarch64-pc-windows-msvc/xdu.exe`         | 417792    | PASS                      |
| `x86_64-apple-darwin`          | PASS   | `dist/x86_64-apple-darwin/xdu`                 | 578312    | PASS                      |
| `aarch64-apple-darwin`         | PASS   | `dist/aarch64-apple-darwin/xdu`                | 569984    | PASS                      |

**Aggregate:** 6 / 6 PASS, 0 / 6 FAIL, 0 / 6 OVERSIZE — all 6 within
1.5 MiB budget. Largest artefact is `x86_64-unknown-linux-musl/xdu`
at 766632 B (~48.7 % of the 1572864-byte ceiling), still leaving
~787 KiB of headroom against the worst case.

## Pre-DONE gate (§12.8.1)

Run inside the `test` service per §8.2:

```sh
docker compose run --rm test sh -c \
    'cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --all --locked'
```

- `cargo fmt --all -- --check` — clean (no diff).
- `cargo clippy --all-targets --locked -- -D warnings` — clean (no
  warnings, no errors).
- `cargo test --all --locked` — 106 / 106 PASS:
  - lib unit tests: 97 passed
  - bin unit tests: 0 passed
  - `integration_test`: 6 passed
  - `snapshot_test`: 3 passed
  - doc-tests: 0 passed

Gate result: GREEN. The `cross-tools` layer change is isolated to the
Windows-MSVC drivers and does not regress the base test surface.

## §8.2 host-isolation compliance

All builds, the pre-DONE gate, and the `/xwin` inspection were run
through `docker compose build` / `docker compose run`. The only
host-side commands were observational `file(1)` and `wc -c` against
already-produced artefacts under `dist/`, plus `docker --version` /
`docker compose version`. No `cargo build` / `cargo test` was invoked
on the host. Compliance preserved.

## §9.5 cross-agent attack review

Skipped per session-wide rule (no second-agent vendor reachable; user
2026-05-06).

## Provenance

- BUG-005 backlog: [`../backlog/BUG-005-xwin-splat-arch-flag-removed.md`](../backlog/BUG-005-xwin-splat-arch-flag-removed.md)
- BUG-005 Speck: [`../specks/SDD-BUG-005-xwin-splat-arch-flag-removed.md`](../specks/SDD-BUG-005-xwin-splat-arch-flag-removed.md)
- Original (failing) TASK-007-Q transcript that this transcript
  supersedes: [`./cross-build-2026-06-03.md`](./cross-build-2026-06-03.md)
- Fix commit: see commit message for the BUG-005 DONE commit.
