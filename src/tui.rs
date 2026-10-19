//! Terminal UI: runtime, tree-view widget, header / footer / help / errors
//! overlays, and rescan flow. Combines TASK-001-H/I/J/K leaves into one
//! module to keep cross-references compact.
//!
//! Async (BUG-002): the TUI holds an `Arc<Mutex<ScanResult>>` shared with a
//! background scanner thread. Every render and every key handler that
//! touches the tree locks the mutex briefly. Cells whose value is not yet
//! finalized render an animated spinner.

use crate::disk_space;
use crate::docker_shrink::{self, SharedProgress};
use crate::export;
use crate::format::{bar, format_size, format_ymd, SizeMode};
use crate::lock_detect::{self, LockHolder};
use crate::scanner::{scan_async, ScanError, ScanHandle, ScanOpts, ScanResult};
use crate::time_machine;
use crate::trash;
use crate::tree::{Node, NodeId, NodeKind, SortKey};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use ratatui::Terminal;
use std::collections::HashSet;
use std::io::{self, stdout, Stdout};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

pub struct App {
    state: Arc<Mutex<ScanResult>>,
    /// Kept to express ownership of the worker. Not joined; the worker either
    /// finishes on its own or the process exits.
    _scan_thread: Option<JoinHandle<()>>,
    root_path: PathBuf,
    opts: ScanOpts,
    current: NodeId,
    selection: usize,
    back_stack: Vec<(NodeId, usize)>,
    sort: SortKey,
    show_help: bool,
    show_errors: bool,
    size_mode: SizeMode,
    status: Option<String>,
    /// Active confirmation modal (currently only used by `d` / delete). When
    /// `Some`, all keystrokes route through `handle_prompt_key`. Default focus
    /// on a fresh prompt is `[ No ]` (`yes_focused = false`) per BR-001 amend.
    confirm_prompt: Option<ConfirmPrompt>,
    /// Active "file is locked" modal (BUG-004). Shown after a delete fails
    /// because another process holds the file open. Default focus on `[ Retry ]`.
    lock_prompt: Option<LockPrompt>,
    /// Active "force-close handles?" sub-confirmation (TASK-006). Default
    /// focus on `[ No ]`. Yes invokes `lock_detect::force_close_handles_for_path`
    /// then re-runs `perform_delete` on the same target.
    force_prompt: Option<ForceConfirmPrompt>,
    /// When `true`, the `d` delete path routes through `trash::move_to_trash`
    /// instead of `remove_file`/`remove_dir_all` (TASK-007-J). Off by default;
    /// enabled via `--trash` on the command line.
    trash_mode: bool,
    /// Running Docker prune + VHDX shrink job (`z`). While `Some`, the UI
    /// shows a busy overlay and ignores keys other than Ctrl-C.
    shrink_job: Option<SharedProgress>,
    /// Entries marked with Space for a batch delete (`d`). Cleared on rescan.
    marks: HashSet<NodeId>,
    quit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfirmAction {
    Delete,
    DeleteMarked,
    Shrink,
    Prune,
    TimeMachine,
}

#[derive(Debug, Clone)]
pub struct ConfirmPrompt {
    target: NodeId,
    message: String,
    yes_focused: bool,
    /// What `Yes` does.
    action: ConfirmAction,
}

#[derive(Debug, Clone)]
pub struct LockPrompt {
    target: NodeId,
    path: PathBuf,
    holders: Vec<LockHolder>,
    retry_focused: bool,
}

#[derive(Debug, Clone)]
pub struct ForceConfirmPrompt {
    target: NodeId,
    path: PathBuf,
    holders: Vec<LockHolder>,
    yes_focused: bool,
}

impl App {
    pub fn new(
        root_path: PathBuf,
        opts: ScanOpts,
        handle: ScanHandle,
        size_mode: SizeMode,
    ) -> Self {
        Self::with_trash(root_path, opts, handle, size_mode, false)
    }

    pub fn with_trash(
        root_path: PathBuf,
        opts: ScanOpts,
        handle: ScanHandle,
        size_mode: SizeMode,
        trash_mode: bool,
    ) -> Self {
        let current = handle.state.lock().unwrap().tree.root();
        let mut s = Self {
            state: handle.state,
            _scan_thread: handle.join_handle,
            root_path,
            opts,
            current,
            selection: 0,
            back_stack: Vec::new(),
            sort: SortKey::SizeDesc,
            show_help: false,
            show_errors: false,
            size_mode,
            status: None,
            confirm_prompt: None,
            lock_prompt: None,
            force_prompt: None,
            trash_mode,
            shrink_job: None,
            marks: HashSet::new(),
            quit: false,
        };
        s.sort_current();
        s
    }

    fn lock(&self) -> MutexGuard<'_, ScanResult> {
        self.state.lock().unwrap()
    }

    fn sort_current(&mut self) {
        let cur = self.current;
        let key = self.sort;
        self.lock().tree.sort(cur, key);
    }

    fn children_count(&self) -> usize {
        self.lock().tree.children(self.current).count()
    }

    fn can_ascend(&self) -> bool {
        if !self.back_stack.is_empty() {
            return true;
        }
        let canonical =
            std::fs::canonicalize(&self.root_path).unwrap_or_else(|_| self.root_path.clone());
        canonical
            .parent()
            .is_some_and(|p| !p.as_os_str().is_empty() && p != canonical.as_path())
    }

    fn dotdot_offset(&self) -> usize {
        if self.can_ascend() {
            1
        } else {
            0
        }
    }

    fn total_rows(&self) -> usize {
        self.children_count() + self.dotdot_offset()
    }

    fn move_selection(&mut self, delta: i32) {
        let n = self.total_rows();
        if n == 0 {
            self.selection = 0;
            return;
        }
        let cur = self.selection as i32;
        let max = n as i32 - 1;
        self.selection = cur.saturating_add(delta).clamp(0, max) as usize;
    }

    fn descend(&mut self) {
        let offset = self.dotdot_offset();
        if offset == 1 && self.selection == 0 {
            self.ascend();
            return;
        }
        let target = {
            let g = self.lock();
            let kids: Vec<NodeId> = g.tree.children(self.current).collect();
            kids.get(self.selection - offset).copied()
        };
        let Some(target) = target else { return };
        let kind = self.lock().tree.get(target).map(|n| n.kind);
        if matches!(kind, Some(NodeKind::Dir)) {
            self.back_stack.push((self.current, self.selection));
            self.current = target;
            self.selection = 0;
            self.sort_current();
        }
    }

    fn ascend(&mut self) {
        if let Some((parent, sel)) = self.back_stack.pop() {
            self.current = parent;
            self.selection = sel;
            return;
        }
        self.scan_filesystem_parent();
    }

    fn scan_filesystem_parent(&mut self) {
        let canonical =
            std::fs::canonicalize(&self.root_path).unwrap_or_else(|_| self.root_path.clone());
        let Some(parent) = canonical.parent().map(Path::to_path_buf) else {
            self.status = Some("at filesystem root — cannot go up".into());
            return;
        };
        if parent == canonical {
            self.status = Some("at filesystem root — cannot go up".into());
            return;
        }
        self.start_new_scan(parent, "scanned parent directory");
    }

    fn start_new_scan(&mut self, path: PathBuf, ok_msg: &str) {
        if let Err(e) = self.spawn_scan(path) {
            self.status = Some(format!("scan failed: {e}"));
            self.lock().errors.push(ScanError {
                path: PathBuf::new(),
                message: format!("{e}"),
            });
            return;
        }
        self.status = Some(ok_msg.to_string());
    }

    fn spawn_scan(&mut self, path: PathBuf) -> Result<(), ScanError> {
        let handle = scan_async(path.clone(), self.opts.clone())?;
        self.state = handle.state;
        self._scan_thread = handle.join_handle;
        self.root_path = path;
        let new_root = self.lock().tree.root();
        self.current = new_root;
        self.marks.clear();
        self.selection = 0;
        self.back_stack.clear();
        self.sort_current();
        Ok(())
    }

    fn cycle_sort(&mut self) {
        self.sort = match self.sort {
            SortKey::SizeDesc => SortKey::NameAsc,
            SortKey::NameAsc => SortKey::CountDesc,
            SortKey::CountDesc => SortKey::SizeAsc,
            SortKey::SizeAsc => SortKey::NameDesc,
            SortKey::NameDesc => SortKey::CountAsc,
            SortKey::CountAsc => SortKey::SizeDesc,
        };
        self.sort_current();
        self.selection = 0;
    }

    fn toggle_size_mode(&mut self) {
        self.size_mode = match self.size_mode {
            SizeMode::Iec => SizeMode::Si,
            SizeMode::Si => SizeMode::Iec,
        };
    }

