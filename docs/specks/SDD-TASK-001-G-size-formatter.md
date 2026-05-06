# SDD-TASK-001-G — Size formatter + bar generator

**Backlog item:** TASK-001-G
**Impacted UCs:** UC-003
**Impacted BR/WF:** BR-006

## Scope / Non-goals

**In scope:** pure functions in `src/format.rs`:
- `SizeMode::{Iec, Si}` enum
- `format_size(bytes: u64, mode: SizeMode) -> String` — `"0 B"`, `"1023 B"`, `"1.0 KiB"`, … up through `EiB` (IEC) / `EB` (SI)
- `bar(fraction: f64, width: usize, utf8: bool) -> String` — UTF-8 block-element bar with ASCII (`#`/space) fallback, fraction clamped `[0,1]`, width clamped `[0,256]`
- Exhaustive unit tests + `[SEC]` cases (panic-safety on extreme inputs)

**Out of scope:** locale formatting (always ASCII digits + `.`); coloring (TASK-001-I); rendering of full row layout (TASK-001-I).

## Acceptance Criteria

- AC-1: `format_size(0, _) == "0 B"`, `format_size(1023, Iec) == "1023 B"`, `format_size(1024, Iec) == "1.0 KiB"`, `format_size(1024², Iec) == "1.0 MiB"`, `format_size(1024⁶, Iec) == "1.0 EiB"`.
- AC-2: SI mode: `format_size(1000, Si) == "1.0 KB"`; divisor 1000.
- AC-3: Values 100…999 in their unit format with no decimals; values 1.0…99.9 with one decimal.
- AC-4: `format_size(u64::MAX, _)` does not panic and ends with `EiB`/`EB`.
- AC-5: `bar(0.0, 10, true) == "          "` (10 spaces).
- AC-6: `bar(1.0, 10, true) == "██████████"`.
- AC-7: `bar(1.0, 10, false) == "##########"`.
- AC-8: `bar(0.5, 10, true) == "█████     "` (5 full + 5 empty, exact).
- AC-9: `bar(0.55, 10, true)` uses a partial block in cell 6 (5 full + 1 partial + 4 empty).
- AC-10: `bar(_, 0, _) == ""`.
- AC-11: `bar` clamps fraction to `[0.0, 1.0]` (negative → empty, >1 → full).
- AC-12: `bar` clamps width to ≤ 256 cells (defense against pathological input).

## Security ACs

- SEC-1: `format_size(u64::MAX, _)` does not panic.
- SEC-2: `bar(NaN, _, _)`, `bar(±∞, _, _)`, `bar(_, usize::MAX, _)` do not panic.
- SEC-3: No `unsafe` code in this module.
- SEC-4: No allocations beyond the returned `String`; no globals; no I/O.

## Failure Modes

The functions are total — no error returns. Pathological inputs are clamped to safe defaults rather than failing; clamping is part of the contract (AC-11 / AC-12).

## Test Matrix

| AC    | Unit | Integration | Build | Notes                        |
|-------|------|-------------|-------|------------------------------|
| AC-1  | ✅   | —           | —     |                              |
| AC-2  | ✅   | —           | —     |                              |
| AC-3  | ✅   | —           | —     | covered via specific values  |
| AC-4  | ✅   | —           | —     |                              |
| AC-5  | ✅   | —           | —     |                              |
| AC-6  | ✅   | —           | —     |                              |
| AC-7  | ✅   | —           | —     |                              |
| AC-8  | ✅   | —           | —     |                              |
| AC-9  | ✅   | —           | —     |                              |
| AC-10 | ✅   | —           | —     |                              |
| AC-11 | ✅   | —           | —     | negative + >1 cases          |
| AC-12 | ✅   | —           | —     | usize::MAX width             |
| SEC-1 | ✅   | —           | —     |                              |
| SEC-2 | ✅   | —           | —     |                              |
| SEC-3 | static | —         | —     | grep `unsafe` — must be 0    |
| SEC-4 | review | —         | —     |                              |

## Forbidden during this leaf

- Adding any crate dependency (per ADR §7).
- Any I/O.
- `unsafe` blocks.
