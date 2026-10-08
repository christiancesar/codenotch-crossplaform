//! Start at sign-in through an XDG autostart entry. v0.3 only had the Windows Run key and failed
//! on Linux; the entry carries the same `--silent` flag.

use std::path::PathBuf;

fn entry_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("autostart").join("codenotch.desktop"))
}

pub fn desktop_entry(exe: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Codenotch\nComment=Usage notch for coding assistants\nExec=\"{exe}\" --silent\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
    )
}

pub fn is_enabled() -> bool {
    entry_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|t| !t.lines().any(|l| l.trim() == "Hidden=true"))
        .unwrap_or(false)
}

pub fn enable() -> Result<String, String> {
    let path = entry_path().ok_or("cannot find the config directory")?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    crate::storage::atomic::write(&path, desktop_entry(&exe.display().to_string()).as_bytes())
        .map_err(|e| e.to_string())?;
    Ok("start at sign-in enabled (silent until a session appears)".into())
}

pub fn disable() -> Result<String, String> {
    let path = entry_path().ok_or("cannot find the config directory")?;
    match std::fs::remove_file(&path) {
        Ok(()) => Ok("start at sign-in disabled".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("start at sign-in was not enabled".into()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn entry_quotes_the_path_and_keeps_the_silent_flag() {
        let e = super::desktop_entry("/opt/My Apps/codenotch");
        assert!(e.contains("Exec=\"/opt/My Apps/codenotch\" --silent\n"));
        assert!(e.starts_with("[Desktop Entry]\n"));
    }
}
