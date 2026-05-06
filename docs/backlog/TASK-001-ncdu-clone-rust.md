# TASK-001 — xdu (Rust disk-usage explorer, ncdu-inspired)

**Status:** DECOMPOSED
**Type:** TASK
**Entry Mode:** IDEA_FIRST (greenfield; functional clone, NOT a §13 migration)
**User Mode:** GUIDED
**Guided Flow Stage:** PLAN_READY (sub-tasks A & B DONE; C IN_PROGRESS this turn)
**Initial code-change expectation:** YES
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Raw Request

> ok, your task is to create app: clone of ncdu that should work under any system. use rust, prioritize static 0 dependency size optimized output

## Clarification Round 1 (2026-05-06)

> idea - tool to see where the space is lost. so 1:1 interface is not needed. it should:
> * allow navigate with storage
> * display relative size (as text bars) and absolute size in b/kb/mb/...
> tool should not have interface to delete or change anything. only information for humans.
> license mit

## Resolved Answers + Assumptions

| Q  | Topic                | Answer                                                                                          |
|----|----------------------|-------------------------------------------------------------------------------------------------|
| Q1 | Parity strictness    | **FUNCTIONAL CLONE** (confirmed: "1:1 interface is not needed"). §13 does NOT apply.            |
| Q2 | Platforms            | **Assumed default:** Linux x86_64 + aarch64 (musl, fully static); Windows x86_64 (MSVC `+crt-static`); macOS x86_64 + aarch64 (self-contained, libSystem dynamic — limit of platform). |
| Q3 | Features (read-only) | Confirmed read-only: NO delete, NO modify. Scan, navigate, bars + absolute size, sort, refresh, JSON export, help. NO JSON import (read-only philosophy). |
| Q4 | Names                | **Assumed:** binary `xdu`, crate `xdu`.                                                         |
| Q5 | License              | **MIT** (confirmed).                                                                            |
| Q6 | Build/test execution | **Assumed default:** all toolchain + builds + tests run in Docker (lawbook §8.2). Static binaries extracted as artifacts. |

GUIDED-mode prerequisites surfaced (none required for this CLI tool):
- No mail / payment / analytics / auth / external API integrations are in scope.
- Telemetry: explicitly OUT.

## Lifecycle Coverage Map

| Stage          | State        | Notes                                                  |
|----------------|--------------|--------------------------------------------------------|
| CLARIFICATION  | DONE         | All blocking questions resolved or defaulted           |
| ESTIMATION     | IN_PROGRESS  | Provisional VERY_HARD; decomposition follows           |
| DECOMPOSITION  | IN_PROGRESS  | Skeleton listed below; sub-task files in next turn     |
| DESIGN         | IN_PROGRESS  | ADR-001 authored this turn                             |
| FRONTEND       | PENDING      | TUI                                                    |
| BACKEND        | PENDING      | Scanner / tree / exporter                              |
| TESTING        | PENDING      | Unit + integration + TUI snapshot, container-first     |
| POST_AUDIT     | PENDING      | Mandatory pre-merge baseline gate (§9.4)               |

## Guided Flow History

| From           | To           | When                | Evidence                                            |
|----------------|--------------|---------------------|-----------------------------------------------------|
| —              | IDEA         | 2026-05-06 intake   | Raw request                                         |
| IDEA           | CLARIFICATION| 2026-05-06          | Six blocking questions raised                       |
| CLARIFICATION  | SPEC_READY   | 2026-05-06          | Q1/Q3/Q5 answered; Q2/Q4/Q6 defaulted; spec drafted |
| SPEC_READY     | ARCH_READY   | 2026-05-06          | ADR-001 authored                                    |

## Refined Description

**Scope (v0.1.0):** A read-only disk-usage explorer named `xdu`, written in Rust,
distributed as a single statically-linked size-optimized binary per supported target,
with a terminal UI that lets the user scan a directory and navigate the resulting tree.
Each entry displays an absolute size (B/KiB/MiB/GiB/TiB) and a relative text bar.

