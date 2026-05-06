# SDD-TASK-001-C — Repo scaffold

**Backlog item:** TASK-001-C
**Impacted UCs:** none directly (infrastructure)
**Impacted BR/WF:** none

## Scope / Non-goals

**In scope:**
- `LICENSE` (MIT, year 2026)
- `.gitignore` (Rust + IDE + OS noise + lawbook `./.dev-data/` + `./.dev-temp/`)
- `.dockerignore` (mirror `.gitignore` plus `.git/`)
- `Cargo.toml` with package metadata and `release-min` profile per ADR-001
- `rust-toolchain.toml` pinning channel `stable`
- `src/main.rs` minimal stub: print version + exit 0 — must compile cleanly
- `README.md` skeleton (replaced fully by TASK-001-N)
- `.editorconfig`
- `git init` + initial commit referencing TASK-001-C

**Out of scope:**
- `Dockerfile.build`, `docker-compose.yml`, build scripts → TASK-001-D
- Source modules (scanner, tree, tui, etc.) → TASK-001-E…L
- Tests → TASK-001-M
- Final user-facing README + manuals → TASK-001-N

## Acceptance Criteria

- AC-1: All scaffold files listed above exist at the documented paths.
- AC-2: `Cargo.toml`'s `[package]` block declares: `name = "xdu"`, `version = "0.1.0"`, `edition = "2021"`, `license = "MIT"`, `rust-version = "1.78"`.
- AC-3: `Cargo.toml` defines `[profile.release-min]` inheriting `release` with `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`.
- AC-4: `[dependencies]` is empty in this leaf (deps added by their owning leaves).
- AC-5: `src/main.rs` compiles under stable Rust and prints `xdu v0.1.0` to stdout, exits 0.
- AC-6: `LICENSE` is the unmodified MIT text with year `2026` and copyright holder placeholder; **no agent attribution**.
- AC-7: `.gitignore` includes `/target/`, `/.dev-data/`, `/.dev-temp/`, `/out/`, IDE files (`.idea/`, `.vscode/`, `*.swp`), OS noise (`Thumbs.db`, `.DS_Store`), and `Cargo.lock` is **NOT** ignored (binary crate convention is to commit it).
- AC-8: `.dockerignore` excludes `.git/`, `target/`, `.dev-data/`, `.dev-temp/`, `out/`, `docs/` (build cache need not see docs), `*.md` at root EXCEPT `README.md` (kept for build metadata).
- AC-9: `git init` succeeds and produces a single commit titled `TASK-001-C: repo scaffold`.

## Security ACs

- SEC-1: No file written outside the working directory.
- SEC-2: `.gitignore` excludes any local-secret patterns conventionally (`.env*` excluding `.env.example`).
- SEC-3: No hidden directories (other than `.git`, `.dev-data`, `.dev-temp`, `.editorconfig`).
- SEC-4: Initial commit excludes target/, build artifacts, and any host-machine-specific paths.
- SEC-5: No code in `src/main.rs` reads, opens, or writes any file at runtime in the scaffold version.

## Failure Modes / Error Mapping

- Cargo build failure (e.g., unknown profile field) → fix Cargo.toml syntax; re-attempt.
- `git init` failure (already initialized) → check `.git/` and either reuse or refuse.

## Test Matrix

| AC    | Unit | Integration | Build | TUI snapshot | Production stack |
|-------|------|-------------|-------|--------------|-----------------|
| AC-1  | —    | —           | ✅    | —            | —                |
| AC-2  | —    | —           | ✅    | —            | —                |
| AC-3  | —    | —           | ✅    | —            | —                |
| AC-4  | —    | —           | ✅    | —            | —                |
| AC-5  | —    | ✅ (TASK-001-D) | ✅ | —            | —                |
| AC-6  | —    | —           | manual| —            | —                |
| AC-7  | —    | —           | manual| —            | —                |
| AC-8  | —    | —           | ✅ (TASK-001-D) | —    | —                |
| AC-9  | —    | —           | manual| —            | —                |
| SEC-1 | —    | —           | manual| —            | —                |
| SEC-5 | —    | ✅ (TASK-001-M) | — | —            | —                |

Build / runtime ACs (AC-3, AC-5) are validated end-to-end by TASK-001-D's pipeline and TASK-001-M's tests; this leaf only ensures the scaffold *exists* in the right shape.

## Dependencies & guardrails

- Lawbook §5.2 line / function / class limits apply (none breached at scaffold scope).
- Lawbook §5.3 "no placeholders / TODOs / dead code" — `main.rs` prints version, which is real working code, not a placeholder.
- Lawbook §5.4 stealth identity — no agent / model / vendor attribution anywhere.
- Lawbook §8.2 host isolation — no command run on host other than `git` (allowed lightweight repo op).
