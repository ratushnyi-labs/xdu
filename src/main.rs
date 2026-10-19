use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;

use xdu::drives_win;
use xdu::format::{format_size, SizeMode};
use xdu::scanner::{scan_async, ScanOpts};
use xdu::trash;
use xdu::tui;

fn main() {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("xdu: {msg}");
            process::exit(2);
        }
    };
    if args.show_help {
        print_help();
        return;
    }
    if args.show_version {
        println!("xdu v{}", env!("CARGO_PKG_VERSION"));
        return;
    }
    // Resolve the scan root. On Windows with no path argument, prompt for a
    // drive selection (TASK-007-H). Elsewhere, default to ".".
    let path = match args.path {
        Some(p) => p,
        None => match resolve_default_root() {
            Some(p) => p,
            None => {
                eprintln!("xdu: no path given and no drive could be selected");
                process::exit(2);
            }
        },
    };
    let opts = ScanOpts {
        cross_fs: args.cross_fs,
        threads: args.threads,
        ..ScanOpts::default()
    };
    let handle = match scan_async(path.clone(), opts.clone()) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("xdu: {e}");
            process::exit(2);
        }
    };
    let mode = if args.si { SizeMode::Si } else { SizeMode::Iec };
    if args.trash && !trash::is_supported() {
        eprintln!("xdu: --trash is not supported on this platform");
        process::exit(2);
    }
    if let Err(e) = tui::run_with(path, opts, handle, mode, args.trash) {
        eprintln!("xdu: TUI error: {e}");
        process::exit(3);
    }
}

struct Args {
    path: Option<PathBuf>,
    cross_fs: bool,
    threads: usize,
    si: bool,
    trash: bool,
    show_help: bool,
    show_version: bool,
}

fn parse_args<I: IntoIterator<Item = String>>(it: I) -> Result<Args, String> {
    let mut path: Option<PathBuf> = None;
    let mut cross_fs = false;
    let mut threads = 0_usize;
    let mut si = false;
    let mut trash = false;
    let mut show_help = false;
    let mut show_version = false;
    let mut it = it.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => show_help = true,
            "-V" | "--version" => show_version = true,
            "--cross-fs" => cross_fs = true,
            "-j" | "--threads" => {
                threads = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or_else(|| format!("{arg} needs a number (0 = auto)"))?;
            }
            "--si" => si = true,
            "--trash" => trash = true,
            s if s.starts_with('-') => return Err(format!("unknown flag: {s}")),
            s => {
                if path.is_some() {
                    return Err(format!("extra positional argument: {s}"));
                }
                path = Some(PathBuf::from(s));
            }
        }
    }
    Ok(Args {
        path,
        cross_fs,
        threads,
        si,
        trash,
        show_help,
        show_version,
    })
}

/// Pick the default scan root when no path is supplied on the command line.
/// On Windows: enumerate drives and prompt the user to choose one. On other
/// platforms: return the current directory (".").
fn resolve_default_root() -> Option<PathBuf> {
    let drives = drives_win::list_drives();
    if drives.is_empty() {
        return Some(PathBuf::from("."));
    }
    println!("xdu — pick a drive to scan:");
    println!();
    for (i, d) in drives.iter().enumerate() {
        let label = if d.label.is_empty() {
            String::from("(no label)")
        } else {
            d.label.clone()
        };
        let size_part = match (d.free_bytes, d.total_bytes) {
            (Some(free), Some(total)) if total > 0 => {
                format!(
                    " — {} free / {} total",
                    format_size(free, SizeMode::Iec),
                    format_size(total, SizeMode::Iec)
                )
            }
            _ => String::new(),
        };
        println!(
            "  {idx}) {letter}:\\  [{ty}]  {label}{size}",
            idx = i + 1,
            letter = d.letter,
            ty = d.drive_type.short(),
            label = label,
            size = size_part
        );
    }
    println!();
    print!("Selection [1-{n}, or Enter for 1]: ", n = drives.len());
    let _ = io::stdout().flush();
    let mut line = String::new();
    let stdin = io::stdin();
    if stdin.lock().read_line(&mut line).is_err() {
        return None;
    }
    let trimmed = line.trim();
    let idx = if trimmed.is_empty() {
        1
    } else {
        match trimmed.parse::<usize>() {
            Ok(n) if (1..=drives.len()).contains(&n) => n,
            _ => return None,
        }
    };
    Some(drives[idx - 1].root.clone())
}

fn print_help() {
    println!(
        "xdu v{} — read-only disk-usage explorer",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("usage: xdu [path] [flags]");
    println!();
    println!("On Windows, omitting [path] presents an interactive drive picker;");
    println!("on other platforms it defaults to the current directory.");
    println!();
    println!("flags:");
    println!("  --cross-fs    scan across filesystem boundaries (Unix)");
    println!("  -j, --threads N  scanner threads (default: auto; 1 = sequential)");
    println!("  --si          use 1000-based units (KB, MB, ...)");
    println!("  --trash       send deleted items to the OS trash (Linux + Windows)");
    println!("  -V, --version print version and exit");
    println!("  -h, --help    show this help and exit");
    println!();
    println!("interactive keybindings:");
    println!("  Up/k Down/j   move      s    cycle sort     ?/F1   help");
    println!("  Right/l Enter enter     u    toggle units   q/^C   quit");
    println!("  Left/h Bksp   up        r    rescan         e      export");
    println!("                          d    delete (with [No]-default prompt)");
    println!("                          Space mark, a mark all, Esc clear, then d deletes marked");
    println!("                          z    shrink selected .vhdx, prunes Docker (Windows)");
    println!("                          P    Docker prune only (Windows)");
    println!("                          !    show errors");
    println!();
    println!("xdu is read-only by default; deletion is exposed only behind 'd'");
    println!("plus a confirmation modal whose default focus is [ No ].");
}