    fn rescan(&mut self) {
        let path = self.root_path.clone();
        self.start_new_scan(path, "rescan started");
    }

    fn absolute_path_for(&self, node_id: NodeId) -> Option<PathBuf> {
        let parts: Vec<String> = {
            let g = self.lock();
            g.tree.path(node_id).into_iter().map(String::from).collect()
        };
        if parts.is_empty() {
            return None;
        }
        let mut path = self.root_path.clone();
        for part in parts.into_iter().skip(1) {
            path.push(part);
        }
        Some(path)
    }

    fn is_scan_root(&self, path: &Path) -> bool {
        let canonical_target = std::fs::canonicalize(path).ok();
        let canonical_root = std::fs::canonicalize(&self.root_path).ok();
        match (canonical_target, canonical_root) {
            (Some(a), Some(b)) => a == b,
            _ => path == self.root_path,
        }
    }

    fn start_delete_prompt(&mut self) {
        let offset = self.dotdot_offset();
        if offset == 1 && self.selection == 0 {
            self.status = Some("cannot delete '..'".into());
            return;
        }
        let target = {
            let g = self.lock();
            let kids: Vec<NodeId> = g.tree.children(self.current).collect();
            kids.get(self.selection.saturating_sub(offset)).copied()
        };
        let Some(target) = target else { return };
        let Some(path) = self.absolute_path_for(target) else {
            return;
        };
        if self.is_scan_root(&path) {
            self.status = Some("cannot delete the scan root".into());
            return;
        }
        let kind = self.lock().tree.get(target).map(|n| n.kind);
        let kind_label = match kind {
            Some(NodeKind::Dir) => "directory (recursive)",
            Some(NodeKind::Symlink) => "symlink",
            _ => "file",
        };
        let (verb, tail) = if self.trash_mode {
            (
                "Move to trash",
                "Trashed items can be restored from the OS trash.",
            )
        } else {
            ("Delete", "This cannot be undone.")
        };
        let message = format!("{verb} {} ?\n  {}\n{tail}", kind_label, path.display());
        self.confirm_prompt = Some(ConfirmPrompt {
            target,
            message,
            yes_focused: false,
            action: ConfirmAction::Delete,
        });
    }

    /// Selected non-`..` child of the current directory, with its absolute path.
    fn selected_entry(&self) -> Option<(NodeId, PathBuf)> {
        let offset = self.dotdot_offset();
        if self.selection < offset {
            return None;
        }
        let target = {
            let g = self.lock();
            let kids: Vec<NodeId> = g.tree.children(self.current).collect();
            kids.get(self.selection - offset).copied()
        }?;
        let path = self.absolute_path_for(target)?;
        Some((target, path))
    }

    fn start_shrink_prompt(&mut self) {
        if !docker_shrink::is_supported() {
            self.status = Some("vhdx shrink is only available on Windows".into());
            return;
        }
        let Some((target, path)) = self.selected_entry() else {
            return;
        };
        if !docker_shrink::is_vhdx(&path) {
            self.status =
                Some("z: select a .vhdx file (e.g. Docker's docker_data.vhdx) to shrink".into());
            return;
        }
        let message = if docker_shrink::is_docker_disk(&path) {
            format!(
                "Docker prune + shrink\n  {}\n\n\
                 1. Prunes Docker (prune -a --volumes): DELETES stopped containers,\n\
                 \x20  ALL unused images, build cache and ALL UNUSED VOLUMES\n\
                 \x20  (volume data is lost).\n\
                 2. Force-stops Docker Desktop and WSL: ALL containers stop.\n\
                 3. Compacts the VHDX (Windows asks for admin / UAC).\n\
                 4. Restarts Docker Desktop.",
                path.display()
            )
        } else {
            format!(
                "Shrink virtual disk\n  {}\n\n\
                 1. Shuts down WSL (wsl --shutdown) and Docker Desktop:\n\
                 \x20  everything running in WSL / Docker stops.\n\
                 2. Compacts the VHDX (Windows asks for admin / UAC).\n\
                 3. Restarts Docker Desktop if it was running.",
                path.display()
            )
        };
        self.confirm_prompt = Some(ConfirmPrompt {
            target,
            message,
            yes_focused: false,
            action: ConfirmAction::Shrink,
        });
    }

    fn start_prune_prompt(&mut self) {
        if !docker_shrink::is_supported() {
            self.status = Some("docker prune is only available on Windows".into());
            return;
        }
        let message = [
            "Docker prune",
            "",
            "Runs: docker container / image -a / builder -a / volume -a prune -f",
            "DELETES stopped containers, ALL unused images, build cache and",
            "ALL UNUSED VOLUMES (volume data is lost), then trims free blocks.",
            "Docker keeps running. The .vhdx file does not shrink until you",
            "compact it (select the .vhdx and press z).",
        ]
        .join("\n");
        self.confirm_prompt = Some(ConfirmPrompt {
            target: self.current,
            message,
            yes_focused: false,
            action: ConfirmAction::Prune,
        });
    }

    fn start_time_machine_prompt(&mut self) {
        if !time_machine::is_supported() {
            self.status = Some("Time Machine snapshots are only available on macOS".into());
            return;
        }
        let snapshots = match time_machine::list_snapshots(&mut docker_shrink::SystemRunner) {
            Ok(s) => s,
            Err(e) => {
                self.status = Some(format!("tmutil listlocalsnapshots failed: {e}"));
                return;
            }
        };
        if snapshots.is_empty() {
            self.status = Some("No Time Machine local snapshots to delete".into());
            return;
        }
        let mut lines = vec![
            format!("Delete Time Machine local snapshots ({})", snapshots.len()),
            String::new(),
        ];
        lines.extend(snapshots.iter().take(6).map(|d| format!("  {d}")));
        if snapshots.len() > 6 {
            lines.push(format!("  … and {} more", snapshots.len() - 6));
        }
        lines.extend([
            String::new(),
            "Runs: tmutil deletelocalsnapshots <date> for each one.".into(),
            "These are the on-disk APFS snapshots of this Mac; they cannot be".into(),
            "restored afterwards. Backups on the Time Machine disk are untouched.".into(),
            "macOS may ask for your administrator password.".into(),
        ]);
        self.confirm_prompt = Some(ConfirmPrompt {
            target: self.current,
            message: lines.join("\n"),
            yes_focused: false,
            action: ConfirmAction::TimeMachine,
        });
    }

    fn start_shrink(&mut self, target: NodeId) {
        let Some(path) = self.absolute_path_for(target) else {
            self.status = Some("shrink: cannot resolve path".into());
            return;
        };
        if !docker_shrink::is_vhdx(&path) {
            self.status = Some("shrink: not a .vhdx file".into());
            return;
        }
        self.shrink_job = Some(docker_shrink::spawn(path));
    }

    /// Called every UI tick: collect the result of a finished shrink job.
    fn poll_shrink(&mut self) {
        let Some(job) = self.shrink_job.as_ref() else {
            return;
        };
        let (title, result) = {
            let g = job.lock().unwrap();
            if !g.done {
                return;
            }
            (g.title.clone(), g.result.clone())
        };
        self.shrink_job = None;
        self.status = Some(match result {
            Some(Ok(msg)) => format!("{title}: {msg}"),
            Some(Err(e)) => format!("{title} failed: {e}"),
            None => format!("{title} finished"),
        });
    }

    fn confirm_accept(&mut self, target: NodeId, action: ConfirmAction) {
        match action {
            ConfirmAction::Delete => self.perform_delete(target),
            ConfirmAction::DeleteMarked => self.perform_delete_marked(),
            ConfirmAction::Shrink => self.start_shrink(target),
            ConfirmAction::Prune => self.shrink_job = Some(docker_shrink::spawn_prune()),
            ConfirmAction::TimeMachine => self.shrink_job = Some(time_machine::spawn_delete()),
        }
    }

    fn delete_path(&self, path: &Path) -> io::Result<()> {
        if self.trash_mode {
            trash::move_to_trash(path).map(|_| ())
        } else if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
    }

    /// Toggle the mark on the selected entry, then move the cursor down.
    fn toggle_mark(&mut self) {
        let offset = self.dotdot_offset();
        if self.selection < offset {
            return;
        }
        let id = {
            let g = self.lock();
            g.tree.children(self.current).nth(self.selection - offset)
        };
        let Some(id) = id else { return };
        if !self.marks.remove(&id) {
            self.marks.insert(id);
        }
        self.move_selection(1);
    }

    /// Mark every entry of the current directory, or clear them all if every
    /// entry is already marked.
    fn toggle_mark_all(&mut self) {
        let kids: Vec<NodeId> = self.lock().tree.children(self.current).collect();
        if kids.iter().all(|k| self.marks.contains(k)) {
            for k in &kids {
                self.marks.remove(k);
            }
        } else {
            self.marks.extend(kids);
        }
    }

