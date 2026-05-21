# TASK-007-A — investigate ratushnyi-labs org conventions

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

Inherited from TASK-007 plus the user's explicit Q2 answer:

> _"q2 - you can clone in temporary folders the apps and investigate"_

## Refined Description

**Scope:** discover and document the publishing conventions used across the
`ratushnyi-labs` GitHub org so TASK-007-N can apply them to xdu. Concretely:

1. `WebFetch` `https://github.com/ratushnyi-labs` (public org page) to enumerate repos.
2. `git clone` 1–3 representative repos into `.dev-temp/ratushnyi-labs-investigation/`
   — project-scoped temp dir per lawbook §8.3 (git-ignored, safe-to-delete).
3. For each cloned repo, inspect:
   - `LICENSE` — header form, copyright holder line.
   - `README.md` — section structure, badge set, screenshots, install instructions.
   - `.github/` — issue templates, PR templates, workflows (CI matrix, release flow,
     dependabot config).
   - `CHANGELOG.md` (if present) — format style (Keep-a-Changelog, conventional, etc.).
   - Build/manifest files (`Cargo.toml`, `package.json`, `pyproject.toml`, etc.) —
     conventional metadata (keywords, categories, repository link, homepage,
     authors).
   - Branch and tag scheme (release tag prefix, branch names).
   - `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md` — presence and shape.
4. Write findings into this backlog file under `## Findings`.

**Non-goals:** applying the conventions to xdu (that is TASK-007-N).

**Impacted UCs:** none directly (research).
**Impacted BR/WF:** none.

## Estimation

**Complexity:** EASY  **Justification:** read-only investigation; bounded by a small
number of public repos; no code change.

## Acceptance Criteria

- AC-1: At least one representative repo from the org is cloned into
  `.dev-temp/ratushnyi-labs-investigation/`.
- AC-2: This file's `## Findings` section documents license-header form, README
  structure, CI workflow shape, issue/PR template presence, CHANGELOG format, tag
  scheme, and any other org-wide conventions observed.
- AC-3: A short delta list captures what xdu needs to add or modify to match.

## SEC ACs

- SEC-1: `git clone` is read-only. CRR-3 N/A.
- SEC-2: No secrets sent to the org or to any third party. WebFetch hits only
  `https://github.com/ratushnyi-labs` and the specific public-repo URLs.
- SEC-3: `.dev-temp/` is git-ignored — the cloned content does not contaminate xdu's
  own git history.

## Compliance Notes — §9.5 cross-agent attack review

Skipped per the session-wide rule (no second-agent vendor reachable; user statement
2026-05-06). This is a read-only investigation task; risk is negligible.

## Findings

Cloned to `.dev-temp/ratushnyi-labs-investigation/`:
- `xsfx` (Rust, 0.2.0) — older, most-polished repo, has CHANGELOG.
- `spad` (Rust) — newer, simpler, no CHANGELOG.
- `trim` (Rust) — newest, multilingual docs, no CHANGELOG.

The 4th repo (`sci-rnr`, TeX) is a research artefact and not a code-conventions reference.

### LICENSE

| Repo | Copyright line |
|------|----------------|
| xsfx | `Copyright (c) 2025 neemle` (older identity) |
| spad | `Copyright (c) 2026 Pavlo Ratushnyi` |
| trim | `Copyright (c) 2026 Pavlo Ratushnyi` |

**Current convention** (spad/trim): `Copyright (c) 2026 Pavlo Ratushnyi`.
**xdu currently has** `Copyright (c) 2026 xdu contributors` — must update.

### `Cargo.toml`

Canonical metadata:
- `edition = "2021"`, `license = "MIT"`
- `repository = "https://github.com/ratushnyi-labs/<name>"`, `homepage = <same>`
- `readme = "README.md"`
- `keywords` + `categories` (xsfx, not always in spad/trim)
- `authors = ["Pavlo Ratushnyi"]` (trim)
- `[profile.release]` (standard, NOT a custom `release-min`):
  ```
  opt-level = "z"
  lto = true
  codegen-units = 1
  panic = "abort"
  strip = true
  ```

**xdu currently:** uses custom `release-min` profile + missing `repository`/`homepage`/`authors`. Realign.

