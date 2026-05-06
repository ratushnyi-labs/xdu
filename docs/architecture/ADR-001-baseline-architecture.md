# ADR-001 — Baseline architecture for `xdu` v0.1.0

**Status:** PROPOSED (awaiting user acceptance)
**Date:** 2026-05-06
**Owner:** TASK-001
**Decision drivers (in priority order, derived from user request):**
1. Single static binary, "zero runtime dependency", per supported target
2. Size-optimized output (smaller is better; budget tentatively < 1.5 MB stripped)
3. "Any system" — Linux, macOS, Windows, with reasonable terminal support
4. Read-only feature scope (UC-001 … UC-007)
5. Deterministic, low-risk, no telemetry, no network

## 1. Context

The tool runs locally, has no network or shared state, and produces visual output in
a terminal. Topology is therefore fixed: one process, one binary. The architectural
choices that materially change cost / size / risk are:

- **Filesystem traversal strategy** (sync stdlib vs parallel walker)
- **TUI library** (mature crate vs hand-rolled ANSI vs Unix-only crate)
- **Tree storage** (arena-indexed vs `Box`-tree vs flat hashmap)
- **Error/concurrency boundary** (single-thread vs scanner thread + UI thread)
- **Build & distribution pipeline** (cross via Docker images, target triples, profile)

Per lawbook §3.1.3 this is a new product idea, so architecture option analysis is mandatory.

## 2. Options considered

### Option A — `crossterm` + `ratatui` + sync stdlib walker + arena tree + scanner thread

| Aspect            | Detail                                                                    |
|-------------------|---------------------------------------------------------------------------|
| TUI               | `ratatui` (active, Apache-2.0/MIT, mature) on `crossterm` backend         |
| Walker            | `std::fs::read_dir` + `Metadata`, recursive, single-threaded              |
| Tree              | `Vec<Node>` arena indexed by `NodeId(u32)` (parent / first_child / next_sib)|
| Concurrency       | One scanner thread; UI on main; `mpsc` channel for progress + completion  |
| Crates added      | `crossterm`, `ratatui`, `unicode-width`, `clap` (or hand-rolled), tiny atomics |
| Pros              | Good Windows console support; well-tested; modest LOC; predictable size   |
| Cons              | ~150–250 KB of dependency code in binary even after `opt-level=z` + LTO   |
| Estimated stripped musl-x86_64 binary | ~700 KB – 1.2 MB                                          |
| Dev velocity      | High                                                                      |
| Risk              | Low                                                                       |

### Option B — Hand-rolled ANSI + sync stdlib walker + arena tree + scanner thread

| Aspect            | Detail                                                                    |
|-------------------|---------------------------------------------------------------------------|
| TUI               | Direct ANSI escape sequences; manual `termios` (Unix) and `ENABLE_VIRTUAL_TERMINAL_PROCESSING` (Windows) |
| Walker            | Same as A                                                                 |
| Tree              | Same as A                                                                 |
| Concurrency       | Same as A                                                                 |
| Crates added      | Optional `windows-sys` (tiny, manual API) or none if using `std::os::windows::raw::HANDLE` + FFI |
| Pros              | Smallest binary (~250–400 KB); no third-party TUI                         |
| Cons              | Substantial extra LOC; reinvents resize/colors/parsing/Windows console quirks |
| Estimated stripped musl-x86_64 binary | ~250 – 450 KB                                              |
| Dev velocity      | Low                                                                       |
| Risk              | Medium-high (terminal edge cases, Windows VT mode, resize signals)        |

### Option C — `crossterm` + `ratatui` + parallel walker (`jwalk` or custom rayon-based) + arena + multi-thread

| Aspect            | Detail                                                                    |
|-------------------|---------------------------------------------------------------------------|
| TUI               | Same as A                                                                 |
| Walker            | Parallel directory walk; results pushed into a lock-protected arena       |
| Tree              | `Vec<Node>` arena behind `Mutex` or sharded                               |
| Concurrency       | Many scanner threads + UI thread                                          |
| Crates added      | `jwalk` (pulls `rayon`, `crossbeam`) ~+150 KB; or custom thread-pool      |
| Pros              | 2–4× faster scans on SSD/NVMe                                             |
| Cons              | Larger binary; more complex error model; non-deterministic ordering       |
| Estimated stripped musl-x86_64 binary | ~1.0 – 1.6 MB                                              |
| Dev velocity      | Medium                                                                    |
| Risk              | Medium                                                                    |

## 3. Tradeoff matrix

