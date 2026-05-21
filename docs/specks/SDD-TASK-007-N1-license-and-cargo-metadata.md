# SDD-TASK-007-N1 — LICENSE + Cargo.toml metadata align

**Backlog item:** TASK-007-N1
**Impacted UCs:** none (metadata only)

## Scope
- `LICENSE`: change copyright holder line to `Copyright (c) 2026 Pavlo Ratushnyi`.
- `Cargo.toml` `[package]`: add `repository = "https://github.com/ratushnyi-labs/xdu"`, `homepage = ...`, `authors = ["Pavlo Ratushnyi"]`.
- `Cargo.toml` profile: remove `[profile.release-min]` and update `[profile.release]` to the org settings (`opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`).
- Update consumers: `Dockerfile.build` (`--profile release-min` → `--release`) and `scripts/build.sh` (same).

## ACs
- AC-1: `LICENSE` copyright line is `Copyright (c) 2026 Pavlo Ratushnyi`.
- AC-2: `Cargo.toml` exposes `repository`, `homepage`, `authors` matching the convention.
- AC-3: `[profile.release-min]` is removed; `[profile.release]` carries the size-optimized settings.
- AC-4: `Dockerfile.build` and `scripts/build.sh` use `--release` (no `release-min` references).
- AC-5: Multi-target build still produces stripped binaries under 1.5 MiB each (no size regression).

## SEC ACs
- SEC-1: No new code paths; no `unsafe`; no behaviour change. Read-only invariant unaffected.
- SEC-2: §5.4 stealth identity preserved — no agent attribution introduced.
- SEC-3: No new crate dependency.

## §9.5 cross-agent attack review
Skipped per session-wide rule (no second-agent vendor reachable). See parent TASK-007.
