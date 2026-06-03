# BUG-005 — `cross-tools` Dockerfile stage fails: `xwin splat` no longer accepts `--arch`

**Status:** DONE  **Type:** BUG
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

## Active-item notes

| Timestamp           | Verbatim user note |
|---------------------|--------------------|
| 2026-06-03          | _"b"_ — answer to the supervisor's two-path question. **Fix path (b): update the Dockerfile command to xwin 0.9.0's new syntax** (not path (a), which would have pinned xwin to a known-good older version). DEV must verify the actual 0.9.0 syntax from `xwin --help` / changelog inside a container BEFORE editing the Dockerfile — no guessing per CRR-1. |

## Atomic Check

ATOMIC — one regression (xwin 0.9.0 dropped `--arch` from the `splat`
subcommand) → one mechanical Dockerfile fix. No decomposition.

## Refined Description

**Problem.** `xwin v0.9.0` (the latest version, installed unpinned via
`cargo install --locked xwin` in `Dockerfile` line 86) removed the
`--arch` flag from its `splat` subcommand. The wave-4 `Dockerfile`
lines 91-95 invoke:

```
ENV XWIN_CACHE_DIR=/xwin
RUN xwin --accept-license splat \
        --output /xwin \
        --arch x86_64 \
        --arch aarch64
```

which fails inside the shared `cross-tools 4/4` layer with
`error: unexpected argument '--arch' found`. Because all six
release-matrix targets share the `cross-tools` stage, every one of
them dies at the same layer — see `docs/audits/cross-build-2026-06-03.md`.
On a v0.1.0 tag, `release.yml` would publish zero artefacts.

**Fix path (user 2026-06-03 active-item note: "b").** Update the
`Dockerfile` command to xwin 0.9.0's actual current syntax. **Do NOT
pin** xwin to an older version (path (a) is rejected); the
`cargo install --locked xwin` line on Dockerfile line 86 stays
unpinned.

**Verification gate (CRR-1, mandatory before any Dockerfile edit).**
DEV MUST discover the real 0.9.0 syntax — not guess it — by one of:

- `docker compose run --rm test sh -c 'xwin --help && xwin splat --help'`
  inside a throwaway container, OR
- reading the published xwin 0.9.0 release notes / changelog / README /
  source on crates.io or the upstream repo.

The discovered syntax (with its source URL or the verbatim `--help`
output) MUST be quoted in the fix commit message. No edit may be
proposed before that artefact exists.

**Definition of done.** The 6-target smoke test
`TARGETS="x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-msvc aarch64-pc-windows-msvc x86_64-apple-darwin aarch64-apple-darwin" sh scripts/build.sh`
reproduces 6/6 PASS; a fresh transcript at
`docs/audits/cross-build-2026-06-03-after-bug005-fix.md` supersedes
the TASK-007-Q transcript and records per-target `file(1)` + `wc -c` +
ceiling check (every binary ≤ 1.5 MiB or explicitly flagged oversize).

## Estimation

**EASY** if xwin 0.9.0 only renamed the surface (e.g. `--arch` moved
to a comma-separated value, or to a global flag placed before `splat`)
— a one- or two-line `Dockerfile` diff, mechanical.

**ABOVE_EASY** if 0.9.0 restructured the splat surface further (e.g.
splatting per arch now requires separate invocations, or a new
subcommand replaces `splat`). In that case the layer's shape may need
to change (still single layer, but multi-step `RUN`), and DEV should
verify the resulting `/xwin` cache still contains both arches.

Both branches stay within a single atomic leaf — the choice is made
after the verification gate produces the actual help text.

## Speck Reference

[`docs/specks/SDD-BUG-005-xwin-splat-arch-flag-removed.md`](../specks/SDD-BUG-005-xwin-splat-arch-flag-removed.md)

## Subagent Spawn Log

| Time (UTC) | Role | Purpose | Status | Notes |
|------------|------|---------|--------|-------|
| 2026-06-03 | BA   | Refine + author Speck per user-confirmed path (b) | SUCCESS | 4 ACs + 3 SEC ACs; atomic verdict ATOMIC; CRR-1 verify-syntax-before-edit gate baked into Speck; estimation EASY (or ABOVE_EASY if 0.9.0 restructured `splat` further) |
| 2026-06-03 | DEV  | Apply Dockerfile fix per Speck; rerun Q smoke-test | SUCCESS | Verified xwin 0.9.0 syntax via throwaway `rust:1.95-bookworm` container; Dockerfile lines 92-104 updated to global `--arch x86_64,aarch64` before `splat`; 6/6 PASS smoke-test (max binary 766632 B for `x86_64-unknown-linux-musl` ≪ 1572864 B = 1.5 MiB ceiling); `/xwin` contains both arches (`aarch64` and `x86_64` subtrees under `/xwin/crt/lib` and `/xwin/sdk/lib/um`); pre-DONE gate green (fmt + clippy + 106/106 tests). Transcript: `docs/audits/cross-build-2026-06-03-after-bug005-fix.md`. |

## Compliance Notes — §9.5

Skipped per session-wide rule (no second-agent vendor reachable; user 2026-05-06).
