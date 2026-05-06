//! Terminal UI: runtime, tree-view widget, header / footer / help / errors
//! overlays, and rescan flow. Combines TASK-001-H/I/J/K leaves into one
//! module to keep cross-references compact.

use crate::export;
use crate::format::{bar, format_size, SizeMode};
use crate::scanner::{scan, ScanOpts, ScanResult};
use crate::tree::{NodeId, NodeKind, SortKey};
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
use std::time::Duration;

pub struct App {
    root_path: PathBuf,
    opts: ScanOpts,
    scan: ScanResult,
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
        scan_result: ScanResult,
        size_mode: SizeMode,
    ) -> Self {
        let mut s = Self {
            current: scan_result.tree.root(),
            scan: scan_result,
            root_path,
            opts,
            selection: 0,
            back_stack: Vec::new(),
            sort: SortKey::SizeDesc,
            show_help: false,
            show_errors: false,
            size_mode,
            status: None,
            quit: false,
        };
        let cur = s.current;
        s.scan.tree.sort(cur, s.sort);
        s
    }

    fn children_count(&self) -> usize {
        self.scan.tree.children(self.current).count()
    }

    fn move_selection(&mut self, delta: i32) {
        let n = self.children_count();
        if n == 0 {
            self.selection = 0;
            return;
        }
        let cur = self.selection as i32;
        let max = n as i32 - 1;
        self.selection = cur.saturating_add(delta).clamp(0, max) as usize;
    }

    fn descend(&mut self) {
        let kids: Vec<NodeId> = self.scan.tree.children(self.current).collect();
        let Some(&target) = kids.get(self.selection) else {
            return;
        };
        let kind = self.scan.tree.get(target).map(|n| n.kind);
        if matches!(kind, Some(NodeKind::Dir)) {
            self.back_stack.push((self.current, self.selection));
            self.current = target;
            self.selection = 0;
            self.scan.tree.sort(self.current, self.sort);
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
        match scan(&parent, &self.opts) {
            Ok(r) => {
                self.root_path = parent;
                self.scan = r;
                self.current = self.scan.tree.root();
                self.selection = 0;
                self.back_stack.clear();
                let cur = self.current;
                self.scan.tree.sort(cur, self.sort);
                self.status = Some("scanned parent directory".into());
            }
            Err(e) => self.status = Some(format!("cannot scan parent: {e}")),
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
        self.scan.tree.sort(self.current, self.sort);
        self.selection = 0;
    }

    fn toggle_size_mode(&mut self) {
        self.size_mode = match self.size_mode {
            SizeMode::Iec => SizeMode::Si,
            SizeMode::Si => SizeMode::Iec,
        };
    }

    fn rescan(&mut self) {
        match scan(&self.root_path, &self.opts) {
            Ok(r) => {
                self.scan = r;
                self.current = self.scan.tree.root();
                self.selection = 0;
                self.back_stack.clear();
                let cur = self.current;
                self.scan.tree.sort(cur, self.sort);
                self.status = Some("rescan complete".to_string());
            }
            Err(e) => {
                self.status = Some(format!("rescan failed: {e}"));
                self.scan.errors.push(e);
            }
        }
    }

    fn export(&mut self) {
        let path: PathBuf = "xdu-export.json".into();
        match export::write_to_path(&self.scan.tree, self.current, &self.opts, &path) {
            Ok(_) => self.status = Some(format!("exported to {}", path.display())),
            Err(e) => self.status = Some(format!("export failed: {e}")),
        }
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
            KeyCode::End => self.selection = self.children_count().saturating_sub(1),
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
    scan_result: ScanResult,
    size_mode: SizeMode,
) -> io::Result<()> {
    let mut term = init_terminal()?;
    let result = run_loop(&mut term, App::new(root_path, opts, scan_result, size_mode));
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
    render_header(f, chunks[0], app);
    render_body(f, chunks[1], app);
    render_footer(f, chunks[2], app);
    if app.show_help {
        render_help(f, area);
    }
    if app.show_errors {
        render_errors(f, area, app);
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let path = app.scan.tree.path(app.current).join("/");
    let text = format!(" {path} ");
    let s = truncate_left(&text, area.width as usize);
    let style = Style::default().add_modifier(Modifier::REVERSED);
    f.render_widget(Paragraph::new(s).style(style), area);
}

fn render_body(f: &mut Frame, area: Rect, app: &App) {
    let tree = &app.scan.tree;
    let kids: Vec<NodeId> = tree.children(app.current).collect();
    let cur = match tree.get(app.current) {
        Some(n) => n,
        None => return,
    };
    let dir_total = cur.size.max(1);
    let utf8 = is_utf8_terminal();
    let height = area.height as usize;
    let n = kids.len();
    let start = if n <= height {
        0
    } else {
        app.selection.saturating_sub(height / 2).min(n - height)
    };
    let end = (start + height).min(n);
    let mut lines: Vec<Line> = Vec::with_capacity(end.saturating_sub(start));
    for (i, &id) in kids[start..end].iter().enumerate() {
        let absolute_i = start + i;
        let Some(node) = tree.get(id) else { continue };
        let frac = (node.size as f64) / (dir_total as f64);
        let s = format!(
            "{}  {:>9}  {}",
            bar(frac, 10, utf8),
            format_size(node.size, app.size_mode),
            display_name(&node.name, node.kind)
        );
        let style = row_style(frac, node.kind, node.error, absolute_i == app.selection);
        lines.push(Line::from(Span::styled(s, style)));
    }
    f.render_widget(Paragraph::new(lines), area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let s = if let Some(msg) = app.status.as_deref() {
        format!(" {msg}  (any key to dismiss) ")
    } else {
        format!(
            " sort:{}  errors:{}  ?:help  s:sort  u:unit  r:rescan  e:export  !:errors  q:quit ",
            sort_label(app.sort),
            app.scan.errors.len()
        )
    };
    let s = truncate_left(&s, area.width as usize);
    let style = Style::default().add_modifier(Modifier::REVERSED);
    f.render_widget(Paragraph::new(s).style(style), area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from("xdu — keybindings"),
        Line::from(""),
        Line::from("  Up / k          move up"),
        Line::from("  Down / j        move down"),
        Line::from("  Right/l/Enter   enter directory"),
        Line::from("  Left/h/Bksp     go up (rescans FS parent at scan root)"),
        Line::from("  PgUp / PgDn     page"),
        Line::from("  Home / End      jump"),
        Line::from("  s               cycle sort"),
        Line::from("  u               toggle KiB / KB units"),
        Line::from("  r               rescan from root (synchronous)"),
        Line::from("  e               export current subtree to xdu-export.json"),
        Line::from("  !               show errors"),
        Line::from("  ? / F1          this help"),
        Line::from("  q / Ctrl-C      quit"),
        Line::from(""),
        Line::from("xdu is read-only; no file is ever modified or deleted."),
        Line::from(""),
        Line::from("Press any key to dismiss."),
    ];
    let block = Block::default().borders(Borders::ALL).title(" Help ");
    let r = centered_rect(area, 60, 22);
    f.render_widget(Clear, r);
    f.render_widget(Paragraph::new(lines).block(block), r);
}

fn render_errors(f: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();
    for e in app.scan.errors.iter().take(50) {
        lines.push(Line::from(format!("{}: {}", e.path.display(), e.message)));
    }
    if app.scan.overflow_errors {
        lines.push(Line::from("(error log capped — additional errors dropped)"));
    }
    if lines.is_empty() {
        lines.push(Line::from("No errors recorded."));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Press any key to dismiss."));
    let title = format!(" Errors ({}) ", app.scan.errors.len());
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn bug001_ascend_at_root_walks_filesystem_parent() {
        use crate::scanner::{scan, ScanOpts};
        use std::fs;
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(9000);
        let n = N.fetch_add(1, Ordering::SeqCst);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let td_path = std::env::temp_dir().join(format!(
            "xdu-tui-asc-{}-{}-{}",
            std::process::id(),
            nanos,
            n
        ));
        let inner = td_path.join("inner");
        fs::create_dir_all(&inner).unwrap();
        fs::write(inner.join("a.txt"), b"hello").unwrap();

        let opts = ScanOpts::default();
        let r = scan(&inner, &opts).unwrap();
        let mut app = App::new(inner.clone(), opts, r, SizeMode::Iec);

        assert!(app.back_stack.is_empty());
        assert_eq!(app.scan.tree.get(app.scan.tree.root()).unwrap().size, 5);

        app.ascend();

        let canonical_td = std::fs::canonicalize(&td_path).unwrap_or(td_path.clone());
        let canonical_app = std::fs::canonicalize(&app.root_path).unwrap_or(app.root_path.clone());
        assert_eq!(canonical_app, canonical_td);
        let parent_size = app.scan.tree.get(app.scan.tree.root()).unwrap().size;
        assert!(parent_size >= 5);

        let _ = fs::remove_dir_all(&td_path);
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
}
