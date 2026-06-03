**EN** · [UA](user-manual-ua.md) · [ES](user-manual-es.md) · [FR](user-manual-fr.md) · [IT](user-manual-it.md) · [PT](user-manual-pt.md)

# xdu — user manual

This is the v0.1.0 end-user manual for `xdu`, a read-only disk-usage explorer
with a terminal UI. It is intentionally short. Everything described here works
in the released binaries; nothing is forward-looking.

## What xdu does

`xdu` scans a directory tree and presents the result as a navigable terminal
UI. Each row shows a name, a relative-size bar, the absolute size of that
entry (including all descendants for directories), and the last-access date.
The footer's right edge shows the free space on the volume containing the
scan root. You move around with the arrow keys or vim keys, descend into a
directory with Enter, and ascend with Backspace. There is an export action
(`e`) that writes the focused subtree to a JSON file, an errors overlay (`!`)
for per-path scan failures, and an in-app help panel (`?` or F1).

## What xdu does NOT do

xdu is read-only by default. It does not delete, move, rename, `chmod`,
truncate, kill processes, or modify your files in any way unless you press a
specific key and confirm. It is not a service or a daemon. It runs only while
you are interacting with it. It does not phone home, collect telemetry, send
analytics, or open any network connection. It does not have an auto-delete
mode, a background cleaner, a "smart" suggestion engine, or a GUI mode in
v0.1.0.

## Install

Pre-built binaries for v0.1.0 are published on the GitHub Releases page:
<https://github.com/ratushnyi-labs/xdu/releases>. Download the archive for
your platform, extract it, and place the `xdu` (or `xdu.exe` on Windows)
binary somewhere on your `PATH`.

**Linux (musl).** Two static builds are published: `x86_64-unknown-linux-musl`
and `aarch64-unknown-linux-musl`. Both are fully static — no glibc, no shared
library dependencies — and run on any Linux kernel 3.2 or newer. Pick the one
matching `uname -m` (`x86_64` or `aarch64`).

**macOS.** Two builds are published: `x86_64-apple-darwin` (Intel) and
`aarch64-apple-darwin` (Apple Silicon). After downloading, you may need to
clear the quarantine attribute that macOS attaches to downloaded binaries:
`xattr -d com.apple.quarantine ./xdu`.

**Windows (MSVC).** Two PE32+ console builds are published:
`x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`. They are standalone
`.exe` files — no installer, no DLL dependencies beyond the MSVC runtime that
ships with Windows. Run from `cmd.exe`, PowerShell, or Windows Terminal.

## First run

Launching `xdu` with no argument scans the current working directory on Unix.
On Windows, launching with no argument shows a numbered list of mounted
drives (system drive, removable media, network shares with a known total
size); type the number and press Enter, or just press Enter to pick the first
entry. Pass an explicit path (`xdu /home/me`, `xdu C:\Users`) to skip the
picker. While the initial scan runs you see a progress line; once the scan
finishes the TUI takes over and you can navigate immediately.

## Navigation

Move the selection with the up/down arrows or `k` / `j`. Descend into the
selected directory with the right arrow, `l`, or Enter. Ascend to the parent
with the left arrow, `h`, or Backspace; alternatively, select the `..` row
at the top of the listing and press Enter. Ascending past the original scan
root triggers a fresh filesystem-level rescan of the new parent. Press `q`
or Ctrl-C to quit. Press `?` or F1 at any time for a panel summarising every
keybinding.

## Sort and units

Press `s` to cycle the sort key. The cycle order is: size descending (the
default — biggest first), size ascending, name ascending, name descending,
file-count descending, file-count ascending. Press `s` again to keep going;
the active sort key is shown in the footer.

Press `u` to toggle the size unit between IEC binary units (`KiB`, `MiB`,
`GiB`, `TiB` — powers of 1024) and SI decimal units (`KB`, `MB`, `GB`, `TB`
— powers of 1000). You can also start in SI mode by passing `--si` on the
command line. The mode applies to every size on screen consistently —
selected row, free-space footer, and any export written while the mode is
active.

## Delete and trash

xdu is read-only by default. To delete the selected entry, press `d`. A
confirmation modal appears with `[ No ]` focused; pressing Enter or Esc
cancels. To proceed, navigate to `[ Yes ]` (Tab or right-arrow) and press
Enter. This [No]-default behaviour is a deliberate safety choice and is the
same in every mode.

Run `xdu --trash` to switch into trash mode. In trash mode the confirmation
modal reads "Move to trash" instead of "Delete", and the action targets the
operating system's native trash:

- **Linux:** the freedesktop.org Trash specification v1.0. Files move into
  `$XDG_DATA_HOME/Trash/files/` (falling back to `$HOME/.local/share/Trash/
  files/`), with a sibling `.trashinfo` metadata file. The standard
  file-manager "Restore from trash" actions work afterwards.
- **Windows:** the system Recycle Bin, via the same kernel path Explorer uses.
- **macOS:** not supported in v0.1.0. Running `xdu --trash` on macOS exits
  non-zero with a clear message. The non-trash `d`-key delete continues to
  work normally on macOS.

Trash mode is still gated by the [No]-default confirmation modal — there is
no "delete without asking" path anywhere in xdu.

## Lock-holder and force-close

When a delete on Windows fails because another process has the file open,
xdu enumerates the holders via the Windows Restart Manager and shows an
informational modal listing the process names and PIDs. From that modal you
can press `f` to attempt a force-close (this calls `DuplicateHandle` with
`DUPLICATE_CLOSE_SOURCE` on each foreign handle); the force-close itself is
gated by a separate [No]-default confirmation modal. Aggressive: only use it
when you know what the lock-holder is.

On Linux, before a delete is attempted, xdu opportunistically walks
`/proc/*/fd/` and lists any process with an open file descriptor pointing at
the selected path. The list is informational only — Linux's POSIX `unlink`
succeeds even when there are open holders, so the delete proceeds on
confirmation. The intent is to let you decide whether to delete a file that
is currently in use.

## Errors overlay

A scan over a real filesystem will occasionally hit a per-path error: a
permission-denied directory, a stale NFS handle, a broken symlink. xdu
records every such error without stopping the scan; press `!` to open the
errors overlay and read them. The overlay scrolls with the arrow keys and
closes with Esc.

## Export

Press `e` to export the currently focused subtree to a JSON file. xdu prompts
for the output path with `xdu-export.json` in the current directory as the
default; press Enter to accept the default or type a new path. The file
format is documented in `docs/spec.md` (schema tag `xdu-export-1`). The
export is purely a write of the chosen file — it does not touch the scanned
filesystem.

## Safety note

xdu is read-only by default. The only writes it ever performs are:

1. An explicit delete via the `d` key, gated by a [No]-default confirmation
   modal.
2. An optional trash-mode delete via `--trash` + `d`, still gated by the
   [No]-default modal, targeting the OS-native trash where supported.
3. A JSON export via the `e` key, writing one file at the path you confirm.

There is no auto-delete, no background mutation, no rename, no `chmod`, no
`truncate`, no `kill` / `TerminateProcess`, and no network activity of any
kind. If something on your filesystem changes while xdu is running, it is
because you pressed a key and confirmed.
