# Cross-build smoke-test — 2026-06-03

**Date:** 2026-06-03 (UTC)
**Wall-clock window:** 09:43:53 → 09:47:18 UTC (~3 min 25 s; the
   `cross-tools` layer cache held after the first failure so each
   subsequent target re-hit the broken layer in seconds)
**Host tools:**
  - Docker Engine: `29.5.2`
  - Docker Compose: `v5.1.4`
  - Host rustc / cargo (informational; builds run inside containers
    per §8.2): `cargo 1.93.1 (083ac5135 2025-12-15)` /
    `rustc 1.93.1 (01f6ddf75 2026-02-11)`
**Container toolchain (per `Dockerfile` `base` stage):** Rust
   `1.95-bookworm` with the 6-target `rustup target add` set.
**`cross-tools` stage installed:** Zig `0.13.0` (pinned + SHA256
   verified), `cargo-xwin v0.22.0` (locked), `cargo-zigbuild v0.22.3`
   (locked), `xwin v0.9.0` (locked).
**Backlog:** [TASK-007-Q](../backlog/TASK-007-Q-cross-build-smoke-test.md)
**Speck:** [SDD-TASK-007-Q](../specks/SDD-TASK-007-Q-cross-build-smoke-test.md)

## Command run (verbatim)

```sh
TARGETS="x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-msvc aarch64-pc-windows-msvc x86_64-apple-darwin aarch64-apple-darwin" sh scripts/build.sh
```

Stdout/stderr captured to `dist/.smoke-test.log` (the script's own
narrative). The verbose Docker BuildKit stderr — which `scripts/build.sh`
silences with `>/dev/null 2>&1` on line 20 — was re-captured by direct
`TARGET=<t> docker compose build build` invocations for the first
target (`dist/.smoke-t1-build.log`, ~35 KB) and a cross-check target
(`dist/.smoke-t6-build.log`, ~3.4 KB) so the failure stderr is
preserved verbatim in this transcript.

**Exit code:** `1` (per `scripts/build.sh` final exit when any target
failed).

**Size ceiling:** `SIZE_CEILING_BYTES=1572864` (1.5 MiB) per
`scripts/build.sh` and ADR-001 §4. Not exercised — no binaries
produced.

## Foundational failure (shared root cause for all 6 targets)

All 6 per-target sections below fail at the same shared Docker
stage — `[cross-tools 4/4]`. The `cross-tools` stage is a dependency
of the `build` stage for every target, so each `TARGET=<t> docker
compose build build` invocation re-hits the same broken layer.
Verbatim stderr (from `dist/.smoke-t1-build.log` lines 138-149,
identical in `dist/.smoke-t6-build.log` lines 7-18):

```
#17 [cross-tools 4/4] RUN xwin --accept-license splat \
        --output /xwin \
        --arch x86_64 \
        --arch aarch64
#17 0.235 error: unexpected argument '--arch' found
#17 0.235
#17 0.235 Usage: xwin splat --output <OUTPUT>
#17 0.235
#17 0.235 For more information, try '--help'.
#17 ERROR: process "/bin/sh -c xwin --accept-license splat \
        --output /xwin --arch x86_64 --arch aarch64" did not
        complete successfully: exit code: 2
------
 > [cross-tools 4/4] RUN xwin --accept-license splat \
        --output /xwin --arch x86_64 --arch aarch64:
0.235 error: unexpected argument '--arch' found
0.235
0.235 Usage: xwin splat --output <OUTPUT>
0.235
0.235 For more information, try '--help'.
------
failed to solve: process "/bin/sh -c xwin --accept-license splat \
    --output /xwin --arch x86_64 --arch aarch64" did not complete
    successfully: exit code: 2
```

`xwin v0.9.0` no longer accepts the `--arch` flag on the `splat`
subcommand; `Dockerfile` lines 91-95 use it. Root-cause diagnosis and
remediation are out of scope for Q per §1.2.13.5; they are tracked in
[BUG-005](../backlog/BUG-005-xwin-splat-arch-flag-removed.md).

## Per-target results

### `x86_64-unknown-linux-musl`

- **Status:** FAIL (image build)
- **Failing Docker stage:** `[cross-tools 4/4]` —
  `xwin --accept-license splat --output /xwin --arch x86_64 --arch aarch64`
- **Last ~50 lines of stderr:** see *Foundational failure* above
  (verbatim from `dist/.smoke-t1-build.log`).
- **Binary path expected:** `dist/x86_64-unknown-linux-musl/xdu`
- **Binary present:** no
- **`file(1)` output:** N/A (no binary produced)
- **`wc -c` byte size:** N/A
- **Ceiling check (vs 1.5 MiB / 1572864 B):** N/A — no binary to
  measure.
- **Follow-up bug:**
  [BUG-005](../backlog/BUG-005-xwin-splat-arch-flag-removed.md).

### `aarch64-unknown-linux-musl`

- **Status:** FAIL (image build)
- **Failing Docker stage:** `[cross-tools 4/4]` — same as above
  (shared layer).
- **Last ~50 lines of stderr:** identical to *Foundational failure*;
  BuildKit served the previously-failed layer from cache and the
  invocation returned in milliseconds.
