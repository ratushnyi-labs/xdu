//! End-to-end integration tests for the xdu library API.
//!
//! Each test builds an ephemeral filesystem under `std::env::temp_dir()`,
//! runs the public scanner / tree / format / export entry points, and
//! asserts the round-trip behaviour described in `docs/spec.md` UC-001
//! through UC-006. Cleanup happens via `TempDir`'s `Drop`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use xdu::export;
use xdu::format::{format_size, SizeMode};
use xdu::scanner::{scan, ScanOpts};

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("xdu-it-{label}-{pid}-{nanos}-{n}"));
        fs::create_dir_all(&p).unwrap();
        TempDir { path: p }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn end_to_end_scan_format_export_round_trip() {
    let td = TempDir::new("e2e");
    fs::write(td.path.join("readme.txt"), b"hello world").unwrap();
    fs::create_dir(td.path.join("src")).unwrap();
    fs::write(td.path.join("src").join("main.rs"), b"fn main() {}\n").unwrap();

    let result = scan(&td.path, &ScanOpts::default()).expect("scan must succeed");
    let root = result.tree.get(result.tree.root()).unwrap();
    assert_eq!(root.size, 11 + 13, "sum of file sizes");
    assert_eq!(root.entries, 4, "root + readme + src + main.rs");

    let formatted = format_size(root.size, SizeMode::Iec);
    assert!(formatted.ends_with(" B") || formatted.ends_with(" KiB"));

    let export_path = td.path.join("xdu-export.json");
    export::write_to_path(
        &result.tree,
        result.tree.root(),
        &ScanOpts::default(),
        &export_path,
    )
    .expect("export must succeed");

    let exported = fs::read_to_string(&export_path).unwrap();
    assert!(exported.contains(r#""schema":"xdu-export-1""#));
    assert!(exported.contains(r#""name":"readme.txt""#));
    assert!(exported.contains(r#""name":"main.rs""#));
    assert!(exported.contains(r#""kind":"dir""#));
    assert!(exported.contains(r#""kind":"file""#));
}

#[test]
fn scan_returns_error_on_missing_path() {
    let bogus = std::env::temp_dir().join("xdu-it-definitely-missing-99999999999");
    let r = scan(&bogus, &ScanOpts::default());
    assert!(r.is_err(), "missing path must return Err, not panic");
}

#[test]
fn scan_returns_error_on_non_directory() {
    let td = TempDir::new("nondir");
    let f = td.path.join("file.txt");
    fs::write(&f, b"x").unwrap();
    let r = scan(&f, &ScanOpts::default());
    assert!(r.is_err());
}

#[test]
fn empty_dir_scans_to_root_only() {
    let td = TempDir::new("empty");
    let r = scan(&td.path, &ScanOpts::default()).unwrap();
    assert_eq!(r.tree.len(), 1);
    let root = r.tree.get(r.tree.root()).unwrap();
    assert_eq!(root.entries, 1);
    assert_eq!(root.size, 0);
}

#[test]
fn export_includes_aggregated_directory_sums() {
    let td = TempDir::new("agg");
    fs::create_dir(td.path.join("a")).unwrap();
    fs::write(td.path.join("a").join("f"), b"abcdefghij").unwrap();
    let r = scan(&td.path, &ScanOpts::default()).unwrap();
    let mut buf = Vec::new();
    export::write_to_writer(&r.tree, r.tree.root(), &ScanOpts::default(), &mut buf).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains(r#""size_bytes":10"#));
    assert!(out.contains(r#""kind":"dir""#));
}

#[test]
fn export_path_security_rejects_traversal() {
    let td = TempDir::new("sec");
    fs::write(td.path.join("a"), b"x").unwrap();
    let r = scan(&td.path, &ScanOpts::default()).unwrap();
    let bad: &Path = Path::new("a/../../escape.json");
    let res = export::write_to_path(&r.tree, r.tree.root(), &ScanOpts::default(), bad);
    assert!(res.is_err(), "must reject `..` in export path");
}