    /// Marked nodes minus any whose ancestor is also marked (deleting the
    /// ancestor already removes them), in a stable order.
    fn marked_roots(&self) -> Vec<NodeId> {
        let g = self.lock();
        let mut out: Vec<NodeId> = self
            .marks
            .iter()
            .copied()
            .filter(|&id| {
                let mut p = g.tree.get(id).and_then(|n| n.parent);
                while let Some(pid) = p {
                    if self.marks.contains(&pid) {
                        return false;
                    }
                    p = g.tree.get(pid).and_then(|n| n.parent);
                }
                true
            })
            .collect();
        out.sort_by_key(|id| id.index());
        out
    }

    fn start_delete_marked_prompt(&mut self) {
        let roots = self.marked_roots();
        if roots.is_empty() {
            return;
        }
        let mut paths: Vec<PathBuf> = Vec::new();
        let mut total: u64 = 0;
        for &id in &roots {
            let Some(path) = self.absolute_path_for(id) else {
                continue;
            };
            if self.is_scan_root(&path) {
                self.status = Some("cannot delete the scan root (unmark it first)".into());
                return;
            }
            total = total.saturating_add(self.lock().tree.get(id).map_or(0, |n| n.size));
            paths.push(path);
        }
        const SHOWN: usize = 8;
        let mut list: Vec<String> = paths
            .iter()
            .take(SHOWN)
            .map(|p| format!("  {}", p.display()))
            .collect();
        if paths.len() > SHOWN {
            list.push(format!("  … and {} more", paths.len() - SHOWN));
        }
        let (verb, tail) = if self.trash_mode {
            (
                "Move to trash",
                "Trashed items can be restored from the OS trash.",
            )
        } else {
            ("Delete", "This cannot be undone.")
        };
        let message = format!(
            "{verb} {} marked items ({}) ?\n{}\n{tail}",
            paths.len(),
            format_size(total, self.size_mode),
            list.join("\n")
        );
        self.confirm_prompt = Some(ConfirmPrompt {
            target: roots[0],
            message,
            yes_focused: false,
            action: ConfirmAction::DeleteMarked,
        });
    }

    /// Delete every marked entry. Failures are collected (and listed in the
    /// `!` errors panel) instead of stopping the batch; failed entries stay
    /// marked so they can be retried.
    fn perform_delete_marked(&mut self) {
        let roots = self.marked_roots();
        let mut deleted = 0_usize;
        let mut failures: Vec<(PathBuf, String)> = Vec::new();
        for id in roots {
            let Some(path) = self.absolute_path_for(id) else {
                continue;
            };
            if self.is_scan_root(&path) {
                failures.push((path, "refusing to remove the scan root".into()));
                continue;
            }
            match self.delete_path(&path) {
                Ok(()) if !path.exists() => {
                    self.lock().tree.detach(id);
                    self.marks.remove(&id);
                    deleted += 1;
                }
                Ok(()) => failures.push((path, "still on disk after delete".into())),
                Err(e) => {
                    let raw = e.raw_os_error().unwrap_or(0);
                    let msg = if matches!(raw, 5 | 32 | 33 | 145) {
                        let holders = lock_detect::holders(&path);
                        if holders.is_empty() {
                            format!("in use ({e})")
                        } else {
                            let names: Vec<String> = holders
                                .iter()
                                .map(|h| format!("{} (pid {})", h.name, h.pid))
                                .collect();
                            format!("in use by {}", names.join(", "))
                        }
                    } else {
                        e.to_string()
                    };
                    failures.push((path, msg));
                }
            }
        }
        let last = self.total_rows().saturating_sub(1);
        self.selection = self.selection.min(last);
        let verb = if self.trash_mode {
            "moved to trash"
        } else {
            "deleted"
        };
        self.status = Some(match failures.first() {
            None => format!("{verb} {deleted} item(s)"),
            Some((p, m)) => format!(
                "{verb} {deleted}, {} failed (see !): {}: {m}",
                failures.len(),
                p.display()
            ),
        });
        let mut g = self.lock();
        for (path, message) in failures {
            g.errors.push(ScanError { path, message });
        }
    }

    fn perform_delete(&mut self, node_id: NodeId) {
        let Some(path) = self.absolute_path_for(node_id) else {
            self.status = Some("delete: cannot resolve path".into());
            return;
        };
        if self.is_scan_root(&path) {
            self.status = Some("delete: refusing to remove the scan root".into());
            return;
        }
        let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        let result = self.delete_path(&path);
        match result {
            Ok(_) => {
                if path.exists() {
                    // BUG-003 defensive read-back caught the failed delete.
                    // BUG-004: try to identify the holding process.
                    if !self.try_open_lock_prompt(node_id, canonical.clone()) {
                        self.status = Some(format!(
                            "delete reported success but {} is still on disk — try `r` to refresh",
                            canonical.display()
                        ));
                    }
                } else {
                    let m = if self.trash_mode {
                        format!("moved {} to trash", canonical.display())
                    } else {
                        format!("deleted {}", canonical.display())
                    };
                    // Update the in-memory tree instead of rescanning the disk.
                    self.marks.remove(&node_id);
                    let detached = self.lock().tree.detach(node_id);
                    if detached {
                        let last = self.total_rows().saturating_sub(1);
                        self.selection = self.selection.min(last);
                        self.status = Some(m);
                    } else {
                        let root = self.root_path.clone();
                        let _ = self.spawn_scan(root);
                        self.status = Some(m);
                    }
                }
            }
            Err(e) => {
                // BUG-004: when the OS reports the file is in use, try to
                // identify the holding processes and offer a Retry modal.
                let raw = e.raw_os_error().unwrap_or(0);
                // 145 = ERROR_DIR_NOT_EMPTY, surfaced when a file inside a
                // directory could not be removed.
                let is_in_use = matches!(raw, 5 | 32 | 33 | 145);
                if is_in_use && self.try_open_lock_prompt(node_id, canonical.clone()) {
                    return;
                }
                self.status = Some(format_delete_error(&canonical, &e));
            }
        }
    }

    fn try_open_lock_prompt(&mut self, target: NodeId, path: PathBuf) -> bool {
        let holders = lock_detect::holders(&path);
        if holders.is_empty() {
            return false;
        }
        self.lock_prompt = Some(LockPrompt {
            target,
            path,
            holders,
            retry_focused: true,
        });
        true
    }

    fn export(&mut self) {
        let path: PathBuf = "xdu-export.json".into();
        let result = {
            let g = self.lock();
            export::write_to_path(&g.tree, self.current, &self.opts, &path)
        };
        self.status = Some(match result {
            Ok(_) => format!("exported to {}", path.display()),
            Err(e) => format!("export failed: {e}"),
        });
    }

    fn handle_key(&mut self, key: KeyCode, mods: KeyModifiers) {
        self.status = None;
        // Ctrl-C always quits, even from a modal prompt.
        if key == KeyCode::Char('c') && mods.contains(KeyModifiers::CONTROL) {
            self.quit = true;
            return;
        }
        // A prune + shrink job is stopping/compacting Docker: ignore input so
        // it cannot be interrupted half-way by an accidental `q`.
        if self.shrink_job.is_some() {
            return;
        }
        if self.handle_prompt_key(key) {
            return;
        }
        if matches!(key, KeyCode::Char('q')) {
            self.quit = true;
            return;
        }
        if self.show_help || self.show_errors {
            self.show_help = false;
            self.show_errors = false;
            return;
        }
        match key {
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => self.descend(),
            KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => self.ascend(),
            KeyCode::PageUp => self.move_selection(-10),
            KeyCode::PageDown => self.move_selection(10),
            KeyCode::Home => self.selection = 0,
            KeyCode::End => self.selection = self.total_rows().saturating_sub(1),
            KeyCode::Char('s') => self.cycle_sort(),
            KeyCode::Char('?') | KeyCode::F(1) => self.show_help = true,
            KeyCode::Char('!') => self.show_errors = true,
            KeyCode::Char('u') => self.toggle_size_mode(),
            KeyCode::Char('r') => self.rescan(),
            KeyCode::Char('e') => self.export(),
            KeyCode::Char('d') => {
                if self.marks.is_empty() {
                    self.start_delete_prompt();
                } else {
                    self.start_delete_marked_prompt();
                }
            }
            KeyCode::Char(' ') => self.toggle_mark(),
            KeyCode::Char('a') => self.toggle_mark_all(),
            KeyCode::Esc => self.marks.clear(),
            KeyCode::Char('z') => self.start_shrink_prompt(),
            KeyCode::Char('P') => self.start_prune_prompt(),
            KeyCode::Char('T') => self.start_time_machine_prompt(),
            _ => {}
        }
    }

