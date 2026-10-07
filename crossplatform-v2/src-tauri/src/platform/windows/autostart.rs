//! Start at sign-in: the `Codenotch` value under HKCU\...\Run (per user, no administrator),
//! written with reg.exe. Name and `--silent` flag are frozen contracts.

use std::os::windows::process::CommandExt;
use std::process::Command;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const NAME: &str = "Codenotch";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn reg(args: &[&str]) -> Option<(bool, String)> {
    let o = Command::new("reg").args(args).creation_flags(CREATE_NO_WINDOW).output().ok()?;
    let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    Some((o.status.success(), text))
}

pub fn is_enabled() -> bool {
    reg(&["query", RUN_KEY, "/v", NAME]).map(|(ok, out)| ok && out.contains(NAME)).unwrap_or(false)
}

pub fn enable() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let val = format!("\"{}\" --silent", exe.display());
    match reg(&["add", RUN_KEY, "/v", NAME, "/t", "REG_SZ", "/d", &val, "/f"]) {
        Some((true, _)) => Ok("start at sign-in enabled (silent until a session appears)".into()),
        Some((false, out)) => Err(out),
        None => Err("reg.exe failed to run".into()),
    }
}

pub fn disable() -> Result<String, String> {
    if !is_enabled() {
        // Checked up front instead of parsing reg.exe's "not found", which comes back translated
        return Ok("start at sign-in was not enabled".into());
    }
    match reg(&["delete", RUN_KEY, "/v", NAME, "/f"]) {
        Some((true, _)) => Ok("start at sign-in disabled".into()),
        Some((false, out)) => Err(out),
        None => Err("reg.exe failed to run".into()),
    }
}
