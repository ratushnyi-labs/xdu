# SDD-TASK-007-B — code hygiene sweep

**Backlog item:** TASK-007-B
**Impacted UCs:** none

## Scope
Inspection of every `src/*.rs` module + targeted small edits where the lawbook §5.2 limits, ADR §7 `unsafe`-doc rules, or clippy posture have drifted.

## ACs
- AC-1: `cargo clippy --all-targets --all-features --locked -- -D warnings` is green.
- AC-2: No function exceeds **30 non-empty / non-comment lines**, or an extraction is performed and recorded.
- AC-3: Every `unsafe` block has a `// SAFETY:` comment immediately above it.
- AC-4: No `#[allow(dead_code)]`, no commented-out logic, no `TODO`/`FIXME` markers outside lawful documentation.
- AC-5: `cargo fmt --all -- --check` is green.

## SEC ACs
- SEC-1: No behaviour change beyond mechanical extractions.
- SEC-2: §5.4 stealth identity preserved.
- SEC-3: BR-001 invariant preserved.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
