//! Terminal UI: runtime, tree-view widget, header / footer / help / errors
//! overlays, and rescan flow. Combines TASK-001-H/I/J/K leaves into one
//! module to keep cross-references compact.
//!
//! Async (BUG-002): the TUI holds an `Arc<Mutex<ScanResult>>` shared with a
//! background scanner thread. Every render and every key handler that
//! touches the tree locks the mutex briefly. Cells whose value is not yet
//! finalized render an animated spinner.

use crate::disk_space;
use crate::export;
use crate::format::{bar, format_size, format_ymd, SizeMode};
use crate::scanner::{scan_async, ScanError, ScanHandle, ScanOpts, ScanResult};
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
    quit: bool,
}

impl App {
    pub fn new(
        root_path: PathBuf,
        opts: ScanOpts,
        handle: ScanHandle,
        size_mode: SizeMode,
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
        match scan_async(path.clone(), self.opts.clone()) {
            Ok(handle) => {
                self.state = handle.state;
                self._scan_thread = handle.join_handle;
                self.root_path = path;
                let new_root = self.lock().tree.root();
                self.current = new_root;
                self.selection = 0;
                self.back_stack.clear();
                self.sort_current();
                self.status = Some(ok_msg.to_string());
            }
            Err(e) => {
                self.status = Some(format!("scan failed: {e}"));
                self.lock().errors.push(ScanError {
                    path: PathBuf::new(),
                    message: format!("{e}"),
                });
            }
        }
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
        if (key == KeyCode::Char('c') && mods.contains(KeyModifiers::CONTROL))
            || matches!(key, KeyCode::Char('q'))
        {
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
            _ => {}
        }
    }
}

pub fn run(
    root_path: PathBuf,
    opts: ScanOpts,
    handle: ScanHandle,
    size_mode: SizeMode,
) -> io::Result<()> {
    let mut term = init_terminal()?;
    let result = run_loop(&mut term, App::new(root_path, opts, handle, size_mode));
    teardown_terminal();
    result
}

fn run_loop(term: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<()> {
    while !app.quit {
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
) -> Line<'static> {
    let name = display_name(&node.name, node.kind);
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
    let style = row_style(frac, node.kind, node.error, selected);
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
        format!(
            " sort:{}  errors:{}  ?:help  s:sort  u:unit  r:rescan  e:export  d:delete  !:errors  q:quit ",
            sort_label(app.sort),
            state.errors.len()
        )
    };
    let right = disk_space::format_free(&app.root_path, app.size_mode);
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
        Line::from("  !               show errors"),
        Line::from("  ? / F1          this help"),
        Line::from("  q / Ctrl-C      quit"),
        Line::from(""),
        Line::from("xdu is read-only; no file is ever modified or deleted."),
        Line::from("Cells with a spinner are not yet calculated; they update live."),
        Line::from(""),
        Line::from("Press any key to dismiss."),
    ];
    let block = Block::default().borders(Borders::ALL).title(" Help ");
    let r = centered_rect(area, 64, 24);
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
    match k {
        SortKey::SizeDesc => "size-",
        SortKey::SizeAsc => "size+",
        SortKey::NameAsc => "name+",
        SortKey::NameDesc => "name-",
        SortKey::CountDesc => "count-",
        SortKey::CountAsc => "count+",
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