    fn handle_prompt_key(&mut self, key: KeyCode) -> bool {
        if self.force_prompt.is_some() {
            self.handle_force_prompt_key(key);
            return true;
        }
        if self.lock_prompt.is_some() {
            self.handle_lock_prompt_key(key);
            return true;
        }
        let Some(prompt) = self.confirm_prompt.as_mut() else {
            return false;
        };
        match key {
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('q') => {
                self.confirm_prompt = None;
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let (target, action) = (prompt.target, prompt.action);
                self.confirm_prompt = None;
                self.confirm_accept(target, action);
            }
            KeyCode::Tab
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l') => {
                prompt.yes_focused = !prompt.yes_focused;
            }
            KeyCode::Enter => {
                let yes = prompt.yes_focused;
                let (target, action) = (prompt.target, prompt.action);
                self.confirm_prompt = None;
                if yes {
                    self.confirm_accept(target, action);
                }
            }
            _ => {}
        }
        true
    }

    fn handle_lock_prompt_key(&mut self, key: KeyCode) {
        let Some(prompt) = self.lock_prompt.as_mut() else {
            return;
        };
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('n') | KeyCode::Char('N') => {
                self.lock_prompt = None;
                self.status = Some("delete cancelled while file was in use".into());
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                let target = prompt.target;
                self.lock_prompt = None;
                self.perform_delete(target);
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                let target = prompt.target;
                let path = prompt.path.clone();
                let holders = prompt.holders.clone();
                self.lock_prompt = None;
                self.force_prompt = Some(ForceConfirmPrompt {
                    target,
                    path,
                    holders,
                    yes_focused: false,
                });
            }
            KeyCode::Tab
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l') => {
                prompt.retry_focused = !prompt.retry_focused;
            }
            KeyCode::Enter => {
                let retry = prompt.retry_focused;
                let target = prompt.target;
                self.lock_prompt = None;
                if retry {
                    self.perform_delete(target);
                } else {
                    self.status = Some("delete cancelled while file was in use".into());
                }
            }
            _ => {}
        }
    }

    fn handle_force_prompt_key(&mut self, key: KeyCode) {
        let Some(prompt) = self.force_prompt.as_mut() else {
            return;
        };
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('n') | KeyCode::Char('N') => {
                self.force_prompt = None;
                self.status = Some("force-close cancelled".into());
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let target = prompt.target;
                let path = prompt.path.clone();
                let pids: Vec<u32> = prompt.holders.iter().map(|h| h.pid).collect();
                self.force_prompt = None;
                self.run_force_close_then_retry(target, &path, &pids);
            }
            KeyCode::Tab
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l') => {
                prompt.yes_focused = !prompt.yes_focused;
            }
            KeyCode::Enter => {
                let yes = prompt.yes_focused;
                let target = prompt.target;
                let path = prompt.path.clone();
                let pids: Vec<u32> = prompt.holders.iter().map(|h| h.pid).collect();
                self.force_prompt = None;
                if yes {
                    self.run_force_close_then_retry(target, &path, &pids);
                } else {
                    self.status = Some("force-close cancelled".into());
                }
            }
            _ => {}
        }
    }

    fn run_force_close_then_retry(&mut self, target: NodeId, path: &Path, pids: &[u32]) {
        let n = lock_detect::force_close_handles_for_path(path, pids);
        if n == 0 {
            self.status = Some(format!(
                "no handles closed (need same-user/admin privileges, or path could not be resolved); {} still locked",
                path.display()
            ));
            return;
        }
        // Retry the delete now that the foreign handles are gone.
        self.status = Some(format!("force-closed {n} handle(s); retrying delete"));
        self.perform_delete(target);
    }
}

fn format_delete_error(path: &Path, e: &io::Error) -> String {
    let raw = e.raw_os_error().unwrap_or(0);
    match raw {
        5 | 32 | 33 | 145 => format!(
            "delete failed: {} is in use, but the holding app could not be identified (it may be a system/elevated process, or a shell/terminal whose working directory is inside). Try Sysinternals Process Explorer (Find > Handle) or Resource Monitor, then retry.",
            path.display()
        ),
        _ => format!("delete failed: {}: {}", path.display(), e),
    }
}

pub fn run(
    root_path: PathBuf,
    opts: ScanOpts,
    handle: ScanHandle,
    size_mode: SizeMode,
) -> io::Result<()> {
    run_with(root_path, opts, handle, size_mode, false)
}

pub fn run_with(
    root_path: PathBuf,
    opts: ScanOpts,
    handle: ScanHandle,
    size_mode: SizeMode,
    trash_mode: bool,
) -> io::Result<()> {
    let mut term = init_terminal()?;
    let result = run_loop(
        &mut term,
        App::with_trash(root_path, opts, handle, size_mode, trash_mode),
    );
    teardown_terminal();
    result
}

/// Render the App into a virtual terminal at `(width, height)` and return the
/// rendered cells as a `\n`-joined grid of symbols.
///
/// This is the snapshot-test surface for TASK-007-F (`LATER-004`): pair it
/// with golden files under `docs/ui-baselines/snapshots/` to lock in TUI
/// output. No styling/colour is captured — only cell symbols — which is
/// sufficient to catch layout, wording, and column-width regressions while
/// staying robust against ANSI-style choices.
pub fn render_snapshot(app: &App, width: u16, height: u16) -> String {
    use ratatui::backend::TestBackend;
    let backend = TestBackend::new(width, height);
    let mut term = Terminal::new(backend).expect("TestBackend::new infallible");
    term.draw(|f| render(f, app))
        .expect("TestBackend draw infallible");
    let buf = term.backend().buffer();
    let mut out = String::with_capacity((width as usize + 1) * height as usize);
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

/// Construct an `App` from a freshly-built tree without running the scanner.
/// Useful for snapshot tests and for any future deterministic-fixture tests.
///
/// The returned App is wired to a non-running `ScanResult` (no worker thread)
/// so the harness has no async timing to coordinate.
pub fn app_from_tree_for_test(root_path: PathBuf, tree: crate::tree::Tree) -> App {
    let state = Arc::new(Mutex::new(ScanResult {
        tree,
        errors: Vec::new(),
        overflow_errors: false,
        running: false,
    }));
    let handle = ScanHandle {
        state,
        join_handle: None,
    };
    App::new(root_path, ScanOpts::default(), handle, SizeMode::Iec)
}

fn run_loop(term: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<()> {
    while !app.quit {
        app.poll_shrink();
        term.draw(|f| render(f, &app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(ke) = event::read()? {
                if ke.kind == KeyEventKind::Press {
                    app.handle_key(ke.code, ke.modifiers);
                }
            }
        }
    }
    Ok(())
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        teardown_terminal();
        original_hook(info);
    }));
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn teardown_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen);
}

fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    let g = app.state.lock().unwrap();
    render_header(f, chunks[0], app, &g);
    render_body(f, chunks[1], app, &g);
    render_footer(f, chunks[2], app, &g);
    drop(g);

    if app.show_help {
        render_help(f, area);
    }
    if app.show_errors {
        render_errors(f, area, app);
    }
    if let Some(prompt) = app.confirm_prompt.as_ref() {
        render_confirm(f, area, prompt);
    }
    if let Some(prompt) = app.lock_prompt.as_ref() {
        render_lock_prompt(f, area, prompt);
    }
    if let Some(prompt) = app.force_prompt.as_ref() {
        render_force_prompt(f, area, prompt);
    }
    if let Some(job) = app.shrink_job.as_ref() {
        render_shrink_busy(f, area, job);
    }
}

fn render_shrink_busy(f: &mut Frame, area: Rect, job: &SharedProgress) {
    let (title, step) = job
        .lock()
        .map(|g| (g.title.clone(), g.step.clone()))
        .unwrap_or_default();
    let lines = vec![
        Line::from(format!("{} {step}…", spinner_char())),
        Line::from(""),
        Line::from("This can take several minutes."),
        Line::from("Input is disabled until it finishes."),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "));
    let h = (lines.len() as u16 + 2).min(area.height);
    let r = centered_rect(area, 70, h);
    f.render_widget(Clear, r);
    f.render_widget(Paragraph::new(lines).block(block), r);
}

fn render_lock_prompt(f: &mut Frame, area: Rect, prompt: &LockPrompt) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(format!(
        "Cannot delete {} — it is held open by {} process(es):",
        prompt.path.display(),
        prompt.holders.len()
    )));
    lines.push(Line::from(""));
    for h in prompt.holders.iter().take(8) {
        lines.push(Line::from(format!("    {} — {}", h.pid, h.name)));
    }
    if prompt.holders.len() > 8 {
        lines.push(Line::from(format!(
            "    … and {} more",
            prompt.holders.len() - 8
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Close the listed apps in your task manager (xdu does not terminate processes for you), then press Retry.",
    ));
    lines.push(Line::from(""));
    lines.push(retry_buttons_line(prompt.retry_focused));
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Tab/Left/Right: toggle    Enter / r: retry    f: force-close handles    n / Esc / q: cancel",
    ));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" File is in use ");
    let h = (lines.len() as u16 + 2).min(area.height);
    let r = centered_rect(area, 100, h);
    f.render_widget(Clear, r);
    f.render_widget(Paragraph::new(lines).block(block), r);
}

