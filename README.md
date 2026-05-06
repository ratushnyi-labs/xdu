# xdu

Read-only disk-usage explorer with a terminal UI. Single static binary per platform.

**Status:** v0.1.0 — early development. Scaffold only at the moment; see
[`docs/backlog/`](docs/backlog/) for what is being built next and
[`docs/spec.md`](docs/spec.md) for the full specification.

## What it does

`xdu` scans a directory and lets you navigate the resulting tree to see where the
space is used. For each entry it shows a relative size bar and the absolute size
in `B / KiB / MiB / GiB / TiB` (or SI units with `--si`).

The tool is **strictly read-only**. It cannot delete, rename, or modify any file.

## Why

- A single, statically-linked binary that runs the same on Linux, macOS, and Windows.
- Zero runtime dependencies — drop the binary into a recovery shell, a container,
  or a locked-down corporate machine and it works.
- Size-optimized — small enough to embed in build images.
- Read-only by design — safe to run on any path.

## Building

All builds run inside Docker per the project's host-isolation rules
([`docs/architecture/ADR-001-baseline-architecture.md`](docs/architecture/ADR-001-baseline-architecture.md) §5).
Build pipeline lands in **TASK-001-D**; until then, only the scaffold compiles.

## Documentation

- [`docs/spec.md`](docs/spec.md) — specification (UCs, business rules, workflows)
- [`docs/architecture/ADR-001-baseline-architecture.md`](docs/architecture/ADR-001-baseline-architecture.md) — architecture decision record
- [`docs/backlog/`](docs/backlog/) — backlog and sub-tasks

## License

[MIT](LICENSE)
