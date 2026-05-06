//! JSON exporter for the scan tree. Hand-rolled streaming encoder — no
//! `serde_json` dependency. Schema `"xdu-export-1"` per `docs/spec.md`
//! UC-006. Iterative tree walk via explicit `Vec<Frame>` — call-stack-safe.

use crate::scanner::ScanOpts;
use crate::tree::{NodeId, NodeKind, Tree};
use std::io::{self, Write};
use std::path::Path;

const FORBIDDEN_PREFIX_UNIX: &[&str] = &["/proc/", "/sys/", "/dev/"];

pub fn write_to_path(tree: &Tree, root: NodeId, opts: &ScanOpts, path: &Path) -> io::Result<()> {
    validate_path(path)?;
    let mut file = std::fs::File::create(path)?;
    write_to_writer(tree, root, opts, &mut file)
}

pub fn write_to_writer<W: Write>(
    tree: &Tree,
    root_id: NodeId,
    opts: &ScanOpts,
    w: &mut W,
) -> io::Result<()> {
    let scanned_at = current_iso8601();
    let root_node = tree
        .get(root_id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "unknown root node id"))?;
    write!(w, r#"{{"schema":"xdu-export-1","scanned_at":""#)?;
    w.write_all(scanned_at.as_bytes())?;
    write!(w, r#"","root":"#)?;
    write_json_string(w, root_node.name.as_ref())?;
    write!(
        w,
        r#","options":{{"cross_fs":{},"si":false,"follow_symlinks":false}},"tree":"#,
        opts.cross_fs
    )?;
    write_subtree(tree, root_id, w)?;
    writeln!(w, "}}")?;
    Ok(())
}

enum Frame {
    Enter(NodeId),
    Comma,
    Close,
}

fn write_subtree<W: Write>(tree: &Tree, root: NodeId, w: &mut W) -> io::Result<()> {
    let mut stack: Vec<Frame> = vec![Frame::Enter(root)];
    while let Some(frame) = stack.pop() {
        match frame {
            Frame::Comma => write!(w, ",")?,
            Frame::Close => write!(w, "]}}")?,
            Frame::Enter(id) => write_enter(tree, id, w, &mut stack)?,
        }
    }
    Ok(())
}

fn write_enter<W: Write>(
    tree: &Tree,
    id: NodeId,
    w: &mut W,
    stack: &mut Vec<Frame>,
) -> io::Result<()> {
    let Some(n) = tree.get(id) else {
        write!(w, "null")?;
        return Ok(());
    };
    write!(w, r#"{{"name":"#)?;
    write_json_string(w, n.name.as_ref())?;
    write!(
        w,
        r#","kind":"{}","size_bytes":{},"alloc_bytes":{},"entries":{},"error":{}"#,
        kind_str(n.kind),
        n.size,
        n.alloc,
        n.entries,
        n.error
    )?;
    let mut kids: Vec<NodeId> = tree.children(id).collect();
    if kids.is_empty() {
        write!(w, "}}")?;
        return Ok(());
    }
    kids.sort_by(|&a, &b| {
        let na = tree.get(a).map(|x| x.name.as_ref()).unwrap_or("");
        let nb = tree.get(b).map(|x| x.name.as_ref()).unwrap_or("");
        na.cmp(nb)
    });
    write!(w, r#","children":["#)?;
    stack.push(Frame::Close);
    for (i, &cid) in kids.iter().enumerate().rev() {
        stack.push(Frame::Enter(cid));
        if i > 0 {
            stack.push(Frame::Comma);
        }
    }
    Ok(())
}

fn kind_str(k: NodeKind) -> &'static str {
    match k {
        NodeKind::Dir => "dir",
        NodeKind::File => "file",
        NodeKind::Symlink => "symlink",
        NodeKind::Other => "other",
    }
}

fn write_json_string<W: Write>(w: &mut W, s: &str) -> io::Result<()> {
    write!(w, "\"")?;
    for c in s.chars() {
        match c {
            '"' => write!(w, "\\\"")?,
            '\\' => write!(w, "\\\\")?,
            '\n' => write!(w, "\\n")?,
            '\r' => write!(w, "\\r")?,
            '\t' => write!(w, "\\t")?,
            '\u{08}' => write!(w, "\\b")?,
            '\u{0c}' => write!(w, "\\f")?,
            c if (c as u32) < 0x20 => write!(w, "\\u{:04x}", c as u32)?,
            c => write!(w, "{c}")?,
        }
    }
    write!(w, "\"")?;
    Ok(())
}

fn validate_path(path: &Path) -> io::Result<()> {
    let s = path.to_string_lossy();
    if s.contains('\0') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path contains NUL byte",
        ));
    }
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path contains '..' segment",
            ));
        }
    }
    if cfg!(unix) {
        for prefix in FORBIDDEN_PREFIX_UNIX {
            if s.starts_with(prefix) {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("refusing to write into {prefix}"),
                ));
            }
        }
    }
    Ok(())
}

