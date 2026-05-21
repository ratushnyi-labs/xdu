# SDD-TASK-007-J — Trash mode

**Backlog item:** TASK-007-J
**Impacted UCs:** UC-008 (delete confirmation), augmented with a trash
variant.

## Scope
- New module `src/trash.rs`:
  - `pub fn move_to_trash(path: &Path) -> io::Result<()>`.
  - `pub fn is_supported() -> bool`.
  - **Linux:** freedesktop.org Trash spec v1.0:
    - Trash root = `$XDG_DATA_HOME/Trash` (fallback `$HOME/.local/share/Trash`).
    - Allocate a unique name in `Trash/files/` (appending `.1`, `.2`, … on
      collision).
    - `rename(2)` the source into place — atomic on the same fs.
    - Write a sibling `Trash/info/<name>.trashinfo` with `[Trash Info]`,
      `Path=` (RFC-2396-percent-encoded), `DeletionDate=` (ISO 8601 UTC,
      reusing `format::format_iso8601_utc`).
  - **Windows:** `SHFileOperationW(FO_DELETE | FOF_ALLOWUNDO | FOF_NO_UI |
    FOF_NOERRORUI)` — the same kernel path the Explorer "Send to Recycle
    Bin" command uses.
  - **macOS / BSD / others:** returns `io::ErrorKind::Unsupported`. macOS
    (`NSFileManager.trashItemAtURL`) is a v0.2 follow-up.
- `src/lib.rs`: export `trash` module.
- `src/main.rs`:
  - Add `--trash` flag.
  - On startup with `--trash` on an unsupported platform: print
    `xdu: --trash is not supported on this platform` and exit 2.
  - New `tui::run_with(...)` variant that takes a `trash_mode: bool`.
- `src/tui.rs`:
  - `App` gains a `trash_mode: bool` field.
  - New `App::with_trash(...)` constructor; `App::new(...)` defaults to
    `trash_mode: false` (backwards-compatible for existing tests).
  - `start_delete_prompt` message reads "Move to trash {kind} ?" when
    `trash_mode == true`, else "Delete {kind} ?". The default focus on
    `[ No ]` is preserved unchanged.
  - `perform_delete`: when `trash_mode == true`, route through
    `trash::move_to_trash` instead of `remove_file` / `remove_dir_all`.
    Success status reads "moved {path} to trash" instead of "deleted {path}".
  - Footer composition prepends `mode:trash` to the right-side segment when
    active, and the delete-hint reads `d:trash` instead of `d:delete`.

## ACs
- AC-1: `cargo build --locked` succeeds on Linux.
- AC-2: Linux test `linux_trash_round_trip_and_collisions` exercises both
  the basic round-trip (file gone from source, present in trash, valid
  `.trashinfo` written) and the name-collision suffixing.
- AC-3: On non-Linux/non-Windows targets `is_supported()` returns `false`
  and `move_to_trash()` returns `ErrorKind::Unsupported`.
- AC-4: `--trash` on an unsupported platform exits non-zero with a clear
  message; on a supported platform it activates trash mode end-to-end.
- AC-5: BR-001 invariant preserved: trash mode is still gated by the
  `d`-keybinding confirmation modal whose default focus is `[ No ]`. Read-
  only is the default; trash mode is opt-in via `--trash`.

## SEC ACs
- SEC-1: Linux impl uses safe `std::fs::rename` + `File::create` only; no
  new `unsafe` blocks.
- SEC-2: Windows impl is one FFI site, with a `// SAFETY:` block
  documenting buffer lifetime + double-NUL termination per the
  SHFileOperationW contract.
- SEC-3: Path canonicalization happens BEFORE the rename so the
  `.trashinfo` `Path=` field records the absolute source path — never an
  attacker-controlled relative path.
- SEC-4: `url_encode_path` escapes whitespace, `%`, `#`, `?`, control
  bytes, and non-ASCII bytes per the freedesktop spec. Forward slash is
  preserved as a literal because `Path=` is a path, not a URL.
- SEC-5: The `.trashinfo` file is a regular file created via `File::create`
  — no symlink trickery.

## §9.5 cross-agent attack review
Skipped per session-wide rule.
