# xdu — Installation Manual

`xdu` ships as a single static binary per platform. There is no installer,
no daemon, no shared library. Drop the binary anywhere on `PATH` and run it.

## Supported platforms (v0.1.0)

| Target triple                        | OS                              | Distribution form               |
|--------------------------------------|---------------------------------|---------------------------------|
| `x86_64-unknown-linux-musl`          | Linux x86_64                    | Fully static ELF (musl)         |
| `aarch64-unknown-linux-musl`         | Linux aarch64 (ARM64)           | Fully static ELF (musl)         |
| `x86_64-apple-darwin`                | macOS Intel                     | Mach-O 64-bit (libSystem)       |
| `aarch64-apple-darwin`               | macOS Apple Silicon             | Mach-O 64-bit (libSystem)       |
| `x86_64-pc-windows-msvc`             | Windows x86_64                  | PE32+ console (MSVC CRT)        |
| `aarch64-pc-windows-msvc`            | Windows ARM64                   | PE32+ console (MSVC CRT)        |

Other targets (*BSD, RISC-V Linux, etc.) are recorded as v0.2 follow-ups in
[`docs/spec.md`](spec.md) §9 (LATER-005).

## Install

### Linux

```sh
chmod +x xdu
sudo mv xdu /usr/local/bin/xdu
xdu --version
```

The musl static binary runs on any Linux kernel 3.2+ regardless of glibc
version — verified to run inside `alpine:3.20` (which has only musl).

### Windows

1. Place `xdu.exe` somewhere on `PATH` (e.g. `%USERPROFILE%\bin\xdu.exe`).
2. Open a terminal and run `xdu.exe --version`.

ANSI colors require a terminal that handles VT escapes — Windows Terminal,
modern PowerShell, or Windows console host with VT processing enabled.

### macOS

1. Place `xdu` somewhere on `PATH` (e.g. `~/.local/bin/xdu` or
   `/usr/local/bin/xdu`):
   ```sh
   chmod +x xdu
   sudo mv xdu /usr/local/bin/xdu
   xdu --version
   ```
2. On Apple Silicon download `aarch64-apple-darwin`; on Intel macs download
   `x86_64-apple-darwin`. Both link against the system `libSystem` so they
   run on any macOS 11+ host.

## First-use sanity check

```sh
xdu --version
xdu --help
xdu /path/to/inspect
```

`xdu` is **read-only by default**. The only filesystem-mutating action is
the explicit `d` keybinding (delete), which opens a confirmation modal
whose default focus is `[ No ]` — see [`docs/spec.md`](spec.md) §4 BR-001
(amended) and §6 UC-008. There is no `rename`/`chmod`/`truncate` path.

## Uninstall

Delete the binary. There are no config files, no caches, no registry
entries.

## Building your own binary

See [`docs/development-manual.md`](development-manual.md). A single
`docker compose run --rm build` produces a fresh stripped binary of
~600 KB.