fn current_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    epoch_to_iso8601(secs)
}

fn epoch_to_iso8601(secs: u64) -> String {
    let days = secs / 86_400;
    let day_secs = secs % 86_400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;
    let (y, mo, d) = days_to_ymd(days as i64);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 {
        z / 146_097
    } else {
        (z - 146_096) / 146_097
    };
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y_civil = if m <= 2 { y + 1 } else { y };
    (y_civil, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Tree;

    fn build_simple_tree() -> Tree {
        let mut t = Tree::new(NodeKind::Dir, "root");
        let r = t.root();
        let a = t.add_child(r, NodeKind::Dir, "a", 0, 0).unwrap();
        t.add_child(a, NodeKind::File, "x.txt", 5, 4096).unwrap();
        t.add_child(r, NodeKind::File, "y.txt", 10, 4096).unwrap();
        t.aggregate();
        t
    }

    #[test]
    fn export_includes_schema_and_root() {
        let t = build_simple_tree();
        let mut out = Vec::new();
        write_to_writer(&t, t.root(), &ScanOpts::default(), &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains(r#""schema":"xdu-export-1""#));
        assert!(s.contains(r#""root":"root""#));
        assert!(s.contains(r#""kind":"dir""#));
        assert!(s.contains(r#""kind":"file""#));
    }

    #[test]
    fn export_children_sorted_by_name() {
        let t = build_simple_tree();
        let mut out = Vec::new();
        write_to_writer(&t, t.root(), &ScanOpts::default(), &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        let i_a = s.find(r#""name":"a""#).unwrap();
        let i_y = s.find(r#""name":"y.txt""#).unwrap();
        assert!(i_a < i_y);
    }

    #[test]
    fn export_iterative_handles_deep_chain() {
        let mut t = Tree::new(NodeKind::Dir, "root");
        let mut cur = t.root();
        for i in 0..50_000 {
            cur = t
                .add_child(cur, NodeKind::Dir, format!("d{i}"), 0, 0)
                .unwrap();
        }
        t.aggregate();
        let mut out = Vec::with_capacity(2 * 1024 * 1024);
        write_to_writer(&t, t.root(), &ScanOpts::default(), &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains(r#""name":"d49999""#));
    }

    #[test]
    fn json_string_escapes_specials() {
        let mut out = Vec::new();
        write_json_string(&mut out, "a\"b\\c\nd\re\tf\u{08}g\u{0c}h\u{01}i").unwrap();
        let s = String::from_utf8(out).unwrap();
        // Expected JSON-encoded form: "a\"b\\c\nd\re\tf\bg\fhi"
        // (each \X below in the Rust source means a literal two-char escape in
        // the produced JSON.)
        let expected = "\"a\\\"b\\\\c\\nd\\re\\tf\\bg\\fh\\u0001i\"";
        assert_eq!(s, expected);
    }

    #[test]
    fn iso8601_format_known_epoch() {
        assert_eq!(epoch_to_iso8601(0), "1970-01-01T00:00:00Z");
        // 2025-01-01T00:00:00Z = 1735689600 seconds
        assert_eq!(epoch_to_iso8601(1_735_689_600), "2025-01-01T00:00:00Z");
        // 2026-01-01T00:00:00Z = 1767225600 seconds
        assert_eq!(epoch_to_iso8601(1_767_225_600), "2026-01-01T00:00:00Z");
    }

    #[test]
    fn sec_uc006_rejects_nul_in_path() {
        let r = validate_path(Path::new("foo\0bar"));
        assert!(r.is_err());
    }

    #[test]
    fn sec_uc006_rejects_parent_segment() {
        let r = validate_path(Path::new("a/../b"));
        assert!(r.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn sec_uc006_rejects_proc_sys_dev() {
        assert!(validate_path(Path::new("/proc/foo")).is_err());
        assert!(validate_path(Path::new("/sys/x")).is_err());
        assert!(validate_path(Path::new("/dev/null")).is_err());
    }
}
