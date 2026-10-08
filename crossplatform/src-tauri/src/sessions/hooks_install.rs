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

/// A missing file is an empty object; one that does not parse is an error, because writing our
/// merge over it would throw away the user's whole Claude Code configuration.
fn load(path: &Path) -> Result<Value, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(json!({})),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    if text.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str::<Value>(&text)
        .ok()
        .filter(Value::is_object)
        .ok_or_else(|| format!("{} is not a JSON object; left untouched", path.display()))
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
        // Events (and the hooks key) only we had filled are dropped rather than left as []
        hooks.retain(|_, v| {
            let Some(arr) = v.as_array() else { return true };
            let kept: Vec<Value> = arr.iter().filter(|e| !is_ours(e)).cloned().collect();
            let mine = arr.len() - kept.len();
            removed += mine;
            *v = json!(kept);
            !(mine > 0 && kept.is_empty())
        });
        if removed > 0 && hooks.is_empty() {
            root.as_object_mut().map(|o| o.remove("hooks"));
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
    backup_and_write(&path, &merged(load(&path)?, &hook))?;
    Ok(format!("wrote {} ({} events)", path.display(), WIRING.len()))
}

pub fn uninstall() -> Result<String, String> {
    let path = settings_path().ok_or("cannot find the user directory")?;
    if !path.exists() {
        return Ok("settings.json does not exist, nothing to uninstall".into());
    }
    let (root, removed) = without_ours(load(&path)?);
    backup_and_write(&path, &root)?;
    Ok(format!("removed {removed} Codenotch hook(s)"))
}

/// The user's choice from the Settings switch. Only the switch sets it: the uninstaller also runs
/// `uninstall-hooks`, and a later reinstall must wire the hooks again.
pub fn set_declined(dir: &Path, declined: bool) {
    let marker = dir.join(crate::storage::paths::HOOKS_OFF);
    let _ = if declined { std::fs::write(marker, b"") } else { std::fs::remove_file(marker) };
}

/// Run at every start: without the hooks the terminal CLI's sessions never reach the app, and
/// nothing tells the user why. Writes only when our entries are missing or point at another
/// copy of the hook (the app moved), so a normal start leaves settings.json and its backups alone.
/// None when there was nothing to do.
pub fn ensure(dir: &Path) -> Option<Result<String, String>> {
    if dir.join(crate::storage::paths::HOOKS_OFF).exists() {
        return None;
    }
    let path = settings_path()?;
    let hook = hook_exe().ok().filter(|h| h.exists())?;
    let root = match load(&path) {
        Ok(r) => r,
        Err(e) => return Some(Err(e)),
    };
    let wanted = merged(root.clone(), &hook);
    if wanted == root {
        return None;
    }
    Some(backup_and_write(&path, &wanted).map(|_| format!("hooks wired in {}", path.display())))
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
        assert!(clean["hooks"].get("PreToolUse").is_none(), "an event only we used is dropped");
    }

    #[test]
    fn uninstall_leaves_the_file_as_it_was_before_install() {
        let user = json!({"model": "opus", "hooks": {"Stop": [{"hooks": [{"type": "command", "command": "notify-send done"}]}]}});
        let (clean, _) = without_ours(merged(user.clone(), Path::new("/opt/codenotch/codenotch-hook")));
        assert_eq!(clean, user);
        let (clean, _) = without_ours(merged(json!({"model": "opus"}), Path::new("/opt/codenotch/codenotch-hook")));
        assert_eq!(clean, json!({"model": "opus"}));
    }

    #[test]
    fn wiring_twice_changes_nothing_so_a_start_does_not_rewrite() {
        let hook = Path::new("/opt/codenotch/codenotch-hook");
        let once = merged(json!({"hooks": {"Stop": [{"hooks": [{"type": "command", "command": "notify-send done"}]}]}}), hook);
        assert_eq!(merged(once.clone(), hook), once);
        assert_ne!(merged(once.clone(), Path::new("/elsewhere/codenotch-hook")), once, "a moved app is rewired");
    }

    #[test]
    fn an_unreadable_settings_file_is_never_overwritten() {
        let dir = std::env::temp_dir().join(format!("codenotch-hooks-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("settings.json");
        assert_eq!(load(&p).unwrap(), json!({}), "missing is empty");
        std::fs::write(&p, "{ \"model\": \"opus\", ").unwrap();
        assert!(load(&p).is_err());
        std::fs::write(&p, "[1, 2]").unwrap();
        assert!(load(&p).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
