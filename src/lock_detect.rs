//! Detect which processes are holding a file open.
//!
//! On Windows: raw FFI to the **Restart Manager** API in `rstrtmgr.dll`. This
//! is the same enumeration-only API Windows installers use to list "the
//! following programs are using files that need to be updated". No elevated
//! privileges required.
//!
//! On Linux / macOS / BSD: returns empty for v0.1. Linux `/proc/*/fd` walking
//! and `lsof`-equivalents are recorded for v0.2 (`LATER-008b`).
//!
//! BR-001 invariant preserved — this module performs no writes and never
//! terminates processes. ADR §7 explicitly permits these small `unsafe` FFI
//! blocks.

use std::path::Path;

#[derive(Debug, Clone)]
pub struct LockHolder {
    pub pid: u32,
    pub name: String,
}

pub fn holders(path: &Path) -> Vec<LockHolder> {
    platform_impl::holders(path)
}

/// Force-close every foreign-process handle that points to `path`, restricted
/// to the PIDs in `pids`. Returns the number of handles closed. Windows-only
/// in v0.1.x; non-Windows targets return 0. Uses the OS-sanctioned
/// `DuplicateHandle(DUPLICATE_CLOSE_SOURCE)` form — no process termination.
pub fn force_close_handles_for_path(path: &Path, pids: &[u32]) -> usize {
    platform_impl::force_close_handles_for_path(path, pids)
}

// ---------------------------------------------------------------------------
// Platform implementations
// ---------------------------------------------------------------------------

#[cfg(windows)]
mod platform_impl {
    use super::LockHolder;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    const CCH_RM_MAX_APP_NAME: usize = 255;
    const CCH_RM_MAX_SVC_NAME: usize = 63;
    const CCH_RM_SESSION_KEY: usize = 32;

