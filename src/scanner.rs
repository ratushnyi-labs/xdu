//! Filesystem scanner.
//!
//! Parallel walk over a directory subtree, populating a [`Tree`]: a pool of
//! threads pulls directories from a shared work stack, runs the syscalls
//! outside the tree lock and applies results in per-directory batches. Honors the
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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::UNIX_EPOCH;

#[derive(Clone, Debug)]
pub struct ScanOpts {
    pub cross_fs: bool,
    pub max_errors: usize,
    /// Walker threads; `0` picks a count from the available cores.
    pub threads: usize,
}

impl Default for ScanOpts {
    fn default() -> Self {
        Self {
            cross_fs: false,
            max_errors: 10_000,
            threads: 0,
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

/// One directory awaiting (or undergoing) a read.
///
/// `pending` counts unfinished work at and below this directory: 1 for its own
/// read plus 1 per child directory that is not finalized yet. Whichever thread
/// drops it to zero marks the directory finalized and then releases the
/// parent's count in turn.
struct DirJob {
    id: NodeId,
    path: PathBuf,
    parent: Option<Arc<DirJob>>,
    pending: AtomicUsize,
}

struct Queue {
    jobs: Vec<Arc<DirJob>>,
    /// Jobs queued or currently running. Zero means the walk is complete.
    outstanding: usize,
    abort: bool,
}

/// State shared by all walker threads of one scan.
struct Shared<'a> {
    state: &'a Mutex<ScanResult>,
    opts: &'a ScanOpts,
    root_dev: u64,
    hardlinks: Mutex<HashSet<(u64, u64)>>,
    queue: Mutex<Queue>,
    wake: Condvar,
}

/// Stops the other walkers if this one panics, so the scan cannot hang
/// waiting for a job that will never finish.
struct AbortOnPanic<'a, 'b>(&'a Shared<'b>);

impl Drop for AbortOnPanic<'_, '_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.abort();
        }
    }
}

/// Number of walker threads: `requested` when non-zero, otherwise twice the
/// core count (directory walking is syscall/IO bound), clamped to 4..=32.
fn worker_count(requested: usize) -> usize {
    if requested > 0 {
        return requested;
    }
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    (cores * 2).clamp(4, 32)
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

    let root_job = Arc::new(DirJob {
        id: root_id,
        path: root,
        parent: None,
        pending: AtomicUsize::new(1),
    });
    let shared = Shared {
        state,
        opts: &opts,
        root_dev,
        hardlinks: Mutex::new(HashSet::new()),
        queue: Mutex::new(Queue {
            jobs: vec![root_job],
            outstanding: 1,
            abort: false,
        }),
        wake: Condvar::new(),
    };

    std::thread::scope(|scope| {
        for i in 1..worker_count(opts.threads) {
            // A failed spawn only means fewer walkers; the calling thread
            // always walks too.
            let _ = std::thread::Builder::new()
                .name(format!("xdu-walk-{i}"))
                .spawn_scoped(scope, || shared.run_worker());
        }
        shared.run_worker();
    });

    state.lock().unwrap().running = false;
}

impl Shared<'_> {
    fn run_worker(&self) {
        let _guard = AbortOnPanic(self);
        while let Some(job) = self.next_job() {
            if self.process_dir(&job) {
                self.release(job);
            } else {
                self.abort();
            }
            self.finish_job();
        }
    }

    /// Block until a directory is available; `None` once the walk is complete
    /// or aborted.
    fn next_job(&self) -> Option<Arc<DirJob>> {
        let mut q = self.queue.lock().unwrap();
        loop {
            if q.abort {
                return None;
            }
            if let Some(job) = q.jobs.pop() {
                return Some(job);
            }
            if q.outstanding == 0 {
                return None;
            }
            q = self.wake.wait(q).unwrap();
        }
    }

    fn push_jobs(&self, jobs: Vec<Arc<DirJob>>) {
        if jobs.is_empty() {
            return;
        }
        let mut q = self.queue.lock().unwrap();
        q.outstanding += jobs.len();
        q.jobs.extend(jobs);
        drop(q);
        self.wake.notify_all();
    }

    fn finish_job(&self) {
        let mut q = self.queue.lock().unwrap();
        q.outstanding -= 1;
        let done = q.outstanding == 0;
        drop(q);
        if done {
            self.wake.notify_all();
        }
    }

    fn abort(&self) {
        self.queue.lock().unwrap().abort = true;
        self.wake.notify_all();
    }

    /// Give back one unit of `node`'s pending count. A directory whose count
    /// reaches zero is finalized and releases its parent in turn.
    fn release(&self, mut node: Arc<DirJob>) {
        loop {
            if node.pending.fetch_sub(1, Ordering::AcqRel) != 1 {
                return;
            }
            self.state.lock().unwrap().tree.set_finalized(node.id, true);
            match node.parent.clone() {
                Some(parent) => node = parent,
                None => return,
            }
        }
    }
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

