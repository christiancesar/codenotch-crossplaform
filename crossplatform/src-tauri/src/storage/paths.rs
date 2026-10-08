use std::path::PathBuf;

/// `%APPDATA%\codenotch` on Windows, `$XDG_CONFIG_HOME/codenotch` (else `~/.config/codenotch`)
/// on Linux. Frozen: codenotch-hook computes the same directory without this crate.
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("codenotch")
}

pub const CONFIG: &str = "config.json";
/// Present while the user has quit from the tray; codenotch-hook stops relaunching the app.
pub const USER_QUIT: &str = "user-quit";
pub const RUN_LOG: &str = "run.log";
/// Present while the user has switched the Claude Code hooks off; the app stops wiring them.
pub const HOOKS_OFF: &str = "hooks-off";
