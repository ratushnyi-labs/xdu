# SDD-TASK-007-N5 — README rewrite

**Backlog item:** TASK-007-N5
**Impacted UCs:** none (presentation)

## Scope
Rewrite `README.md` to match the org pattern: centered hero, three-badge row (CI / License: MIT / Release), Supported Targets table (6 targets confirmed in wave 4), Quick Start, Install (Pre-built + Build from Source), Testing, Documentation table, Contributing, Security, License. Multilingual user-manual table row is stubbed for TASK-007-N10.

## ACs
- AC-1: First lines render as a centered hero (`<p align="center">...</p>`).
- AC-2: Badges row present with the three canonical badges, each linking to the correct URL.
- AC-3: Supported Targets table lists all 6 targets (x86_64/aarch64 × {linux-musl, apple-darwin, pc-windows-msvc}).
- AC-4: Documentation section links to `docs/spec.md`, `docs/development-manual.md`, `docs/installation-manual.md`, `docs/configuration-manual.md`.
- AC-5: Multilingual user-manual row stubbed (links resolve once N10 lands; broken links are acceptable mid-wave but the row's structure is in place).
- AC-6: License section points at `LICENSE` (MIT).
- AC-7: No agent attribution.

## SEC ACs
- SEC-1: No secrets, no telemetry pixels, no analytics scripts.
- SEC-2: External links use HTTPS.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
