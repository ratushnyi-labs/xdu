//! Free / total space lookup for the volume containing a path.
//!
//! Linux: raw FFI to `statvfs(3)`. Windows: raw FFI to `GetDiskFreeSpaceExW`.
//! Other Unix (macOS / BSDs) returns `None` in v0.1.x — recorded as a v0.2
//! follow-up in `docs/spec.md` §9. ADR §7 explicitly permits these small
//! `unsafe` FFI blocks.

use crate::format::{format_size, SizeMode};
use std::path::Path;

/// `(free_bytes_to_caller, total_bytes)` for the volume containing `path`,
/// or `None` when the syscall fails or the platform is unsupported.
pub fn free_and_total(path: &Path) -> Option<(u64, u64)> {
    platform_impl::free_and_total(path)
}

/// Render the right-side footer segment, e.g. ` free: 12.3 GiB (45%) `.
/// Returns an empty string when free space is unavailable.
pub fn format_free(path: &Path, mode: SizeMode) -> String {
    match free_and_total(path) {
        Some((free, total)) if total > 0 => {
            let pct = ((free as f64 * 100.0 / total as f64).round() as u32).min(100);
            format!(" free: {} ({pct}%) ", format_size(free, mode))
        }
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Platform implementations
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod platform_impl {
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int};
    use std::path::Path;

    // Verified against musl's `bits/statvfs.h` for 64-bit (both x86_64 and
    // aarch64). glibc 64-bit shares the same field offsets for the values
    // we read (f_frsize, f_blocks, f_bavail).
    #[repr(C)]
    struct Statvfs {
        f_bsize: u64,
        f_frsize: u64,
        f_blocks: u64,
        f_bfree: u64,
        f_bavail: u64,
        f_files: u64,
        f_ffree: u64,
        f_favail: u64,
        f_fsid: u64,
        f_flag: u64,
        f_namemax: u64,
        __spare: [u32; 6],
    }

    unsafe extern "C" {
        fn statvfs(path: *const c_char, buf: *mut Statvfs) -> c_int;
    }

    pub fn free_and_total(path: &Path) -> Option<(u64, u64)> {
        let s = path.to_str()?;
        let cstr = CString::new(s).ok()?;
        // SAFETY: `cstr.as_ptr()` is a valid NUL-terminated C string for the
        // duration of this call. `buf` is a stack-allocated, zero-initialized
        // `Statvfs` we only read from after a successful call. `statvfs` is a
        // POSIX libc function with no preconditions beyond a valid path
        // pointer and a writable buffer of the correct layout.
        let (rc, buf) = unsafe {
            let mut buf: Statvfs = std::mem::zeroed();
            let rc = statvfs(cstr.as_ptr(), &mut buf);
            (rc, buf)
        };
        if rc != 0 {
            return None;
        }
        let bsize = if buf.f_frsize > 0 {
            buf.f_frsize
        } else {
            buf.f_bsize
        };
        let total = buf.f_blocks.saturating_mul(bsize);
        let free = buf.f_bavail.saturating_mul(bsize);
        Some((free, total))
    }
}

#[cfg(windows)]
mod platform_impl {
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    pub fn free_and_total(path: &Path) -> Option<(u64, u64)> {
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        wide.push(0);
        let mut free_caller: u64 = 0;
        let mut total: u64 = 0;
        let mut free_total: u64 = 0;
        // SAFETY: `wide` is a NUL-terminated UTF-16 buffer kept alive for the
        // duration of the call. The three `&mut u64` outputs are valid stack
        // pointers. `GetDiskFreeSpaceExW` is a kernel32 export with no
        // preconditions beyond what is supplied here.
        let rc = unsafe {
            GetDiskFreeSpaceExW(wide.as_ptr(), &mut free_caller, &mut total, &mut free_total)
        };
        if rc == 0 {
            return None;
        }
        Some((free_caller, total))
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
mod platform_impl {
    use std::path::Path;
    pub fn free_and_total(_path: &Path) -> Option<(u64, u64)> {
        None
    }
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn supported() -> bool {
        cfg!(any(target_os = "linux", windows))
    }

    #[test]
    fn free_and_total_for_temp_dir_is_plausible_on_supported_platforms() {
        if !supported() {
            return;
        }
        let p: PathBuf = std::env::temp_dir();
        let r = free_and_total(&p);
        assert!(r.is_some(), "supported platform must return Some");
        let (free, total) = r.unwrap();
        assert!(total > 0, "total bytes must be > 0");
        assert!(free <= total, "free <= total");
    }

    #[test]
    fn format_free_includes_pct_and_unit_on_supported_platforms() {
        if !supported() {
            return;
        }
        let p: PathBuf = std::env::temp_dir();
        let s = format_free(&p, SizeMode::Iec);
        assert!(s.contains("free:"), "got: {s}");
        assert!(s.contains('%'), "got: {s}");
    }

    #[test]
    fn format_free_returns_empty_when_path_invalid() {
        // Path with NUL is rejected by CString::new() on Linux and produces
        // a malformed wide string on Windows; in both cases free_and_total
        // returns None and format_free returns "".
        let p = PathBuf::from("invalid\0path");
        let s = format_free(&p, SizeMode::Iec);
        assert!(s.is_empty(), "got: {s:?}");
    }
}
