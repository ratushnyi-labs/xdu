# BUG-005 — `cross-tools` Dockerfile stage fails: `xwin splat` no longer accepts `--arch`

**Status:** INTAKE  **Type:** BUG
**Created:** 2026-06-03  **Updated:** 2026-06-03

## Raw observation (verbatim)

Surfaced by the TASK-007-Q 6-target cross-build smoke-test (see
`docs/audits/cross-build-2026-06-03.md`). Filed per §1.2.13.5
mid-task-discovery — DEV recorded the failure and continued the
smoke-test for the remaining targets without attempting any fix.

Smoke-test invocation (run on the host, 2026-06-03 09:47:03 UTC):

```
TARGETS="x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-msvc aarch64-pc-windows-msvc x86_64-apple-darwin aarch64-apple-darwin" sh scripts/build.sh
```

Per-target outcome (all 6 fail identically — the `cross-tools` Docker
stage is shared by every target so each `docker compose build build`
re-hits the same broken layer):

```
==> Building x86_64-unknown-linux-musl
    [error] image build failed for x86_64-unknown-linux-musl
==> Building aarch64-unknown-linux-musl
    [error] image build failed for aarch64-unknown-linux-musl
==> Building x86_64-pc-windows-msvc
    [error] image build failed for x86_64-pc-windows-msvc
==> Building aarch64-pc-windows-msvc
    [error] image build failed for aarch64-pc-windows-msvc
==> Building x86_64-apple-darwin
    [error] image build failed for x86_64-apple-darwin
==> Building aarch64-apple-darwin
    [error] image build failed for aarch64-apple-darwin
==> Build summary
    failed targets: x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-msvc aarch64-pc-windows-msvc x86_64-apple-darwin aarch64-apple-darwin
```

`scripts/build.sh` swallows the `docker compose build` stderr with
`>/dev/null 2>&1` (line 20), so the per-target log line above is just
the script's own diagnostic. The verbose stderr was captured via a
direct `TARGET=... docker compose build build` invocation (see
`dist/.smoke-t1-build.log` and `dist/.smoke-t6-build.log`):

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
    --output /xwin --arch x86_64 --arch aarch64" did not complete
    successfully: exit code: 2
------
failed to solve: process "/bin/sh -c xwin --accept-license splat \
    --output /xwin --arch x86_64 --arch aarch64" did not complete
    successfully: exit code: 2
```

Container-side step that fails: `cross-tools 4/4`, command verbatim:

```
xwin --accept-license splat --output /xwin --arch x86_64 --arch aarch64
```

Source of the offending command: `Dockerfile` lines 91-95.

`xwin` version installed during the `cross-tools 3/4` step
immediately above (line 137.4 of `dist/.smoke-t1-build.log`):

```
Installing /usr/local/cargo/bin/xwin
Installed package `xwin v0.9.0` (executable `xwin`)
```

The `xwin v0.9.0` CLI no longer recognises the `--arch` flag on the
`splat` subcommand. Its current `--help` output is just
`Usage: xwin splat --output <OUTPUT>` — architecture selection moved
to a different surface in `xwin >=0.7` (likely a global
`--arch <arch>[,...]` placed before the subcommand, not after; not
verified here per single-task discipline).

## Impact

All 6 release-matrix targets fail at the `cross-tools` Docker stage.
The `release.yml` GitHub Actions workflow on a v0.1.0 tag would
publish ZERO artefacts — every job hits the same broken layer.

## Out of scope for this INTAKE file

Per §1.2.13.5 the DEV-subagent did NOT diagnose root cause, propose
a fix, pin `xwin` to an older version, or edit `Dockerfile`. That is a
separate refinement leaf.

## Refined Description

_pending refinement_

## Estimation

_pending refinement_

## Speck Reference

_pending refinement_
