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
}

#[cfg(not(windows))]
mod platform_impl {
    use super::LockHolder;
    use std::path::Path;
    pub fn holders(_path: &Path) -> Vec<LockHolder> {
        Vec::new()
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
}
