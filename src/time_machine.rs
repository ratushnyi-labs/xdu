//! Reclaim disk on macOS by deleting Time Machine local snapshots.
//!
//! Time Machine keeps hourly APFS snapshots of the boot volume while a backup
//! destination is unavailable (and for up to 24 h otherwise). They hold on to
//! deleted data, show up as "System Data" / purgeable space and are invisible
//! to a directory scan. `tmutil listlocalsnapshots /` enumerates them and
//! `tmutil deletelocalsnapshots <date>` drops one.
//!
//! Backups stored on the Time Machine *destination* disk are not touched.
//!
//! Like [`crate::docker_shrink`], the orchestration is written against the
//! [`Runner`] trait so it is testable without macOS; only [`SystemRunner`]
//! spawns processes.

use crate::docker_shrink::{finish, new_titled, set_step, Runner, SharedProgress, SystemRunner};

const SNAPSHOT_PREFIX: &str = "com.apple.TimeMachine.";
const SNAPSHOT_SUFFIX: &str = ".local";

/// `tmutil` only exists on macOS.
pub fn is_supported() -> bool {
    cfg!(target_os = "macos")
}

/// Extract snapshot dates (`2024-01-15-123456`) from `tmutil listlocalsnapshots`
/// output. Header lines and anything not shaped like a Time Machine snapshot
/// are ignored, so the result is safe to pass back to `tmutil` / a shell.
pub fn parse_snapshots(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|l| {
            let date = l
                .trim()
                .strip_prefix(SNAPSHOT_PREFIX)?
                .strip_suffix(SNAPSHOT_SUFFIX)?;
            (!date.is_empty() && date.chars().all(|c| c.is_ascii_digit() || c == '-'))
                .then(|| date.to_string())
        })
        .collect()
}

/// List local snapshots of the boot volume. `Err` carries `tmutil`'s message.
pub fn list_snapshots<R: Runner>(runner: &mut R) -> Result<Vec<String>, String> {
    let out = runner.run("tmutil", &["listlocalsnapshots".into(), "/".into()]);
    if out.ok {
        Ok(parse_snapshots(&out.text))
    } else {
        Err(out.text)
    }
}

fn needs_root(text: &str) -> bool {
    let t = text.to_lowercase();
    t.contains("root") || t.contains("privilege") || t.contains("not permitted")
}

/// Delete every local snapshot. Tries `tmutil` directly first; when macOS
/// demands root, retries once for all remaining snapshots through the system
/// administrator-password dialog (`osascript … with administrator privileges`).
pub fn run_delete<R: Runner>(runner: &mut R, progress: &SharedProgress) {
    set_step(progress, "listing local snapshots");
    let snapshots = match list_snapshots(runner) {
        Ok(s) => s,
        Err(e) => return finish(progress, Err(format!("tmutil listlocalsnapshots: {e}"))),
    };
    if snapshots.is_empty() {
        return finish(progress, Ok("no local snapshots to delete".into()));
    }

    let total = snapshots.len();
    let mut deleted = 0;
    let mut remaining = snapshots.as_slice();
    let mut last_err = String::new();
    while let Some((date, rest)) = remaining.split_first() {
        set_step(
            progress,
            &format!("deleting snapshot {}/{total} ({date})", deleted + 1),
        );
        let out = runner.run("tmutil", &["deletelocalsnapshots".into(), date.clone()]);
        if out.ok {
            deleted += 1;
            remaining = rest;
        } else if needs_root(&out.text) {
            set_step(progress, "waiting for administrator password");
            let script = format!(
                "for d in {}; do /usr/bin/tmutil deletelocalsnapshots $d || exit 1; done",
                remaining.join(" ")
            );
            let osa = format!("do shell script \"{script}\" with administrator privileges");
            let out = runner.run("osascript", &["-e".into(), osa]);
            if !out.ok {
                last_err = out.text;
            }
            break;
        } else {
            last_err = out.text;
            break;
        }
    }

    // Trust the OS, not our counters: report what is actually left.
    let left = list_snapshots(runner)
        .map(|s| s.len())
        .unwrap_or(total - deleted);
    if left == 0 {
        finish(progress, Ok(format!("deleted {total} local snapshot(s)")))
    } else {
        let why = if last_err.is_empty() {
            String::new()
        } else {
            format!(": {last_err}")
        };
        finish(
            progress,
            Err(format!("{left} of {total} snapshot(s) remain{why}")),
        )
    }
}

