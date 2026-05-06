//! xdu — read-only disk-usage explorer.
//!
//! Library entry point. Modules implement scanner, tree model, formatting,
//! TUI, and export. The binary in `src/main.rs` wires them together.

pub mod format;
