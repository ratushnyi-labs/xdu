# xdu — Installation Manual

`xdu` ships as a single static binary per platform. There is no installer,
no daemon, no shared library. Drop the binary anywhere on `PATH` and run it.

## Supported platforms (v0.1.0)

| Target triple                        | OS                              | Distribution form          |
|--------------------------------------|---------------------------------|----------------------------|
| `x86_64-unknown-linux-musl`          | Linux x86_64                    | Fully static ELF (musl)    |
| `aarch64-unknown-linux-musl`         | Linux aarch64 (ARM64)           | Fully static ELF (musl)    |
| `x86_64-pc-windows-gnu`              | Windows x86_64                  | PE32+ console executable   |

macOS and *BSD targets are recorded as v0.2 follow-ups in
[`docs/spec.md`](spec.md) §9 (LATER-005). Build from source on macOS via
the host Rust toolchain — the same `cargo build --profile release-min`
invocation used by the Docker pipeline works.

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

### macOS (build from source)

1. Install Rust via `rustup` (host toolchain — note this is a deviation
   from the Docker-only project rule, accepted for macOS until osxcross
   automation lands in v0.2).
2. From the project root:
   ```sh
   cargo build --profile release-min
   ```
3. Binary at `target/release-min/xdu`.

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