fn render_force_prompt(f: &mut Frame, area: Rect, prompt: &ForceConfirmPrompt) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "Force-close {} handle(s) on {} ?",
            prompt.holders.len(),
            prompt.path.display()
        ),
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(
        "This calls DuplicateHandle(DUPLICATE_CLOSE_SOURCE) on each holder's",
    ));
    lines.push(Line::from(
        "file handle. The owning processes are NOT terminated, but the next",
    ));
    lines.push(Line::from(
        "write through the now-invalid handle will fail; unsaved data in the",
    ));
    lines.push(Line::from("listed apps may be lost:"));
    lines.push(Line::from(""));
    for h in prompt.holders.iter().take(8) {
        lines.push(Line::from(format!("    {} — {}", h.pid, h.name)));
    }
    if prompt.holders.len() > 8 {
        lines.push(Line::from(format!(
            "    … and {} more",
            prompt.holders.len() - 8
        )));
    }
    lines.push(Line::from(""));
    lines.push(confirm_buttons_line(prompt.yes_focused));
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Tab/Left/Right: toggle    Enter: activate    y: yes    n / Esc / q: cancel",
    ));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Force-close handles? ")
        .border_style(Style::default().fg(Color::Red));
    let h = (lines.len() as u16 + 2).min(area.height);
    let r = centered_rect(area, 100, h);
    f.render_widget(Clear, r);
    f.render_widget(Paragraph::new(lines).block(block), r);
}

fn retry_buttons_line(retry_focused: bool) -> Line<'static> {
    let retry_style = if retry_focused {
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let cancel_style = if !retry_focused {
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Line::from(vec![
        Span::raw("    "),
        Span::styled(" Retry ", retry_style),
        Span::raw("        "),
        Span::styled(" Cancel ", cancel_style),
        Span::raw("    "),
    ])
}

fn render_confirm(f: &mut Frame, area: Rect, prompt: &ConfirmPrompt) {
    let mut lines: Vec<Line> = prompt
        .message
        .lines()
        .map(|l| Line::from(l.to_string()))
        .collect();
    lines.push(Line::from(""));
    lines.push(confirm_buttons_line(prompt.yes_focused));
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Tab/Left/Right: toggle    Enter: activate    y: yes    n / Esc / q: cancel",
    ));
    let title = match prompt.action {
        ConfirmAction::Shrink => " Confirm Docker prune + shrink ",
        ConfirmAction::Prune => " Confirm Docker prune ",
        ConfirmAction::TimeMachine => " Confirm delete Time Machine snapshots ",
        ConfirmAction::Delete | ConfirmAction::DeleteMarked => " Confirm delete ",
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let h = (lines.len() as u16 + 2).min(area.height);
    let r = centered_rect(area, 84, h);
    f.render_widget(Clear, r);
    f.render_widget(Paragraph::new(lines).block(block), r);
}

fn confirm_buttons_line(yes_focused: bool) -> Line<'static> {
    let yes_style = if yes_focused {
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::Red)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let no_style = if !yes_focused {
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Line::from(vec![
        Span::raw("    "),
        Span::styled(" Yes ", yes_style),
        Span::raw("        "),
        Span::styled(" No ", no_style),
        Span::raw("    "),
    ])
}

fn render_header(f: &mut Frame, area: Rect, app: &App, state: &ScanResult) {
    let path = state.tree.path(app.current).join("/");
    let text = format!(" {path} ");
    let s = truncate_left(&text, area.width as usize);
    let style = Style::default().add_modifier(Modifier::REVERSED);
    f.render_widget(Paragraph::new(s).style(style), area);
}

fn render_body(f: &mut Frame, area: Rect, app: &App, state: &ScanResult) {
    let tree = &state.tree;
    let kids: Vec<NodeId> = tree.children(app.current).collect();
    let cur = match tree.get(app.current) {
        Some(n) => n,
        None => return,
    };
    let dir_total = cur.size.max(1);
    let dir_finalized = cur.finalized;
    let utf8 = is_utf8_terminal();
    let dotdot = app.dotdot_offset() == 1;
    let height = area.height as usize;
    let total = kids.len() + if dotdot { 1 } else { 0 };
    let start = if total <= height {
        0
    } else {
        app.selection.saturating_sub(height / 2).min(total - height)
    };
    let end = (start + height).min(total);
    let name_w = name_column_width(area.width as usize);
    let spinner = spinner_char();

    let mut lines: Vec<Line> = Vec::with_capacity(end.saturating_sub(start));
    for absolute_i in start..end {
        let line = if dotdot && absolute_i == 0 {
            render_dotdot_row(name_w, absolute_i == app.selection)
        } else {
            let kids_idx = absolute_i - if dotdot { 1 } else { 0 };
            let Some(&id) = kids.get(kids_idx) else {
                continue;
            };
            let Some(node) = tree.get(id) else { continue };
            render_entry_row(
                node,
                dir_total,
                dir_finalized,
                name_w,
                utf8,
                app.size_mode,
                spinner,
                absolute_i == app.selection,
                app.marks.contains(&id),
            )
        };
        lines.push(line);
    }
    f.render_widget(Paragraph::new(lines), area);
}

const BAR_W: usize = 10;
const SIZE_W: usize = 9;
const DATE_W: usize = 10;
const SEP_W: usize = 2;

fn name_column_width(area_w: usize) -> usize {
    let fixed = BAR_W + SEP_W + SIZE_W + SEP_W + DATE_W + SEP_W;
    area_w.saturating_sub(fixed).max(8)
}

#[allow(clippy::too_many_arguments)]
fn render_entry_row(
    node: &Node,
    dir_total: u64,
    dir_finalized: bool,
    name_w: usize,
    utf8: bool,
    size_mode: SizeMode,
    spinner: char,
    selected: bool,
    marked: bool,
) -> Line<'static> {
    let mut name = display_name(&node.name, node.kind);
    if marked {
        name.insert_str(0, "* ");
    }
    let name_padded = pad_or_truncate(&name, name_w);
    let (bar_str, size_str) = if node.finalized {
        let frac = (node.size as f64) / (dir_total as f64);
        (
            bar(frac, BAR_W, utf8),
            format!("{:>SIZE_W$}", format_size(node.size, size_mode)),
        )
    } else {
        // Not yet calculated: blank bar + spinner-as-size (BUG-002).
        (
            " ".repeat(BAR_W),
            format!("{:>SIZE_W$}", spinner.to_string()),
        )
    };
    let date = format_ymd(node.accessed_unix);
    let s = format!("{name_padded}  {bar_str}  {size_str}  {date}");
    let frac = if dir_finalized && node.finalized {
        (node.size as f64) / (dir_total as f64)
    } else {
        0.0
    };
    let mut style = row_style(frac, node.kind, node.error, selected);
    if marked {
        style = style.add_modifier(Modifier::BOLD).fg(Color::Yellow);
    }
    Line::from(Span::styled(s, style))
}

fn render_dotdot_row(name_w: usize, selected: bool) -> Line<'static> {
    let name = pad_or_truncate("..", name_w);
    let s = format!(
        "{name}  {bar:BAR_W$}  {size:>SIZE_W$}  {date:DATE_W$}",
        bar = "",
        size = "",
        date = "",
    );
    let mut style = Style::default().add_modifier(Modifier::BOLD);
    if selected {
        style = style.add_modifier(Modifier::REVERSED);
    }
    Line::from(Span::styled(s, style))
}

fn pad_or_truncate(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count == width {
        return s.to_string();
    }
    if count < width {
        let mut out = String::with_capacity(width);
        out.push_str(s);
        for _ in count..width {
            out.push(' ');
        }
        return out;
    }
    if width == 0 {
        return String::new();
    }
    let take = width.saturating_sub(1);
    let mut out: String = s.chars().take(take).collect();
    out.push('…');
    out
}

/// `true` when shrinking is available here and the selected row is a `.vhdx`.
/// Reads the already-locked `state` (the render path holds the mutex).
fn selected_is_vhdx(app: &App, state: &ScanResult) -> bool {
    if !docker_shrink::is_supported() {
        return false;
    }
    let offset = app.dotdot_offset();
    if app.selection < offset {
        return false;
    }
    state
        .tree
        .children(app.current)
        .nth(app.selection - offset)
        .and_then(|id| state.tree.get(id))
        .is_some_and(|n| n.kind == NodeKind::File && docker_shrink::is_vhdx(Path::new(&*n.name)))
}

