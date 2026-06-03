# Dependency audit — 2026-06-03

**Date:** 2026-06-03 (UTC)
**Tool:** `cargo-audit-audit 0.22.1`
**Rust toolchain:** `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` / `rustc 1.95.0 (59807616e 2026-04-14)`
**Advisory database:** RustSec advisory-db (1103 advisories loaded)
**Command:** `scripts/audit.sh` → `docker compose run --rm -T test sh -c 'cargo audit'`
**Container:** `xdu-test` (Dockerfile `base` stage; lawbook §8.2 host isolation preserved)
**Backlog:** [TASK-007-P](../backlog/TASK-007-P-audit-and-coverage-baseline.md)
**Speck:** [SDD-TASK-007-P](../specks/SDD-TASK-007-P-audit-and-coverage-baseline.md)

## Captured output

```
==> cargo audit (workspace, locked)
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 1103 security advisories (from /usr/local/cargo/advisory-db)
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (70 crate dependencies)
Crate:     paste
Version:   1.0.15
Warning:   unmaintained
Title:     paste - no longer maintained
Date:      2024-10-07
ID:        RUSTSEC-2024-0436
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0436
Dependency tree:
paste 1.0.15
└── ratatui 0.29.0
    └── xdu 0.1.0

Crate:     lru
Version:   0.12.5
Warning:   unsound
Title:     `IterMut` violates Stacked Borrows by invalidating internal pointer
Date:      2026-01-07
ID:        RUSTSEC-2026-0002
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0002
Dependency tree:
lru 0.12.5
└── ratatui 0.29.0
    └── xdu 0.1.0

warning: 2 allowed warnings found
```

Process exit code: `0` (cargo-audit classifies `unmaintained` / `unsound`
notices as warnings rather than vulnerabilities; no `RUSTSEC` vulnerability
advisories triggered a non-zero exit).

## Verdict

**2 surviving advisories — both informational (warnings, not vulnerabilities).**

| ID                 | Crate         | Kind          | Reaches xdu via         | First-party fix available? |
|--------------------|---------------|---------------|-------------------------|----------------------------|
| RUSTSEC-2024-0436  | paste 1.0.15  | unmaintained  | ratatui 0.29.0 → paste  | No — transitive only       |
| RUSTSEC-2026-0002  | lru 0.12.5    | unsound       | ratatui 0.29.0 → lru    | No — transitive only       |

Neither advisory carries a CVSS score or marks an exploitable vulnerability.
Both are transitive dependencies pulled in by `ratatui 0.29.0`, the TUI
toolkit xdu depends on directly. xdu does not call `paste!` macros nor
`lru::IterMut` itself; the unsound `IterMut` codepath is reachable only
through ratatui's internal caches, none of which xdu drives via the
affected API.

No vulnerability-class advisories surfaced. Zero hard-fail RUSTSEC IDs.

## Compliance Notes — surviving advisories pending user approval

Per lawbook §7 + Speck AC-2, surviving advisories require per-advisory
rationale + explicit user approval before TASK-007-O (publication) fires.

### RUSTSEC-2024-0436 — `paste` unmaintained

- **Reachability:** transitive, via `ratatui 0.29.0 → paste 1.0.15`. xdu
  does not import `paste` directly.
- **Exploitability:** none. "Unmaintained" is a maintenance signal, not a
  vulnerability. The crate compiles and functions; the upstream author has
  stopped responding to issues.
- **Mitigation path:** upstream ratatui is tracking a replacement
  (https://github.com/ratatui-org/ratatui/issues for the in-flight
  migration). xdu inherits any fix automatically by bumping ratatui once a
  paste-free release lands.
- **Recommendation:** ACCEPT for v0.1.0. Re-check on every dependabot bump
  of ratatui and drop this note once ratatui releases a paste-free version.

### RUSTSEC-2026-0002 — `lru` IterMut unsound

- **Reachability:** transitive, via `ratatui 0.29.0 → lru 0.12.5`. xdu does
  not import `lru` directly nor invoke `IterMut`.
- **Exploitability:** sub-low. The unsoundness is a Stacked-Borrows
  violation in `IterMut`; reachable only when code mutably iterates an
  `lru::LruCache`. Ratatui uses LRU internally for layout/style caches and
  does not expose `IterMut` to consumers, so xdu's surface is not
  affected.
- **Mitigation path:** lru 0.13.0+ ships the fix. Upstream ratatui bump to
  a fixed lru is the natural path; xdu inherits the fix on the next
  ratatui release.
- **Recommendation:** ACCEPT for v0.1.0. Re-check on every dependabot bump
  of ratatui / lru and drop this note once a fixed version is in the
  dependency tree.

**Action for user (TASK-007-O gate):** approve or reject these two notes
before publishing v0.1.0. If rejected, the alternative is to fork ratatui
or pin an older ratatui that predates the affected versions — both are
strictly worse trade-offs than accepting two transitive warnings.
