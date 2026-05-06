# SDD-TASK-001-J — Header / footer / help overlay / errors overlay

**Backlog item:** TASK-001-J
**Impacted UCs:** UC-002, UC-004, UC-007

## Scope
Persistent header (root path + breadcrumb, reverse-video bar). Persistent footer (sort indicator, error count, key hints, reverse-video bar). Centered modal help overlay listing every keybinding. Centered modal error overlay listing recorded `ScanError`s (capped at 50 visible).

## ACs
- AC-1: Header shows breadcrumb path (root/.../current).
- AC-2: Footer shows `sort:<key><dir>  errors:<n>  ?:help  s:sort  u:unit  q:quit`.
- AC-3: `?` / `F1` opens help overlay; any keypress dismisses.
- AC-4: `e` opens errors overlay; any keypress dismisses.
- AC-5: Help overlay lists every accepted keybinding.

Implemented in `src/tui.rs`.