impl Shared<'_> {
    /// Read one directory: syscalls (`read_dir`, `lstat`) run outside the tree
    /// lock; inserts are applied in batches. Returns `false` when the tree is
    /// out of capacity and the whole scan must stop.
    fn process_dir(&self, job: &Arc<DirJob>) -> bool {
        let entries = match std::fs::read_dir(&job.path) {
            Ok(e) => e,
            Err(e) => {
                let mut guard = self.state.lock().unwrap();
                let s: &mut ScanResult = &mut guard;
                push_err(
                    &mut s.errors,
                    &mut s.overflow_errors,
                    self.opts.max_errors,
                    &job.path,
                    &e,
                );
                s.tree.mark_error(job.id);
                return true;
            }
        };

        // Stage all entries' inserts + deferred errors locally; apply under a
        // single lock acquisition per batch (TASK-007-D).
        let mut batch: Vec<StagedEntry> = Vec::with_capacity(64);
        let mut deferred_errors: Vec<(PathBuf, io::Error)> = Vec::new();

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    deferred_errors.push((job.path.clone(), e));
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
            if !self.opts.cross_fs && dev_of(&meta) != self.root_dev {
                continue;
            }
            let kind = classify(&meta);
            let name = entry.file_name().to_string_lossy().into_owned();
            let (size, alloc) = compute_sizes(&meta, kind, &self.hardlinks);
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
                && !self.apply_batch(job, &mut batch, &mut deferred_errors)
            {
                return false;
            }
        }

        self.apply_batch(job, &mut batch, &mut deferred_errors)
    }

    /// Apply one batch of staged inserts + deferred errors under a single
    /// lock, bubbling the batch total up the ancestor chain once. Child
    /// directories become new jobs, queued after the lock is released.
    /// Returns `false` if the tree's arena hit `OutOfCapacity` mid-batch.
    fn apply_batch(
        &self,
        job: &Arc<DirJob>,
        batch: &mut Vec<StagedEntry>,
        deferred_errors: &mut Vec<(PathBuf, io::Error)>,
    ) -> bool {
        if batch.is_empty() && deferred_errors.is_empty() {
            return true;
        }
        let mut new_jobs: Vec<Arc<DirJob>> = Vec::with_capacity(batch.len() / 4);
        let mut overflowed = false;
        {
            let mut guard = self.state.lock().unwrap();
            let s: &mut ScanResult = &mut guard;
            for (path, e) in deferred_errors.drain(..) {
                push_err(
                    &mut s.errors,
                    &mut s.overflow_errors,
                    self.opts.max_errors,
                    &path,
                    &e,
                );
                s.tree.mark_error(job.id);
            }
            let mut first_added: Option<NodeId> = None;
            let (mut sum_size, mut sum_alloc, mut added) = (0_u64, 0_u64, 0_u64);
            for entry in batch.drain(..) {
                match s
                    .tree
                    .add_child(job.id, entry.kind, entry.name, entry.size, entry.alloc)
                {
                    Ok(id) => {
                        s.tree.set_accessed(id, entry.accessed_secs);
                        first_added.get_or_insert(id);
                        sum_size = sum_size.saturating_add(entry.size);
                        sum_alloc = sum_alloc.saturating_add(entry.alloc);
                        added += 1;
                        if matches!(entry.kind, NodeKind::Dir) {
                            job.pending.fetch_add(1, Ordering::AcqRel);
                            new_jobs.push(Arc::new(DirJob {
                                id,
                                path: entry.entry_path,
                                parent: Some(job.clone()),
                                pending: AtomicUsize::new(1),
                            }));
                        }
                    }
                    Err(TreeError::OutOfCapacity) => {
                        overflowed = true;
                        break;
                    }
                    Err(TreeError::InvalidParent) => {
                        s.tree.mark_error(job.id);
                    }
                }
            }
            if let Some(id) = first_added {
                s.tree.bubble_up_add(id, sum_size, sum_alloc, added);
            }
        }
        if overflowed {
            return false;
        }
        self.push_jobs(new_jobs);
        true
    }
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

