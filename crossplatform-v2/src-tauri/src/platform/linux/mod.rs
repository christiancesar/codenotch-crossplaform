//! Linux: /proc, EWMH over X11 (the app runs under XWayland, see window.rs in phase 4), XDG.

mod autostart;
mod process;
mod x11;

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
        let Some(x) = x11::Ewmh::connect() else { return false };
        let windows = x.client_windows_with_pid();
        match super::terminal_window(&windows, &chain, &maps.ppid) {
            Some(w) => x.activate(w),
            None => false,
        }
    }

    fn focus_claude_desktop(&self) -> bool {
        let maps = process::proc_maps();
        let Some(x) = x11::Ewmh::connect() else { return false };
        let windows: Vec<_> = x
            .client_windows_with_pid()
            .into_iter()
            .map(|(w, pid)| (w, pid, x.area(w)))
            .collect();
        match super::claude_desktop_window(&windows, &maps.name) {
            Some(w) => x.activate(w),
            None => false,
        }
    }

    fn foreground_pid(&self) -> u32 {
        x11::Ewmh::connect().and_then(|x| x.active_window_pid()).unwrap_or(0)
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
        vec![base.to_string()]
    }
}

impl Locale for Platform {
    fn system_lang(&self) -> &'static str {
        // gettext's precedence: LC_ALL, then LC_MESSAGES, then LANG
        ["LC_ALL", "LC_MESSAGES", "LANG"]
            .into_iter()
            .filter_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty()))
            .find_map(|v| super::lang_from_locale(&v))
            .unwrap_or("en")
    }
}
