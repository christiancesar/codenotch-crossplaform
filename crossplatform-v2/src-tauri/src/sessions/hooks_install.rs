//! Merges codenotch-hook into ~/.claude/settings.json without touching the user's own hooks.
//! Our entries are recognized by "codenotch-hook" in the command (a frozen contract); a backup
//! is written first.

use crate::platform::{Executables, Platform};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// (Claude Code event, needs a matcher, event reported to the app)
const WIRING: &[(&str, bool, &str)] = &[
    ("SessionStart", false, "session_start"),
    ("UserPromptSubmit", false, "running"),
    ("PreToolUse", true, "running"),
    ("PostToolUse", true, "running"),
    ("Notification", false, "attention"),
    ("Stop", false, "done"),
    ("SessionEnd", false, "session_end"),
];

pub fn settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
}

/// The hook binary next to this executable: codenotch-hook.exe on Windows, codenotch-hook on Linux.
fn hook_exe() -> Result<PathBuf, String> {
    let dir = std::env::current_exe().map_err(|e| e.to_string())?.parent().ok_or("cannot locate the program directory")?.to_path_buf();
    let name = Platform.exe_names("codenotch-hook").into_iter().next().unwrap_or_else(|| "codenotch-hook".into());
    Ok(dir.join(name))
}

fn is_ours(entry: &Value) -> bool {
    entry["hooks"].as_array().is_some_and(|hs| {
        hs.iter().any(|h| {
            h["command"].as_str().is_some_and(|c| c.contains("codenotch-hook") || c.contains("eatbean-hook") || c.contains("pacman-hook"))
        })
    })
}

fn load(path: &Path) -> Value {
    std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).filter(Value::is_object).unwrap_or_else(|| json!({}))
}

fn backup_and_write(path: &Path, root: &Value) -> Result<(), String> {
    if path.exists() {
        let _ = std::fs::copy(path, path.with_extension(format!("json.codenotch-bak-{}", crate::support::time::now_ms() / 1000)));
    }
    let txt = serde_json::to_string_pretty(root).map_err(|e| e.to_string())?;
    crate::storage::atomic::write(path, txt.as_bytes()).map_err(|e| e.to_string())
}

pub fn is_installed() -> bool {
    settings_path().and_then(|p| std::fs::read_to_string(p).ok()).is_some_and(|t| t.contains("codenotch-hook"))
}

/// Our entries for every wired event; older entries of ours are replaced, everything else kept.
pub fn merged(mut root: Value, hook: &Path) -> Value {
    if !root["hooks"].is_object() {
        root["hooks"] = json!({});
    }
    for (event, need_matcher, internal) in WIRING {
        let mut arr: Vec<Value> = root["hooks"][*event].as_array().cloned().unwrap_or_default().into_iter().filter(|e| !is_ours(e)).collect();
        let mut entry = json!({ "hooks": [{ "type": "command", "command": format!("\"{}\" {internal}", hook.display()), "timeout": 5 }] });
        if *need_matcher {
            entry["matcher"] = json!("*");
        }
        arr.push(entry);
        root["hooks"][*event] = json!(arr);
    }
    root
}

/// Every entry of ours removed; returns how many.
pub fn without_ours(mut root: Value) -> (Value, usize) {
    let mut removed = 0;
    if let Some(hooks) = root["hooks"].as_object_mut() {
        for v in hooks.values_mut() {
            if let Some(arr) = v.as_array() {
                let kept: Vec<Value> = arr.iter().filter(|e| !is_ours(e)).cloned().collect();
                removed += arr.len() - kept.len();
                *v = json!(kept);
            }
        }
    }
    (root, removed)
}

pub fn install() -> Result<String, String> {
    let path = settings_path().ok_or("cannot find the user directory")?;
    let hook = hook_exe()?;
    if !hook.exists() {
        return Err(format!("missing {}", hook.display()));
    }
    backup_and_write(&path, &merged(load(&path), &hook))?;
    Ok(format!("wrote {} ({} events)", path.display(), WIRING.len()))
}

pub fn uninstall() -> Result<String, String> {
    let path = settings_path().ok_or("cannot find the user directory")?;
    if !path.exists() {
        return Ok("settings.json does not exist, nothing to uninstall".into());
    }
    let (root, removed) = without_ours(load(&path));
    backup_and_write(&path, &root)?;
    Ok(format!("removed {removed} Codenotch hook(s)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_keeps_foreign_hooks_and_replaces_its_own() {
        let user = json!({"model": "opus", "hooks": {"Stop": [
            {"hooks": [{"type": "command", "command": "notify-send done"}]},
            {"hooks": [{"type": "command", "command": "\"/old/codenotch-hook\" done"}]}
        ]}});
        let hook = Path::new("/opt/codenotch/codenotch-hook");
        let root = merged(user, hook);
        let stop = root["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 2);
        assert_eq!(stop[0]["hooks"][0]["command"], "notify-send done");
        assert_eq!(stop[1]["hooks"][0]["command"], "\"/opt/codenotch/codenotch-hook\" done");
        assert_eq!(root["hooks"]["PreToolUse"][0]["matcher"], "*");
        assert_eq!(root["model"], "opus");
        let (clean, removed) = without_ours(root);
        assert_eq!(removed, WIRING.len());
        assert_eq!(clean["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }
}
