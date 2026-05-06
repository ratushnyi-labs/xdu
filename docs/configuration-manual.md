# xdu — Configuration Manual

`xdu` has no configuration file. Behavior is controlled entirely by CLI
flags and a small set of standard environment variables. Each setting
listed here is documented in `docs/spec.md`.

## Command-line flags

| Flag                | UC      | Effect                                                                     |
|---------------------|---------|----------------------------------------------------------------------------|
| _(positional path)_ | UC-001  | Directory to scan. Defaults to `.` if omitted.                             |
| `--cross-fs`        | UC-001  | Allow scan to cross filesystem mount/volume boundaries (Unix only — see BR-003 + spec §9). |
| `--si`              | UC-003  | Display sizes in SI decimal units (1 KB = 1000 B). Default is IEC binary (1 KiB = 1024 B). |
| `-V`, `--version`   | —       | Print `xdu vX.Y.Z` and exit 0.                                              |
| `-h`, `--help`      | —       | Print help text and exit 0.                                                 |

Unknown flags exit with code 2 and a stderr message.

## Exit codes

| Code | Meaning                                                                   |
|------|---------------------------------------------------------------------------|
| 0    | Normal exit (scan completed; user quit the TUI).                          |
| 2    | CLI usage error — invalid flag, missing path, or path is not a directory. |
| 3    | Fatal TUI / I/O error during the interactive session.                     |

Per-entry filesystem errors during a scan **do not** raise the exit code
(BR-005). They are recorded in the in-memory error log and visible via
the `!` keybinding in the TUI.

## Environment variables

| Variable    | Effect                                                                              |
|-------------|-------------------------------------------------------------------------------------|
| `NO_COLOR`  | If set to any non-empty value, disables ANSI color output. Honored via ratatui.     |
| `LANG`, `LC_ALL`, `LC_CTYPE` | Inspected for the substrings `UTF-8` / `UTF8`. If found, UTF-8 block-element bars render; otherwise the ASCII (`#` / space) fallback is used. |
| `TERM`      | Honored by `crossterm` for terminal capability detection.                            |
| `TARGET`    | (Build-time only.) Selects the Rust target triple in `Dockerfile.build` and `scripts/build.sh`. |
| `SIZE_CEILING_BYTES` | (Build-time only.) Overrides the default 1.5 MiB binary-size hard ceiling. |

`xdu` does **not** read any environment variable for telemetry, network,
or update checking — it makes no network calls anywhere.

## Color decision tree

1. If `NO_COLOR` is set → all rendering uses default terminal style.
2. Otherwise:
   - Directory rows use `Color::Cyan`.
   - Rows whose share of the current directory's total exceeds **25%** become `Color::Red`.
   - Rows whose share exceeds **10%** but is ≤ 25% become `Color::Yellow`.
   - Rows on a node flagged `error == true` become `Color::Red` regardless of share.
   - The currently-selected row gains `Modifier::REVERSED` on top of the above.

## Size formatting

`format_size(bytes, mode)` produces:

- For `SizeMode::Iec` (default): `B / KiB / MiB / GiB / TiB / PiB / EiB`, divisor 1024.
- For `SizeMode::Si` (`--si`): `B / KB / MB / GB / TB / PB / EB`, divisor 1000.
- 0–999 in the chosen unit render with no decimal: `500 KiB`.
- 1.0–99.9 render with one decimal: `1.4 MiB`.

## Bar rendering

10-cell relative bar per row.

- UTF-8 mode (default on Windows; Unix when `LANG`/`LC_ALL`/`LC_CTYPE`
  contain `UTF-8`/`UTF8`): full block `█` with 7 partial-block glyphs at
  1/8 increments.
- ASCII fallback: `#` for filled cells, space for empty cells.

## Keybindings (TUI)

| Key            | Action                                                                  |
|----------------|-------------------------------------------------------------------------|
| `↑` / `k`      | Move selection up.                                                       |
| `↓` / `j`      | Move selection down.                                                     |
| `→` / `l` / Enter | Enter directory.                                                       |
| `←` / `h` / Backspace | Go up.                                                              |
| `PgUp` / `PgDn` | Scroll one page.                                                         |
| `Home` / `End` | Jump to first / last entry.                                              |
| `s`            | Cycle sort key (size↓ → name↑ → count↓ → size↑ → name↓ → count↑ → …).   |
| `u`            | Toggle IEC / SI size units.                                              |
| `r`            | Rescan from root (synchronous; v0.2 will offer per-subtree async rescan). |
| `e`            | Export the current subtree to `xdu-export.json` in CWD.                  |
| `!`            | Show error log overlay.                                                  |
| `?` / `F1`     | Show help overlay.                                                       |
| `q` / Ctrl-C   | Quit (terminal restored on every exit path including panic).            |

## Build-time budgets (ADR-001 §5)

- Hard ceiling: **1.5 MiB** per stripped target binary. Configurable via `SIZE_CEILING_BYTES`.
- Target: < **1.0 MiB**. Achieved (v0.1.0): musl-x86_64 ≈ 675 KiB, musl-aarch64 ≈ 608 KiB, windows-gnu-x86_64 ≈ 476 KiB.
- Profile: `release-min` (`opt-level="z"`, `lto="fat"`, `codegen-units=1`, `panic="abort"`, `strip="symbols"`).
- No new crate dependencies without an ADR amendment.

## What `xdu` will NEVER do

- Read file contents (only metadata).
- Modify, delete, or move any file.
- Open network sockets.
- Spawn subprocesses.
- Send telemetry or analytics anywhere.
- Persist state between runs (no cache, no config file).
