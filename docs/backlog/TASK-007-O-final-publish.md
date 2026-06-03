# TASK-007-O — Final pre-publish gate + tag v0.1.0 + push + GH release

**Status:** IN PROGRESS  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** BUILD
**Priority:** MEDIUM
**Atomic Check:** ATOMIC — the publish sequence is a single interlocked
  action ("ship v0.1.0 publicly"); the sub-steps (verify gate → advisory
  approval → tag → push commits → push tag → release) cannot be
  meaningfully delivered independently (you can't push the tag without
  the verify; you can't get the release without the tag push). §12.3.3
  waiver implicit by interlock per §12.4.1.
**Conflict Resolution Reference:** —
**Created:** 2026-06-03  **Updated:** 2026-06-03

## Raw Request

Final wave of TASK-007. Parent decomposition entry:

| TASK-007-O    | Final pre-publish gate + `git tag v0.1.0` + GH release (CRR-3)   | INFRA    | EASY        | all above         |

Authorisation gate (parent line 96-98):
> CRR-3 reminder: the actual `git push` to a public remote needs explicit
> per-action confirmation from you.

## Refined Description

A single interlocked publish sequence. Each numbered step is a precondition
for the next:

1. **Pre-publish verification gate (DEV scope).** Inside the Docker test
   container:
   - `cargo fmt --all -- --check` → clean.
   - `cargo clippy --all-targets --locked -- -D warnings` → clean.
   - `cargo test --all --locked` → 106/106 pass.
   - `cargo audit` → 2 known informational warnings (RUSTSEC-2024-0436,
     RUSTSEC-2026-0002) — both already documented in
     `docs/audits/dependency-audit-2026-06-03.md`; user explicit approval
     for these two required at step 2 below.
   - Optional re-run of `scripts/build.sh` for the 6-target matrix (cheap
     since `cross-tools` is cached from BUG-005 verification).

2. **User-approval gates (CRR-1 / governance).**
   - Approve the 2 informational advisories from P's audit (paste
     unmaintained + lru unsound, both transitive via ratatui), OR direct
     the supervisor to seek a different remediation path.
   - Approve the CHANGELOG `[Unreleased]` → `[0.1.0]` section flip + date
     stamp (the entries written across waves 2-6 collectively become the
     0.1.0 release notes).
   - Approve the README "Status" line / shield update if any (currently
     reads draft; tighten if requested).

3. **Local tag (reversible).** `git tag v0.1.0` after CHANGELOG flip is
   committed. `git tag -d v0.1.0` undoes it cleanly if anything is
   discovered before push.

4. **CRR-3 push gate #1 — commits.** `git push origin main` pushes
   wave-2..wave-6 commits to the public remote. Requires explicit
   per-action user confirmation. Once pushed, the commits are public.

5. **CRR-3 push gate #2 — tag.** `git push origin v0.1.0` pushes the tag
   and triggers `.github/workflows/release.yml`, which builds the 6
   targets on native GitHub runners and creates the GitHub Release with
   attached artefacts. Requires a SECOND explicit per-action confirmation
   (this is the one that publishes the binaries).

6. **Release workflow verification.** Watch `release.yml` complete on the
   GitHub Actions side; confirm all 6 artefacts are attached to the v0.1.0
   release page; confirm the release notes are populated (the workflow
   uses `generate_release_notes: true` per TASK-007-N9).

7. **Post-release.** Mark TASK-007 + every child leaf DONE in the parent
   decomposition table; update the CHANGELOG with the actual release URL;
   close any open `BUG-###` / `TASK-###` items that resolved by shipping.

## Out of scope

- Cargo crates.io publishing (xdu is a binary distribution; no crate
  publish in v0.1.0). If wanted, file as separate v0.2 leaf.
- Homebrew / scoop / package-manager submissions (post-v0.1.0).
- Announcement / blog post / social (CRR-3 external-messaging territory —
  separate leaf with its own per-action confirmations).
- Patch releases (v0.1.1+). New BUG / TASK items will go through the
  normal lifecycle.

## Estimation

**Complexity:** EASY in mechanical effort (a handful of `git` invocations
+ a CHANGELOG flip). The hard part is the CRR-3 gating discipline + the
advisory-approval surfacing — both supervisor-inline work.

## Speck Reference

[`docs/specks/SDD-TASK-007-O-final-publish.md`](../specks/SDD-TASK-007-O-final-publish.md)

## Subagent Spawn Log

| Time (UTC) | Role | Purpose | Status | Notes |
|------------|------|---------|--------|-------|
| 2026-06-03 | BA   | Refine + author Speck | SUCCESS | wrote `docs/specks/SDD-TASK-007-O-final-publish.md` (101 lines, 8 ACs, 6 SEC ACs); Status left IN PROGRESS per §12.4.1 — O completes only when the actual publish lands |
| 2026-06-03 | DEV  | Pre-publish verification gate (Step 1 of Speck) | SUCCESS | 4/4 gates green inside Docker (fmt clean, clippy `-D warnings` clean, `cargo test --all --locked` 106/106, `cargo audit` exit 0 with the 2 known informational warnings RUSTSEC-2024-0436 + RUSTSEC-2026-0002 unchanged from P); transcript at `docs/audits/pre-publish-verification-2026-06-03.md`; READY-FOR-USER-APPROVAL for Step 2 |
| 2026-06-03 | DEV | Step 3: CHANGELOG flip [Unreleased] -> [0.1.0] - 2026-06-03 | SUCCESS | User verbatim approval quoted in commit message; new empty Unreleased section seeded |
| 2026-06-03 | DEV | Step 3 follow-up: merge duplicate [0.1.0] sections | SUCCESS | User verbatim: "Merge into one [0.1.0] - 2026-06-03 (Recommended)". 2026-05-06 section folded into 2026-06-03; entries grouped Added/Changed/Fixed/Security. |

## Compliance Notes — §9.5

Skipped per session-wide rule (no second-agent vendor reachable; user 2026-05-06).
