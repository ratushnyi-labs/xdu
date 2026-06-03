# TASK-007-Q — 6-target cross-build smoke-test

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Priority:** MEDIUM
**Created:** 2026-06-03  **Updated:** 2026-06-03

## Raw Request

Supervisor-derived pre-publish hardening leaf identified in the 2026-06-03
audit. Wave 4 added apple-darwin (via `cargo-zigbuild`) and windows-msvc (via
`cargo-xwin`) to the build matrix in `Dockerfile` + `scripts/build.sh`, but
the multi-target run was never executed end-to-end. If zigbuild's macOS
sysroot misses a symbol or xwin's MSVC SDK splat is mis-routed, the
`release.yml` GitHub Action would publish broken artefacts on the v0.1.0 tag.
Pre-publish should run `TARGETS="<all 6>" scripts/build.sh` locally, confirm
each target produces a binary under the 1.5 MiB ceiling, and record the
captured `file(1)` + `wc -c` output as a transcript.

## Refined Description

Run the cross-build matrix once and record the result.

Changes:
- No source / Dockerfile / script edits expected. This leaf records what
  the existing pipeline produces; any failure surfaces as a NEW INTAKE
  backlog item (per DEV §1.2.13.5 mid-task-discovery rule), not as widened
  scope here.
- New transcript `docs/audits/cross-build-2026-06-03.md` captures, per target:
  - command run
  - `file(1)` output on the produced binary
  - `wc -c` byte size
  - pass/fail vs the 1.5 MiB ceiling (`SIZE_CEILING_BYTES=1572864`)
- If any target fails, the DEV-subagent creates a new INTAKE `BUG-###` file
  with the captured error and DOES NOT auto-fix (single-task discipline).

**Out of scope:** fixing whatever cross-build problem surfaces. That is a
separate leaf, refined and estimated on its own merits.

**Note on cost:** building the `cross-tools` stage downloads ~1–2 GiB of
MSVC SDK via `xwin splat`. First-run cache fill is slow; subsequent
incremental rebuilds are cheap.

## Estimation

**Complexity:** EASY (in workflow terms) — one script invocation + one
transcript. Wall-clock cost is high (cross-tools image build); compute
cost is non-trivial but bounded.

## Speck Reference

[`docs/specks/SDD-TASK-007-Q-cross-build-smoke-test.md`](../specks/SDD-TASK-007-Q-cross-build-smoke-test.md)

## Subagent Spawn Log

| Time (UTC)         | Role | Purpose                                  | Status   | Notes |
|--------------------|------|------------------------------------------|----------|-------|
| 2026-06-03 | BA   | Refine + author Speck                    | SUCCESS  | 4 ACs + 3 SEC ACs; BA scope respected |
| 2026-06-03 | DEV  | Run 6-target build + capture transcript  | SUCCESS  | Smoke-test ran end-to-end; transcript at `docs/audits/cross-build-2026-06-03.md`. All 6 targets FAIL at shared `cross-tools 4/4` Docker step (`xwin v0.9.0` removed the `--arch` flag used in `Dockerfile` lines 91-95). Single umbrella follow-up: BUG-005. No source/Dockerfile/script/workflow edits per Q observational scope. Pre-DONE gate (fmt/clippy/test) skipped per spawn-prompt — Q does not touch source. |

## Compliance Notes — §9.5 cross-agent attack review

Skipped per session-wide rule (no second-agent vendor reachable; gemini/gpt
cut off per user 2026-05-06).
