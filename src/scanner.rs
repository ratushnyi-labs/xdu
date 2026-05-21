//! Filesystem scanner.
//!
//! Iterative DFS over a directory subtree, populating a [`Tree`]. Honors the
//! v0.1.0 BR set: BR-002 hardlink dedup (Unix), BR-003 mount boundary (Unix),
//! BR-004 no symlink follow, BR-005 errors don't abort, BR-006 size mode is
//! a display concern (handled by [`crate::format`]).
//!
//! Both synchronous [`scan`] and asynchronous [`scan_async`] share the private
//! `scan_into` core. The asynchronous variant returns a [`ScanHandle`] backed
//! by `Arc<Mutex<ScanResult>>`; the worker thread mutates the shared state
//! incrementally so the TUI can read partial results between scanner steps
//! (BUG-002).

use crate::tree::{NodeId, NodeKind, Tree, TreeError};
use std::collections::HashSet;
use std::fs::Metadata;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::UNIX_EPOCH;

#[derive(Clone, Debug)]
pub struct ScanOpts {
    pub cross_fs: bool,
    pub max_errors: usize,
}

impl Default for ScanOpts {
    fn default() -> Self {
        Self {
            cross_fs: false,
            max_errors: 10_000,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScanError {
    pub path: PathBuf,
    pub message: String,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.message)
    }
}

impl std::error::Error for ScanError {}

#[derive(Debug)]
pub struct ScanResult {
    pub tree: Tree,
    pub errors: Vec<ScanError>,
    pub overflow_errors: bool,
    /// `true` while a worker thread is still mutating this `ScanResult`.
    /// Synchronous [`scan`] returns with `running == false`; asynchronous
    /// [`scan_async`] starts with `running == true` and the worker flips it
    /// to `false` as its very last act.
    pub running: bool,
}

pub struct ScanHandle {
    pub state: Arc<Mutex<ScanResult>>,
    /// Kept to express ownership of the worker thread. We never `.join()`
    /// from the TUI — the worker either runs to completion (after which the
    /// JoinHandle simply drops) or the process exits and the OS reaps it.
    pub join_handle: Option<JoinHandle<()>>,
}

pub fn scan(root: &Path, opts: &ScanOpts) -> Result<ScanResult, ScanError> {
    let state = init_state(root)?;
    scan_into(&state, root.to_path_buf(), opts.clone());
    // Sync path: only one Arc clone exists (the local `state`). try_unwrap
    // succeeds; if it ever didn't, that would indicate a programming error
    // and we panic loudly rather than swallow it.
    let mutex = match Arc::try_unwrap(state) {
        Ok(m) => m,
        Err(_) => panic!("scan: stray Arc reference after sync scan"),
    };
    let inner = match mutex.into_inner() {
        Ok(s) => s,
        Err(poisoned) => poisoned.into_inner(),
    };
    Ok(inner)
}

pub fn scan_async(root: PathBuf, opts: ScanOpts) -> Result<ScanHandle, ScanError> {
    let state = init_state(&root)?;
    let state_for_thread = state.clone();
    let join_handle = std::thread::Builder::new()
        .name("xdu-scanner".to_string())
        .spawn(move || {
            scan_into(&state_for_thread, root, opts);
        })
        .map_err(|e| ScanError {
            path: PathBuf::from("xdu-scanner"),
            message: format!("spawn failed: {e}"),
        })?;
    Ok(ScanHandle {
        state,
        join_handle: Some(join_handle),
    })
}

fn init_state(root: &Path) -> Result<Arc<Mutex<ScanResult>>, ScanError> {
    if root.as_os_str().to_string_lossy().contains('\0') {
        return Err(err(root, "path contains NUL byte"));
    }
    let root_meta = std::fs::metadata(root).map_err(|e| Scan::from_io(root, &e))?;
    if !root_meta.is_dir() {
        return Err(err(root, "not a directory"));
    }
    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string_lossy().into_owned());
    let tree = Tree::new(NodeKind::Dir, root_name);
    Ok(Arc::new(Mutex::new(ScanResult {
        tree,
        errors: Vec::new(),
        overflow_errors: false,
        running: true,
    })))
}

enum Frame {
    Pre(NodeId, PathBuf),
    Post(NodeId),
}

