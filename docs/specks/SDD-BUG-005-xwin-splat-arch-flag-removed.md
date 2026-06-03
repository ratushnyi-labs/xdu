# SDD-BUG-005 — `cross-tools` Dockerfile stage fails: `xwin splat` no longer accepts `--arch`

**Backlog item:** BUG-005
**Impacted UCs:** none (release-matrix pipeline regression; observational on
user-visible behaviour but unblocks all 6 cross-build targets)

## Scope

### Verification gate (CRR-1 — MUST precede any Dockerfile edit)

DEV discovers the real `xwin 0.9.0` `splat` syntax — not by guessing —
via one of:

- `docker compose run --rm test sh -c 'xwin --help && xwin splat --help'`
  inside a throwaway container, OR
- the published `xwin 0.9.0` release notes / changelog / README / source on
  crates.io or the upstream repo.

The verbatim `--help` output or the source URL MUST be quoted in the fix
commit message. No `Dockerfile` edit may be staged before this artefact
exists.

### `Dockerfile` (lines 91-95)

Current block:

```
ENV XWIN_CACHE_DIR=/xwin
RUN xwin --accept-license splat \
        --output /xwin \
        --arch x86_64 \
        --arch aarch64
```

Update to xwin 0.9.0's actual current syntax discovered above. The
layer's intent — splat both arches' MSVC SDK into `/xwin` for downstream
`cross-tools` consumers — is unchanged. Diff stays minimal: a one- or
two-line replacement in the EASY branch, or a small multi-step `RUN` in
the ABOVE_EASY branch if 0.9.0 requires per-arch invocations.

`cargo install --locked xwin` on `Dockerfile` line 86 stays UNPINNED
(user 2026-06-03 path (b), not path (a)).

### `docs/audits/cross-build-2026-06-03-after-bug005-fix.md` (new)

Fresh 6-target smoke-test transcript reproduced after the fix lands.
Format matches the TASK-007-Q transcript:

- exact `TARGETS=... sh scripts/build.sh` invocation
- per-target section: command, `file(1)` output, `wc -c` byte size,
  pass/fail vs `SIZE_CEILING_BYTES=1572864`
- final summary table (target, status, binary path, byte size,
  ceiling check)

This transcript supersedes `docs/audits/cross-build-2026-06-03.md`; the
older transcript is left untouched as historical record.

**Out of scope:**

- Any other `Dockerfile` changes (e.g. base-image bumps, additional
  cross-tool installs, layer reordering).
- Any edit under `src/`, `Cargo.toml`, `scripts/`, `tests/`, or
  `.github/workflows/`.
- Pinning `xwin` to an older version (path (a), explicitly rejected by
  the user).
- Modifying the existing TASK-007-Q transcript or the BUG-005 backlog
  body (the new transcript supersedes; the backlog is closed via status
  flip only).

## ACs

- AC-1: `Dockerfile` no longer passes the removed `--arch` flag to the
  `xwin splat` subcommand; the `cross-tools 4/4` layer builds
  successfully.
- AC-2: After the layer builds, `/xwin` contains both x86_64 AND aarch64
  MSVC SDK content (verified by `ls /xwin` or equivalent listing
  showing both arch trees, captured in the new transcript).
- AC-3: 6-target smoke-test produces 6/6 PASS via
  `TARGETS="x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-msvc aarch64-pc-windows-msvc x86_64-apple-darwin aarch64-apple-darwin" sh scripts/build.sh`;
  `docs/audits/cross-build-2026-06-03-after-bug005-fix.md` exists with
  per-target `file(1)` + `wc -c` + ceiling check (every binary
  ≤ 1.5 MiB or explicitly recorded as oversize) and a final summary
  table.
- AC-4: The fix commit message explicitly cites the xwin 0.9.0
  help/changelog/source reference (URL or quoted `--help` output) used
  to derive the new syntax. No guessing.

## SEC ACs

- SEC-1: `xwin --accept-license` flag remains in the new invocation;
  Microsoft MSVC EULA acceptance is preserved.
- SEC-2: All builds (verification, smoke-test, final reproduce) run
  inside Docker per §8.2 host isolation; no host `cargo install xwin`
  or host `xwin` invocation.
- SEC-3: No new shipping crate dependency added; no new privileged
  operation introduced; `xwin` remains a build-time cross-tool only.

## §9.5 cross-agent attack review

Skipped per session-wide rule.
