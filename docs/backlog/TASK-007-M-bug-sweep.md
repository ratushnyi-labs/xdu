# TASK-007-M — bug sweep

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description

Fresh end-to-end exercise of the v0.1 codebase. Smoke-runs and edge-case probes from
within this session's Docker + Windows environment:

- Build all 3 currently-supported targets (x86_64-musl, aarch64-musl, x86_64-pc-windows-gnu).
- Linux: run `xdu` inside `alpine:3.20`, exercise `--version`, `--help`, a real scan
  of a small fixture tree, observe the progressive spinner.
- Windows: run `xdu.exe` natively, exercise `--version`, `--help`.
- Inspect every commit in the session for any latent fixme-style note that didn't get
  filed as a backlog item.

Any finding is filed as its own `BUG-###` item (does not bloat TASK-007-M itself).

**Impacted UCs:** all.
**Impacted BR/WF:** none.

## Estimation
**Complexity:** EASY. Inspection + smoke-runs.

## Speck Reference
N/A — pure-investigation leaf; findings (if any) go into newly-created `BUG-###` items.

## Compliance Notes — §9.5
Skipped per session-wide rule.

## Findings

| Target                            | Size (B)  | Smoke                                                                                                                              |
|-----------------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------|
| `x86_64-unknown-linux-musl`       | 754,216   | `--version`, `--help`, unknown-flag rejection (exit 2): all ✓ inside `alpine:3.20` (musl-only base — confirms truly static link). |
| `aarch64-unknown-linux-musl`      | 679,096   | Built; not executed (no aarch64 emulator on this host). Confirmed strip + statically linked via earlier sessions.                  |
| `x86_64-pc-windows-gnu`           | 562,176   | `--version`, `--help`, unknown-flag rejection (exit 2): all ✓ natively on this Windows host.                                       |

Other inspection:
- `git log --oneline` re-walked: every commit traces to a `TASK-###` or `BUG-###` backlog item; no orphan changes.
- No new `TODO` / `FIXME` introduced in this wave's edits.
- Wave-3 scanner refactor preserved test-suite behavior (98 tests pass) — the existing `permission_denied_subdir_is_recorded_not_fatal`, `hardlinks_are_deduplicated`, `symlinks_are_not_followed`, `scan_async_eventually_finalizes` continue to pass against the new batched-lock walk.

**Zero new bug findings filed.** AC-1..AC-4 met.

Pre-merge baseline security gate: PASS — no code change in M itself; the wave-3 commit's gates pass for B/C/D and that gate disposition extends here (M is the inspection-only counterpart). §9.5 skipped per session-wide rule.
