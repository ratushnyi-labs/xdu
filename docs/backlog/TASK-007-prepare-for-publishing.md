# TASK-007 — prepare app for MIT-license publishing under ratushnyi-labs

**Status:** DONE
**Type:** TASK
**Entry Mode:** CODE_FIRST (existing codebase; polish + publishing prep)
**User Mode:** GUIDED
**Guided Flow Stage:** CLARIFICATION → SPEC_READY (advancing after investigation TASK-007-A)
**Initial code-change expectation:** YES (polish work + repo-publishing scaffolding)
**Complexity:** **VERY_HARD** — touches every layer (code, TUI, perf, docs, tests, platform extensions, bug sweep, publishing infra). MUST decompose per §12.6/§12.7.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> i need prepare this app for publishing under mit license like others: https://github.com/ratushnyi-labs. but before that we need to polish the app.

## Lifecycle Coverage Map

| Stage          | State        |
|----------------|--------------|
| CLARIFICATION  | IN_PROGRESS  |
| ESTIMATION     | PENDING      |
| DECOMPOSITION  | PENDING      |
| DESIGN         | PENDING      |
| FRONTEND       | PENDING      |
| BACKEND        | PENDING      |
| TESTING        | PENDING      |
| POST_AUDIT     | PENDING      |

## Guided Flow History

| From  | To             | When                | Evidence                                       |
|-------|----------------|---------------------|------------------------------------------------|
| —     | IDEA           | 2026-05-06 (intake) | Raw request recorded                           |
| IDEA  | CLARIFICATION  | 2026-05-06          | Scope ambiguities require user input            |

## Open Questions (blocking — CRR-1: no guessing)

Two clusters of ambiguity. Both block `REFINED`.

### Q1 — "Polish" scope

The word "polish" can mean several distinct workstreams. Tick everything in scope (or
add your own):

- (a) Code hygiene: re-review every module's clippy/lint posture, remove dead code,
  tighten function lengths to lawbook §5.2 limits, audit `unsafe` blocks for
  documentation completeness.
- (b) UX polish in the TUI: visual consistency, color tuning, edge-case layout
  fixes (very narrow / very wide terminals), keybinding ergonomics, status-bar
  layout, error-message wording.
- (c) Performance polish: scan throughput on huge trees, lock contention in async
  scanner, memory footprint per node, binary-size optimization sweep.
- (d) Documentation polish: rewrite manuals for an external reader, add a
  CHANGELOG, expand the README with screenshots / asciinema, write a
  CONTRIBUTING guide.
- (e) Test polish: tighten existing assertions, add the missing TUI snapshot
  harness (`LATER-004`), close the `cargo-llvm-cov` coverage gap (`LATER-006`).
- (f) Outstanding deferred items from `docs/spec.md` §9 known-limitations:
  `TASK-003` (Windows drive list), `LATER-005` (macOS targets), `LATER-007`
  (trash mode), `LATER-008b` (Linux lock-holder), `LATER-009` (macOS / BSD
  free-space).
- (g) Bug sweep: anything you've encountered while exercising v0.1.x that hasn't
  been filed yet.

Pick the subset you want as the v0.1 polish goal. Anything not picked stays in
the deferred-follow-up list.

### Q2 — Publishing conventions at `ratushnyi-labs`

I have not yet inspected the public conventions at https://github.com/ratushnyi-labs.
Per CRR-1 I will not guess them. Pick the option you want:

- (a) **You list the conventions** (license-header style, README structure, CI
  workflow files, issue/PR templates, CHANGELOG format, version-tag scheme,
  release-artifact format, badges, branch protection, anything else).
- (b) **I fetch one representative repo** from the org with WebFetch, summarise
  the conventions, and you confirm what to match. (Network call — no secrets
  involved; the repo is public.)
- (c) **You point me at one specific repo** in the org whose conventions xdu
  should match.

## Tentative checklist (subject to your Q1 + Q2 answers)

Pre-publish hygiene that's repo-independent and almost certainly in scope:

- [ ] LICENSE already MIT — verify copyright holder line matches the org / your name.
- [ ] README usable by a new visitor with no context (install + usage + screenshot).
- [ ] CHANGELOG.md created (Keep-a-Changelog format unless you specify otherwise).
- [ ] Version tags: confirm `Cargo.toml` `version` semver scheme and tag `v0.1.0` (or whatever lands).
- [ ] CI workflow under `.github/workflows/` matching org convention.
- [ ] Release artifacts: pre-built binaries per target attached to GitHub Release.
- [ ] Issue / PR templates under `.github/` matching org convention.
- [ ] `CODE_OF_CONDUCT.md` / `CONTRIBUTING.md` matching org convention.
- [ ] `cargo-audit` / `cargo-deny` pass — no advisories in the dep tree.
- [ ] Stealth identity (§5.4) re-checked across all artifacts before public push.
- [ ] CRR-3 reminder: the actual `git push` to a public remote needs explicit
  per-action confirmation from you.

