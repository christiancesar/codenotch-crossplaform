//! Runs a CLI inside a Windows pseudo console (ConPTY), bounded by a job object that kills every
//! descendant on exit or timeout. Redirected pipes historically came back empty for Antigravity's
//! CLI on Windows; the pseudo console makes it print as it would in a terminal.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::time::Duration;

/// Quotes a command line argument according to Windows CommandLineToArgvW rules.
fn quote_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }
    if !arg.contains([' ', '\t', '\n', '\x0b', '\"']) {
        return arg.to_string();
    }
    let mut res = String::with_capacity(arg.len() + 2);
    res.push('"');
    let mut backslashes = 0;
    for c in arg.chars() {
        if c == '\\' {
            backslashes += 1;
        } else {
            for _ in 0..(if c == '"' {
                backslashes * 2 + 1
            } else {
                backslashes
            }) {
                res.push('\\');
            }
            backslashes = 0;
            res.push(c);
        }
    }
    for _ in 0..backslashes * 2 {
        res.push('\\');
    }
    res.push('"');
    res
}

/// Drains at most 64 KB of output concurrently, enforces the timeout and cleans up descendants.
pub fn run_captured(
    program: &Path,
    args: &[&str],
    cwd: Option<&Path>,
    timeout: Duration,
) -> Result<String, String> {
    use std::io::Read;
    use std::os::windows::io::FromRawHandle;
    use ::windows::core::{PCWSTR, PWSTR};
    use ::windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
    use ::windows::Win32::System::Console::{ClosePseudoConsole, CreatePseudoConsole, COORD, HPCON};
    use ::windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use ::windows::Win32::System::Pipes::CreatePipe;
    use ::windows::Win32::System::Threading::{
        CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess,
        InitializeProcThreadAttributeList, ResumeThread, UpdateProcThreadAttribute,
        WaitForSingleObject, CREATE_SUSPENDED, EXTENDED_STARTUPINFO_PRESENT,
        LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
        STARTF_USESTDHANDLES, STARTUPINFOEXW,
    };

    if !program.is_file() {
        return Err(format!("Program not found: {}", program.display()));
    }

    struct Job(HANDLE);
    impl Drop for Job {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    let job = unsafe {
        let h = CreateJobObjectW(None, PCWSTR::null())
            .map_err(|e| format!("Cannot create CLI process job: {e}"))?;
        let job = Job(h);
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        SetInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            std::mem::size_of_val(&limits) as u32,
        )
        .map_err(|e| format!("Cannot configure CLI process job limits: {e}"))?;
        job
    };

    let mut in_read = HANDLE::default();
    let mut in_write = HANDLE::default();
    let mut out_read = HANDLE::default();
    let mut out_write = HANDLE::default();

    unsafe {
        CreatePipe(&mut in_read, &mut in_write, None, 0)
            .map_err(|e| format!("CreatePipe(in) failed: {e}"))?;
        if let Err(e) = CreatePipe(&mut out_read, &mut out_write, None, 0) {
            let _ = CloseHandle(in_read);
            let _ = CloseHandle(in_write);
            return Err(format!("CreatePipe(out) failed: {e}"));
        }
    }

    let console_size = COORD { X: 160, Y: 60 };
    let hpc_res = unsafe { CreatePseudoConsole(console_size, in_read, out_write, 0) };

    unsafe {
        let _ = CloseHandle(in_read);
        let _ = CloseHandle(out_write);
    }

    let hpc = match hpc_res {
        Ok(h) => h,
        Err(e) => {
            unsafe {
                let _ = CloseHandle(in_write);
                let _ = CloseHandle(out_read);
            }
            return Err(format!("CreatePseudoConsole failed: {e}"));
        }
    };

    struct PseudoConsoleGuard(HPCON);
    impl Drop for PseudoConsoleGuard {
        fn drop(&mut self) {
            unsafe {
                ClosePseudoConsole(self.0);
            }
        }
    }
    let _input_guard = Job(in_write);
    let pty_guard = PseudoConsoleGuard(hpc);
    let mut file = unsafe { std::fs::File::from_raw_handle(out_read.0 as _) };
    let reader_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        while let Ok(n) = file.read(&mut chunk) {
            if n == 0 {
                break;
            }
            let keep = n.min((crate::platform::MAX_CAPTURE + 1).saturating_sub(buf.len()));
            buf.extend_from_slice(&chunk[..keep]);
        }
        buf
    });

    let mut attr_size = 0usize;
    let _ = unsafe {
        InitializeProcThreadAttributeList(
            LPPROC_THREAD_ATTRIBUTE_LIST(std::ptr::null_mut()),
            1,
            0,
            &mut attr_size,
        )
    };

    let mut attr_storage = vec![0u8; attr_size];
    let attr_list = LPPROC_THREAD_ATTRIBUTE_LIST(attr_storage.as_mut_ptr() as *mut _);

    struct AttrListGuard(LPPROC_THREAD_ATTRIBUTE_LIST);
    impl Drop for AttrListGuard {
        fn drop(&mut self) {
            unsafe {
                DeleteProcThreadAttributeList(self.0);
            }
        }
    }

    let _attr_guard = unsafe {
        InitializeProcThreadAttributeList(attr_list, 1, 0, &mut attr_size)
            .map_err(|e| format!("InitializeProcThreadAttributeList failed: {e}"))?;
        AttrListGuard(attr_list)
    };

    unsafe {
        UpdateProcThreadAttribute(
            attr_list,
            0,
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
            Some(hpc.0 as *const core::ffi::c_void),
            std::mem::size_of::<HPCON>(),
            None,
            None,
        )
        .map_err(|e| format!("UpdateProcThreadAttribute failed: {e}"))?;
    }

    let mut cmd_line_str = quote_arg(program.to_str().unwrap_or_default());
    let program_u16: Vec<u16> = program
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    for arg in args {
        cmd_line_str.push(' ');
        cmd_line_str.push_str(&quote_arg(arg));
    }
    let mut cmd_line_u16: Vec<u16> = cmd_line_str
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let cwd_u16: Option<Vec<u16>> = cwd.map(|p| {
        p.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    });

    let mut si_ex = STARTUPINFOEXW::default();
    si_ex.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    // Prevent inherited parent output handles from bypassing the pseudo console.
    si_ex.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    si_ex.lpAttributeList = attr_list;

    let mut proc_info = PROCESS_INFORMATION::default();

    let spawn_res = unsafe {
        CreateProcessW(
            PCWSTR(program_u16.as_ptr()),
            PWSTR(cmd_line_u16.as_mut_ptr()),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_SUSPENDED,
            None,
            cwd_u16
                .as_ref()
                .map_or(PCWSTR::null(), |v| PCWSTR(v.as_ptr())),
            &si_ex.StartupInfo,
            &mut proc_info,
        )
    };

    if let Err(e) = spawn_res {
        return Err(format!("CreateProcessW failed: {e}"));
    }

    if unsafe { AssignProcessToJobObject(job.0, proc_info.hProcess).is_err() } {
        unsafe {
            let _ = ::windows::Win32::System::Threading::TerminateProcess(proc_info.hProcess, 1);
            let _ = CloseHandle(proc_info.hThread);
            let _ = CloseHandle(proc_info.hProcess);
        }
        return Err("Cannot attach CLI process to job object".into());
    }

    let resumed = unsafe {
        let result = ResumeThread(proc_info.hThread);
        let _ = CloseHandle(proc_info.hThread);
        result != u32::MAX
    };
    let _process_guard = Job(proc_info.hProcess);
    if !resumed {
        drop(job);
        return Err("Cannot resume CLI process".into());
    }

    let started = std::time::Instant::now();
    let mut exit_code = 0u32;
    let mut timed_out = false;

    loop {
        let wait = unsafe { WaitForSingleObject(proc_info.hProcess, 100) };
        if wait == WAIT_OBJECT_0 {
            let _ = unsafe { GetExitCodeProcess(proc_info.hProcess, &mut exit_code) };
            break;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            unsafe {
                let _ = TerminateJobObject(job.0, 1);
            }
            break;
        }
    }

    // Stop our remaining descendants before closing their console.
    drop(job);
    drop(pty_guard);

    let raw_bytes = reader_thread
        .join()
        .map_err(|_| "CLI output reader thread panicked".to_string())?;

    if timed_out {
        return Err(format!("{} timed out after {timeout:?}", program.display()));
    }

    if exit_code != 0 {
        return Err(format!("{} failed with exit code {exit_code}", program.display()));
    }

    if raw_bytes.len() > crate::platform::MAX_CAPTURE {
        return Err("output is too large".into());
    }

    Ok(String::from_utf8_lossy(&raw_bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn quoted_paths_keep_backslashes() {
        assert_eq!(quote_arg(r"C:\Program Files\agy.exe"), r#""C:\Program Files\agy.exe""#);
        assert_eq!(quote_arg("path with space\\"), "\"path with space\\\\\"");
        assert_eq!(quote_arg("a\\\"b"), "\"a\\\\\\\"b\"");
    }

    fn cmd() -> Option<PathBuf> {
        let c = std::env::var_os("ComSpec").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows\System32\cmd.exe"));
        c.is_file().then_some(c)
    }

    #[test]
    fn bounded_output_and_no_deadlock() {
        let Some(cmd) = cmd() else { return };
        let out = run_captured(&cmd, &["/c", "echo hello from conpty"], None, Duration::from_secs(10)).expect("echo");
        assert!(out.contains("hello from conpty"));
        let large = run_captured(&cmd, &["/c", "for /L %i in (1,1,1000) do @echo 012345678901234567890123456789012345678901234567890123456789"], None, Duration::from_secs(15)).expect("large output");
        assert!(!large.is_empty() && large.len() <= crate::platform::MAX_CAPTURE);
    }

    #[test]
    fn timeout_terminates_the_process_tree() {
        let Some(cmd) = cmd() else { return };
        let start = std::time::Instant::now();
        let res = run_captured(&cmd, &["/c", "ping -n 10 127.0.0.1 >nul"], None, Duration::from_millis(400));
        assert!(res.unwrap_err().contains("timed out"));
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn a_missing_program_is_an_error() {
        assert!(run_captured(Path::new(r"C:\non\existent\agy.exe"), &["--print"], None, Duration::from_secs(5)).is_err());
    }
}