fn render_footer(f: &mut Frame, area: Rect, app: &App, state: &ScanResult) {
    let left = if let Some(msg) = app.status.as_deref() {
        format!(" {msg}  (any key to dismiss) ")
    } else if state.running {
        format!(
            " scanning ({} entries, {} errors) {}  q:quit ",
            state.tree.len(),
            state.errors.len(),
            spinner_char()
        )
    } else {
        let delete_hint = if app.trash_mode {
            "d:trash"
        } else {
            "d:delete"
        };
        if app.marks.is_empty() && selected_is_vhdx(app, state) {
            // A virtual disk is selected: surface the shrink action, which
            // would otherwise be cut off the end of the default footer.
            format!(" z:shrink this .vhdx (prune Docker + compact)  {delete_hint}  ?:help  q:quit ")
        } else if app.marks.is_empty() {
            format!(
                " sort:{}  errors:{}  ?:help  s:sort  u:unit  r:rescan  e:export  {delete_hint}  !:errors  q:quit ",
                sort_label(app.sort),
                state.errors.len()
            )
        } else {
            format!(
                " marked:{}  Space:mark  a:all  Esc:clear  {delete_hint} marked  ?:help  q:quit ",
                app.marks.len()
            )
        }
    };
    let mut right = disk_space::format_free(&app.root_path, app.size_mode);
    if app.trash_mode {
        right = format!(" mode:trash {right}");
    }
    let s = compose_footer(&left, &right, area.width as usize);
    let style = Style::default().add_modifier(Modifier::REVERSED);
    f.render_widget(Paragraph::new(s).style(style), area);
}

fn compose_footer(left: &str, right: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let lc = left.chars().count();
    let rc = right.chars().count();
    if rc == 0 {
        return truncate_left(left, width);
    }
    if lc + rc <= width {
        let pad = width - lc - rc;
        return format!("{left}{}{right}", " ".repeat(pad));
    }
    if rc < width {
        let take = width.saturating_sub(rc + 1);
        let left_truncated: String = left.chars().take(take).collect();
        return format!("{left_truncated} {right}");
    }
    truncate_left(right, width)
}

fn render_help(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from("xdu — keybindings"),
        Line::from(""),
        Line::from("  Up / k          move up"),
        Line::from("  Down / j        move down"),
        Line::from("  Right/l/Enter   enter directory"),
        Line::from("  Left/h/Bksp     go up (rescans FS parent at scan root)"),
        Line::from("  ..              row at top of every navigable directory"),
        Line::from("  PgUp / PgDn     page"),
        Line::from("  Home / End      jump"),
        Line::from("  s               cycle sort"),
        Line::from("  u               toggle KiB / KB units"),
        Line::from("  r               rescan from root (async)"),
        Line::from("  e               export current subtree to xdu-export.json"),
        Line::from("  d               delete the selected entry (with confirmation)"),
        Line::from("  Space / a / Esc mark entry / mark all / clear; d deletes marked"),
        Line::from("  z               shrink the selected .vhdx (+ Docker prune)"),
        Line::from("  P               Docker prune only (no stop, no compact)"),
        Line::from("  T               delete Time Machine local snapshots (macOS)"),
        Line::from("  !               show errors"),
        Line::from("  ? / F1          this help"),
        Line::from("  q / Ctrl-C      quit"),
        Line::from(""),
        Line::from("xdu is read-only by default; deletion is exposed only behind 'd'"),
        Line::from("plus a confirmation modal whose default focus is [ No ]."),
        Line::from("Cells with a spinner are not yet calculated; they update live."),
        Line::from(""),
        Line::from("Press any key to dismiss."),
    ];
    let block = Block::default().borders(Borders::ALL).title(" Help ");
    let r = centered_rect(area, 68, 27);
    f.render_widget(Clear, r);
    f.render_widget(Paragraph::new(lines).block(block), r);
}

fn render_errors(f: &mut Frame, area: Rect, app: &App) {
    let g = app.state.lock().unwrap();
    let mut lines: Vec<Line> = Vec::new();
    for e in g.errors.iter().take(50) {
        lines.push(Line::from(format!("{}: {}", e.path.display(), e.message)));
    }
    if g.overflow_errors {
        lines.push(Line::from("(error log capped — additional errors dropped)"));
    }
    if lines.is_empty() {
        lines.push(Line::from("No errors recorded."));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Press any key to dismiss."));
    let title = format!(" Errors ({}) ", g.errors.len());
    let block = Block::default().borders(Borders::ALL).title(title);
    let r = centered_rect(area, 80, 22);
    f.render_widget(Clear, r);
    f.render_widget(Paragraph::new(lines).block(block), r);
}

fn sort_label(k: SortKey) -> &'static str {
    sort_label_for(k, is_utf8_terminal())
}

fn sort_label_for(k: SortKey, utf8: bool) -> &'static str {
    if utf8 {
        match k {
            SortKey::SizeDesc => "size↓",
            SortKey::SizeAsc => "size↑",
            SortKey::NameAsc => "name↑",
            SortKey::NameDesc => "name↓",
            SortKey::CountDesc => "count↓",
            SortKey::CountAsc => "count↑",
        }
    } else {
        match k {
            SortKey::SizeDesc => "size-",
            SortKey::SizeAsc => "size+",
            SortKey::NameAsc => "name+",
            SortKey::NameDesc => "name-",
            SortKey::CountDesc => "count-",
            SortKey::CountAsc => "count+",
        }
    }
}

fn display_name(name: &str, kind: NodeKind) -> String {
    let mut s = sanitize(name);
    match kind {
        NodeKind::Dir => s.push('/'),
        NodeKind::Symlink => s.push('@'),
        _ => {}
    }
    s
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect()
}

fn truncate_left(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let take = max.saturating_sub(1);
    let chars: Vec<char> = s.chars().collect();
    let start = chars.len().saturating_sub(take);
    let mut out = String::from("…");
    out.extend(&chars[start..]);
    out
}

fn row_style(frac: f64, kind: NodeKind, error: bool, selected: bool) -> Style {
    let mut s = match kind {
        NodeKind::Dir => Style::default().fg(Color::Cyan),
        _ => Style::default(),
    };
    if frac > 0.25 {
        s = s.fg(Color::Red);
    } else if frac > 0.10 {
        s = s.fg(Color::Yellow);
    }
    if error {
        s = s.fg(Color::Red);
    }
    if selected {
        s = s.add_modifier(Modifier::REVERSED);
    }
    s
}