### Build targets

Org canonical matrix (6 targets):
- `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`
- `x86_64-apple-darwin`, `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`

**xdu currently:** 3 targets — `x86_64/aarch64 linux-musl` + `x86_64-pc-windows-gnu`.
Deltas: add apple-darwin (`LATER-005`), add aarch64 windows, **switch Windows from `-gnu` to `-msvc`**.

### CI strategy

**Org canonical:** GitHub Actions native runners (ubuntu / macos / windows) per matrix entry, `dtolnay/rust-toolchain`, `cargo-zigbuild` for musl cross from non-musl runners. Two workflows:
- `.github/workflows/ci.yml` — on push/PR; matrix build + integration tests in Docker.
- `.github/workflows/release.yml` — on `v*` tag; produces `*.tar.gz` (Unix) / `*.zip` (Windows) and a GitHub Release via `softprops/action-gh-release@v2`.

**xdu currently:** Docker-only `scripts/build.sh` + `scripts/ci.sh`, no GitHub Actions. Add both workflows.

### `.github/` tree (canonical)

```
.github/
  CODEOWNERS                            # `* @paveldhq`
  SECURITY.md                            # Supported Versions / Reporting / Timeline / Scope
  PULL_REQUEST_TEMPLATE.md               # Summary + Checklist + Test Plan
  ISSUE_TEMPLATE/
    bug_report.yml                       # version + OS dropdown + description + repro + logs
    feature_request.yml                  # problem + solution + alternatives
    config.yml                           # blank_issues_enabled: false, contact_links to docs
  workflows/
    ci.yml
    release.yml
```

xdu has none of these. Must add.

### `.gitignore` (canonical, brief)

```
.idea
target
lcov.info
*.profraw
dist/
.build/
.dev-data/
.dev-temp/
```

xdu's is more verbose with IDE/OS-noise entries. Simplify to match.

### `.gitattributes` (canonical)

```
* text=auto eol=lf
*.exe binary
*.dll binary
*.so binary
*.dylib binary
*.png binary
*.jpg binary
*.gz binary
*.zip binary
*.tar binary
*.crt binary
```

xdu's includes per-extension EOL overrides — replace with the simpler canonical form.

### `.dockerignore` (canonical)

```
target
.git
.idea
.dev-data
.dev-temp
*.profraw
lcov.info
dist
.build
```

### `.env` (canonical)

Comment-only file committed for lawbook §8.1 compliance:

```
# Safe local defaults for <repo> development
# No ports exposed — <repo> is a CLI tool with no running services.
# This file exists for rules §8.1 compliance.
```

xdu has `.env.example` only — should also commit a comment-only `.env`.

### `README.md` shape

- Centered title via `<p align="center">` and a one-line tagline (spad, trim) **or** flat `# name` (xsfx).
- Three badges in a row: CI, License: MIT, Release.
- Sections in order: hero → optional Supported Targets table → Quick Start / Usage → Install (Pre-built binaries via curl + Build from Source) → Testing → Documentation table → Contributing → Security → License.
- Documentation table with relative links to `docs/spec.md`, `docs/development-manual.md`, etc.
- Multilingual user-manual table (en, ua, es, fr, it, pt) — present in xsfx and trim; not mandatory but on-pattern.

xdu's current README is reasonable but lacks badges, multilingual manuals, and the org's structural style.

### `CONTRIBUTING.md`

Sections: Development Setup → Workflow → Code Style → Safety Rules → Testing Requirements → Conventional Commits → PR Checklist.

xdu doesn't have one. Must add.

### `CHANGELOG.md`

Keep-a-Changelog format with semver. Sections per version: Added / Changed / Fixed / Security. Date format `YYYY-MM-DD`. Only `xsfx` has one of the three.

xdu doesn't have one. Adding is in scope (`(d)` polish workstream).

### Docker layout

- `Dockerfile` (not `Dockerfile.build`) — single multi-stage; `target: test` is the canonical stage name.
- `docker-compose.yml` — one `test` service: `build: { context: ., target: test }`.
- Dockerfile builds in `rust:1.x-slim-bookworm`, adds rustfmt/clippy/llvm-tools, `cargo-binstall` + `cargo-llvm-cov` + `cargo-audit`, then `CMD cargo fmt --check && clippy && llvm-cov && audit`.