## Continuity flag from earlier in this session

The prior items in this chat — `BUG-003 ae66380`, `BUG-004 2ca8cac`, `TASK-006 eeb5e7a` —
were authored under lawbook v0.7.0 and do not carry the §9.5 cross-agent attack review
(LIGHT + FULL) that v0.8.0 now requires for implementable items.

**Resolved (2026-05-06):** user statement (verbatim): _"this laptop has access to claude
only, the gemini/gpt are cut-off"_. Per §9.5.1 (human-attacker substitution) and §9.5.2
(skip with explicit `## Compliance Notes` entry stating "no second agent reachable"),
the three items now carry retroactive `## Compliance Notes` blocks recording the skip
with the user's statement as the concrete reason. **This same constraint applies to
TASK-007 itself and to every subsequent implementable item produced from this laptop**
until either the second-agent vendor access is restored or the user elects to perform
the human-attacker substitution.

## Active-item notes

| Timestamp           | Verbatim user note                                                       |
|---------------------|--------------------------------------------------------------------------|
| 2026-05-06 (intake) | _"i need prepare this app for publishing under mit license like others: https://github.com/ratushnyi-labs. but before that we need to polish the app."_ |
| 2026-05-06          | _"this laptop has access to claude only, the gemini/gpt are cut-off"_ — environment constraint for §9.5; resolves the prior 1/2/3 question by option-2-analogue (skip with active Compliance Notes). |
| 2026-05-06          | _"q1 - all"_ — every polish workstream (a, b, c, d, e, f, g) is in scope. |
| 2026-05-06          | _"q2 - you can clone in temporary folders the apps and investigate"_ — authorisation to `git clone` representative repos from the org into `.dev-temp/` (project-scoped, git-ignored, safe-to-delete per lawbook §8.3) and summarise conventions for matching. |
| 2026-05-06          | _"d1: Switch Windows target"_ — Windows target switches from `x86_64-pc-windows-gnu` to `x86_64-pc-windows-msvc`. Downstream coupling NOT yet confirmed (see Open follow-ups below). |
| 2026-05-06          | _"d2 - add also"_ — multilingual `docs/user-manual-{en,ua,es,fr,it,pt}.md` are in scope. |
| 2026-05-06          | _"d3 - by now mackine translated"_ — translations are machine-generated for v0.1; each non-English manual carries a header note flagging it as machine-translated, pending human review for a future revision. |
| 2026-05-06          | _"D1.1 - both"_ — add `x86_64-apple-darwin` AND `aarch64-apple-darwin`. |
| 2026-05-06          | _"D1.2 - yep both"_ — add `x86_64-pc-windows-msvc` AND `aarch64-pc-windows-msvc`. |
| 2026-05-06          | _"D1.3 - for local docker llvm based build we can keep all 6"_ — local Docker keeps building all 6 targets via LLVM tooling (`cargo-zigbuild` for musl + apple-darwin, `cargo-xwin` for windows-msvc). GitHub Actions native-runner pipeline also added per org pattern for the publish path. |
| 2026-05-06          | _"fo"_ — interpreted as `go` (typo of the offered approval option, matches the pattern of recent typos: `qnd`, `tet=rmination`, `mackine`). Executing wave 2 (N1…N5) and pausing before wave 3. CRR-1: if interpretation is wrong, reply `stop` and I will revert. |
| 2026-05-06          | _"retry"_ — resume N5 (README rewrite); the prior turn announced N5 but stopped before writing the file. |
| 2026-05-06          | _"go"_ — Docker is back up; running wave-2 verification (fmt + clippy + test + scripts/build.sh smoke) and committing on green. |
| 2026-05-06          | _"go"_ — Wave 3 (B/C/D/M) executed. Committed `ff020a0`. |
| 2026-05-06          | _"go"_ — Wave 4 (N6/N8a/N8b/I/N7/N9) executed. Committed `2861ab6`. Multi-target verification deferred to TASK-007-O. |
| 2026-05-21          | _"go"_ — proceeding to wave 5 (F/G/H/J/K/L: TUI snapshot, llvm-cov, Windows drives, trash, Linux lock-holder, macOS/BSD free-space). |
| 2026-05-21          | Wave 5 complete: all 6 leaves DONE. 97 unit + 6 integration + 3 snapshot = 106 tests pass. Apple-darwin / windows-msvc cross-build verification still deferred to TASK-007-O. |
| 2026-06-03          | _"what else should we do before publishing the app?"_ — supervisor audit of the pre-publish gap list before scheduling waves 6 + 7. |
| 2026-06-03          | _"go"_ — drain remaining backlog per §1.2.12. New leaves added: P (audit + llvm-cov baseline), Q (cross-build smoke-test), R (parent Speck + stealth-identity audit). Switching from inline execution to supervisor-subagent model (§1.2 + §1.2.13) for all remaining leaves. Stop directive: AT_BACKLOG_DRAIN; CRR-3 halt scheduled at O (push to public remote). |
| 2026-06-03          | P DONE (`f5a9d78`) — clean cargo-audit (2 informational warnings flagged for O gate), coverage baseline 74.06% (no fail-under gate added). Q DONE (`8ef9fa1`) — smoke-test caught BUG-005 (xwin 0.9.0 CLI break). BUG-005 DONE (`66ea297`) — 6/6 PASS, largest binary 766 KiB ≪ 1.5 MiB. Cross-build pipeline now verified end-to-end. |
| 2026-06-03          | _"Split into R-1 + R-2"_ — AskUserQuestion answer to §12.4.1 atomic-split prompt. R replaced in decomposition table with TASK-007-R1 (parent Speck) + TASK-007-R2 (stealth-identity audit). Original R never had its own backlog file, so no DUPLICATE flip needed; SPLIT_INTO recorded here and in the table. |
| 2026-06-03          | TASK-007-O AC-8 housekeeping: parent Status flipped to DONE. v0.1.0 shipped at `https://github.com/ratushnyi-labs/xdu/releases/tag/v0.1.0` (2026-06-03T14:47:47Z). See `## v0.1.0 outcome — 2026-06-03` section below for the full child-leaf roll-up. |