fn centered_rect(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

fn is_utf8_terminal() -> bool {
    if cfg!(windows) {
        return true;
    }
    let lang = std::env::var("LANG").unwrap_or_default();
    let lc_all = std::env::var("LC_ALL").unwrap_or_default();
    let lc_ctype = std::env::var("LC_CTYPE").unwrap_or_default();
    let v = format!("{lang} {lc_all} {lc_ctype}").to_uppercase();
    v.contains("UTF-8") || v.contains("UTF8")
}

fn spinner_char() -> char {
    const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    FRAMES[((now_ms / 100) as usize) % FRAMES.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{scan, scan_async, ScanOpts};

    fn handle_from_sync(path: PathBuf, opts: ScanOpts) -> ScanHandle {
        let result = scan(&path, &opts).unwrap();
        ScanHandle {
            state: Arc::new(Mutex::new(result)),
            join_handle: None,
        }
    }

    fn unique_temp(label: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("xdu-tui-{label}-{pid}-{nanos}-{n}"))
    }

    #[test]
    fn truncate_left_short_unchanged() {
        assert_eq!(truncate_left("hello", 10), "hello");
    }

    #[test]
    fn truncate_left_long_uses_ellipsis() {
        let out = truncate_left("0123456789abcdef", 6);
        assert_eq!(out.chars().count(), 6);
        assert!(out.starts_with('…'));
        assert!(out.ends_with("bcdef"));
    }

    #[test]
    fn sanitize_replaces_control_chars() {
        let r = sanitize("a\nb\rc\tD");
        assert_eq!(r, "a?b?c?D");
    }

    #[test]
    fn display_name_marks_dir_and_symlink() {
        assert_eq!(display_name("a", NodeKind::Dir), "a/");
        assert_eq!(display_name("a", NodeKind::Symlink), "a@");
        assert_eq!(display_name("a", NodeKind::File), "a");
    }

    #[test]
    fn sort_labels_cover_all_keys() {
        for k in [
            SortKey::SizeDesc,
            SortKey::SizeAsc,
            SortKey::NameAsc,
            SortKey::NameDesc,
            SortKey::CountDesc,
            SortKey::CountAsc,
        ] {
            assert!(!sort_label(k).is_empty());
        }
    }

    #[test]
    fn task007c_sort_label_uses_arrows_in_utf8() {
        assert_eq!(sort_label_for(SortKey::SizeDesc, true), "size↓");
        assert_eq!(sort_label_for(SortKey::SizeAsc, true), "size↑");
        assert_eq!(sort_label_for(SortKey::NameAsc, true), "name↑");
        assert_eq!(sort_label_for(SortKey::NameDesc, true), "name↓");
        assert_eq!(sort_label_for(SortKey::CountDesc, true), "count↓");
        assert_eq!(sort_label_for(SortKey::CountAsc, true), "count↑");
    }

    #[test]
    fn task007c_sort_label_falls_back_to_ascii() {
        assert_eq!(sort_label_for(SortKey::SizeDesc, false), "size-");
        assert_eq!(sort_label_for(SortKey::SizeAsc, false), "size+");
        assert_eq!(sort_label_for(SortKey::CountAsc, false), "count+");
    }

    #[test]
    fn task007c_name_column_width_floors_at_8() {
        // Width < fixed (35 cells of bar/size/date/separators) → name column floors at 8.
        assert_eq!(name_column_width(30), 8);
        assert_eq!(name_column_width(0), 8);
        assert_eq!(name_column_width(35), 8);
        // Wider terminal yields proportional name column.
        assert_eq!(
            name_column_width(80),
            80 - (BAR_W + SEP_W + SIZE_W + SEP_W + DATE_W + SEP_W)
        );
    }

    #[test]
    fn task007c_compose_footer_narrower_than_right_does_not_panic() {
        // Width 5, right segment is ` right ` (7 chars) — the right wins via
        // `truncate_left(right, width)`.
        let s = compose_footer(" left ", " right ", 5);
        assert_eq!(s.chars().count(), 5);
    }

    #[test]
    fn task007c_compose_footer_width_zero_is_empty() {
        assert_eq!(compose_footer("anything", "more", 0), "");
    }

    #[test]
    fn spinner_char_is_one_of_the_frames() {
        let c = spinner_char();
        assert!(['|', '/', '-', '\\'].contains(&c));
    }

    #[test]
    fn bug001_ascend_at_root_walks_filesystem_parent() {
        let td_path = unique_temp("asc");
        let inner = td_path.join("inner");
        std::fs::create_dir_all(&inner).unwrap();
        std::fs::write(inner.join("a.txt"), b"hello").unwrap();

        let opts = ScanOpts::default();
        let h = handle_from_sync(inner.clone(), opts.clone());
        let mut app = App::new(inner.clone(), opts, h, SizeMode::Iec);

        assert!(app.back_stack.is_empty());
        let inner_size = app.lock().tree.get(app.current).unwrap().size;
        assert_eq!(inner_size, 5);

        app.ascend();

        // After ascend, App started a fresh async scan of the parent. Wait
        // for it to finish so the assertion is deterministic.
        wait_for_running_false(&app, std::time::Duration::from_secs(5));

        let canonical_td = std::fs::canonicalize(&td_path).unwrap_or(td_path.clone());
        let canonical_app = std::fs::canonicalize(&app.root_path).unwrap_or(app.root_path.clone());
        assert_eq!(canonical_app, canonical_td);
        let parent_size = app.lock().tree.get(app.current).unwrap().size;
        assert!(parent_size >= 5);

        let _ = std::fs::remove_dir_all(&td_path);
    }

    fn wait_for_running_false(app: &App, timeout: std::time::Duration) {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if !app.lock().running {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn compose_footer_pads_when_both_fit() {
        let s = compose_footer(" left ", " right ", 20);
        assert_eq!(s.chars().count(), 20);
        assert!(s.starts_with(" left "));
        assert!(s.ends_with(" right "));
    }

    #[test]
    fn compose_footer_truncates_left_when_narrow() {
        let s = compose_footer(" sort:size  errors:0  ?:help ", " free: 10 GiB (50%) ", 25);
        assert!(s.contains("free:"));
    }

    #[test]
    fn compose_footer_empty_right_just_truncates() {
        let s = compose_footer(" hello ", "", 100);
        assert_eq!(s, " hello ");
    }

    #[test]
    fn task005_confirm_buttons_default_focus_no() {
        // yes_focused = false -> No is highlighted, Yes is dim.
        let line = confirm_buttons_line(false);
        let spans = line.spans;
        // Find the spans for Yes and No labels.
        let yes_span = spans
            .iter()
            .find(|s| s.content.contains("Yes"))
            .expect("Yes span present");
        let no_span = spans
            .iter()
            .find(|s| s.content.contains("No"))
            .expect("No span present");
        assert!(
            no_span.style.add_modifier.contains(Modifier::REVERSED),
            "No must be reverse-video on default focus"
        );
        assert!(
            !yes_span.style.add_modifier.contains(Modifier::REVERSED),
            "Yes must NOT be reverse-video on default focus"
        );
    }

    #[test]
    fn task005_confirm_buttons_yes_focused_swaps() {
        let line = confirm_buttons_line(true);
        let yes = line
            .spans
            .iter()
            .find(|s| s.content.contains("Yes"))
            .unwrap();
        let no = line
            .spans
            .iter()
            .find(|s| s.content.contains("No"))
            .unwrap();
        assert!(yes.style.add_modifier.contains(Modifier::REVERSED));
        assert!(!no.style.add_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn task005_open_prompt_then_cancel_does_not_delete() {
        let td_path = unique_temp("del-cancel");
        std::fs::create_dir_all(&td_path).unwrap();
        let target = td_path.join("a.txt");
        std::fs::write(&target, b"keepme").unwrap();

        let opts = ScanOpts::default();
        let h = handle_from_sync(td_path.clone(), opts.clone());
        let mut app = App::new(td_path.clone(), opts, h, SizeMode::Iec);
        // Select the only entry (offset 0 since at scan root with no FS parent
        // detectable to test scan setup; we just pick selection 0 directly).
        app.selection = if app.dotdot_offset() == 1 { 1 } else { 0 };
        app.start_delete_prompt();
        assert!(app.confirm_prompt.is_some(), "prompt should open");
        // Cancel via Esc.
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(app.confirm_prompt.is_none(), "prompt should close");
        assert!(target.exists(), "file must NOT be deleted on cancel");

        let _ = std::fs::remove_dir_all(&td_path);
    }

    #[test]
    fn task005_yes_actually_deletes_file() {
        let td_path = unique_temp("del-yes");
        std::fs::create_dir_all(&td_path).unwrap();
        let target = td_path.join("byebye.txt");
        std::fs::write(&target, b"goodbye").unwrap();

        let opts = ScanOpts::default();
        let h = handle_from_sync(td_path.clone(), opts.clone());
        let mut app = App::new(td_path.clone(), opts, h, SizeMode::Iec);
        app.selection = if app.dotdot_offset() == 1 { 1 } else { 0 };
        app.start_delete_prompt();
        assert!(app.confirm_prompt.is_some());
        // Toggle to Yes via Tab, then Enter.
        app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(app.confirm_prompt.is_none());
        assert!(!target.exists(), "file must be deleted after Yes+Enter");

        let _ = std::fs::remove_dir_all(&td_path);
    }

    fn app_with_files(label: &str, names: &[&str]) -> (App, PathBuf) {
        let td_path = unique_temp(label);
        std::fs::create_dir_all(&td_path).unwrap();
        for (i, n) in names.iter().enumerate() {
            std::fs::write(td_path.join(n), vec![0u8; 100 * (i + 1)]).unwrap();
        }
        let opts = ScanOpts::default();
        let h = handle_from_sync(td_path.clone(), opts.clone());
        let app = App::new(td_path.clone(), opts, h, SizeMode::Iec);
        (app, td_path)
    }

    #[test]
    fn space_marks_and_unmarks_and_advances_cursor() {
        let (mut app, td) = app_with_files("mark-toggle", &["a.txt", "b.txt", "c.txt"]);
        app.selection = app.dotdot_offset();
        let start = app.selection;
        app.handle_key(KeyCode::Char(' '), KeyModifiers::NONE);
        assert_eq!(app.marks.len(), 1);
        assert_eq!(app.selection, start + 1);
        app.selection = start;
        app.handle_key(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(app.marks.is_empty(), "second Space unmarks");
        app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(app.marks.len(), 3);
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(app.marks.is_empty());
        let _ = std::fs::remove_dir_all(&td);
    }

    #[test]
    fn d_with_marks_deletes_only_marked_files_after_confirm() {
        let (mut app, td) = app_with_files("mark-del", &["a.txt", "b.txt", "c.txt"]);
        let state_before = std::sync::Arc::as_ptr(&app.state);
        app.selection = app.dotdot_offset();
        app.handle_key(KeyCode::Char(' '), KeyModifiers::NONE); // marks row 1, moves to row 2
        app.handle_key(KeyCode::Char(' '), KeyModifiers::NONE); // marks row 2
        assert_eq!(app.marks.len(), 2);
        app.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
        let prompt = app.confirm_prompt.as_ref().expect("batch prompt opens");
        assert!(
            prompt.message.contains("2 marked items"),
            "{}",
            prompt.message
        );
        assert_eq!(prompt.action, ConfirmAction::DeleteMarked);
        // Nothing deleted yet; default focus is No.
        assert_eq!(std::fs::read_dir(&td).unwrap().count(), 3);
        app.handle_key(KeyCode::Char('y'), KeyModifiers::NONE);

        assert_eq!(std::fs::read_dir(&td).unwrap().count(), 1);
        assert_eq!(app.children_count(), 1);
        assert!(app.marks.is_empty());
        assert_eq!(
            state_before,
            std::sync::Arc::as_ptr(&app.state),
            "no rescan"
        );
        assert!(app.status.as_deref().unwrap_or("").contains("2 item"));
        let _ = std::fs::remove_dir_all(&td);
    }

    #[test]
    fn d_with_marks_cancel_deletes_nothing() {
        let (mut app, td) = app_with_files("mark-cancel", &["a.txt", "b.txt"]);
        app.selection = app.dotdot_offset();
        app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('n'), KeyModifiers::NONE);
        assert_eq!(std::fs::read_dir(&td).unwrap().count(), 2);
        assert_eq!(app.marks.len(), 2, "marks are kept after cancel");
        let _ = std::fs::remove_dir_all(&td);
    }

    #[test]
    fn marked_roots_skips_children_of_marked_directories() {
        let td = unique_temp("mark-nested");
        std::fs::create_dir_all(td.join("dir")).unwrap();
        std::fs::write(td.join("dir").join("inner.txt"), b"x").unwrap();
        let opts = ScanOpts::default();
        let h = handle_from_sync(td.clone(), opts.clone());
        let mut app = App::new(td.clone(), opts, h, SizeMode::Iec);
        let (dir, inner) = {
            let g = app.lock();
            let dir = g.tree.children(g.tree.root()).next().unwrap();
            let inner = g.tree.children(dir).next().unwrap();
            (dir, inner)
        };
        app.marks.insert(dir);
        app.marks.insert(inner);
        assert_eq!(app.marked_roots(), vec![dir]);
        let _ = std::fs::remove_dir_all(&td);
    }

    #[test]
    fn shift_p_opens_prune_prompt_defaulting_to_no_or_reports_unsupported() {
        let (mut app, td) = app_with_files("prune-key", &["a.txt"]);
        app.handle_key(KeyCode::Char('P'), KeyModifiers::NONE);
        if docker_shrink::is_supported() {
            let p = app.confirm_prompt.as_ref().expect("prune prompt opens");
            assert_eq!(p.action, ConfirmAction::Prune);
            assert!(!p.yes_focused, "default focus must be No");
            assert!(p.message.contains("UNUSED VOLUMES"));
        } else {
            assert!(app.confirm_prompt.is_none());
            assert!(app.status.as_deref().unwrap_or("").contains("Windows"));
        }
        assert!(app.shrink_job.is_none(), "nothing runs before confirmation");
        let _ = std::fs::remove_dir_all(&td);
    }

    #[test]
    fn shift_t_never_runs_anything_before_confirmation() {
        let (mut app, td) = app_with_files("tm-key", &["a.txt"]);
        app.handle_key(KeyCode::Char('T'), KeyModifiers::NONE);
        if let Some(p) = app.confirm_prompt.as_ref() {
            assert_eq!(p.action, ConfirmAction::TimeMachine);
            assert!(!p.yes_focused, "default focus must be No");
        } else {
            assert!(app.status.is_some(), "explains why no prompt opened");
        }
        assert!(app.shrink_job.is_none(), "nothing runs before confirmation");
        let _ = std::fs::remove_dir_all(&td);
    }

    #[test]
    fn z_on_non_vhdx_does_not_open_shrink_prompt() {
        let td_path = unique_temp("shrink-nonvhdx");
        std::fs::create_dir_all(&td_path).unwrap();
        std::fs::write(td_path.join("a.txt"), b"x").unwrap();

        let opts = ScanOpts::default();
        let h = handle_from_sync(td_path.clone(), opts.clone());
        let mut app = App::new(td_path.clone(), opts, h, SizeMode::Iec);
        app.selection = app.dotdot_offset();
        app.handle_key(KeyCode::Char('z'), KeyModifiers::NONE);
        assert!(app.confirm_prompt.is_none());
        assert!(app.shrink_job.is_none());
        assert!(app.status.is_some());

        let _ = std::fs::remove_dir_all(&td_path);
    }

    #[test]
    fn delete_updates_tree_in_place_without_rescan() {
        let td_path = unique_temp("del-nores");
        std::fs::create_dir_all(&td_path).unwrap();
        std::fs::write(td_path.join("a.txt"), vec![0u8; 1000]).unwrap();
        std::fs::write(td_path.join("b.txt"), vec![0u8; 10]).unwrap();

        let opts = ScanOpts::default();
        let h = handle_from_sync(td_path.clone(), opts.clone());
        let mut app = App::new(td_path.clone(), opts, h, SizeMode::Iec);
        let state_before = std::sync::Arc::as_ptr(&app.state);
        let root = app.lock().tree.root();
        let total_before = app.lock().tree.get(root).unwrap().size;
        app.selection = app.dotdot_offset(); // largest first: a.txt
        app.start_delete_prompt();
        app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        assert!(!td_path.join("a.txt").exists());
        assert_eq!(
            state_before,
            std::sync::Arc::as_ptr(&app.state),
            "delete must not start a new scan"
        );
        assert_eq!(app.children_count(), 1);
        let total_after = app.lock().tree.get(root).unwrap().size;
        assert_eq!(total_before - total_after, 1000);

        let _ = std::fs::remove_dir_all(&td_path);
    }

    #[test]
    fn bug003_status_reports_canonical_path_on_success() {
        let td_path = unique_temp("del-canon");
        std::fs::create_dir_all(&td_path).unwrap();
        let target = td_path.join("zap.txt");
        std::fs::write(&target, b"x").unwrap();
        let canonical = std::fs::canonicalize(&target).unwrap_or(target.clone());

        let opts = ScanOpts::default();
        let h = handle_from_sync(td_path.clone(), opts.clone());
        let mut app = App::new(td_path.clone(), opts, h, SizeMode::Iec);
        app.selection = if app.dotdot_offset() == 1 { 1 } else { 0 };
        app.start_delete_prompt();
        app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!target.exists(), "file must be gone");
        let s = app.status.as_deref().unwrap_or_default();
        assert!(s.contains("deleted"), "got: {s}");
        // Status must show canonical path (or at least the file's basename).
        assert!(
            s.contains(&canonical.display().to_string()) || s.contains("zap.txt"),
            "status missing canonical path: {s}"
        );

        let _ = std::fs::remove_dir_all(&td_path);
    }

    #[test]
    fn task005_refuses_to_delete_scan_root() {
        let td_path = unique_temp("del-root");
        std::fs::create_dir_all(&td_path).unwrap();
        std::fs::write(td_path.join("x"), b"y").unwrap();

        let opts = ScanOpts::default();
        let h = handle_from_sync(td_path.clone(), opts.clone());
        let mut app = App::new(td_path.clone(), opts, h, SizeMode::Iec);
        // Force the operation by calling perform_delete on the root NodeId.
        let root_id = app.lock().tree.root();
        app.perform_delete(root_id);
        assert!(td_path.exists(), "scan root must not be deleted");
        assert!(app
            .status
            .as_deref()
            .unwrap_or_default()
            .contains("scan root"),);
        let _ = std::fs::remove_dir_all(&td_path);
    }

    #[test]
    fn bug002_async_scan_progresses_to_finalized() {
        let path = unique_temp("async");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("a"), b"123").unwrap();
        std::fs::create_dir(path.join("sub")).unwrap();
        std::fs::write(path.join("sub").join("b"), b"4567").unwrap();

        let handle = scan_async(path.clone(), ScanOpts::default()).expect("spawn");
        let app = App::new(path.clone(), ScanOpts::default(), handle, SizeMode::Iec);

        wait_for_running_false(&app, std::time::Duration::from_secs(5));
        let g = app.lock();
        assert!(!g.running);
        assert!(g.tree.get(g.tree.root()).unwrap().finalized);
        assert_eq!(g.tree.get(g.tree.root()).unwrap().size, 7);
        drop(g);

        let _ = std::fs::remove_dir_all(&path);
    }
}