fn scan_into(state: &Mutex<ScanResult>, root: PathBuf, opts: ScanOpts) {
    let root_meta = match std::fs::metadata(&root) {
        Ok(m) => m,
        Err(_) => {
            state.lock().unwrap().running = false;
            return;
        }
    };
    let root_dev = dev_of(&root_meta);
    let root_id = {
        let mut g = state.lock().unwrap();
        let id = g.tree.root();
        g.tree.set_accessed(id, accessed_unix_secs(&root_meta));
        id
    };

    let mut hardlinks: HashSet<(u64, u64)> = HashSet::new();
    let mut stack: Vec<Frame> = vec![Frame::Pre(root_id, root)];

    while let Some(frame) = stack.pop() {
        match frame {
            Frame::Post(id) => {
                state.lock().unwrap().tree.set_finalized(id, true);
            }
            Frame::Pre(parent_id, parent_path) => {
                stack.push(Frame::Post(parent_id));
                if !walk_one(
                    state,
                    &mut hardlinks,
                    &mut stack,
                    parent_id,
                    &parent_path,
                    root_dev,
                    &opts,
                ) {
                    break;
                }
            }
        }
    }

    state.lock().unwrap().running = false;
}

/// Soft cap on batched inserts per lock acquisition. Keeps the lock duration
/// bounded for pathological directories (>4096 entries). TASK-007-D performance
/// polish.
const WALK_BATCH_CAP: usize = 4096;

struct StagedEntry {
    kind: NodeKind,
    name: String,
    size: u64,
    alloc: u64,
    accessed_secs: u64,
    entry_path: PathBuf,
}

#[allow(clippy::too_many_arguments)]
fn walk_one(
    state: &Mutex<ScanResult>,
    hardlinks: &mut HashSet<(u64, u64)>,
    stack: &mut Vec<Frame>,
    parent_id: NodeId,
    parent_path: &Path,
    root_dev: u64,
    opts: &ScanOpts,
) -> bool {
    let entries = match std::fs::read_dir(parent_path) {
        Ok(e) => e,
        Err(e) => {
            let mut guard = state.lock().unwrap();
            let s: &mut ScanResult = &mut guard;
            push_err(
                &mut s.errors,
                &mut s.overflow_errors,
                opts.max_errors,
                parent_path,
                &e,
            );
            s.tree.mark_error(parent_id);
            return true;
        }
    };

    // Stage all entries' inserts + deferred errors locally; apply under a single
    // lock acquisition per batch (TASK-007-D). The previous design took the lock
    // once per entry — N acquisitions for N entries. This brings it to ~1.
    let mut batch: Vec<StagedEntry> = Vec::with_capacity(64);
    let mut deferred_errors: Vec<(PathBuf, io::Error)> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                deferred_errors.push((parent_path.to_path_buf(), e));
                continue;
            }
        };
        let entry_path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                deferred_errors.push((entry_path, e));
                continue;
            }
        };
        if !opts.cross_fs && dev_of(&meta) != root_dev {
            continue;
        }
        let kind = classify(&meta);
        let name = entry.file_name().to_string_lossy().into_owned();
        let (size, alloc) = compute_sizes(&meta, kind, hardlinks);
        let accessed_secs = accessed_unix_secs(&meta);
        batch.push(StagedEntry {
            kind,
            name,
            size,
            alloc,
            accessed_secs,
            entry_path,
        });
        if batch.len() >= WALK_BATCH_CAP
            && !apply_batch(
                state,
                stack,
                parent_id,
                &mut batch,
                &mut deferred_errors,
                opts,
            )
        {
            return false;
        }
    }

    apply_batch(
        state,
        stack,
        parent_id,
        &mut batch,
        &mut deferred_errors,
        opts,
    )
}

/// Apply one batch of staged inserts + deferred errors under a single lock.
/// Returns `false` if the tree's arena hit `OutOfCapacity` mid-batch (signals
/// the caller to stop). Deferred stack pushes are drained AFTER the lock is
/// released to keep the critical section short.
fn apply_batch(
    state: &Mutex<ScanResult>,
    stack: &mut Vec<Frame>,
    parent_id: NodeId,
    batch: &mut Vec<StagedEntry>,
    deferred_errors: &mut Vec<(PathBuf, io::Error)>,
    opts: &ScanOpts,
) -> bool {
    if batch.is_empty() && deferred_errors.is_empty() {
        return true;
    }
    let mut stack_pushes: Vec<(NodeId, PathBuf)> = Vec::with_capacity(batch.len() / 4);
    let mut overflowed = false;
    {
        let mut guard = state.lock().unwrap();
        let s: &mut ScanResult = &mut guard;
        for (path, e) in deferred_errors.drain(..) {
            push_err(
                &mut s.errors,
                &mut s.overflow_errors,
                opts.max_errors,
                &path,
                &e,
            );
            s.tree.mark_error(parent_id);
        }
        for entry in batch.drain(..) {
            match s
                .tree
                .add_child(parent_id, entry.kind, entry.name, entry.size, entry.alloc)
            {
                Ok(id) => {
                    s.tree.set_accessed(id, entry.accessed_secs);
                    s.tree.bubble_up_add(id, entry.size, entry.alloc, 1);
                    if matches!(entry.kind, NodeKind::Dir) {
                        stack_pushes.push((id, entry.entry_path));
                    }
                }
                Err(TreeError::OutOfCapacity) => {
                    overflowed = true;
                    break;
                }
                Err(TreeError::InvalidParent) => {
                    s.tree.mark_error(parent_id);
                }
            }
        }
    }
    for (id, path) in stack_pushes {
        stack.push(Frame::Pre(id, path));
    }
    !overflowed
}