/// macOS `SF_DATALESS`: the file's contents live in the cloud, not on disk.
const SF_DATALESS: u32 = 0x4000_0000;

fn flags_dataless(flags: u32) -> bool {
    flags & SF_DATALESS != 0
}

#[cfg(target_os = "macos")]
fn is_dataless(meta: &Metadata) -> bool {
    use std::os::macos::fs::MetadataExt;
    flags_dataless(meta.st_flags())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn is_dataless(_meta: &Metadata) -> bool {
    false
}

#[cfg(unix)]
fn compute_sizes(
    meta: &Metadata,
    kind: NodeKind,
    hardlinks: &Mutex<HashSet<(u64, u64)>>,
) -> (u64, u64) {
    // Directory entries carry their own listing size (typically one block on
    // ext-family filesystems). Per ncdu convention we count only the sum of
    // children for dirs — return (0, 0) so eager bubble-up doesn't double-add.
    if matches!(kind, NodeKind::Dir) {
        return (0, 0);
    }
    use std::os::unix::fs::MetadataExt;
    // Online-only placeholders (Dropbox / iCloud / OneDrive File Provider
    // files) report their full logical length but occupy nothing locally.
    if is_dataless(meta) {
        return (0, 0);
    }
    let size = meta.len();
    let alloc = meta.blocks().saturating_mul(512);
    if matches!(kind, NodeKind::File) && meta.nlink() > 1 {
        let key = (meta.dev(), meta.ino());
        if !hardlinks.lock().unwrap().insert(key) {
            return (size, 0);
        }
    }
    (size, alloc)
}

#[cfg(not(unix))]
fn compute_sizes(
    meta: &Metadata,
    kind: NodeKind,
    _hardlinks: &Mutex<HashSet<(u64, u64)>>,
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
    fn dataless_flag_detection() {
        assert!(flags_dataless(0x4000_0020), "Dropbox placeholder flags");
        assert!(!flags_dataless(0x0000_0020), "compressed only");
        assert!(!flags_dataless(0));
    }

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

    /// Build a wide + deep tree and require that every thread count yields
    /// identical totals, every directory finalized, and no lost entries.
    #[test]
    fn parallel_scan_matches_sequential_and_finalizes_everything() {
        let td = TempDir::new();
        let mut expected_size = 0_u64;
        let mut expected_entries = 1_u64; // root
        for a in 0..8 {
            for b in 0..8 {
                let dir = td.path.join(format!("a{a}")).join(format!("b{b}"));
                std::fs::create_dir_all(&dir).unwrap();
                for f in 0..20 {
                    let body = vec![b'x'; 10 + f];
                    write_file(&dir.join(format!("f{f}")), &body);
                    expected_size += body.len() as u64;
                }
            }
        }
        // 8 a-dirs + 64 b-dirs + 64*20 files
        expected_entries += 8 + 64 + 64 * 20;
        let empty = td.path.join("empty");
        std::fs::create_dir(&empty).unwrap();
        expected_entries += 1;

        for threads in [1, 2, 8, 0] {
            let opts = ScanOpts {
                threads,
                ..ScanOpts::default()
            };
            let r = scan(&td.path, &opts).unwrap();
            let root = r.tree.get(r.tree.root()).unwrap();
            assert_eq!(root.size, expected_size, "threads={threads}");
            assert_eq!(root.entries, expected_entries, "threads={threads}");
            assert!(!r.running);
            let mut stack = vec![r.tree.root()];
            while let Some(id) = stack.pop() {
                let n = r.tree.get(id).unwrap();
                assert!(n.finalized, "{} not finalized (threads={threads})", n.name);
                stack.extend(r.tree.children(id));
            }
        }
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
