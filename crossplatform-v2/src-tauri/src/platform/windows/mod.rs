//! Windows: ToolHelp snapshots, top-level window enumeration, the HKCU Run key.

mod autostart;
mod process;
mod windows;

use super::{Autostart, Executables, Focus, Locale, ProcMaps, Processes};

pub struct Platform;

impl Processes for Platform {
    fn proc_maps(&self) -> ProcMaps {
        process::proc_maps()
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
