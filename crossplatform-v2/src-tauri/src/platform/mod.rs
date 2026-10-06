//! Everything that differs between Windows and Linux. The choice is made once, here; no other
//! module carries a `cfg` for an OS. Traits are small so a consumer asks only for what it uses
//! and tests can pass a fake.

use std::collections::HashMap;

pub struct ProcMaps {
    pub ppid: HashMap<u32, u32>,
    /// Lower-case executable name
    pub name: HashMap<u32, String>,
}

pub trait Processes {
    fn proc_maps(&self) -> ProcMaps;
}

pub trait Focus {
    /// Raises the terminal window hosting the Claude CLI process `claude_pid`.
    fn focus_terminal(&self, claude_pid: u32) -> bool;
    /// Raises the Claude desktop app's main window.
    fn focus_claude_desktop(&self) -> bool;
    /// Process owning the focused window, 0 when unknown.
    fn foreground_pid(&self) -> u32;
}

pub trait Autostart {
    fn is_enabled(&self) -> bool;
    fn enable(&self) -> Result<String, String>;
    fn disable(&self) -> Result<String, String>;
}

pub trait Locale {
    /// UI language for "auto": "en", "zh", "ja", "ko" or "pt".
    fn system_lang(&self) -> &'static str;
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::Platform;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::Platform;

/// `pid` and its ancestors, at most 8 levels: node, shell, terminal host...
pub fn chain_of(pid: u32, ppid: &HashMap<u32, u32>) -> Vec<u32> {
    let mut chain = vec![pid];
    let mut cur = pid;
    for _ in 0..8 {
        match ppid.get(&cur) {
            Some(&p) if p != 0 && !chain.contains(&p) => {
                chain.push(p);
                cur = p;
            }
            _ => break,
        }
    }
    chain
}

/// Whether `pid` belongs to the chain: itself on it, or its parent (the conhost case).
pub fn pid_hits_chain(pid: u32, chain: &[u32], maps: &ProcMaps) -> bool {
    chain.contains(&pid) || maps.ppid.get(&pid).map(|p| chain.contains(p)).unwrap_or(false)
}

/// Among windows `(id, owner pid)`, the one owned furthest up the chain: the real terminal host,
/// not the shell inside it.
pub fn terminal_window<W: Copy>(windows: &[(W, u32)], chain: &[u32], ppid: &HashMap<u32, u32>) -> Option<W> {
    let score = |pid: u32| {
        chain
            .iter()
            .position(|&c| c == pid)
            .or_else(|| ppid.get(&pid).and_then(|pp| chain.iter().position(|c| c == pp)))
    };
    windows
        .iter()
        .filter_map(|&(w, pid)| score(pid).map(|s| (s, w)))
        .max_by_key(|(s, _)| *s)
        .map(|(_, w)| w)
}

/// The Claude desktop app's main window: the largest window whose process name contains
/// "claude" (Codenotch itself excluded). Input is `(id, owner pid, area)`.
pub fn claude_desktop_window<W: Copy>(windows: &[(W, u32, i64)], names: &HashMap<u32, String>) -> Option<W> {
    windows
        .iter()
        .filter(|(_, pid, _)| {
            names.get(pid).map(|n| n.contains("claude") && !n.contains("codenotch")).unwrap_or(false)
        })
        .max_by_key(|(_, _, area)| *area)
        .map(|(w, _, _)| *w)
}

/// Maps a locale tag ("pt_BR.UTF-8", "zh-Hans-CN") to a supported UI language.
pub fn lang_from_locale(tag: &str) -> Option<&'static str> {
    let lower = tag.to_lowercase();
    ["zh", "ja", "ko", "pt"].into_iter().find(|l| lower.starts_with(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ppid(pairs: &[(u32, u32)]) -> HashMap<u32, u32> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn chain_stops_at_cycles_and_pid_zero() {
        let m = ppid(&[(10, 9), (9, 8), (8, 0), (5, 6), (6, 5)]);
        assert_eq!(chain_of(10, &m), vec![10, 9, 8]);
        assert_eq!(chain_of(5, &m), vec![5, 6]);
    }

    #[test]
    fn terminal_is_the_window_highest_up_the_chain() {
        // claude 10 -> shell 9 -> terminal 8; a conhost 20 whose parent is the shell
        let m = ppid(&[(10, 9), (9, 8), (20, 9)]);
        let chain = chain_of(10, &m);
        assert_eq!(terminal_window(&[("conhost", 20), ("term", 8), ("other", 99)], &chain, &m), Some("term"));
        assert_eq!(terminal_window(&[("conhost", 20)], &chain, &m), Some("conhost"));
        assert_eq!(terminal_window(&[("other", 99)], &chain, &m), None);
    }

    #[test]
    fn claude_desktop_is_the_largest_claude_window_but_never_codenotch() {
        let names: HashMap<u32, String> =
            [(1, "claude.exe"), (2, "claude.exe"), (3, "codenotch-claude"), (4, "code")]
                .into_iter()
                .map(|(p, n)| (p, n.to_string()))
                .collect();
        let wins = [("small", 1, 10), ("big", 2, 500), ("notch", 3, 9999), ("vscode", 4, 9999)];
        assert_eq!(claude_desktop_window(&wins, &names), Some("big"));
    }

    #[test]
    fn locale_tags_map_to_supported_languages() {
        assert_eq!(lang_from_locale("pt_BR.UTF-8"), Some("pt"));
        assert_eq!(lang_from_locale("zh-Hans-CN"), Some("zh"));
        assert_eq!(lang_from_locale("en_US.UTF-8"), None);
    }
}
