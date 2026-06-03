# TASK-007-N10 — Multilingual user manuals (en/ua/es/fr/it/pt)

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Priority:** MEDIUM
**Atomic Check:** ATOMIC — cohesive single-feature deliverable: the
  user-manual set in 6 languages. Per §12.4.1, multiple language outputs
  serving one outcome (the user-manual set) are atomic, not umbrella.
**Created:** 2026-06-03  **Updated:** 2026-06-03

## Raw Request

Resolved from earlier session (TASK-007 parent decomposition; user 2026-05-06):

- _"d2 - add also"_ — multilingual `docs/user-manual-{en,ua,es,fr,it,pt}.md` are in scope.
- _"d3 - by now mackine translated"_ — translations are machine-generated for v0.1; each non-English manual carries a header note flagging it as machine-translated, pending human review for a future revision.

## Refined Description

Author the user-manual set in 6 languages under `docs/`:

- `docs/user-manual-en.md` — authoritative English source.
- `docs/user-manual-ua.md` — Ukrainian.
- `docs/user-manual-es.md` — Spanish.
- `docs/user-manual-fr.md` — French.
- `docs/user-manual-it.md` — Italian.
- `docs/user-manual-pt.md` — Portuguese.

Content scope (end-user perspective, not contributor-facing):

- One-paragraph overview: what xdu does, what it does NOT do (no auto-delete,
  no service, no telemetry).
- Install: one paragraph per platform (Linux musl, macOS, Windows MSVC), each
  pointing at the GitHub Releases page once it exists.
- First run: what happens when you launch with no argument (default to CWD on
  Unix, drive picker on Windows).
- Navigation: arrow keys / vim keys / Enter / Backspace / q.
- Sort + unit toggles: `s`, `u`.
- Delete + trash (TASK-007-J) + lock-holder modal (BUG-004) + force-close
  (TASK-006) — brief, with the [No]-default confirmation behaviour spelled out.
- Errors overlay: `!`.
- Export: `e` → `xdu-export.json`.
- A read-only-by-default safety note.

Non-English manuals carry a header note (translated into each language) of the
shape:

> **Note.** This document is a machine translation for v0.1.0. Pending human
> review. Please open an issue at https://github.com/ratushnyi-labs/xdu if you
> spot a translation error.

The English manual carries no such note. README's existing "Documentation"
table (added in N5) is updated to link the 6 manuals.

**Out of scope:**

- Translations from a paid commercial service. Inline machine-translation
  output is acceptable per user 2026-05-06 ("by now mackine translated").
- Linking the manuals from `man(1)` / unix `MANPATH` (separate v0.2 leaf).
- Generating language-specific screenshot assets (English screenshots can be
  reused; v0.2 leaf for localised UI captures if and when xdu ever localises
  its own strings — currently English-only TUI labels).

## Estimation

**Complexity:** ABOVE_EASY. 6 markdown files, ~150-250 lines each. The
authoritative English content is the main effort; the 5 translations are
mechanical once that's locked.

## Speck Reference

[`docs/specks/SDD-TASK-007-N10-multilingual-user-manuals.md`](../specks/SDD-TASK-007-N10-multilingual-user-manuals.md)

## Subagent Spawn Log

| Time (UTC) | Role | Purpose | Status | Notes |
|------------|------|---------|--------|-------|
| 2026-06-03 | BA   | Refine + author Speck + write all 6 manuals (BA scope expanded by supervisor authorisation for `docs/user-manual-*.md` paths, which are documentation aligned with BA's `docs/spec.md` doctrine role) | SUCCESS | First spawn crashed mid-run on a socket error after landing Speck + en + ua. Second spawn (this one) completed es/fr/it/pt machine translations following the en source + ua header-note pattern, and updated README's User Manual table (was a placeholder pointing at this TASK; now links the six manuals with a machine-translation caveat line). All six manuals present; non-English manuals carry a translated machine-translation header note per Speck. |

## Compliance Notes — §9.5

Skipped per session-wide rule (no second-agent vendor reachable; user 2026-05-06).
