# TASK-007-N1 — LICENSE + Cargo.toml metadata align to org

**Status:** DONE  **Type:** TASK  **Parent:** TASK-007
**User Mode:** GUIDED  **Guided Flow Stage:** POST_AUDIT

Verification: `cargo fmt`/`clippy`/`test --locked` all clean (93 tests). Smoke release build of `x86_64-unknown-linux-musl` succeeded → 752,360 B (~735 KiB), under the 1 MiB target. Pre-merge baseline security gate: PASS — metadata-only; no code paths added; BR-001 preserved. §9.5 cross-agent attack review skipped per session-wide rule.
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request
Inherited from TASK-007 (user answers D1/D1.1/D1.2/D1.3/D2/D3 plus `fo`→`go`).

## Refined Description
- LICENSE: change `Copyright (c) 2026 xdu contributors` → `Copyright (c) 2026 Pavlo Ratushnyi`.
- Cargo.toml `[package]`: add `repository`, `homepage`, `authors`. Keep `keywords` / `categories`.
- Cargo.toml `[profile.release]`: replace the custom `[profile.release-min]` with the org-standard `[profile.release]` settings (`opt-level="z"`, `lto=true`, `codegen-units=1`, `panic="abort"`, `strip=true`).
- Update consumers of the old `release-min` name: `Dockerfile.build` and `scripts/build.sh` switch from `--profile release-min` to `--release`.

**Impacted UCs:** none (metadata only).
**Impacted BR/WF:** none.

## Estimation
**Complexity:** EASY. Single-author metadata change + two consumer updates.

## Speck Reference
[`docs/specks/SDD-TASK-007-N1-license-and-cargo-metadata.md`](../specks/SDD-TASK-007-N1-license-and-cargo-metadata.md)

## Compliance Notes — §9.5
Cross-agent attack review skipped per session-wide rule (no second-agent vendor reachable). See parent TASK-007.