/// [`run_delete`] on a background thread.
pub fn spawn_delete() -> SharedProgress {
    let progress = new_titled("Time Machine snapshots");
    let worker = progress.clone();
    std::thread::Builder::new()
        .name("xdu-tm-snapshots".to_string())
        .spawn(move || run_delete(&mut SystemRunner, &worker))
        .map(|_| ())
        .unwrap_or_else(|e| finish(&progress, Err(format!("cannot start worker: {e}"))));
    progress
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docker_shrink::CmdOutput;

    const LISTING: &str = "Snapshots for volume group containing disk '/':\n\
        com.apple.TimeMachine.2024-01-15-123456.local\n\
        com.apple.TimeMachine.2024-01-15-133456.local\n\
        com.apple.os.update-ABCDEF\n";

    /// Scripted `tmutil`: holds a snapshot list that `deletelocalsnapshots`
    /// shrinks; `deny_direct` simulates the "requires root" failure.
    struct Fake {
        snapshots: Vec<String>,
        deny_direct: bool,
        calls: Vec<String>,
    }

    impl Fake {
        fn new(n: usize, deny_direct: bool) -> Self {
            Self {
                snapshots: (0..n).map(|i| format!("2024-01-15-1{i:05}")).collect(),
                deny_direct,
                calls: Vec::new(),
            }
        }
    }

    impl Runner for Fake {
        fn run(&mut self, program: &str, a: &[String]) -> CmdOutput {
            self.calls.push(format!("{program} {}", a.join(" ")));
            let ok = |text: &str| CmdOutput {
                ok: true,
                text: text.into(),
            };
            match (program, a.first().map(String::as_str)) {
                ("tmutil", Some("listlocalsnapshots")) => ok(&self
                    .snapshots
                    .iter()
                    .map(|d| format!("{SNAPSHOT_PREFIX}{d}{SNAPSHOT_SUFFIX}"))
                    .collect::<Vec<_>>()
                    .join("\n")),
                ("tmutil", Some("deletelocalsnapshots")) if self.deny_direct => CmdOutput {
                    ok: false,
                    text: "tmutil: deletelocalsnapshots requires root privileges".into(),
                },
                ("tmutil", Some("deletelocalsnapshots")) => {
                    self.snapshots.retain(|d| d != &a[1]);
                    ok("")
                }
                ("osascript", _) => {
                    self.snapshots.clear();
                    ok("")
                }
                _ => CmdOutput {
                    ok: false,
                    text: "unexpected".into(),
                },
            }
        }
    }

    fn result(p: &SharedProgress) -> Result<String, String> {
        p.lock().unwrap().result.clone().expect("job finished")
    }

    #[test]
    fn parse_keeps_only_time_machine_snapshots() {
        assert_eq!(
            parse_snapshots(LISTING),
            vec!["2024-01-15-123456", "2024-01-15-133456"]
        );
    }

    #[test]
    fn parse_rejects_names_that_could_inject_into_a_shell() {
        let out = "com.apple.TimeMachine.2024-01-15-1; rm -rf ~.local\n";
        assert!(parse_snapshots(out).is_empty());
    }

    #[test]
    fn deletes_each_snapshot_directly() {
        let mut r = Fake::new(3, false);
        let p = new_titled("t");
        run_delete(&mut r, &p);
        assert_eq!(result(&p), Ok("deleted 3 local snapshot(s)".into()));
        let deletes = r
            .calls
            .iter()
            .filter(|c| c.starts_with("tmutil deletelocalsnapshots"))
            .count();
        assert_eq!(deletes, 3);
        assert!(r.calls.iter().all(|c| !c.starts_with("osascript")));
    }

    #[test]
    fn falls_back_to_admin_dialog_when_root_required() {
        let mut r = Fake::new(2, true);
        let p = new_titled("t");
        run_delete(&mut r, &p);
        assert_eq!(result(&p), Ok("deleted 2 local snapshot(s)".into()));
        let osa: Vec<_> = r
            .calls
            .iter()
            .filter(|c| c.starts_with("osascript"))
            .collect();
        assert_eq!(osa.len(), 1, "one password prompt for all snapshots");
        assert!(osa[0].contains("with administrator privileges"));
    }

    #[test]
    fn nothing_to_delete_is_not_an_error() {
        let mut r = Fake::new(0, false);
        let p = new_titled("t");
        run_delete(&mut r, &p);
        assert_eq!(result(&p), Ok("no local snapshots to delete".into()));
    }

    #[test]
    fn reports_remaining_snapshots_when_delete_fails() {
        struct Broken(Fake);
        impl Runner for Broken {
            fn run(&mut self, program: &str, a: &[String]) -> CmdOutput {
                if a.first().map(String::as_str) == Some("deletelocalsnapshots") {
                    return CmdOutput {
                        ok: false,
                        text: "boom".into(),
                    };
                }
                self.0.run(program, a)
            }
        }
        let mut r = Broken(Fake::new(2, false));
        let p = new_titled("t");
        run_delete(&mut r, &p);
        assert_eq!(result(&p), Err("2 of 2 snapshot(s) remain: boom".into()));
    }
}