**xdu currently:** `Dockerfile.build` (different name) + `docker-compose.yml` with multiple services (`build`, `dev`). Different design. Renaming + restructure is in scope of TASK-007-N.

### Test convention

- Tests in `tests/integration.rs`.
- 100% lines + functions coverage enforced via `cargo-llvm-cov --fail-under-lines 100 --fail-under-functions 100`.
- Security-test naming convention: **`test_sec_ucXXX_*`** prefix (different from the lawbook's `[SEC][UC-###]` Playwright-flavoured form).

xdu uses `sec_uc<num>_` prefix on Rust tests. **Org pattern is `test_sec_uc###_`** — small but visible delta.

### `docs/` layout

- `spec.md`, `development-manual.md`, `installation-manual.md`, `configuration-manual.md`.
- `user-manual-{en,ua,es,fr,it,pt}.md` — multilingual user manual table (xsfx, trim).
- `backlog/`, `specks/` (xsfx) — same lawbook governance structure xdu already uses.

xdu has the first four; multilingual manuals are an optional extra under workstream `(d)`.

### `scripts/`

Org pattern is one-line wrappers around `docker compose run --build --rm test`. xdu's `scripts/build.sh` + `scripts/ci.sh` are more elaborate; depending on how aggressively we restructure the Docker layout, either keep or simplify.

---

## Delta list — what xdu needs to add or modify

1. **LICENSE** — update copyright line to `Copyright (c) 2026 Pavlo Ratushnyi`.
2. **Cargo.toml** — add `repository`, `homepage`, `authors`; replace custom `release-min` profile with standard `[profile.release]`; ensure `keywords` + `categories` present.
3. **Targets** — switch Windows from `-gnu` to `-msvc`; add `apple-darwin` x86_64 + aarch64 (currently `LATER-005`); add `aarch64-pc-windows-msvc`.
4. **`.gitignore`** — replace with canonical 8-line form.
5. **`.gitattributes`** — replace with canonical 11-line form.
6. **`.dockerignore`** — replace with canonical 9-line form.
7. **`.env`** — add a comment-only `.env` file.
8. **`.github/CODEOWNERS`** — `* @paveldhq` (per the org pattern).
9. **`.github/SECURITY.md`** — Supported Versions, Reporting, Timeline, Scope.
10. **`.github/PULL_REQUEST_TEMPLATE.md`** — Summary + Checklist + Test Plan.
11. **`.github/ISSUE_TEMPLATE/bug_report.yml`** — xdu version + OS dropdown + description + repro + logs.
12. **`.github/ISSUE_TEMPLATE/feature_request.yml`** — problem + solution + alternatives.
13. **`.github/ISSUE_TEMPLATE/config.yml`** — `blank_issues_enabled: false`, contact link to docs.
14. **`.github/workflows/ci.yml`** — native-runner matrix, build + test.
15. **`.github/workflows/release.yml`** — on `v*` tag, build all targets, package as `*.tar.gz` / `*.zip`, GitHub Release via `softprops/action-gh-release@v2`.
16. **`CONTRIBUTING.md`** — org-pattern sections.
17. **`CHANGELOG.md`** — Keep-a-Changelog with `[0.1.0]` first entry covering everything done so far.
18. **`README.md`** — restructure with badges + sections matching the org style.
19. **Dockerfile rename / restructure** — `Dockerfile` (not `Dockerfile.build`) with a `test` stage; `docker-compose.yml` reduced to a single `test` service.
20. **Coverage gate** — wire `cargo-llvm-cov` with `--fail-under-lines 100 --fail-under-functions 100`. Already in scope as `LATER-006`.
21. **Test naming** — rename existing `sec_uc<num>_*` Rust tests to `test_sec_uc###_*` per org convention.
22. **Optional: multilingual user manuals** — `docs/user-manual-{en,ua,es,fr,it,pt}.md`. Lowest priority.

Status: **DONE** — findings recorded; AC-1, AC-2, AC-3 met.

## Speck Reference

N/A — pure-research sub-task whose deliverable IS the `## Findings` section here
(per §3.4 wording allowing pure-docs leaves without a Speck file).
