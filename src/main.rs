use std::path::PathBuf;
use std::process;

use xdu::format::SizeMode;
use xdu::scanner::{scan, ScanOpts};
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
    let opts = ScanOpts {
        cross_fs: args.cross_fs,
        ..ScanOpts::default()
    };
    let result = match scan(&args.path, &opts) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("xdu: {e}");
            process::exit(2);
        }
    };
    let mode = if args.si { SizeMode::Si } else { SizeMode::Iec };
    if let Err(e) = tui::run(args.path, opts, result, mode) {
        eprintln!("xdu: TUI error: {e}");
        process::exit(3);
    }
}

struct Args {
    path: PathBuf,
    cross_fs: bool,
    si: bool,
    show_help: bool,
    show_version: bool,
}

fn parse_args<I: IntoIterator<Item = String>>(it: I) -> Result<Args, String> {
    let mut path: Option<PathBuf> = None;
    let mut cross_fs = false;
    let mut si = false;
    let mut show_help = false;
    let mut show_version = false;
    for arg in it {
        match arg.as_str() {
            "-h" | "--help" => show_help = true,
            "-V" | "--version" => show_version = true,
            "--cross-fs" => cross_fs = true,
            "--si" => si = true,
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
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        cross_fs,
        si,
        show_help,
        show_version,
    })
}

fn print_help() {
    println!(
        "xdu v{} — read-only disk-usage explorer",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("usage: xdu [path] [flags]");
    println!();
    println!("flags:");
    println!("  --cross-fs    scan across filesystem boundaries (Unix)");
    println!("  --si          use 1000-based units (KB, MB, ...)");
    println!("  -V, --version print version and exit");
    println!("  -h, --help    show this help and exit");
    println!();
    println!("interactive keybindings:");
    println!("  Up/k Down/j   move      s    cycle sort     ?/F1   help");
    println!("  Right/l Enter enter     u    toggle units   q/^C   quit");
    println!("  Left/h Bksp   up        r    rescan         e      errors");
    println!();
    println!("xdu is read-only; no file is ever modified or deleted.");
}