- **Binary path expected:** `dist/aarch64-unknown-linux-musl/xdu`
- **Binary present:** no
- **`file(1)` output:** N/A
- **`wc -c` byte size:** N/A
- **Ceiling check (vs 1.5 MiB / 1572864 B):** N/A
- **Follow-up bug:**
  [BUG-005](../backlog/BUG-005-xwin-splat-arch-flag-removed.md).

### `x86_64-pc-windows-msvc`

- **Status:** FAIL (image build)
- **Failing Docker stage:** `[cross-tools 4/4]` — shared layer.
- **Last ~50 lines of stderr:** identical to *Foundational failure*.
- **Binary path expected:** `dist/x86_64-pc-windows-msvc/xdu.exe`
- **Binary present:** no
- **`file(1)` output:** N/A
- **`wc -c` byte size:** N/A
- **Ceiling check (vs 1.5 MiB / 1572864 B):** N/A
- **Follow-up bug:**
  [BUG-005](../backlog/BUG-005-xwin-splat-arch-flag-removed.md).

### `aarch64-pc-windows-msvc`

- **Status:** FAIL (image build)
- **Failing Docker stage:** `[cross-tools 4/4]` — shared layer.
- **Last ~50 lines of stderr:** identical to *Foundational failure*.
- **Binary path expected:** `dist/aarch64-pc-windows-msvc/xdu.exe`
- **Binary present:** no
- **`file(1)` output:** N/A
- **`wc -c` byte size:** N/A
- **Ceiling check (vs 1.5 MiB / 1572864 B):** N/A
- **Follow-up bug:**
  [BUG-005](../backlog/BUG-005-xwin-splat-arch-flag-removed.md).

### `x86_64-apple-darwin`

- **Status:** FAIL (image build)
- **Failing Docker stage:** `[cross-tools 4/4]` — shared layer.
- **Last ~50 lines of stderr:** identical to *Foundational failure*.
- **Binary path expected:** `dist/x86_64-apple-darwin/xdu`
- **Binary present:** no
- **`file(1)` output:** N/A
- **`wc -c` byte size:** N/A
- **Ceiling check (vs 1.5 MiB / 1572864 B):** N/A
- **Follow-up bug:**
  [BUG-005](../backlog/BUG-005-xwin-splat-arch-flag-removed.md).

### `aarch64-apple-darwin`

- **Status:** FAIL (image build)
- **Failing Docker stage:** `[cross-tools 4/4]` — shared layer.
- **Last ~50 lines of stderr:** verbatim re-confirmation captured
  separately in `dist/.smoke-t6-build.log` (a direct
  `TARGET=aarch64-apple-darwin docker compose build build` invocation)
  to demonstrate the failure is not specific to the linux-musl driver.
  Stderr is identical to *Foundational failure* above.
- **Binary path expected:** `dist/aarch64-apple-darwin/xdu`
- **Binary present:** no
- **`file(1)` output:** N/A
- **`wc -c` byte size:** N/A
- **Ceiling check (vs 1.5 MiB / 1572864 B):** N/A
- **Follow-up bug:**
  [BUG-005](../backlog/BUG-005-xwin-splat-arch-flag-removed.md).

## Summary

| Target                         | Status         | Binary path                                       | Byte size | Ceiling check (≤ 1.5 MiB) |
|--------------------------------|----------------|---------------------------------------------------|-----------|---------------------------|
| `x86_64-unknown-linux-musl`    | FAIL (image)   | `dist/x86_64-unknown-linux-musl/xdu` (missing)    | N/A       | N/A                       |
| `aarch64-unknown-linux-musl`   | FAIL (image)   | `dist/aarch64-unknown-linux-musl/xdu` (missing)   | N/A       | N/A                       |
| `x86_64-pc-windows-msvc`       | FAIL (image)   | `dist/x86_64-pc-windows-msvc/xdu.exe` (missing)   | N/A       | N/A                       |
| `aarch64-pc-windows-msvc`      | FAIL (image)   | `dist/aarch64-pc-windows-msvc/xdu.exe` (missing)  | N/A       | N/A                       |
| `x86_64-apple-darwin`          | FAIL (image)   | `dist/x86_64-apple-darwin/xdu` (missing)          | N/A       | N/A                       |
| `aarch64-apple-darwin`         | FAIL (image)   | `dist/aarch64-apple-darwin/xdu` (missing)         | N/A       | N/A                       |

**Aggregate:** 0 / 6 PASS, 6 / 6 FAIL, 0 / 6 OVERSIZE. No binaries
produced, so the 1.5 MiB ceiling was not exercised.

## Follow-up bugs

- [BUG-005 — `cross-tools` Dockerfile stage fails: `xwin splat` no
  longer accepts `--arch`](../backlog/BUG-005-xwin-splat-arch-flag-removed.md)
  — single umbrella bug covers all 6 target failures because the
  failing step is in the shared `cross-tools` Docker stage. Filed at
  `INTAKE` per §1.2.13.5; no diagnosis or remediation attempted in
  this leaf.

## §8.2 host-isolation compliance

All build attempts were issued through `docker compose build` /
`docker compose run` per `scripts/build.sh`. No `cargo build` was
invoked on the host. Compliance preserved.

## §9.5 cross-agent attack review

Skipped per session-wide rule.
