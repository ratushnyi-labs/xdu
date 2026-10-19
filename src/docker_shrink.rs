//! Reclaim host disk from a `.vhdx`: (Docker) prune, then compact the VHDX.
//!
//! A `.vhdx` only ever grows: deleting data inside the guest frees space
//! *inside* the virtual disk, never on the host. Getting it back needs a
//! compaction, driven from here behind an explicit confirmation modal in the
//! TUI. The sequence mirrors a manual cleanup script that is known to work:
//!
//! 1. **Docker disks only** (`docker_data.vhdx`, anything under a `Docker`
//!    folder): `docker container|image -a|builder -a|volume -a prune -f`
//!    (the equivalent of `docker system prune -a --volumes`, which also drops
//!    *named* unused volumes), then `fstrim` inside the `docker-desktop` VM so
//!    the freed blocks are reportable to Windows.
//! 2. Force-stop Docker Desktop, `wsl --shutdown`, wait for the disk to be
//!    released.
//! 3. Compact elevated (UAC): `Optimize-VHD -Mode Full` when the Hyper-V
//!    module exists, otherwise `diskpart` (`compact vdisk`).
//! 4. Start Docker Desktop again if it was running.
//!
//! The orchestration is written against the [`Runner`] trait so it is testable
//! without Docker or Windows. Only [`SystemRunner`] touches real processes.

use crate::format::{format_size, SizeMode};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Compaction relies on Windows-only tooling (`wsl`, `Optimize-VHD` /
/// `diskpart`, UAC elevation).
pub fn is_supported() -> bool {
    cfg!(windows)
}

/// `true` for paths that look like a virtual-disk image this module can compact.
pub fn is_vhdx(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("vhdx"))
}

/// `true` when `path` is Docker Desktop's disk (so a Docker prune applies).
pub fn is_docker_disk(path: &Path) -> bool {
    path.to_string_lossy().to_lowercase().contains("docker")
}

#[derive(Debug, Clone)]
pub struct CmdOutput {
    pub ok: bool,
    pub text: String,
}

pub trait Runner {
    /// Run `program` with `args` to completion. Spawn failures are reported as
    /// `ok == false` with the error text.
    fn run(&mut self, program: &str, args: &[String]) -> CmdOutput;

    /// Wait for `d` (overridden in tests).
    fn sleep(&mut self, d: Duration) {
        std::thread::sleep(d);
    }
}

pub struct SystemRunner;

impl Runner for SystemRunner {
    fn run(&mut self, program: &str, args: &[String]) -> CmdOutput {
        match Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .output()
        {
            Ok(o) => {
                let mut text = String::from_utf8_lossy(&o.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&o.stderr));
                CmdOutput {
                    ok: o.status.success(),
                    text: text.trim().to_string(),
                }
            }
            Err(e) => CmdOutput {
                ok: false,
                text: format!("{program}: {e}"),
            },
        }
    }
}

/// Shared between the worker thread and the TUI.
#[derive(Debug, Default, Clone)]
pub struct Progress {
    /// Name of the job ("Docker prune + shrink" / "Docker prune").
    pub title: String,
    /// Human-readable description of the step currently running.
    pub step: String,
    pub done: bool,
    /// `Some` once `done`: `Ok(summary)` or `Err(reason)`.
    pub result: Option<Result<String, String>>,
}

pub type SharedProgress = Arc<Mutex<Progress>>;

pub fn new_progress() -> SharedProgress {
    new_titled("Docker prune + shrink")
}

pub fn new_titled(title: &str) -> SharedProgress {
    Arc::new(Mutex::new(Progress {
        title: title.to_string(),
        step: "starting".into(),
        ..Progress::default()
    }))
}

pub fn set_step(p: &SharedProgress, step: &str) {
    if let Ok(mut g) = p.lock() {
        g.step = step.to_string();
    }
}

pub fn finish(p: &SharedProgress, result: Result<String, String>) {
    if let Ok(mut g) = p.lock() {
        g.done = true;
        g.result = Some(result);
    }
}

