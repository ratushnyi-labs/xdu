# SDD-TASK-007-O — Final pre-publish gate + tag v0.1.0 + push + GH release

**Backlog item:** TASK-007-O
**Impacted UCs:** none (publish ceremony; no behavioural change)

## Scope

A single interlocked publish sequence (§12.3.3 atomicity waiver by
interlock per §12.4.1 — each step is a precondition for the next).
The 7 sub-steps:

1. **Pre-publish verification gate (DEV).** Inside the Docker test
   container: `cargo fmt --all -- --check` clean, `cargo clippy
   --all-targets --locked -- -D warnings` clean, `cargo test --all
   --locked` → 106/106 pass, `cargo audit` → ≤ 2 known informational
   warnings (RUSTSEC-2024-0436 paste unmaintained, RUSTSEC-2026-0002 lru
   unsound — both already in `docs/audits/dependency-audit-2026-06-03.md`).
   Optional rerun of `scripts/build.sh` for the 6-target matrix
   (`cross-tools` cached from BUG-005).
2. **User-approval gates (CRR-1 / governance).** Supervisor surfaces the
   2 informational advisories for explicit approval (or remediation,
   e.g. ratatui bump); the CHANGELOG `[Unreleased]` → `[0.1.0] - <DATE>`
   flip; any README "Status" line / shield tightening.
3. **Local tag (reversible).** `git tag v0.1.0` after the CHANGELOG flip
   commit lands. Reversible via `git tag -d v0.1.0` before any push.
4. **CRR-3 push gate #1 — commits.** `git push origin main`. Requires
   explicit per-action user confirmation (irreversible / public).
5. **CRR-3 push gate #2 — tag.** `git push origin v0.1.0` → triggers
   `.github/workflows/release.yml`. Requires a SECOND, separate
   per-action user confirmation (this is the binary-publish step).
6. **Release workflow verification.** Watch `release.yml` complete on
   GH Actions; confirm 6 artefacts attached to the v0.1.0 release page;
   confirm `generate_release_notes: true` produced populated notes.
7. **Post-release housekeeping.** Mark parent TASK-007 + every child leaf
   DONE in the decomposition table; append the actual release URL to the
   CHANGELOG `[0.1.0]` entry; close any items resolved by shipping.

The CHANGELOG flip preserves all wave-2..wave-6 entries verbatim — they
collectively become the 0.1.0 release notes. `Cargo.toml`
`version = "0.1.0"` is unchanged.

**Out of scope:** crates.io publish; package-manager submissions (brew /
scoop / winget); announcement / blog / social posts (separate CRR-3
leaf); v0.1.1+ patch releases.

## ACs

- AC-1: All 4 pre-publish DEV gates pass inside the Docker test container
  — fmt clean, clippy `-D warnings` clean, `cargo test --all --locked`
  reports 106/106 pass, `cargo audit` reports ≤ 2 known informational
  warnings.
- AC-2: Advisory approval is recorded — EITHER via supervisor
  AskUserQuestion confirming the 2 informational warnings, OR by
  remediation (e.g. ratatui bump). The decision is quoted verbatim in
  the publish commit body or in a `docs/audits/` amendment note.
- AC-3: `CHANGELOG.md` has its `[Unreleased]` heading flipped to
  `[0.1.0] - <DATE>` (DATE = actual publish date) with the existing
  entries preserved verbatim. `Cargo.toml` `version = "0.1.0"` is
  unchanged.
- AC-4: `git tag v0.1.0` is created locally BEFORE any `git push`.
  Tag is reversible (`git tag -d v0.1.0`) up until step 5 fires.
- AC-5: `git push origin main` is preceded by an explicit per-action
  user confirmation. The user message granting confirmation is quoted
  verbatim in the publish commit body or the supervisor turn record.
- AC-6: `git push origin v0.1.0` is preceded by a SECOND, separate
  explicit per-action user confirmation (distinct from AC-5). The
  user message is quoted verbatim.
- AC-7: `.github/workflows/release.yml` completes successfully on the
  v0.1.0 tag push, producing 6 release artefacts attached to the GitHub
  Release page (per SDD-TASK-007-N9 AC-3).
- AC-8: After the release lands, parent `docs/backlog/TASK-007-prepare-for-publishing.md`
  Status is updated to DONE, the child-leaf decomposition table marks
  every leaf DONE, and the CHANGELOG `[0.1.0]` entry has the actual
  release URL appended.

## SEC ACs

- SEC-1: §5.4 stealth identity preserved across every public artefact
  (commits, tags, release notes, attached binaries). The R2 audit
  verdict (CLEAN, per `docs/audits/`) is the baseline; no regression
  introduced by the publish ceremony.
- SEC-2: No force-push, no history rewrite, no `--no-verify` on commits
  or pushes. Pre-commit hooks (if any) run normally; signing
  configuration (if any) is not bypassed.
- SEC-3: The two informational advisories (RUSTSEC-2024-0436 paste
  unmaintained, RUSTSEC-2026-0002 lru unsound — both transitive via
  ratatui) are EITHER user-approved with the recommendation recorded
  in `docs/audits/dependency-audit-2026-06-03.md`, OR remediated (e.g.
  ratatui bump). Silently shipping over them is a §10 forbidden action.
- SEC-4: Microsoft MSVC EULA acceptance preserved in the published
  Dockerfile (`--accept-license` already in place per N8a/N8b); no
  silent removal during the publish ceremony.
- SEC-5: No new shipping crate dependency added in O. The publish
  ceremony is mechanical (`git tag` + `git push` + workflow watch).
- SEC-6: CRR-3 confirmations (AC-5, AC-6) are quoted verbatim in the
  publish commit body or supervisor turn record. Silent / implied
  confirmation is forbidden.

## §9.5 cross-agent attack review

Skipped per session-wide rule.
