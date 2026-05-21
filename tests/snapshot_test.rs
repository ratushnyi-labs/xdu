//! TUI snapshot tests (TASK-007-F / LATER-004).
//!
//! Each test:
//!   1. Builds a fixed in-memory `Tree` (no scanner I/O).
//!   2. Wraps it in an `App` via `tui::app_from_tree_for_test`.
//!   3. Renders to a virtual terminal at a fixed size.
//!   4. Compares against a golden file under `docs/ui-baselines/snapshots/`.
//!
//! To regenerate baselines after an intentional UI change:
//!
//! ```sh
//! XDU_BLESS_SNAPSHOTS=1 cargo test --test snapshot_test
//! ```
//!
//! The bless mode writes new baselines instead of asserting. CI must never
//! set the env var.

use std::path::PathBuf;
use xdu::tree::{NodeKind, Tree};
use xdu::tui::{app_from_tree_for_test, render_snapshot};

fn baseline_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("ui-baselines")
        .join("snapshots")
        .join(name)
}

fn assert_or_bless(name: &str, actual: &str) {
    let path = baseline_path(name);
    let bless = std::env::var("XDU_BLESS_SNAPSHOTS")
        .map(|v| v != "0" && !v.is_empty())
        .unwrap_or(false);

    if bless || !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create baseline dir");
        }
        std::fs::write(&path, actual).expect("write baseline");
        if !bless {
            // First-run convenience: when the baseline did not exist we wrote
            // it; surface that fact in the test output but do not fail. The
            // committed baseline now locks in the layout.
            eprintln!("[snapshot] created new baseline at {}", path.display());
        }
        return;
    }

    let expected = std::fs::read_to_string(&path).expect("read baseline");
    if expected != actual {
        // Print a unified-ish diff hint without pulling in a diff crate.
        eprintln!(
            "\n--- expected ({})\n{}\n--- actual\n{}\n---",
            path.display(),
            expected,
            actual
        );
        panic!(
            "snapshot mismatch for {name}.\n\
             If this change is intentional, re-run with XDU_BLESS_SNAPSHOTS=1 to update."
        );
    }
}

/// Build a fixture tree with a stable set of children. All sizes/aliases are
/// finalized so we render real numbers (not the in-progress spinner).
fn fixture_tree_three_children() -> Tree {
    let mut t = Tree::new(NodeKind::Dir, "fixture-root");
    let root = t.root();
    let _a = t
        .add_child(root, NodeKind::Dir, "alpha", 0, 0)
        .expect("add alpha");
    let _b = t
        .add_child(root, NodeKind::File, "beta.bin", 5_000_000, 5_000_000)
        .expect("add beta");
    let _c = t
        .add_child(root, NodeKind::File, "gamma.log", 1_500_000, 1_500_000)
        .expect("add gamma");
    // Make alpha non-empty so it gets a real size after aggregation.
    let alpha = t
        .add_child(_a, NodeKind::File, "inner.dat", 800_000, 800_000)
        .expect("add inner.dat");
    t.bubble_up_add(alpha, 800_000, 800_000, 1);
    t.bubble_up_add(_b, 5_000_000, 5_000_000, 1);
    t.bubble_up_add(_c, 1_500_000, 1_500_000, 1);
    // Finalize everything so size cells render (no spinners).
    t.finalize_all();
    t
}

#[test]
fn snapshot_main_list_80x24() {
    let tree = fixture_tree_three_children();
    let app = app_from_tree_for_test(PathBuf::from("/fixture-root"), tree);
    let out = render_snapshot(&app, 80, 24);
    assert_or_bless("main_list_80x24.txt", &out);
}

#[test]
fn snapshot_main_list_narrow_40x20() {
    let tree = fixture_tree_three_children();
    let app = app_from_tree_for_test(PathBuf::from("/fixture-root"), tree);
    let out = render_snapshot(&app, 40, 20);
    assert_or_bless("main_list_narrow_40x20.txt", &out);
}

#[test]
fn snapshot_empty_tree_80x24() {
    let mut t = Tree::new(NodeKind::Dir, "empty-root");
    t.finalize_all();
    let app = app_from_tree_for_test(PathBuf::from("/empty-root"), t);
    let out = render_snapshot(&app, 80, 24);
    assert_or_bless("empty_tree_80x24.txt", &out);
}
