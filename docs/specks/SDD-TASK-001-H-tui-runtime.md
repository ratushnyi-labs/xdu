# SDD-TASK-001-H — TUI runtime

**Backlog item:** TASK-001-H
**Impacted UCs:** UC-001 (TUI init), UC-002 (input loop), UC-007

## Scope
Terminal init/teardown (alt screen, raw mode), event loop, panic-safe restoration via `Drop` guard + `panic::set_hook`, ≥10 Hz redraw cap, UTF-8 detection from `LANG`/`LC_ALL`/`LC_CTYPE`, `NO_COLOR` honoring delegated to ratatui's `Style::default()` policies.

## ACs
- AC-1: `tui::run(scan, mode)` enables raw mode + alt screen, runs event loop, restores on quit.
- AC-2: A panic during render restores the terminal before propagating (panic hook).
- AC-3: Ctrl-C and `q` always quit.
- AC-4: Event poll uses 100 ms timeout — frame loop tolerates 10 Hz redraw without busy-spin.
- AC-5: `is_utf8_terminal()` returns true on Windows (default) or when locale env names contain `UTF-8`.

## SEC ACs
- SEC-1: Terminal restored on every exit path (normal, error, panic).
- SEC-2: No input echo during raw mode.
- SEC-3: `Drop` guard ensures restoration even when caller forgets.

## Test
TUI runtime is verified by:
- Running the binary in a real terminal (manual).
- Multi-target builds confirm cross-platform compilation.
TUI snapshot harness is deferred to v0.2 (recorded in `docs/spec.md` §9).

Implemented in `src/tui.rs` (combined with I/J/K).
