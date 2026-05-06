# TASK-001-G — Size formatter + bar generator

**Status:** DONE  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
**Scope:** `format_size(bytes, mode)` producing strings like `1.4 MiB` / `1.4 MB`; `bar(fraction, width, utf8)` producing Unicode block bar with ASCII fallback.
**Non-goals:** locale formatting (always ASCII digits + `.` decimal).
**Impacted UCs:** UC-003.
**Impacted BR/WF:** BR-006.
**Depends on:** TASK-001-C.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE | ESTIMATION | DONE | DECOMPOSITION | N/A | DESIGN | DONE | BACKEND | PENDING | TESTING | PENDING | POST_AUDIT | PENDING |

## Estimation
**Complexity:** EASY  **Justification:** pure functions, exhaustive unit tests.

## Acceptance (pointers)
- AC-1: `0 → "0 B"`, `1023 → "1023 B"`, `1024 → "1.0 KiB"`, `1024*1024 → "1.0 MiB"`, etc. up to `EiB`.
- AC-2: `--si` mode flips divisor to 1000 and unit suffix to `KB/MB/GB/...`.
- AC-3: `bar(0.0, 10, true)` = `"          "`; `bar(1.0, 10, true)` = `"██████████"`; non-integer fractions use partial blocks.
- AC-4: `utf8 = false` → `"##########"` / spaces.
- SEC-1: width clamped to `[0, 256]`; fraction clamped to `[0.0, 1.0]`.

## Speck Reference
[`docs/specks/SDD-TASK-001-G-size-formatter.md`](../specks/SDD-TASK-001-G-size-formatter.md)

## Verified results

- 21 unit tests pass: 18 functional + 3 `[SEC]` (NaN handling, u64::MAX no-panic, extreme width clamp).
- `cargo fmt --check` clean.
- `cargo clippy --all-targets --all-features --locked -- -D warnings` clean.
- No `unsafe`, no I/O, no allocations beyond returned `String`.

## Pre-merge baseline security gate (§9.4)

| Audit                          | Depth     | Disposition | Rationale                                                                |
|--------------------------------|-----------|-------------|--------------------------------------------------------------------------|
| SECURITY_AUDIT                 | NORMAL    | PASS        | Pure functions; total domain (clamping for pathological inputs); no I/O; no `unsafe`; no globals. SEC-1 / SEC-2 / SEC-3 / SEC-4 of Speck verified by tests + grep. |
| DEPENDENCY_TREE_SECURITY_AUDIT | FAST      | PASS        | No new crate dependency added (per ADR §7).                              |
| MEMORY_LEAK_AUDIT              | FAST      | PASS        | Only allocation is the returned `String`; freed by caller on drop. No retained state. |

Findings: none. Continuity check vs Speck ACs 1–12 + SEC-1…4: every AC mapped to a passing test or to a static review gate (SEC-3 grep, SEC-4 review) — all pass.

Status: **DONE**.