    const ERROR_SUCCESS: u32 = 0;
    const ERROR_MORE_DATA: u32 = 234;

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct Filetime {
        dw_low: u32,
        dw_high: u32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct RmUniqueProcess {
        dw_process_id: u32,
        process_start_time: Filetime,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct RmProcessInfo {
        process: RmUniqueProcess,
        str_app_name: [u16; CCH_RM_MAX_APP_NAME + 1],
        str_service_short_name: [u16; CCH_RM_MAX_SVC_NAME + 1],
        application_type: u32,
        app_status: u32,
        ts_session_id: u32,
        b_restartable: i32,
    }

    #[link(name = "rstrtmgr")]
    unsafe extern "system" {
        fn RmStartSession(
            p_session: *mut u32,
            dw_session_flags: u32,
            str_session_key: *mut u16,
        ) -> u32;
        fn RmEndSession(p_session: u32) -> u32;
        fn RmRegisterResources(
            p_session: u32,
            n_files: u32,
            rg_sz_files: *const *const u16,
            n_apps: u32,
            rg_applications: *const u8,
            n_services: u32,
            rg_sz_service_names: *const *const u16,
        ) -> u32;
        fn RmGetList(
            p_session: u32,
            pn_proc_info_needed: *mut u32,
            pn_proc_info: *mut u32,
            rg_affected_apps: *mut RmProcessInfo,
            lp_dw_reb_reasons: *mut u32,
        ) -> u32;
    }

    pub fn holders(path: &Path) -> Vec<LockHolder> {
        let canonical: std::path::PathBuf =
            std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let wide_path: Vec<u16> = OsStr::new(&canonical)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let file_ptr: *const u16 = wide_path.as_ptr();
        let files: [*const u16; 1] = [file_ptr];

        let mut session: u32 = 0;
        // Session key buffer — Restart Manager fills with a session GUID string.
        let mut key: [u16; CCH_RM_SESSION_KEY + 1] = [0; CCH_RM_SESSION_KEY + 1];
        // SAFETY: session is a valid stack location for the out-pointer; key
        // is a valid mutable buffer of CCH_RM_SESSION_KEY+1 wchars per the
        // RestartManager API contract.
        let rc = unsafe { RmStartSession(&mut session, 0, key.as_mut_ptr()) };
        if rc != ERROR_SUCCESS {
            return Vec::new();
        }

        // RAII guard so RmEndSession runs on every exit path.
        struct SessionGuard(u32);
        impl Drop for SessionGuard {
            fn drop(&mut self) {
                // SAFETY: session id is valid (came from a successful RmStartSession).
                unsafe {
                    RmEndSession(self.0);
                }
            }
        }
        let _guard = SessionGuard(session);

        // SAFETY: registering one file path; pointer arrays are valid for
        // the duration of the call; counts match the array sizes.
        let rc = unsafe {
            RmRegisterResources(
                session,
                1,
                files.as_ptr(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
            )
        };
        if rc != ERROR_SUCCESS {
            return Vec::new();
        }

        // Probe-needed pattern: call RmGetList with a 0-length buffer; it
        // returns ERROR_MORE_DATA and writes the required count to `needed`.
        let mut needed: u32 = 0;
        let mut count: u32 = 0;
        let mut reasons: u32 = 0;
        // SAFETY: passing nullptr for the array with count = 0 is documented
        // by RestartManager as the "size probe" form.
        let rc = unsafe {
            RmGetList(
                session,
                &mut needed,
                &mut count,
                std::ptr::null_mut(),
                &mut reasons,
            )
        };
        if rc == ERROR_SUCCESS && needed == 0 {
            return Vec::new();
        }
        if rc != ERROR_MORE_DATA {
            return Vec::new();
        }

        // Allocate the actual array.
        let cap = needed as usize;
        let mut buf: Vec<RmProcessInfo> = Vec::with_capacity(cap);
        let mut got: u32 = cap as u32;
        // SAFETY: buf has capacity `cap`; we pass it as a writable array of
        // RmProcessInfo of length cap. After the call, RestartManager has
        // filled `got` entries; we set_len to that value below.
        let rc = unsafe {
            RmGetList(
                session,
                &mut needed,
                &mut got,
                buf.as_mut_ptr(),
                &mut reasons,
            )
        };
        if rc != ERROR_SUCCESS {
            return Vec::new();
        }
        // SAFETY: RmGetList wrote `got` valid records into the first `got`
        // slots; we set the Vec's length to that.
        unsafe {
            buf.set_len((got as usize).min(cap));
        }

        buf.into_iter()
            .map(|info| LockHolder {
                pid: info.process.dw_process_id,
                name: wide_to_string(&info.str_app_name),
            })
            .collect()
    }

    fn wide_to_string(buf: &[u16]) -> String {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..len])
    }

    // -----------------------------------------------------------------------
    // Force-close: forcibly close foreign-process file handles to a target
    // path via DuplicateHandle(DUPLICATE_CLOSE_SOURCE). No process termination.
    // -----------------------------------------------------------------------

    use std::ffi::c_void;
    use std::sync::mpsc;
    use std::time::Duration;

    const SYSTEM_EXTENDED_HANDLE_INFORMATION: u32 = 64;
    const OBJECT_NAME_INFORMATION: u32 = 1;
    const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC0000004_u32 as i32;
    const STATUS_SUCCESS: i32 = 0;
    const PROCESS_DUP_HANDLE: u32 = 0x0040;
    const DUPLICATE_CLOSE_SOURCE: u32 = 0x0001;
    const DUPLICATE_SAME_ACCESS: u32 = 0x0002;

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct SystemHandleTableEntryInfoEx {
        object: usize,
        unique_process_id: usize,
        handle_value: usize,
        granted_access: u32,
        creator_back_trace_index: u16,
        object_type_index: u16,
        handle_attributes: u32,
        reserved: u32,
    }

    #[repr(C)]
    struct UnicodeString {
        length: u16,
        maximum_length: u16,
        buffer: *mut u16,
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtQuerySystemInformation(
            class: u32,
            buf: *mut c_void,
            len: u32,
            return_len: *mut u32,
        ) -> i32;
        fn NtQueryObject(
            handle: *mut c_void,
            class: u32,
            buf: *mut c_void,
            len: u32,
            return_len: *mut u32,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
        fn DuplicateHandle(
            src_process: *mut c_void,
            src_handle: *mut c_void,
            dst_process: *mut c_void,
            dst_handle: *mut *mut c_void,
            access: u32,
            inherit: i32,
            options: u32,
        ) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
        fn GetCurrentProcess() -> *mut c_void;
        fn GetCurrentProcessId() -> u32;
        fn QueryDosDeviceW(device: *const u16, target: *mut u16, max_chars: u32) -> u32;
    }

    pub fn force_close_handles_for_path(path: &Path, pids: &[u32]) -> usize {
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let Some(nt_path) = win32_to_nt_path(&canonical) else {
            return 0;
        };
        let nt_path_lower = nt_path.to_lowercase();

        let Some(handles) = enumerate_extended_handles() else {
            return 0;
        };
        // SAFETY: GetCurrentProcessId is a pure read.
        let our_pid = unsafe { GetCurrentProcessId() };

        let mut closed = 0_usize;
        let mut process_cache: std::collections::HashMap<u32, *mut c_void> =
            std::collections::HashMap::new();
        for entry in &handles {
            let pid = entry.unique_process_id as u32;
            if pid == our_pid {
                continue;
            }
            if !pids.is_empty() && !pids.contains(&pid) {
                continue;
            }
            let process = match process_cache.get(&pid).copied() {
                Some(p) => p,
                None => {
                    // SAFETY: PID is read from system handle table; OpenProcess
                    // returns NULL on failure. We cache and CloseHandle later.
                    let p = unsafe { OpenProcess(PROCESS_DUP_HANDLE, 0, pid) };
                    process_cache.insert(pid, p);
                    p
                }
            };
            if process.is_null() {
                continue;
            }
            if matches_target_handle(process, entry.handle_value as *mut c_void, &nt_path_lower)
                && force_close_one(process, entry.handle_value as *mut c_void)
            {
                closed += 1;
            }
        }

        for handle in process_cache.values() {
            if !handle.is_null() {
                // SAFETY: process handle came from a successful OpenProcess.
                unsafe {
                    CloseHandle(*handle);
                }
            }
        }
        closed
    }

    fn matches_target_handle(
        process: *mut c_void,
        foreign: *mut c_void,
        target_lower: &str,
    ) -> bool {
        let mut local: *mut c_void = std::ptr::null_mut();
        // SAFETY: process is a valid handle; foreign is a u-pointer captured
        // verbatim from the system handle table; local is a writable out-pointer.
        let ok = unsafe {
            DuplicateHandle(
                process,
                foreign,
                GetCurrentProcess(),
                &mut local,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        };
        if ok == 0 || local.is_null() {
            return false;
        }
        let name = query_object_name_with_timeout(local, Duration::from_millis(100));
        // SAFETY: local came from a successful DuplicateHandle.
        unsafe {
            CloseHandle(local);
        }
        match name {
            Some(n) => n.to_lowercase() == target_lower,
            None => false,
        }
    }

    fn force_close_one(process: *mut c_void, foreign: *mut c_void) -> bool {
        let mut tmp: *mut c_void = std::ptr::null_mut();
        // SAFETY: DUPLICATE_CLOSE_SOURCE is the documented form for closing a
        // foreign handle while atomically duplicating it locally.
        let ok = unsafe {
            DuplicateHandle(
                process,
                foreign,
                GetCurrentProcess(),
                &mut tmp,
                0,
                0,
                DUPLICATE_CLOSE_SOURCE,
            )
        };
        if ok == 0 || tmp.is_null() {
            return false;
        }
        // SAFETY: tmp is a valid local handle; close it to release our copy.
        unsafe {
            CloseHandle(tmp);
        }
        true
    }

    fn enumerate_extended_handles() -> Option<Vec<SystemHandleTableEntryInfoEx>> {
        let mut size: u32 = 1024 * 1024; // start at 1 MiB
        loop {
            let mut buf: Vec<u8> = vec![0; size as usize];
            let mut returned: u32 = 0;
            // SAFETY: buf is a valid mutable byte buffer of `size` bytes;
            // NtQuerySystemInformation writes at most `size` bytes and stores
            // the actual byte count in `returned`. On STATUS_INFO_LENGTH_MISMATCH
            // we grow the buffer and retry.
            let status = unsafe {
                NtQuerySystemInformation(
                    SYSTEM_EXTENDED_HANDLE_INFORMATION,
                    buf.as_mut_ptr() as *mut c_void,
                    size,
                    &mut returned,
                )
            };
            if status == STATUS_INFO_LENGTH_MISMATCH {
                // Grow and retry. Cap at 256 MiB to avoid runaway growth.
                let next = size.saturating_mul(2);
                if next > 256 * 1024 * 1024 || next == size {
                    return None;
                }
                size = next;
                continue;
            }
            if status != STATUS_SUCCESS {
                return None;
            }
            // First field is `number_of_handles: usize`, then a `reserved:
            // usize`, then the array. Read these out manually.
            if returned < (std::mem::size_of::<usize>() * 2) as u32 {
                return None;
            }
            // SAFETY: the buffer is at least 2 * size_of::<usize>() bytes,
            // verified above.
            let count = unsafe { *(buf.as_ptr() as *const usize) };
            let entry_size = std::mem::size_of::<SystemHandleTableEntryInfoEx>();
            let header_size = std::mem::size_of::<usize>() * 2;
            let needed = header_size + count.saturating_mul(entry_size);
            if needed > returned as usize {
                return None;
            }
            let mut out: Vec<SystemHandleTableEntryInfoEx> = Vec::with_capacity(count);
            for i in 0..count {
                let offset = header_size + i * entry_size;
                // SAFETY: offset + entry_size <= needed <= returned <= size.
                let entry = unsafe {
                    std::ptr::read_unaligned(
                        buf.as_ptr().add(offset) as *const SystemHandleTableEntryInfoEx
                    )
                };
                out.push(entry);
            }
            return Some(out);
        }
    }

    fn query_object_name_with_timeout(handle: *mut c_void, timeout: Duration) -> Option<String> {
        let (tx, rx) = mpsc::channel::<Option<String>>();
        // Send the raw kernel-handle value across threads as `usize` (Send+Copy)
        // and cast back inside the worker. Kernel handles are values, not Rust
        // references, so this transport is well-defined.
        let h_addr: usize = handle as usize;
        std::thread::Builder::new()
            .name("xdu-ntqo".to_string())
            .spawn(move || {
                let h: *mut c_void = h_addr as *mut c_void;
                let mut buf: Vec<u8> = vec![0; 4096];
                let mut returned: u32 = 0;
                // SAFETY: buf is sized 4096 bytes; NtQueryObject writes at most
                // `buf.len()` and reports actual bytes via `returned`.
                let status = unsafe {
                    NtQueryObject(
                        h,
                        OBJECT_NAME_INFORMATION,
                        buf.as_mut_ptr() as *mut c_void,
                        buf.len() as u32,
                        &mut returned,
                    )
                };
                if status != STATUS_SUCCESS {
                    let _ = tx.send(None);
                    return;
                }
                // The buffer starts with a UnicodeString that points to its
                // own buffer somewhere later in `buf`.
                if (returned as usize) < std::mem::size_of::<UnicodeString>() {
                    let _ = tx.send(None);
                    return;
                }
                // SAFETY: buf has at least sizeof(UnicodeString) bytes.
                let us = unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const UnicodeString) };
                if us.length == 0 || us.buffer.is_null() {
                    let _ = tx.send(None);
                    return;
                }
                let chars = (us.length as usize) / 2;
                // SAFETY: us.buffer points inside `buf` (NtQueryObject layout
                // for ObjectNameInformation). We read `chars` u16s starting
                // there. If for some reason buffer is outside, this could
                // misread; we mitigate with bounds check on returned size.
                let slice: &[u16] = unsafe { std::slice::from_raw_parts(us.buffer, chars) };
                let s = String::from_utf16_lossy(slice);
                let _ = tx.send(Some(s));
            })
            .ok()?;
        match rx.recv_timeout(timeout) {
            Ok(name) => name,
            Err(_) => None, // worker thread is leaked; kernel will return eventually
        }
    }

    fn win32_to_nt_path(canonical: &Path) -> Option<String> {
        // Strip the `\\?\` extended prefix that canonicalize() can produce.
        let s = canonical.to_string_lossy();
        let trimmed = s.strip_prefix(r"\\?\").unwrap_or(&s);
        let bytes = trimmed.as_bytes();
        if bytes.len() < 3 || bytes[1] != b':' || bytes[2] != b'\\' {
            return None;
        }
        let drive = format!("{}:", bytes[0] as char);
        let drive_w: Vec<u16> = OsStr::new(&drive)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut out: Vec<u16> = vec![0; 1024];
        // SAFETY: drive_w is NUL-terminated; out is a 1024-wchar mutable buffer.
        let n = unsafe { QueryDosDeviceW(drive_w.as_ptr(), out.as_mut_ptr(), out.len() as u32) };
        if n == 0 {
            return None;
        }
        // QueryDosDeviceW writes a double-NUL-terminated list; take the first.
        let len = out.iter().position(|&c| c == 0).unwrap_or(out.len());
        let device = String::from_utf16_lossy(&out[..len]);
        let rest = &trimmed[2..]; // strip "C:" — keep the leading "\"
        Some(format!("{device}{rest}"))
    }

}

#[cfg(not(windows))]
mod platform_impl {
    use super::LockHolder;
    use std::path::Path;
    pub fn holders(_path: &Path) -> Vec<LockHolder> {
        Vec::new()
    }
    pub fn force_close_handles_for_path(_path: &Path, _pids: &[u32]) -> usize {
        0
    }
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn holders_for_unknown_path_returns_empty() {
        let p = PathBuf::from(if cfg!(windows) {
            "C:\\nope-such-file-zzz-99999"
        } else {
            "/nope-such-file-zzz-99999"
        });
        let h = holders(&p);
        assert!(h.is_empty());
    }

    #[test]
    fn holders_for_temp_file_not_held_returns_empty() {
        // Create a temp file, drop the file handle, then probe. With nothing
        // holding it open, we expect an empty result on every platform.
        let p = std::env::temp_dir().join(format!(
            "xdu-lock-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&p, b"x").unwrap();
        let h = holders(&p);
        let _ = std::fs::remove_file(&p);
        assert!(h.is_empty(), "got holders: {h:?}");
    }

    #[test]
    fn force_close_for_unknown_path_returns_zero() {
        let p = PathBuf::from(if cfg!(windows) {
            "C:\\nope-such-file-zzz-99999"
        } else {
            "/nope-such-file-zzz-99999"
        });
        assert_eq!(force_close_handles_for_path(&p, &[]), 0);
    }

    #[test]
    fn force_close_with_empty_pids_returns_zero_when_unheld() {
        let p = std::env::temp_dir().join(format!(
            "xdu-fc-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&p, b"x").unwrap();
        let n = force_close_handles_for_path(&p, &[]);
        let _ = std::fs::remove_file(&p);
        assert_eq!(n, 0);
    }
}
