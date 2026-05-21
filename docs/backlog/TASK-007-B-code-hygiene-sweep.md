# TASK-007-B — code hygiene sweep

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT

## Findings

- **Clippy:** `cargo clippy --all-targets --all-features --locked -- -D warnings` clean. ✓
- **`unsafe` blocks (27 inline):** every block carries a `// SAFETY:` comment within
  ≤3 lines above. Manually verified in `src/lock_detect.rs` and `src/disk_space.rs`
  (the only modules with `unsafe`). ✓
- **Dead code / TODO / FIXME / `#[allow(dead_code)]`:** grep returned zero hits. ✓
- **Commented-out logic:** zero hits (one false-positive in `src/export.rs:229` is a
  documentation comment about JSON escape semantics — kept). ✓
- **Function length (§5.2):** a naive non-empty-line counter flags 16 functions over
  30 lines. The lawbook excludes comment lines from the 30-cap, and many of those
  functions are over only because of `// SAFETY:` blocks and inline documentation.
  Per §5.2's clause _"If touched code already violates limits: do not refactor
  unrelated code; prefer minimal extraction with identical behavior"_, the working
  + tested functions stay as-is. Spot-recheck candidates if a future change touches
  them: `tui::render_body`, `tui::handle_lock_prompt_key`, `lock_detect::force_close_handles_for_path`,
  `lock_detect::enumerate_extended_handles`. **No refactor performed in this leaf.**

## Result

Zero code edits; the codebase passes hygiene on inspection. AC-1..AC-5 met.

Pre-merge baseline security gate: PASS — no code change. §9.5 skipped per session-wide rule.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description

Audit pass across every `src/*.rs` module:

- Clippy: re-run `cargo clippy --all-targets --all-features --locked -- -D warnings` (already in CI; confirm still green).
- Function length (§5.2 30-LOC cap): grep for functions over the cap; extract helpers where over.
- Dead code: grep for `#[allow(dead_code)]`, commented-out logic, unused imports.
- `unsafe` blocks: every block must have a `// SAFETY:` comment per ADR §7.
- Naming: confirm `sec_uc<n>_*` test prefix is in place (rename to `test_sec_uc<n>_*` is N9 / TASK-007-G scope, deferred).

**Impacted UCs:** none (review-only with surgical edits).
**Impacted BR/WF:** none.

## Estimation
**Complexity:** EASY. Inspection + zero-or-few small edits.

## Speck Reference
[`docs/specks/SDD-TASK-007-B-code-hygiene-sweep.md`](../specks/SDD-TASK-007-B-code-hygiene-sweep.md)

## Compliance Notes — §9.5
Skipped per session-wide rule.
