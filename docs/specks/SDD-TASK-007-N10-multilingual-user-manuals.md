# SDD-TASK-007-N10 — Multilingual user manuals (en/ua/es/fr/it/pt)

**Backlog item:** TASK-007-N10
**Impacted UCs:** none (end-user documentation; reflects v0.1.0 surface only)

## Scope

Author the user-manual set for v0.1.0 as 6 markdown files under `docs/`,
end-user perspective (not contributor-facing):

- `docs/user-manual-en.md` — authoritative English source.
- `docs/user-manual-ua.md` — Ukrainian, machine-translated.
- `docs/user-manual-es.md` — Spanish, machine-translated.
- `docs/user-manual-fr.md` — French, machine-translated.
- `docs/user-manual-it.md` — Italian, machine-translated.
- `docs/user-manual-pt.md` — Portuguese, machine-translated.

Every manual carries (in this order): a language-bar at the top (bolding the
current language without a link); a translated `# xdu — user manual` title; a
`What xdu does / does NOT do` section; `Install` per platform (Linux musl,
macOS, Windows MSVC, all pointing at the GitHub Releases page); `First run`
(default CWD on Unix, drive picker on Windows per TASK-007-H); `Navigation`
(arrows + vim keys + Enter / Backspace + `q` + `?` / F1); `Sort + units` (`s`,
`u` — IEC vs SI); `Delete + trash` ([No]-default modal; `--trash` opt-in;
Linux freedesktop spec; Windows Recycle Bin; macOS unsupported in v0.1.0);
`Lock-holder + force-close` (Windows Restart Manager modal; Linux /proc walk
informational); `Errors overlay` (`!`); `Export` (`e` → `xdu-export.json`);
`Safety note` (read-only by default).

The 5 non-English manuals additionally carry a one-paragraph header note —
translated into that language — flagging the file as a machine translation for
v0.1.0 pending human review, with the issue-tracker URL
`https://github.com/ratushnyi-labs/xdu`.

`README.md` already has a stubbed `User Manual` language table from N5; the
six placeholder rows are updated to link the corresponding files. No other
README change.

## ACs

- AC-1: All 6 files exist under `docs/` with non-empty content covering the
  9 sections listed in Scope, in the same order.
- AC-2: The 5 non-English manuals carry the machine-translation header note
  translated into their language, with the correct issue-tracker URL.
- AC-3: The English manual is the authoritative source; the 5 translations
  cover the same 9 sections in the same order and at comparable depth.
- AC-4: Each manual carries a small language-bar at the top of the form
  `[EN](user-manual-en.md) · [UA](user-manual-ua.md) · [ES](user-manual-es.md)
   · [FR](user-manual-fr.md) · [IT](user-manual-it.md) · [PT](user-manual-pt.md)`,
  with the link for the current language replaced by its bolded code (e.g.
  `**EN**` in `user-manual-en.md`).
- AC-5: `README.md`'s existing `User Manual` table (N5 stubs) is updated to
  point its 6 rows at the corresponding new files; no other README change.
- AC-6: No agent attribution leaks (§5.4); no `Co-Authored-By` in any file.

## SEC ACs

- SEC-1: No agent attribution leaked into any manual (§5.4) — strict.
- SEC-2: No promises of features not in the v0.1.0 binary (no macOS trash, no
  FreeBSD, no GUI mode, no auto-delete, no service mode).
- SEC-3: No telemetry / network / analytics claims beyond what xdu actually
  does, which is none. External URLs are HTTPS and limited to the public
  GitHub repository and Releases page already referenced from README.

## §9.5 cross-agent attack review

Skipped per session-wide rule.
