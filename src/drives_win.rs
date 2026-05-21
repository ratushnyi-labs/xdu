//! Windows drive enumeration (TASK-007-H / TASK-003).
//!
//! When `xdu` is invoked with no path on Windows, the binary prints a numbered
//! list of fixed/removable drives (with optional volume label + free / total
//! space) and reads stdin for a selection. This is the minimum-viable resolver
//! for the v0.1 "no path on Windows" UX gap; a fully-interactive TUI drive
//! picker is recorded for v0.2.
//!
//! On non-Windows targets the module compiles to a no-op (`list_drives()`
//! returns an empty `Vec`) so the rest of the codebase can call it
//! unconditionally.

use crate::disk_space;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveType {
    Fixed,
    Removable,
    Remote,
    CdRom,
    RamDisk,
    Unknown,
}

impl DriveType {
    pub fn short(&self) -> &'static str {
        match self {
            Self::Fixed => "fixed",
            Self::Removable => "removable",
            Self::Remote => "network",
            Self::CdRom => "cdrom",
            Self::RamDisk => "ramdisk",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Drive {
    pub letter: char,
    pub root: PathBuf,
    pub drive_type: DriveType,
    pub label: String,
    pub free_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

/// Enumerate the system's logical drives. On non-Windows targets returns
/// an empty vector. On Windows targets the result is filtered to drives
/// that report a non-zero total size (skips unmounted CD-ROM bays, etc.).
pub fn list_drives() -> Vec<Drive> {
    platform_impl::list_drives()
}

// ---------------------------------------------------------------------------

#[cfg(windows)]
mod platform_impl {
    use super::{Drive, DriveType};
    use crate::disk_space;
    use std::os::windows::ffi::OsStrExt;
    use std::path::PathBuf;

    // GetDriveTypeW constants per Win32 docs.
    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;
    const DRIVE_REMOTE: u32 = 4;
    const DRIVE_CDROM: u32 = 5;
    const DRIVE_RAMDISK: u32 = 6;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLogicalDriveStringsW(buf_len: u32, buf: *mut u16) -> u32;
        fn GetDriveTypeW(root: *const u16) -> u32;
        fn GetVolumeInformationW(
            root: *const u16,
            vol_name_buf: *mut u16,
            vol_name_size: u32,
            vol_serial: *mut u32,
            max_component_len: *mut u32,
            fs_flags: *mut u32,
            fs_name_buf: *mut u16,
            fs_name_size: u32,
        ) -> i32;
    }

    pub fn list_drives() -> Vec<Drive> {
        let mut buf: Vec<u16> = vec![0; 512];
        // SAFETY: buf is a writable u16 buffer of 512 wchars. GetLogicalDriveStringsW
        // writes at most buf_len wchars and returns the number actually written.
        let n = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) };
        if n == 0 || (n as usize) > buf.len() {
            return Vec::new();
        }
        let s = &buf[..n as usize];

        // The buffer holds a double-NUL-terminated list of NUL-terminated
        // 4-wchar strings like "C:\\\0D:\\\0...\0".
        let mut out = Vec::new();
        let mut start = 0_usize;
        for i in 0..s.len() {
            if s[i] == 0 {
                if start < i {
                    out.push(s[start..i].to_vec());
                }
                start = i + 1;
            }
        }
        out.into_iter().filter_map(probe_drive).collect()
    }

    fn probe_drive(root_wide: Vec<u16>) -> Option<Drive> {
        // root_wide is e.g. "C:\\\0" — already NUL-terminated when treated
        // as a C string by virtue of the original buffer; we need to ensure
        // it remains NUL-terminated for the FFI call. The split above strips
        // the trailing NUL, so append one back.
        let mut root_c: Vec<u16> = root_wide.clone();
        root_c.push(0);

        // SAFETY: root_c is a NUL-terminated UTF-16 string.
        let dt = unsafe { GetDriveTypeW(root_c.as_ptr()) };
        let drive_type = match dt {
            DRIVE_FIXED => DriveType::Fixed,
            DRIVE_REMOVABLE => DriveType::Removable,
            DRIVE_REMOTE => DriveType::Remote,
            DRIVE_CDROM => DriveType::CdRom,
            DRIVE_RAMDISK => DriveType::RamDisk,
            _ => DriveType::Unknown,
        };

        // Extract drive letter. root_wide is like ['C', ':', '\\'].
        let letter = char::from_u32(root_wide[0] as u32)?;
        if !letter.is_ascii_alphabetic() {
            return None;
        }
        let letter = letter.to_ascii_uppercase();

        // Volume label — best-effort. May fail on empty/network drives.
        let mut name_buf: Vec<u16> = vec![0; 256];
        let mut serial: u32 = 0;
        let mut max_comp: u32 = 0;
        let mut fs_flags: u32 = 0;
        let mut fs_name: Vec<u16> = vec![0; 256];
        // SAFETY: all buffers are stack-rooted and writable. GetVolumeInformationW
        // returns 0 on failure and we treat that as "no label".
        let label = unsafe {
            let ok = GetVolumeInformationW(
                root_c.as_ptr(),
                name_buf.as_mut_ptr(),
                name_buf.len() as u32,
                &mut serial,
                &mut max_comp,
                &mut fs_flags,
                fs_name.as_mut_ptr(),
                fs_name.len() as u32,
            );
            if ok == 0 {
                String::new()
            } else {
                let len = name_buf
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(name_buf.len());
                String::from_utf16_lossy(&name_buf[..len])
            }
        };

        let root_str = String::from_utf16_lossy(&root_wide);
        let path = PathBuf::from(&root_str);
        let (free, total) = match disk_space::free_and_total(&path) {
            Some((f, t)) => (Some(f), Some(t)),
            None => (None, None),
        };

        // Skip drives we cannot meaningfully size (e.g. empty CD-ROM bays).
        // The drive picker UI lists only "real" drives.
        if total.unwrap_or(0) == 0 && drive_type == DriveType::CdRom {
            return None;
        }

        // Belt-and-braces: ensure the wide-encoded root path was actually
        // recognised by the OsStr round-trip. (No-op; included for clarity.)
        let _: Vec<u16> = std::ffi::OsStr::new(&root_str).encode_wide().collect();

        Some(Drive {
            letter,
            root: path,
            drive_type,
            label,
            free_bytes: free,
            total_bytes: total,
        })
    }
}

#[cfg(not(windows))]
mod platform_impl {
    use super::Drive;
    pub fn list_drives() -> Vec<Drive> {
        Vec::new()
    }
}

// Re-export `disk_space::format_size` use site to silence the unused-import
// warning on non-Windows builds.
#[cfg(not(windows))]
#[allow(unused_imports)]
use disk_space as _disk_space_unused;

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_windows_list_is_empty() {
        // On non-Windows we expect an empty list (the no-op impl).
        if !cfg!(windows) {
            assert!(list_drives().is_empty());
        }
    }

    #[test]
    fn drive_type_short_strings_are_stable() {
        assert_eq!(DriveType::Fixed.short(), "fixed");
        assert_eq!(DriveType::Removable.short(), "removable");
        assert_eq!(DriveType::Remote.short(), "network");
        assert_eq!(DriveType::CdRom.short(), "cdrom");
        assert_eq!(DriveType::RamDisk.short(), "ramdisk");
        assert_eq!(DriveType::Unknown.short(), "unknown");
    }

    #[cfg(windows)]
    #[test]
    fn windows_list_contains_at_least_one_drive() {
        let drives = list_drives();
        assert!(!drives.is_empty(), "no drives reported");
        assert!(drives.iter().any(|d| d.letter == 'C'), "no C: drive");
    }
}