**Non-goals (v0.1.0):**
- No file/directory deletion or modification (BR-001).
- No graphical UI; terminal only.
- No remote/network scanning.
- No archive-format introspection (zips/tars treated as opaque files).
- No telemetry, analytics, network calls.
- No JSON import (read-only philosophy; export only).

**Impacted UCs:** new spec, see [`docs/spec.md`](../spec.md). UC-001…UC-007.
**Impacted BR/WF:** all new — BR-001…BR-006, WF-001, WF-002.
**Dependencies:** none (greenfield).
**Risks:**
- Cross-platform terminal capability quirks (Windows console vs Unix tty).
- "Static, zero runtime dep" interpretation on macOS (libSystem cannot be statically linked).
- Scanning very deep trees (Linux `PATH_MAX` interactions; symlink loops if symlinks ever enabled).
- Size budget vs. crate ecosystem tradeoffs (covered in ADR-001).

**Architecture decision:** see [`docs/architecture/ADR-001-baseline-architecture.md`](../architecture/ADR-001-baseline-architecture.md).

## Estimation

**Complexity:** **VERY_HARD** (parent only). Will be `DECOMPOSED`. No leaf may exceed `ABOVE_EASY`.

**Justification:** multi-target static build + TUI + scanner + tree + tests across the matrix
exceeds a single implementable scope. Decomposition along architecture/BE/FE/test/build axes
brings each leaf inside `ABOVE_EASY` (§12.5–12.7).

## Sub-task Decomposition (v0.1.0)

Skeleton — individual `docs/backlog/TASK-001-<id>-*.md` files will be created next turn.
Each leaf MUST be re-estimated independently and reach `ABOVE_EASY` or `EASY` before BUILD.

| Sub-task ID         | Title                                            | Layer    | Est.     | Depends on    | Status        |
|---------------------|--------------------------------------------------|----------|----------|---------------|---------------|
| TASK-001-A          | Author `docs/spec.md` v0.1.0                     | DESIGN   | EASY     | —             | DONE          |
| TASK-001-B          | ADR-001: baseline architecture                   | DESIGN   | EASY     | —             | DONE          |
| TASK-001-C          | Repo scaffold (Cargo + LICENSE + git + ignores)  | INFRA    | EASY     | A, B          | IN PROGRESS   |
| TASK-001-D          | Docker build pipeline (multi-stage, multi-target)| INFRA    | ABOVE_EASY| C            | READY         |
| TASK-001-E          | Scanner module (BE)                              | BACKEND  | ABOVE_EASY| C            | READY         |
| TASK-001-F          | Tree model (BE)                                  | BACKEND  | ABOVE_EASY| C            | READY         |
| TASK-001-G          | Size formatter + bar (BE)                        | BACKEND  | EASY     | C             | READY         |
| TASK-001-H          | TUI runtime (FE)                                 | FRONTEND | ABOVE_EASY| C            | READY         |
| TASK-001-I          | Tree view widget (FE)                            | FRONTEND | ABOVE_EASY| F, G, H       | READY         |
| TASK-001-J          | Header + footer + help overlay (FE)              | FRONTEND | EASY     | H             | READY         |
| TASK-001-K          | Refresh / rescan flow                            | INTEG    | EASY     | E, I          | READY         |
| TASK-001-L          | JSON export                                      | BACKEND  | EASY     | F             | READY         |
| TASK-001-M          | Test suite (unit + integration + TUI snapshot)   | TEST     | ABOVE_EASY| E, F, G, H, I | READY         |
| TASK-001-N          | Documentation (manuals + README final)           | DOC      | EASY     | all of E–L    | READY         |

**Execution order (waves):**
1. A, B (this turn, partial)
2. C
3. D, E, F, G (parallelizable)
4. H
5. I, J
6. K, L
7. M, N
8. POST_AUDIT (security gate §9.4)

## Speck Reference

`docs/specks/SDD-TASK-001-xdu.md` (parent) plus per-leaf Specks where the leaf changes
behavior. To be authored at `PLAN_READY`.

## Open Items For User

- Confirm decomposition order above (or request changes).
- Confirm v0.1.0 scope boundary (anything else considered MUST-have for v0.1.0?).
- Anything in §Resolved Assumptions you want to override?
