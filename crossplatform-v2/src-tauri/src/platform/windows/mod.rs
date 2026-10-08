//! Windows: ToolHelp snapshots, top-level window enumeration, the HKCU Run key.

mod autostart;
mod credentials;
mod process;
mod pty;
mod window;
mod windows;

use super::{Autostart, Credentials, Executables, Focus, Input, Locale, ProcMaps, Processes, Pty, Window};

pub struct Platform;

impl Processes for Platform {
    fn proc_maps(&self) -> ProcMaps {
        process::proc_maps()
    }
    fn command_lines(&self, needle: &str) -> Vec<(u32, String)> {
        process::command_lines(needle)
    }
    fn listening_ports(&self, pid: u32) -> Vec<u16> {
        process::listening_ports(pid)
    }
    fn lower_current_thread_priority(&self) {
        use ::windows::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_BELOW_NORMAL};
        unsafe {
            let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
        }
    }
    fn io_counters(&self, pid: u32) -> Option<(u64, u64)> {
        use ::windows::Win32::Foundation::CloseHandle;
        use ::windows::Win32::System::Threading::{GetProcessIoCounters, OpenProcess, IO_COUNTERS, PROCESS_QUERY_LIMITED_INFORMATION};
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut io = IO_COUNTERS::default();
            let ok = GetProcessIoCounters(h, &mut io).is_ok();
            let _ = CloseHandle(h);
            ok.then_some((io.OtherTransferCount, io.ReadTransferCount))
        }
    }
}

impl Pty for Platform {
    fn run_captured(&self, program: &std::path::Path, args: &[&str], cwd: Option<&std::path::Path>, timeout: std::time::Duration) -> Result<String, String> {
        pty::run_captured(program, args, cwd, timeout)
    }
}

impl Credentials for Platform {
    fn read_generic(&self, target: &str) -> Option<Vec<u8>> {
        credentials::read_generic(target)
    }
}

impl Focus for Platform {
    fn focus_terminal(&self, claude_pid: u32) -> bool {
        if claude_pid == 0 {
            return false;
        }
        let maps = process::proc_maps();
        let chain = super::chain_of(claude_pid, &maps.ppid);
        let wins: Vec<_> = windows::visible_titled().into_iter().map(|(h, pid, _)| (h, pid)).collect();
        match super::terminal_window(&wins, &chain, &maps.ppid) {
            Some(h) => windows::raise(h),
            None => false,
        }
    }

    fn focus_claude_desktop(&self) -> bool {
        let maps = process::proc_maps();
        match super::claude_desktop_window(&windows::visible_titled(), &maps.name) {
            Some(h) => windows::raise(h),
            None => false,
        }
    }

    fn foreground_pid(&self) -> u32 {
        windows::foreground_pid()
    }
}

impl Autostart for Platform {
    fn is_enabled(&self) -> bool {
        autostart::is_enabled()
    }
    fn enable(&self) -> Result<String, String> {
        autostart::enable()
    }
    fn disable(&self) -> Result<String, String> {
        autostart::disable()
    }
}

impl Executables for Platform {
    fn exe_names(&self, base: &str) -> Vec<String> {
        vec![format!("{base}.exe"), format!("{base}.cmd")]
    }
    fn hide_console(&self, cmd: &mut std::process::Command) {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

impl Window for Platform {
    fn prepare_process(&self) {
        window::prepare_process();
    }
    fn no_activate(&self, w: &tauri::WebviewWindow) {
        window::no_activate(w);
    }
    /// The pointer position is always current on Windows, so the watchdog toggles click-through
    fn shapes_input(&self) -> bool {
        false
    }
    fn set_input_region(&self, _w: &tauri::WebviewWindow, _rects: Vec<[f64; 4]>) {}
    fn starts_collapsed(&self) -> bool {
        false
    }
}

impl Input for Platform {
    fn left_button_down(&self) -> bool {
        window::left_button_down()
    }
    /// Windows needs no forced collapse: leaving the window is always seen there
    fn focus_signature(&self) -> Option<(u32, u32)> {
        None
    }
}

impl Locale for Platform {
    fn system_lang(&self) -> &'static str {
        use ::windows::Win32::Globalization::GetUserDefaultLocaleName;
        let mut buf = [0u16; 85];
        let n = unsafe { GetUserDefaultLocaleName(&mut buf) };
        if n <= 0 {
            return "en";
        }
        let name = String::from_utf16_lossy(&buf[..(n as usize - 1)]);
        super::lang_from_locale(&name).unwrap_or("en")
    }
}