## Refined Description

**Scope (per user's Q1 = all):** every polish workstream in v0.1 + publishing scaffolding:

- (a) **Code hygiene** — clippy/lint resweep, function-length cap (§5.2), dead-code, `unsafe`-block doc audit.
- (b) **TUI UX** — narrow/wide terminal layouts, color tuning, keybinding ergonomics, status-bar layout, error wording.
- (c) **Performance** — scan throughput, lock contention in async scanner, memory per node, binary-size sweep.
- (d) **Documentation** — README rewrite for outside readers, screenshots/asciinema, CHANGELOG (Keep-a-Changelog), CONTRIBUTING.
- (e) **Test polish** — TUI snapshot harness (`LATER-004`), `cargo-llvm-cov` coverage gate (`LATER-006`).
- (f) **Deferred items from spec §9** — `TASK-003` (Windows drives), `LATER-005` (macOS targets), `LATER-007` (trash mode), `LATER-008b` (Linux lock-holder), `LATER-009` (macOS / BSD free-space).
- (g) **Bug sweep** — fresh end-to-end exercise on Linux + Windows, file what surfaces.

**Publishing prep (per user's main Q):** match ratushnyi-labs org conventions. Investigation
sub-task `TASK-007-A` clones 1–2 representative repos to `.dev-temp/` and summarises.
Then `TASK-007-N` (final) applies the conventions to xdu and prepares the publish action
(behind a CRR-3 per-action confirmation when we get to `git push`).

**Non-goals:**
- Major architecture changes (would be a new top-level item).
- Adding non-MIT licensing.
- Forcing a v0.2 feature scope creep (each `LATER-*` is its own leaf with a clear cap).

**Impacted UCs:** all (touches every code path) plus new ones once `TASK-003`/`LATER-*` leaves materialise.
**Impacted BR/WF:** none modified — BR-001 stays as amended through TASK-005/006.
**Dependencies:** TASK-007-A must complete before TASK-007-N can author the publishing convention application.

## Decomposition (leaves; each gets its own backlog file when picked up)

`A` is DONE. `N` is split into 10 sub-leaves so each is implementable. Order updated to
reflect dependencies surfaced during TASK-007-A investigation.

| ID            | Title                                                            | Layer    | Est.        | Depends on        |
|---------------|------------------------------------------------------------------|----------|-------------|-------------------|
| TASK-007-A    | Investigate ratushnyi-labs conventions (clone + summarise)       | INFRA    | EASY        | —                 |
| TASK-007-N1   | LICENSE + Cargo.toml metadata align                              | INFRA    | EASY        | A                 |
| TASK-007-N2   | .gitignore / .gitattributes / .dockerignore / .env align         | INFRA    | EASY        | A                 |
| TASK-007-N3   | .github/ (CODEOWNERS, SECURITY.md, ISSUE/PR templates)           | INFRA    | EASY        | A                 |
| TASK-007-N4   | CONTRIBUTING.md + CHANGELOG.md                                   | DOC      | EASY        | A                 |
| TASK-007-N5   | README rewrite to org style (badges + sections + Supported Targets) | DOC   | EASY        | A                 |
| TASK-007-B    | Code hygiene sweep (a)                                           | BACKEND  | ABOVE_EASY  | A                 |
| TASK-007-C    | TUI UX polish (b)                                                | FRONTEND | ABOVE_EASY  | A                 |
| TASK-007-D    | Performance polish (c)                                           | BACKEND  | ABOVE_EASY  | A                 |
| TASK-007-M    | Bug sweep (g)                                                    | TEST     | EASY        | A                 |
| TASK-007-N6   | Dockerfile rename + restructure (`test` stage)                   | INFRA    | ABOVE_EASY  | A                 |
| TASK-007-N8a  | Switch Windows target gnu → msvc (`cargo-xwin` in Docker)        | INFRA    | ABOVE_EASY  | A                 |
| TASK-007-N8b  | Add `aarch64-pc-windows-msvc`                                    | INFRA    | EASY        | N8a               |
| TASK-007-I    | Add `x86_64-apple-darwin` + `aarch64-apple-darwin` (`cargo-zigbuild` in Docker) | INFRA | ABOVE_EASY | A           |
| TASK-007-N7   | .github/workflows/ci.yml (native-runner matrix, 6 targets)       | INFRA    | ABOVE_EASY  | N6, N8a, N8b, I   |
| TASK-007-N9   | .github/workflows/release.yml (on `v*` tag, gh release)          | INFRA    | ABOVE_EASY  | N7                |
| TASK-007-F    | TUI snapshot harness `LATER-004` (e.1)                           | TEST     | ABOVE_EASY  | A                 |
| TASK-007-G    | `cargo-llvm-cov` gate + `test_sec_uc###_*` rename `LATER-006`    | TEST     | ABOVE_EASY  | A                 |
| TASK-007-H    | Windows drive list `TASK-003` (f.1)                              | BACKEND  | ABOVE_EASY  | A                 |
| TASK-007-J    | Trash mode `LATER-007` (f.3)                                     | BACKEND  | ABOVE_EASY  | A, TASK-005       |
| TASK-007-K    | Linux lock-holder enumeration `LATER-008b` (f.4)                 | BACKEND  | ABOVE_EASY  | A, BUG-004        |
| TASK-007-L    | macOS / BSD free-space `LATER-009` (f.5)                         | BACKEND  | EASY        | A, TASK-004, I    |
| TASK-007-N10  | Multilingual user manuals (en/ua/es/fr/it/pt, machine-translated) | DOC     | ABOVE_EASY  | N5                |
| TASK-007-P    | cargo-audit install + clean pass; cargo-llvm-cov baseline run    | INFRA    | EASY        | G                 |
| TASK-007-Q    | Local 6-target cross-build smoke-test (before CI sees the matrix) | INFRA   | EASY        | N6, N8a, N8b, I   |
| ~~TASK-007-R~~| ~~Parent SDD-TASK-007 Speck + docs/audits/stealth-identity record~~ — **SPLIT** per §12.4.1 (user confirmed 2026-06-03) into R1 + R2 | — | — | —              |
| TASK-007-R1   | Author parent SDD-TASK-007 Speck (cross-cutting ACs over the 17 child Specks) | DOC | EASY | N5            |
| TASK-007-R2   | docs/audits/stealth-identity-2026-06-03.md — §5.4 sweep record   | DOC      | EASY        | N5                |
| TASK-007-O    | Final pre-publish gate + `git tag v0.1.0` + GH release (CRR-3)   | INFRA    | EASY        | all above         |

Execution waves:
1. **A** (investigation) — DONE.
2. **Wave 2 — repo scaffolding (no behaviour change):** N1, N2, N3, N4, N5. Parallel-safe.
3. **Wave 3 — code/UX polish:** B, C, D, M. Parallel-safe.
4. **Wave 4 — build pipeline restructure:** N6, N8a, N8b, I, N7, N9. Sequenced.
5. **Wave 5 — tests + deferred features:** F, G, H, J, K, L. Parallel-safe.
6. **Wave 6 — multilingual docs:** N10.
7. **CRR-3 confirmation gate (Wave 7):** TASK-007-O. `git push` to public remote and
   `gh release create` happen ONLY after explicit per-action user confirmation.

## Compliance Notes — cross-agent attack review (§9.5)

This item and every leaf produced from this laptop will record the §9.5.2 documented
skip: _no second-agent vendor reachable; gemini/gpt cut off per user (2026-05-06)_.
Per §9.5.1 the user may at any time elect human-attacker substitution for a specific
leaf — the agent will accept and record that on request.

## Estimation

**Parent:** VERY_HARD → `DECOMPOSED`. **Leaves:** A is `EASY`; B–N are `EASY` or
`ABOVE_EASY` — all implementable per §12.5.

## Speck Reference

`docs/specks/SDD-TASK-007-prepare-for-publishing.md` — minimal parent-level Speck after
TASK-007-A reports its findings, so the publishing conventions are documented as ACs in
the Speck before any of B–N start.

## Refined Description

**Pending Q1 + Q2.** Will be authored once polish scope and ratushnyi-labs conventions
are confirmed.

## Estimation

**Pending.** Provisional: **MEDIUM** at minimum (publishing scaffolding + at least one
polish workstream); promotes to `HARD` if (c) performance or (e) full TUI snapshot
harness are in scope, in which case decomposition per §12.7 follows.

## Speck Reference

`docs/specks/SDD-TASK-007-prepare-for-publishing.md` — authored once `REFINED`.

## v0.1.0 outcome — 2026-06-03

**v0.1.0 shipped.** Release URL: `https://github.com/ratushnyi-labs/xdu/releases/tag/v0.1.0`.
Published: `2026-06-03T14:47:47Z`. All 6 binary artefacts attached (3 musl
`.tar.gz` + 2 windows-msvc `.zip` + 1 apple-darwin `.tar.gz`). Repo
remains **PRIVATE** per standing user direction (CRR-3 Publish Log
`post-1` + `post-1b` in `TASK-007-O-final-publish.md`).

**Child-leaf roll-up (all 22 leaves DONE):**

- **Wave 1 — investigation:** TASK-007-A (DONE).
- **Wave 2 — scaffolding:** TASK-007-N1, N2, N3, N4, N5 (DONE).
- **Wave 3 — code/UX polish:** TASK-007-B, C, D, M (DONE).
- **Wave 4 — build pipeline:** TASK-007-N6, N8a, N8b, I, N7, N9 (DONE).
- **Wave 5 — tests + deferred features:** TASK-007-F, G, H, J, K, L (DONE).
- **Wave 6 — multilingual docs:** TASK-007-N10 (DONE).
- **Wave 6.5 — audits + smoke-test:** TASK-007-P (DONE, `f5a9d78`),
  TASK-007-Q (DONE, `8ef9fa1`), TASK-007-R1 (DONE), TASK-007-R2 (DONE).
- **Wave 7 — publish (CRR-3 gate):** TASK-007-O (DONE).
- **Original R:** SPLIT into R1 + R2 (no DUPLICATE flip; never had its own
  backlog file).

**Cancelled-then-retagged churn during Wave 7** (full detail in TASK-007-O
CRR-3 Publish Log `post-1h` + `post-1i`):

- First v0.1.0 tag was pinned at `f370f71`. The `release.yml` run
  `26888633897` was user-cancelled after `x86_64-apple-darwin` queued
  >45 min on `macos-13` (CRR-3 `post-1f` / `post-1h`).
- Recovery path picked: patch `release.yml` to use `ubuntu-latest` +
  `cargo-zigbuild` for `x86_64-apple-darwin` (mirroring the musl pattern),
  then re-tag v0.1.0 to the new SHA (CRR-3 `post-1i`).
- Patch commit `e260fe0`; tag deleted both locally and on remote; new
  annotated v0.1.0 created at SHA `569f83f`; re-pushed; fresh workflow
  run `26891916491` completed green at `2026-06-03T14:47:47Z`.

**Out-of-scope follow-ups intentionally deferred to v0.2:**

- Visibility flip PRIVATE → PUBLIC (user direction `post-1b`: keep private
  indefinitely until user re-opens).
- Cargo crates.io publishing (post-v0.1.0 leaf if wanted).
- Package-manager submissions (Homebrew / scoop / nix).
- Announcement / external messaging (separate CRR-3 leaf).
- macOS / BSD trash mode (TASK-007-J shipped Linux + Windows only).
- The 2 informational `cargo-audit` warnings (RUSTSEC-2024-0436 paste
  unmaintained + RUSTSEC-2026-0002 lru unsound) — both transitive via
  ratatui; remediation tracked in
  `docs/audits/dependency-audit-2026-06-03.md`.
