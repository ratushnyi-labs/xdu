<p align="center">
  <h1 align="center">xdu</h1>
  <p align="center">Read-only disk-usage explorer with a terminal UI. Single static binary per platform.</p>
</p>

<p align="center">
  <a href="https://github.com/ratushnyi-labs/xdu/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/ratushnyi-labs/xdu/actions/workflows/ci.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue.svg"></a>
  <a href="https://github.com/ratushnyi-labs/xdu/releases"><img alt="Release" src="https://img.shields.io/github/v/release/ratushnyi-labs/xdu"></a>
</p>

---

`xdu` scans a directory and lets you navigate the resulting tree to see where the space
is used. Each row shows the entry name, a relative-size bar, the absolute size
(`B / KiB / MiB / GiB / TiB`, or SI units with `--si`), and the last-access date.
The footer's right edge shows free space on the volume containing the scan root.

xdu is **read-only by default**. The only mutating actions are opt-in behind explicit
keybindings — `d` deletes (with a `[ No ]`-default confirmation modal) and, on Windows,
`f` force-closes foreign file handles via `DuplicateHandle(DUPLICATE_CLOSE_SOURCE)` to
unblock a locked file (also gated by a `[ No ]`-default confirmation modal).
No `rename` / `chmod` / `truncate` / `kill` / `TerminateProcess` anywhere.
No network, no telemetry.

## Supported Targets

| Target                            | Arch  | Notes                                              |
|-----------------------------------|-------|----------------------------------------------------|
| `x86_64-unknown-linux-musl`       | x64   | Fully static; runs on any Linux kernel 3.2+        |
| `aarch64-unknown-linux-musl`      | ARM64 | Fully static; runs on any Linux kernel 3.2+        |
| `x86_64-apple-darwin`             | x64   | macOS Intel                                        |
| `aarch64-apple-darwin`            | ARM64 | macOS Apple Silicon                                |
| `x86_64-pc-windows-msvc`          | x64   | PE32+ console; lock-holder detection via `rstrtmgr` |
| `aarch64-pc-windows-msvc`         | ARM64 | PE32+ console; lock-holder detection via `rstrtmgr` |

## Quick Start

```sh
xdu                  # scan current directory
xdu /home/me/code    # scan a specific path
xdu /                # scan the filesystem root (Unix)
xdu C:\              # scan a drive root (Windows)
xdu --si             # use SI units (KB, MB, ...) instead of IEC (KiB, MiB, ...)
xdu --cross-fs       # follow mount points (Unix; v0.2 on Windows)
```

Inside the TUI:

| Key | Action |
|-----|--------|
| `↑` / `k`, `↓` / `j`  | move selection |
| `→` / `l` / Enter      | enter directory |
| `←` / `h` / Backspace  | go up (rescans filesystem parent at scan root) |
| `s`                    | cycle sort key (size↓ / name↑ / count↓ / size↑ / name↓ / count↑) |
| `u`                    | toggle IEC / SI size units |
| `r`                    | rescan from root |
| `e`                    | export current subtree to `xdu-export.json` |
| `d`                    | **delete** selected entry (with `[ No ]`-default confirmation modal) |
| `f`                    | (inside lock-prompt) **force-close** foreign handles (Windows, with confirmation) |
| `!`                    | show error log |
| `?` / `F1`             | show help |
| `q` / Ctrl-C           | quit |

## Install

### Pre-built binaries

Download from [GitHub Releases](https://github.com/ratushnyi-labs/xdu/releases).

```sh
# Linux x64
curl -sSfL https://github.com/ratushnyi-labs/xdu/releases/latest/download/xdu-x86_64-unknown-linux-musl.tar.gz \
    | tar xzf - -C /usr/local/bin

# Linux ARM64
curl -sSfL https://github.com/ratushnyi-labs/xdu/releases/latest/download/xdu-aarch64-unknown-linux-musl.tar.gz \
    | tar xzf - -C /usr/local/bin
```

For macOS and Windows, download the matching archive and place the binary somewhere
on your `PATH`.

### Build from source

All builds run inside Docker per the project's host-isolation rule:

```sh
# Single target
TARGET=x86_64-unknown-linux-musl docker compose run --rm build

# All six targets
sh scripts/build.sh
```

Output: `dist/<TARGET>/xdu(.exe)`. Each binary is checked against a 1.5 MiB hard
ceiling (configurable via `SIZE_CEILING_BYTES` in `.env`).

## Testing

```sh
docker compose run --rm test sh -c "\
  cargo fmt --all -- --check \
  && cargo clippy --all-targets --all-features --locked -- -D warnings \
  && cargo test --locked"
```

## Documentation

| Document                                                   | Audience      |
|------------------------------------------------------------|---------------|
| [Specification](docs/spec.md)                              | Designers     |
| [Installation Manual](docs/installation-manual.md)         | End-users     |
| [Configuration Manual](docs/configuration-manual.md)       | End-users     |
| [Development Manual](docs/development-manual.md)           | Contributors  |
| [Architecture Decision](docs/architecture/ADR-001-baseline-architecture.md) | Contributors |

### User Manual

| Language     | File |
|--------------|------|
| English      | [docs/user-manual-en.md](docs/user-manual-en.md) |
| Українська   | [docs/user-manual-ua.md](docs/user-manual-ua.md) |
| Español      | [docs/user-manual-es.md](docs/user-manual-es.md) |
| Français     | [docs/user-manual-fr.md](docs/user-manual-fr.md) |
| Italiano     | [docs/user-manual-it.md](docs/user-manual-it.md) |
| Português    | [docs/user-manual-pt.md](docs/user-manual-pt.md) |

Non-English manuals are machine translations for v0.1.0, pending human review.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](.github/SECURITY.md).

## License

[MIT](LICENSE)