struct Scan;
impl Scan {
    fn from_io(p: &Path, e: &io::Error) -> ScanError {
        ScanError {
            path: p.to_path_buf(),
            message: e.to_string(),
        }
    }
}

fn err(p: &Path, msg: &str) -> ScanError {
    ScanError {
        path: p.to_path_buf(),
        message: msg.to_string(),
    }
}

fn push_err(errors: &mut Vec<ScanError>, overflow: &mut bool, max: usize, p: &Path, e: &io::Error) {
    if errors.len() < max {
        errors.push(Scan::from_io(p, e));
    } else {
        *overflow = true;
    }
}

fn classify(meta: &Metadata) -> NodeKind {
    let ft = meta.file_type();
    if ft.is_dir() {
        NodeKind::Dir
    } else if ft.is_symlink() {
        NodeKind::Symlink
    } else if ft.is_file() {
        NodeKind::File
    } else {
        NodeKind::Other
    }
}

fn accessed_unix_secs(meta: &Metadata) -> u64 {
    meta.accessed()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(unix)]
fn dev_of(meta: &Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    meta.dev()
}

#[cfg(not(unix))]
fn dev_of(_meta: &Metadata) -> u64 {
    0
}

#[cfg(unix)]
fn compute_sizes(
    meta: &Metadata,
    kind: NodeKind,
    hardlinks: &mut HashSet<(u64, u64)>,
) -> (u64, u64) {
    // Directory entries carry their own listing size (typically one block on
    // ext-family filesystems). Per ncdu convention we count only the sum of
    // children for dirs — return (0, 0) so eager bubble-up doesn't double-add.
    if matches!(kind, NodeKind::Dir) {
        return (0, 0);
    }
    use std::os::unix::fs::MetadataExt;
    let size = meta.len();
    let alloc = meta.blocks().saturating_mul(512);
    if matches!(kind, NodeKind::File) && meta.nlink() > 1 {
        let key = (meta.dev(), meta.ino());
        if !hardlinks.insert(key) {
            return (size, 0);
        }
    }
    (size, alloc)
}

