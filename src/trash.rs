//! Move-to-trash backend (TASK-007-J / LATER-007).
//!
//! Opt-in via `--trash`. When enabled, the `d` delete action moves the target
//! into the OS-native trash instead of permanently removing it.
//!
//! - **Linux:** implements the freedesktop.org Trash specification v1.0.
//!   The trash root is `$XDG_DATA_HOME/Trash` (falling back to
//!   `$HOME/.local/share/Trash`). Each trashed item gets a unique name under
//!   `Trash/files/` plus a sibling `Trash/info/<name>.trashinfo` text file
//!   recording the original path and deletion timestamp.
//! - **Windows:** `SHFileOperationW(FO_DELETE | FOF_ALLOWUNDO | FOF_NO_UI |
//!   FOF_NOERRORUI)` — this is the documented user-facing "send to Recycle
//!   Bin" path used by Explorer itself.
//! - **macOS / BSD / others:** returns `ErrorKind::Unsupported` for v0.1.
//!
//! BR-001 is preserved: the trash move is a single explicit user action gated
//! by the same confirmation modal as permanent delete.

use std::io;
use std::path::Path;

/// Move `path` to the platform's native trash. Returns the new path of the
/// trashed item if known (Linux), or `None` on platforms where the trash
/// location is not enumerable by the calling process (Windows).
pub fn move_to_trash(path: &Path) -> io::Result<()> {
    platform_impl::move_to_trash(path)
}

/// Whether move-to-trash is implemented on the current target. The TUI uses
/// this to decide whether to surface the `--trash` flag at startup with a
/// clear "unsupported on this platform" error.
pub fn is_supported() -> bool {
    platform_impl::IS_SUPPORTED
}

// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod platform_impl {
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub const IS_SUPPORTED: bool = true;

    /// freedesktop.org Trash spec v1.0 — `$XDG_DATA_HOME/Trash` with fallback
    /// to `$HOME/.local/share/Trash`. Tests override the trash root by setting
    /// `XDG_DATA_HOME` to a writable temp directory.
    fn trash_root() -> io::Result<PathBuf> {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            if !xdg.is_empty() {
                return Ok(PathBuf::from(xdg).join("Trash"));
            }
        }
        let home = std::env::var("HOME").map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "$HOME is unset; cannot locate trash directory",
            )
        })?;
        Ok(PathBuf::from(home).join(".local/share/Trash"))
    }

    pub fn move_to_trash(path: &Path) -> io::Result<()> {
        let root = trash_root()?;
        let files = root.join("files");
        let info = root.join("info");
        std::fs::create_dir_all(&files)?;
        std::fs::create_dir_all(&info)?;

        // Compute the absolute source path before we move it — once moved,
        // canonicalize() would fail.
        let abs_source = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        let base_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid file name"))?
            .to_string();

        let (final_files_path, final_info_path, trashed_name) =
            allocate_unique_target(&files, &info, &base_name)?;

        // Move the source. `rename` is atomic on the same filesystem.
        // Cross-filesystem moves require copy+unlink; for v0.1 we report
        // ErrorKind::CrossesDevices via the underlying io::Error so the
        // caller can decide.
        std::fs::rename(path, &final_files_path)?;

        // Write the .trashinfo sidecar.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let iso = format_iso8601_utc(now);
        let info_body = format!(
            "[Trash Info]\nPath={}\nDeletionDate={iso}\n",
            url_encode_path(abs_source.to_string_lossy().as_ref())
        );
        let mut f = std::fs::File::create(&final_info_path)?;
        f.write_all(info_body.as_bytes())?;

        let _ = trashed_name; // currently unused; kept for future logging
        Ok(())
    }

    /// Pick a name that does not collide inside both `files/` and `info/`.
    /// Appends `.1`, `.2`, … to the base name until both sides are free.
    fn allocate_unique_target(
        files: &Path,
        info: &Path,
        base_name: &str,
    ) -> io::Result<(PathBuf, PathBuf, String)> {
        for n in 0..1024 {
            let candidate = if n == 0 {
                base_name.to_string()
            } else {
                format!("{base_name}.{n}")
            };
            let fp = files.join(&candidate);
            let ip = info.join(format!("{candidate}.trashinfo"));
            if !fp.exists() && !ip.exists() {
                return Ok((fp, ip, candidate));
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "trash: ran out of unique name attempts",
        ))
    }

    /// Best-effort URL-encoding of a POSIX path per the freedesktop.org Trash
    /// spec (RFC 2396, percent-encode the reserved set). We only escape the
    /// minimum set that file managers care about: '%', ' ', '#', '?', and
    /// non-ASCII bytes. The forward slash MUST remain literal because the
    /// Path= field is a slash-separated path, not a URL.
    fn url_encode_path(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for b in s.bytes() {
            let escape = matches!(b, b'%' | b' ' | b'#' | b'?' | b'\n' | b'\r' | b'\t')
                || !(0x20..0x7f).contains(&b);
            if escape {
                out.push_str(&format!("%{b:02X}"));
            } else {
                out.push(b as char);
            }
        }
        out
    }

    fn format_iso8601_utc(epoch_secs: u64) -> String {
        // Reuse the formatter from crate::format for consistency with export.
        crate::format::format_iso8601_utc(epoch_secs)
    }
}