| Driver                      | Weight | A   | B   | C   |
|-----------------------------|--------|-----|-----|-----|
| Static, zero runtime deps   | =      | ✓   | ✓   | ✓   | (all are pure-Rust, statically linked)
| Size-optimized              | high   | ◐   | ✓✓  | ✗   |
| Any-system support          | high   | ✓✓  | ◐   | ✓✓  | (B's manual Windows console is risky)
| Read-only / safety surface  | high   | ✓✓  | ◐   | ✓   | (A: smaller LOC = less attack surface than B's hand-rolled paths)
| Dev velocity                | high   | ✓✓  | ✗   | ✓   |
| Maintenance                 | medium | ✓✓  | ✗   | ✓   |
| Scan speed                  | low    | ◐   | ◐   | ✓✓  |

## 4. Decision

**Adopt Option A** (`crossterm` + `ratatui` + sync stdlib walker + arena tree + scanner thread)
as v0.1.0 baseline.

### Rationale
- "Static + zero runtime dep" is interpreted as **no dynamic shared-library dependency at runtime** (musl static / `+crt-static` / `cargo build --release` with selected target). Pure-Rust crates are statically linked into the single binary and do not violate that. Confirmed by user phrasing "static 0 dependency size optimized output."
- Option B saves ~300–700 KB but at the cost of substantial LOC, terminal-edge-case bug surface, and Windows console fragility — directly conflicts with priority-3 ("any system") and increases bug surface despite priority-4 (read-only safety).
- Option C buys speed we do not need at v0.1.0 and bloats the binary in conflict with priority-2.
- Size budget for A on `x86_64-unknown-linux-musl` after `[profile.release-min]` (`opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`): **target < 1.0 MB**, hard ceiling **1.5 MB**.
- If the realized binary exceeds 1.5 MB or the user later raises size priority, **Phase 3 optimization** (lawbook §13.5 conceptual analogue, even though §13 doesn't apply here) revisits B incrementally:
  - drop `clap` for hand-rolled argv parser (saves ~80–150 KB)
  - drop `unicode-width` for inline tables for the few glyphs we use (~30 KB)
  - if needed, move from `ratatui` to direct `crossterm` rendering primitives
  - last resort: full B migration

## 5. Build & distribution pipeline

| Aspect              | Decision                                                                  |
|---------------------|---------------------------------------------------------------------------|
| Toolchain location  | Inside Docker (lawbook §8.2 — host isolation)                             |
| Builder image       | Multi-stage; `rust:1.x-alpine` (stage `build`); per-target images for cross|
| Targets v0.1.0      | `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-pc-windows-gnu` (chosen over MSVC for fully containerized builds via `mingw-w64`), `x86_64-apple-darwin` + `aarch64-apple-darwin` (via `osxcross` toolchain image; documented limitation: macOS link to libSystem cannot be fully static — accepted) |
| Profile             | `release-min`: `opt-level="z"`, `lto="fat"`, `codegen-units=1`, `panic="abort"`, `strip="symbols"`, `incremental=false` |
| Linker              | `mold` (Linux) for build speed; `ld` final pass for size determinism      |
| Output              | `/out/<target>/xdu(.exe)` extracted via `docker compose run --rm build`   |
| Reproducibility     | Pin Rust toolchain via `rust-toolchain.toml`; lock all crate versions     |
| CI simulation (§8.7)| `scripts/ci.sh` running typecheck → lint (`clippy -D warnings`) → unit → integration → TUI snapshot → multi-target release builds → security & dep audit (`cargo-audit`, `cargo-deny`) |
| HEALTHCHECK in image| N/A (build image, not runtime)                                            |
| Production stage    | `scratch` (final binary only) per §8.4 — distroless / scratch is preferred |
| `.dockerignore`     | required                                                                  |

Note on Windows target: `x86_64-pc-windows-gnu` chosen for v0.1.0 to keep all builds inside
Linux Docker (no Windows host needed). MSVC + `+crt-static` is recorded as an alternative
to be evaluated in TASK-001-D if `gnu` target produces measurably larger binaries or
compatibility issues; switching is a non-blocking optimization.

## 6. Test architecture

- **Unit tests:** colocated `#[cfg(test)]`. Pure logic only (formatter, sorter, tree ops, hardlink-set).
- **Integration tests:** `tests/` dir; spin a temp filesystem in `./.dev-temp/` (lawbook §8.3) and run scanner end-to-end.
- **TUI snapshot tests:** custom test harness using a virtual terminal (`vt100` crate or hand-rolled) to render the TUI to a buffer, then assert character + style cells against `docs/ui-baselines/UC-###/<step>.txt`. This is the §13.3.4 spirit applied without §13's strict parity requirement.
- **Security tests:** `[SEC]`-labeled cases for path-traversal arguments, oversize inputs, null-byte filenames, symlink loops, hardlink chains, very deep trees.
- **Coverage gate:** 100% lines + branches + functions on the library crate (lawbook §7.1). Excludes `main.rs` glue (justification recorded per-test-file).
- All test stages run inside the same Docker network as build (lawbook §6.2).

## 7. Forbidden during v0.1.0 implementation

- Adding any crate not listed in §2 Option A's "Crates added" without an ADR amendment.
- Any code path that opens a file for write/append other than: explicit JSON export (UC-006), terminal stdout/stderr.
- `unsafe` outside small clearly-justified blocks (Windows API FFI, vt parser if any). Each block needs a `// SAFETY:` comment.
- Async runtime (`tokio`, `async-std`). Synchronous code is sufficient and smaller.

## 8. Evolution triggers (when to revisit this ADR)

- Realized binary > 1.5 MB after `release-min` profile → trigger size-cut review (move toward B).
- User report of scan time > 60 s on commodity SSD for ~5 M files → trigger Option C reconsideration.
- Need for non-Latin glyph width handling beyond `unicode-width` capability → reassess.
- Request for plugin/scripting architecture → trigger architecture rework, not patches.

## 9. Acceptance

User confirmation moves status `PROPOSED → ACCEPTED` and unblocks `ARCH_READY → PLAN_READY`.
