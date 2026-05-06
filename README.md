# xdu

Read-only disk-usage explorer with a terminal UI. Single static binary per
platform, well under 1 MiB.

```
 /home/me/projects/xdu/                                                          
██████████  675 KiB  out/x86_64-unknown-linux-musl/xdu                          
█████▌      354 KiB  src/
███▏        198 KiB  docs/
██▌         141 KiB  Cargo.lock
▏             3 KiB  README.md
```

`xdu` scans a directory and lets you navigate the result with the keyboard.
Each row shows the entry's share of its parent as a bar plus the absolute
size in `B`/`KiB`/`MiB`/`GiB`/`TiB` (or SI units with `--si`). The footer's
right edge shows free space on the volume containing the scan root.

`xdu` is **read-only by default**. The only filesystem-mutating action is
the explicit `d` keybinding, which opens a confirmation modal whose default
focus is `[ No ]`. There is no `rename` / `chmod` / `truncate` path.
No network calls, no telemetry.

## Features

- Recursive scan with progress.
- Interactive tree navigation (arrow keys + `hjkl`).
- Sort by size / name / count, ascending or descending.
- IEC binary (`KiB`) and SI decimal (`KB`) units.
- Hardlink deduplication (Unix; Windows v0.2).
- Mount-boundary detection (Unix; Windows v0.2).
- JSON export of the current subtree.
- Error log: per-entry I/O failures are recorded but do not abort the scan.
- Read-only by design — no modify / delete keybindings exist.

## Install

### Linux

```sh
chmod +x xdu
./xdu /path/to/inspect
```

The musl static binary runs on any Linux kernel ≥ 3.2 regardless of glibc
version (verified inside `alpine:3.20` which has only musl).

### Windows

```pwsh
.\xdu.exe C:\path\to\inspect
```

### macOS

Build from source (osxcross automation is v0.2 work):

```sh
cargo build --profile release-min
```

See [`docs/installation-manual.md`](docs/installation-manual.md) for
target details and per-platform notes.

## Keybindings

| Key | Action |
|-----|--------|
| `↑` / `k`, `↓` / `j` | move selection |
| `→` / `l` / Enter | enter directory |
| `←` / `h` / Backspace | go up |
| `s` | cycle sort key |
| `u` | toggle IEC / SI units |
| `r` | rescan from root |
| `e` | export current subtree to `xdu-export.json` |
| `d` | **delete** selected entry (with `[ No ]`-default confirmation modal) |
| `!` | show error log |
| `?` / `F1` | show help |
| `q` / Ctrl-C | quit |

Full table in [`docs/configuration-manual.md`](docs/configuration-manual.md).

## Build

All builds run inside Docker per project rule §8.2 (host isolation). No
host install of Rust required.

```sh
sh scripts/build.sh        # all 3 supported targets, with size enforcement
sh scripts/ci.sh           # fmt + clippy + check + test + multi-target build
```

Single-target build:

```sh
TARGET=x86_64-unknown-linux-musl docker compose run --rm build
```

Output: `out/<TARGET>/xdu(.exe)`.

Build-time size budgets per ADR-001:

- Hard ceiling: 1.5 MiB stripped (build fails if exceeded).
- Target: < 1.0 MiB (v0.1.0 hits ~600 KiB).

## Project documentation

- [`docs/spec.md`](docs/spec.md) — specification (UC-001 … UC-007, BR-001 … BR-006, WF-001 / WF-002, known limitations).
- [`docs/architecture/ADR-001-baseline-architecture.md`](docs/architecture/ADR-001-baseline-architecture.md) — architecture decision record.
- [`docs/development-manual.md`](docs/development-manual.md) — Docker-based dev workflow.
- [`docs/installation-manual.md`](docs/installation-manual.md) — per-platform install.
- [`docs/configuration-manual.md`](docs/configuration-manual.md) — every flag, every env var, color rules, size budgets.
- [`docs/specks/`](docs/specks/) — implementation Specks per leaf.
- [`docs/backlog/`](docs/backlog/) — backlog and lifecycle records.

## License

[MIT](LICENSE).