#[cfg(not(unix))]
fn compute_sizes(
    meta: &Metadata,
    kind: NodeKind,
    _hardlinks: &mut HashSet<(u64, u64)>,
) -> (u64, u64) {
    if matches!(kind, NodeKind::Dir) {
        return (0, 0);
    }
    let size = meta.len();
    (size, size)
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempDir {
        path: PathBuf,
    }
    impl TempDir {
        fn new() -> Self {
            let n = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
            let pid = std::process::id();
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let p = std::env::temp_dir().join(format!("xdu-test-{pid}-{nanos}-{n}"));
            fs::create_dir_all(&p).unwrap();
            TempDir { path: p }
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            #[cfg(unix)]
            let _ = chmod_walk(&self.path, 0o755);
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[cfg(unix)]
    fn chmod_walk(p: &Path, mode: u32) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::symlink_metadata(p) {
            let mut perm = meta.permissions();
            perm.set_mode(mode);
            let _ = fs::set_permissions(p, perm);
            if meta.is_dir() {
                if let Ok(entries) = fs::read_dir(p) {
                    for e in entries.flatten() {
                        let _ = chmod_walk(&e.path(), mode);
                    }
                }
            }
        }
        Ok(())
    }

    fn write_file(p: &Path, bytes: &[u8]) {
        fs::write(p, bytes).unwrap();
    }

    #[test]
    fn rejects_nul_byte_in_path() {
        let bad = PathBuf::from("foo\0bar");
        let err = scan(&bad, &ScanOpts::default()).unwrap_err();
        assert!(err.message.contains("NUL byte"));
    }

    #[test]
    fn rejects_missing_root() {
        let p = std::env::temp_dir().join("xdu-test-definitely-missing-99999999999");
        let err = scan(&p, &ScanOpts::default()).unwrap_err();
        assert!(!err.message.is_empty());
    }

    #[test]
    fn rejects_non_directory_root() {
        let td = TempDir::new();
        let f = td.path.join("a.txt");
        write_file(&f, b"hi");
        let err = scan(&f, &ScanOpts::default()).unwrap_err();
        assert!(err.message.contains("not a directory"));
    }

    #[test]
    fn scans_simple_tree() {
        let td = TempDir::new();
        write_file(&td.path.join("a.txt"), b"hello");
        write_file(&td.path.join("b.txt"), b"worldworld");
        fs::create_dir(td.path.join("sub")).unwrap();
        write_file(&td.path.join("sub").join("c.txt"), b"!");
        let r = scan(&td.path, &ScanOpts::default()).unwrap();
        let root = r.tree.get(r.tree.root()).unwrap();
        assert_eq!(root.size, 5 + 10 + 1);
        assert_eq!(root.entries, 5); // root + a + b + sub + c
    }

    #[test]
    fn aggregate_called_after_scan() {
        let td = TempDir::new();
        write_file(&td.path.join("x"), b"abc");
        let r = scan(&td.path, &ScanOpts::default()).unwrap();
        let root = r.tree.get(r.tree.root()).unwrap();
        assert_eq!(root.size, 3);
    }

    #[test]
    fn after_scan_running_is_false_and_root_finalized() {
        let td = TempDir::new();
        write_file(&td.path.join("x"), b"abc");
        let r = scan(&td.path, &ScanOpts::default()).unwrap();
        assert!(!r.running, "sync scan must return with running=false");
        assert!(r.tree.get(r.tree.root()).unwrap().finalized);
    }

    #[test]
    fn scan_async_eventually_finalizes() {
        let td = TempDir::new();
        write_file(&td.path.join("a"), b"ab");
        write_file(&td.path.join("b"), b"cdef");
        let handle = scan_async(td.path.clone(), ScanOpts::default()).unwrap();
        // Wait briefly for the worker.
        let join = handle.join_handle.expect("worker spawned");
        join.join().expect("worker thread completed");
        let g = handle.state.lock().unwrap();
        assert!(!g.running);
        assert_eq!(g.tree.get(g.tree.root()).unwrap().size, 6);
        assert!(g.tree.get(g.tree.root()).unwrap().finalized);
    }

    #[cfg(unix)]
    #[test]
    fn permission_denied_subdir_is_recorded_not_fatal() {
        use std::os::unix::fs::PermissionsExt;
        let td = TempDir::new();
        let blocked = td.path.join("blocked");
        fs::create_dir(&blocked).unwrap();
        write_file(&blocked.join("inner"), b"hello");
        write_file(&td.path.join("ok.txt"), b"yo");
        let mut p = fs::metadata(&blocked).unwrap().permissions();
        p.set_mode(0o000);
        fs::set_permissions(&blocked, p).unwrap();

        if std::fs::read_dir(&blocked).is_ok() {
            eprintln!("[skip] running as root or fs ignores perms");
            return;
        }

        let r = scan(&td.path, &ScanOpts::default()).unwrap();
        let root = r.tree.get(r.tree.root()).unwrap();
        assert!(root.size >= 2);
        assert!(!r.errors.is_empty(), "should have at least one error");
        assert!(root.error, "root marked error due to subdir failure");
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_not_followed() {
        use std::os::unix::fs;
        let td = TempDir::new();
        let target = td.path.join("real");
        std::fs::create_dir(&target).unwrap();
        write_file(&target.join("inside"), b"abcdefghij");
        let link = td.path.join("link-to-real");
        fs::symlink(&target, &link).unwrap();
        let r = scan(&td.path, &ScanOpts::default()).unwrap();
        let root_id = r.tree.root();
        let mut saw_link = false;
        for cid in r.tree.children(root_id) {
            let c = r.tree.get(cid).unwrap();
            if c.name.as_ref() == "link-to-real" {
                assert!(matches!(c.kind, NodeKind::Symlink));
                assert_eq!(c.entries, 1);
                saw_link = true;
            }
        }
        assert!(saw_link);
    }

    #[cfg(unix)]
    #[test]
    fn hardlinks_are_deduplicated() {
        use std::os::unix::fs::MetadataExt;
        let td = TempDir::new();
        let a = td.path.join("a");
        write_file(&a, b"hardlink me");
        let b = td.path.join("b");
        std::fs::hard_link(&a, &b).unwrap();
        let am = std::fs::metadata(&a).unwrap();
        assert!(am.nlink() >= 2);
        let r = scan(&td.path, &ScanOpts::default()).unwrap();
        let mut allocs: Vec<u64> = r
            .tree
            .children(r.tree.root())
            .map(|id| r.tree.get(id).unwrap().alloc)
            .collect();
        allocs.sort_unstable();
        assert_eq!(allocs.len(), 2);
        assert_eq!(allocs[0], 0);
        assert!(allocs[1] > 0);
    }

    #[test]
    fn test_sec_uc001_max_errors_cap_does_not_grow_unbounded() {
        let td = TempDir::new();
        let opts = ScanOpts {
            max_errors: 0,
            ..ScanOpts::default()
        };
        write_file(&td.path.join("a"), b"x");
        let r = scan(&td.path, &opts).unwrap();
        assert!(!r.overflow_errors);
        assert!(r.errors.is_empty());
    }
}