/// Run the whole sequence on a background thread.
pub fn spawn(vhdx: PathBuf) -> SharedProgress {
    let progress = new_progress();
    let worker = progress.clone();
    std::thread::Builder::new()
        .name("xdu-vhdx-shrink".to_string())
        .spawn(move || {
            let mut runner = SystemRunner;
            run_shrink(
                &vhdx,
                &mut runner,
                |p| std::fs::metadata(p).ok().map(|m| m.len()),
                &worker,
            );
        })
        .map(|_| ())
        .unwrap_or_else(|e| finish(&progress, Err(format!("cannot start worker: {e}"))));
    progress
}

fn strip_verbatim(p: &Path) -> String {
    let s = p.to_string_lossy();
    s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
}

/// PowerShell script (run elevated) that compacts `vhdx`: `Optimize-VHD` when
/// the Hyper-V module is present, `diskpart` otherwise (Windows Home).
pub fn compact_script(vhdx: &Path) -> String {
    let p = strip_verbatim(vhdx).replace('\'', "''");
    format!(
        r#"$ErrorActionPreference = 'Stop'
$p = '{p}'
try {{
  if (Get-Command Optimize-VHD -ErrorAction SilentlyContinue) {{
    Optimize-VHD -Path $p -Mode Full
  }} else {{
    $dp = New-TemporaryFile
    @("select vdisk file=`"$p`"", 'attach vdisk readonly', 'compact vdisk', 'detach vdisk', 'exit') | Set-Content $dp -Encoding ascii
    try {{ diskpart /s $dp }} finally {{ Remove-Item $dp -Force }}
  }}
}} catch {{
  Write-Error $_
  exit 1
}}
"#
    )
}

/// Arguments for `powershell` that run the script file elevated (UAC), wait,
/// and propagate its exit code.
fn elevated_script_args(script: &Path) -> Vec<String> {
    let script = script.display().to_string().replace('\'', "''");
    vec![
        "-NoProfile".into(),
        "-NonInteractive".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-Command".into(),
        format!(
            "$ErrorActionPreference='Stop'; $p = Start-Process -FilePath powershell.exe -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','\"{script}\"' -Verb RunAs -Wait -PassThru -WindowStyle Hidden; exit $p.ExitCode"
        ),
    ]
}

fn args(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// Prune commands, equivalent to `docker system prune -a --volumes` (the
/// per-resource form also removes *named* unused volumes).
const PRUNE_CMDS: [&[&str]; 4] = [
    &["container", "prune", "-f"],
    &["image", "prune", "-a", "-f"],
    &["builder", "prune", "-a", "-f"],
    &["volume", "prune", "-a", "-f"],
];

const DOCKER_PROCESSES: [&str; 3] = [
    "Docker Desktop.exe",
    "com.docker.backend.exe",
    "com.docker.build.exe",
];

fn docker_desktop_running<R: Runner>(r: &mut R) -> bool {
    let out = r.run(
        "tasklist",
        &args(&["/FI", "IMAGENAME eq Docker Desktop.exe", "/NH"]),
    );
    out.ok && out.text.contains("Docker Desktop.exe")
}

fn start_docker_desktop<R: Runner>(r: &mut R) {
    if r.run("docker", &args(&["desktop", "start"])).ok {
        return;
    }
    let base = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    let exe = format!(r"{base}\Docker\Docker\Docker Desktop.exe");
    let _ = r.run("cmd", &["/C".into(), "start".into(), "".into(), exe]);
}

/// Prune Docker and `fstrim` its VM. `Ok` is a short note for the summary;
/// `Err` means the Docker daemon could not be reached.
fn prune_docker<R: Runner>(runner: &mut R, progress: &SharedProgress) -> Result<String, String> {
    if !runner.run("docker", &args(&["info"])).ok {
        return Err("Docker daemon not reachable".into());
    }
    set_step(
        progress,
        "pruning Docker (containers, images, build cache, volumes)",
    );
    let mut failed = 0;
    let mut reclaimed: Vec<String> = Vec::new();
    for c in PRUNE_CMDS {
        let out = runner.run("docker", &args(c));
        if !out.ok {
            failed += 1;
        } else if let Some(line) = out
            .text
            .lines()
            .find_map(|l| l.trim().strip_prefix("Total reclaimed space:"))
        {
            reclaimed.push(line.trim().to_string());
        }
    }
    set_step(progress, "trimming free blocks (fstrim)");
    let trimmed = runner
        .run(
            "wsl",
            &args(&["-d", "docker-desktop", "sh", "-c", "fstrim -av"]),
        )
        .ok;
    let mut note = if failed == 0 {
        "Docker pruned".to_string()
    } else {
        format!(
            "Docker prune: {failed} of {} steps failed",
            PRUNE_CMDS.len()
        )
    };
    if !reclaimed.is_empty() {
        note.push_str(&format!(" (reclaimed {})", reclaimed.join(" + ")));
    }
    if !trimmed {
        note.push_str("; fstrim failed (may reclaim less)");
    }
    Ok(note)
}

/// Prune Docker (and `fstrim` its VM) without stopping Docker or compacting
/// any disk: no UAC prompt, no restart.
pub fn run_prune<R: Runner>(runner: &mut R, progress: &SharedProgress) {
    match prune_docker(runner, progress) {
        Ok(note) => finish(
            progress,
            Ok(format!(
                "{note}. The .vhdx file only shrinks after a compact (select it and press z)."
            )),
        ),
        Err(e) => finish(progress, Err(format!("prune failed: {e}"))),
    }
}

/// [`run_prune`] on a background thread.
pub fn spawn_prune() -> SharedProgress {
    let progress = new_titled("Docker prune");
    let worker = progress.clone();
    std::thread::Builder::new()
        .name("xdu-docker-prune".to_string())
        .spawn(move || run_prune(&mut SystemRunner, &worker))
        .map(|_| ())
        .unwrap_or_else(|e| finish(&progress, Err(format!("cannot start worker: {e}"))));
    progress
}

/// Execute the full sequence, reporting through `progress`. `size_of` returns
/// the on-disk size of the VHDX (injected for tests).
pub fn run_shrink<R: Runner>(
    vhdx: &Path,
    runner: &mut R,
    size_of: impl Fn(&Path) -> Option<u64>,
    progress: &SharedProgress,
) {
    if !is_vhdx(vhdx) {
        finish(
            progress,
            Err(format!("{} is not a .vhdx file", vhdx.display())),
        );
        return;
    }

    let prune_note = if is_docker_disk(vhdx) {
        prune_docker(runner, progress).unwrap_or_else(|e| format!("prune skipped ({e})"))
    } else {
        "no Docker prune (not a Docker disk)".to_string()
    };

    let before = size_of(vhdx);
    let was_running = docker_desktop_running(runner);
    if was_running {
        set_step(progress, "stopping Docker Desktop");
        for image in DOCKER_PROCESSES {
            let _ = runner.run("taskkill", &args(&["/F", "/T", "/IM", image]));
        }
    }

    set_step(progress, "shutting down WSL");
    let wsl = runner.run("wsl", &args(&["--shutdown"]));
    if !wsl.ok {
        if was_running {
            start_docker_desktop(runner);
        }
        finish(
            progress,
            Err(format!("wsl --shutdown failed: {}", wsl.text)),
        );
        return;
    }
    // Give the VM a moment to release the disk file.
    runner.sleep(Duration::from_secs(8));

    set_step(
        progress,
        "compacting the VHDX (approve the UAC prompt if shown)",
    );
    let script_path = std::env::temp_dir().join(format!("xdu-compact-{}.ps1", std::process::id()));
    // UTF-8 BOM so Windows PowerShell 5.1 reads non-ASCII paths correctly.
    let body = format!("\u{feff}{}", compact_script(vhdx));
    let compact = match std::fs::write(&script_path, body) {
        Ok(()) => runner.run("powershell", &elevated_script_args(&script_path)),
        Err(e) => CmdOutput {
            ok: false,
            text: format!("cannot write compact script: {e}"),
        },
    };
    let _ = std::fs::remove_file(&script_path);

    if was_running {
        set_step(progress, "restarting Docker Desktop");
        start_docker_desktop(runner);
    }

    if !compact.ok {
        finish(
            progress,
            Err(format!(
                "compaction failed or UAC was declined ({}). {prune_note}",
                compact.text
            )),
        );
        return;
    }

    let after = size_of(vhdx);
    let summary = match (before, after) {
        (Some(b), Some(a)) if a < b => format!(
            "{} → {} (freed {}). {prune_note}. Press r to rescan.",
            format_size(b, SizeMode::Iec),
            format_size(a, SizeMode::Iec),
            format_size(b - a, SizeMode::Iec)
        ),
        (Some(b), Some(_)) => format!(
            "VHDX did not shrink (still {}); it may already be compact, or the compaction was refused. {prune_note}",
            format_size(b, SizeMode::Iec)
        ),
        _ => format!("compaction finished. {prune_note}. Press r to rescan."),
    };
    finish(progress, Ok(summary));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Fake {
        calls: Vec<String>,
        desktop_running: bool,
        fail: Vec<&'static str>,
    }

    impl Fake {
        fn new(desktop_running: bool) -> Self {
            Self {
                calls: Vec::new(),
                desktop_running,
                fail: Vec::new(),
            }
        }
    }

    impl Runner for Fake {
        fn run(&mut self, program: &str, a: &[String]) -> CmdOutput {
            let line = format!("{program} {}", a.join(" "));
            self.calls.push(line.clone());
            let ok = !self.fail.iter().any(|f| line.starts_with(f));
            let text = if program == "tasklist" && self.desktop_running {
                "Docker Desktop.exe   1234 Console".to_string()
            } else {
                String::new()
            };
            CmdOutput { ok, text }
        }
        fn sleep(&mut self, _d: Duration) {}
    }

    fn docker_vhdx() -> PathBuf {
        PathBuf::from(r"C:\Users\x\AppData\Local\Docker\wsl\disk\docker_data.vhdx")
    }

    fn ubuntu_vhdx() -> PathBuf {
        PathBuf::from(r"C:\Users\x\AppData\Local\Packages\Ubuntu\LocalState\ext4.vhdx")
    }

    fn assert_in_order(calls: &[String], order: &[&str]) {
        let mut at = 0;
        for want in order {
            let idx = calls[at..]
                .iter()
                .position(|c| c.starts_with(want))
                .unwrap_or_else(|| panic!("missing {want} in {calls:?}"));
            at += idx + 1;
        }
    }

    #[test]
    fn is_vhdx_matches_extension_case_insensitively() {
        assert!(is_vhdx(Path::new("a/docker_data.vhdx")));
        assert!(is_vhdx(Path::new("A.VHDX")));
        assert!(!is_vhdx(Path::new("a/b.vhd")));
        assert!(!is_vhdx(Path::new("a/vhdx")));
    }

    #[test]
    fn is_docker_disk_detects_docker_paths_only() {
        assert!(is_docker_disk(&docker_vhdx()));
        assert!(!is_docker_disk(&ubuntu_vhdx()));
    }

    #[test]
    fn compact_script_prefers_optimize_vhd_with_diskpart_fallback() {
        let s = compact_script(Path::new(r"\\?\C:\d\it's.vhdx"));
        assert!(s.contains(r"$p = 'C:\d\it''s.vhdx'"));
        let order: Vec<usize> = [
            "Optimize-VHD -Path $p -Mode Full",
            "attach vdisk readonly",
            "compact vdisk",
            "detach vdisk",
        ]
        .iter()
        .map(|c| s.find(c).unwrap())
        .collect();
        assert!(order.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn elevated_args_use_runas_wait_and_propagate_exit_code() {
        let a = elevated_script_args(Path::new(r"C:\it's\s.ps1"));
        let cmd = a.last().unwrap();
        assert!(cmd.contains("-Verb RunAs"));
        assert!(cmd.contains("-Wait"));
        assert!(cmd.contains("exit $p.ExitCode"));
        assert!(cmd.contains("it''s"));
    }

    #[test]
    fn docker_disk_runs_full_sequence_in_order_and_reports_freed_space() {
        let mut r = Fake::new(true);
        let p = new_progress();
        let sizes = Cell::new(0);
        run_shrink(
            &docker_vhdx(),
            &mut r,
            |_| {
                let n = sizes.get();
                sizes.set(n + 1);
                Some(if n == 0 { 50 << 30 } else { 20 << 30 })
            },
            &p,
        );
        assert_in_order(
            &r.calls,
            &[
                "docker info",
                "docker container prune -f",
                "docker image prune -a -f",
                "docker builder prune -a -f",
                "docker volume prune -a -f",
                "wsl -d docker-desktop sh -c fstrim -av",
                "tasklist",
                "taskkill /F /T /IM Docker Desktop.exe",
                "wsl --shutdown",
                "powershell",
                "docker desktop start",
            ],
        );
        let g = p.lock().unwrap();
        assert!(g.done);
        let msg = g.result.as_ref().unwrap().as_ref().unwrap();
        assert!(msg.contains("freed"), "{msg}");
        assert!(msg.contains("Docker pruned"), "{msg}");
    }

    #[test]
    fn non_docker_vhdx_skips_prune_and_fstrim() {
        let mut r = Fake::new(false);
        let p = new_progress();
        run_shrink(&ubuntu_vhdx(), &mut r, |_| Some(1), &p);
        assert!(!r.calls.iter().any(|c| c.starts_with("docker ")));
        assert!(!r.calls.iter().any(|c| c.contains("fstrim")));
        assert_in_order(&r.calls, &["wsl --shutdown", "powershell"]);
    }

    #[test]
    fn docker_not_running_skips_stop_and_restart() {
        let mut r = Fake::new(false);
        let p = new_progress();
        run_shrink(&docker_vhdx(), &mut r, |_| Some(1), &p);
        assert!(!r.calls.iter().any(|c| c.starts_with("taskkill")));
        assert!(!r.calls.iter().any(|c| c.starts_with("docker desktop")));
        assert!(r.calls.iter().any(|c| c.starts_with("wsl --shutdown")));
    }

    #[test]
    fn declined_uac_reports_error_and_still_restarts_docker() {
        let mut r = Fake::new(true);
        r.fail.push("powershell");
        let p = new_progress();
        run_shrink(&docker_vhdx(), &mut r, |_| Some(1), &p);
        assert!(p.lock().unwrap().result.as_ref().unwrap().is_err());
        assert!(r
            .calls
            .iter()
            .any(|c| c.starts_with("docker desktop start")));
    }

    #[test]
    fn wsl_failure_aborts_before_compaction() {
        let mut r = Fake::new(true);
        r.fail.push("wsl --shutdown");
        let p = new_progress();
        run_shrink(&docker_vhdx(), &mut r, |_| Some(1), &p);
        assert!(!r.calls.iter().any(|c| c.starts_with("powershell")));
        assert!(p.lock().unwrap().result.as_ref().unwrap().is_err());
    }

    #[test]
    fn unreachable_daemon_skips_prune_but_still_compacts() {
        let mut r = Fake::new(false);
        r.fail.push("docker info");
        let p = new_progress();
        run_shrink(&docker_vhdx(), &mut r, |_| Some(1), &p);
        assert!(!r.calls.iter().any(|c| c.contains("prune")));
        assert!(r.calls.iter().any(|c| c.starts_with("powershell")));
        let g = p.lock().unwrap();
        let msg = g.result.as_ref().unwrap().as_ref().unwrap();
        assert!(msg.contains("prune skipped"), "{msg}");
    }

    #[test]
    fn prune_only_prunes_and_trims_without_stopping_or_compacting() {
        let mut r = Fake::new(true);
        let p = new_progress();
        run_prune(&mut r, &p);
        assert_in_order(
            &r.calls,
            &[
                "docker info",
                "docker container prune -f",
                "docker image prune -a -f",
                "docker builder prune -a -f",
                "docker volume prune -a -f",
                "wsl -d docker-desktop sh -c fstrim -av",
            ],
        );
        for forbidden in ["taskkill", "wsl --shutdown", "powershell", "docker desktop"] {
            assert!(
                !r.calls.iter().any(|c| c.starts_with(forbidden)),
                "{forbidden} must not run: {:?}",
                r.calls
            );
        }
        let g = p.lock().unwrap();
        assert!(g.done);
        assert!(g.result.as_ref().unwrap().is_ok());
    }

    #[test]
    fn prune_only_reports_error_when_daemon_unreachable() {
        let mut r = Fake::new(false);
        r.fail.push("docker info");
        let p = new_progress();
        run_prune(&mut r, &p);
        assert!(!r.calls.iter().any(|c| c.contains("prune")));
        assert!(p.lock().unwrap().result.as_ref().unwrap().is_err());
    }

    #[test]
    fn non_vhdx_is_rejected_without_running_anything() {
        let mut r = Fake::new(true);
        let p = new_progress();
        run_shrink(Path::new("C:\\x\\file.txt"), &mut r, |_| None, &p);
        assert!(r.calls.is_empty());
        assert!(p.lock().unwrap().result.as_ref().unwrap().is_err());
    }
}