#[cfg(windows)]
mod platform_impl {
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    pub const IS_SUPPORTED: bool = true;

    // SHFileOperationW constants (Win32 docs).
    const FO_DELETE: u32 = 0x0003;
    const FOF_SILENT: u16 = 0x0004;
    const FOF_NOCONFIRMATION: u16 = 0x0010;
    const FOF_ALLOWUNDO: u16 = 0x0040;
    const FOF_NOERRORUI: u16 = 0x0400;
    const FOF_NO_UI: u16 = FOF_SILENT | FOF_NOCONFIRMATION | FOF_NOERRORUI;

    #[repr(C)]
    struct ShFileOpStructW {
        hwnd: *mut std::ffi::c_void,
        w_func: u32,
        p_from: *const u16,
        p_to: *const u16,
        f_flags: u16,
        f_any_operations_aborted: i32,
        h_name_mappings: *mut std::ffi::c_void,
        lpsz_progress_title: *const u16,
    }

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHFileOperationW(file_op: *mut ShFileOpStructW) -> i32;
    }

    pub fn move_to_trash(path: &Path) -> io::Result<()> {
        // SHFileOperationW expects a DOUBLE-NUL-terminated list of source paths.
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        wide.push(0);
        wide.push(0);

        let mut op = ShFileOpStructW {
            hwnd: std::ptr::null_mut(),
            w_func: FO_DELETE,
            p_from: wide.as_ptr(),
            p_to: std::ptr::null(),
            f_flags: FOF_ALLOWUNDO | FOF_NO_UI,
            f_any_operations_aborted: 0,
            h_name_mappings: std::ptr::null_mut(),
            lpsz_progress_title: std::ptr::null(),
        };

        // SAFETY: `wide` is a double-NUL-terminated UTF-16 buffer kept alive
        // for the duration of the call. `op` is a valid mutable SHFILEOPSTRUCTW.
        // SHFileOperationW is a stable Win32 export.
        let rc = unsafe { SHFileOperationW(&mut op) };
        if rc != 0 {
            return Err(io::Error::other(format!(
                "SHFileOperationW failed (code {rc})"
            )));
        }
        if op.f_any_operations_aborted != 0 {
            return Err(io::Error::other("SHFileOperationW: operation aborted"));
        }
        Ok(())
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod platform_impl {
    use std::io;
    use std::path::Path;

    pub const IS_SUPPORTED: bool = false;

    pub fn move_to_trash(_path: &Path) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "trash mode is not implemented on this platform (v0.1)",
        ))
    }
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_platform_returns_error() {
        if cfg!(any(target_os = "linux", windows)) {
            assert!(is_supported());
        } else {
            assert!(!is_supported());
            let r = move_to_trash(std::path::Path::new("/tmp/nope"));
            assert!(r.is_err());
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_trash_round_trip_and_collisions() {
        // Single serialized test rather than two parallel ones — env vars in
        // Rust's test runner are process-global, so parallel tests racing on
        // XDG_DATA_HOME would interfere with each other. We exercise both
        // round-trip and collision behaviour in one sequence.
        let work = std::env::temp_dir().join(format!(
            "xdu-trash-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&work).unwrap();
        let xdg = work.join("xdg-data");
        let src = work.join("victim.txt");
        std::fs::write(&src, b"bye").unwrap();

        // SAFETY: env var mutation is sound; we contain races by keeping all
        // env-touching steps inside a single test.
        unsafe {
            std::env::set_var("XDG_DATA_HOME", &xdg);
        }
        move_to_trash(&src).expect("first move_to_trash failed");

        assert!(!src.exists(), "source should be gone");
        let trashed = xdg.join("Trash/files/victim.txt");
        assert!(
            trashed.exists(),
            "trash copy missing at {}",
            trashed.display()
        );
        let info = xdg.join("Trash/info/victim.txt.trashinfo");
        assert!(info.exists(), "trashinfo missing at {}", info.display());
        let info_body = std::fs::read_to_string(&info).unwrap();
        assert!(info_body.starts_with("[Trash Info]"), "got: {info_body}");
        assert!(info_body.contains("Path="), "got: {info_body}");
        assert!(info_body.contains("DeletionDate="), "got: {info_body}");

        // Collision: a second file with the same name lands at <name>.1.
        let src2 = work.join("victim.txt");
        std::fs::write(&src2, b"second").unwrap();
        move_to_trash(&src2).expect("second move_to_trash failed");

        assert!(xdg.join("Trash/files/victim.txt").exists());
        assert!(xdg.join("Trash/files/victim.txt.1").exists());
        assert!(xdg.join("Trash/info/victim.txt.trashinfo").exists());
        assert!(xdg.join("Trash/info/victim.txt.1.trashinfo").exists());

        unsafe {
            std::env::remove_var("XDG_DATA_HOME");
        }
        let _ = std::fs::remove_dir_all(&work);
    }
}
