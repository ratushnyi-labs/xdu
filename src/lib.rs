//! xdu — read-only disk-usage explorer.
//!
//! Library entry point. Modules implement scanner, tree model, formatting,
//! TUI, and export. The binary in `src/main.rs` wires them together.

pub mod disk_space;
pub mod docker_shrink;
pub mod drives_win;
pub mod export;
pub mod format;
pub mod lock_detect;
pub mod scanner;
pub mod time_machine;
pub mod trash;
pub mod tree;
pub mod tui;
