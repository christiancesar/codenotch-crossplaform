use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

pub fn config(dir: &Path) -> String {
    let r = crate::config::load(dir);
    if r.missing {
        return format!("{} not found: defaults (it is written on first start)", dir.join("config.json").display());
    }
    format!("port={} lang={} tray_mode={} ({})", r.config.port, r.config.lang, r.config.tray_mode.as_str(), dir.join("config.json").display())
}

pub fn port(dir: &Path) -> String {
    let port = crate::config::load(dir).config.port;
    match std::net::TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => format!("{port} free: no Codenotch instance is running"),
        Err(_) => format!("{port} in use: an instance is already running (quit it from the tray before starting a new build)"),
    }
}

fn collect(dir: &Path, depth: usize, out: &mut Vec<(PathBuf, SystemTime)>) {
    if depth > 10 {
        return;
    }
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, depth + 1, out);
        } else if crate::sessions::watcher::is_session_jsonl(&p) {
            if let Ok(t) = e.metadata().and_then(|m| m.modified()) {
                out.push((p, t));
            }
        }
    }
}

/// Each watch root, its five newest transcripts, and whether their tail parses.
pub fn sessions() -> String {
    let mut o = String::new();
    for root in crate::sessions::watcher::roots() {
        if !root.exists() {
            o += &format!("{} [missing]\n", root.display());
            continue;
        }
        o += &format!("{}\n", root.display());
        let mut files = Vec::new();
        collect(&root, 0, &mut files);
        files.sort_by_key(|(_, m)| std::cmp::Reverse(*m));
        if files.is_empty() {
            o += "  (no session transcripts)\n";
        }
        for (p, m) in files.into_iter().take(5) {
            let age = SystemTime::now().duration_since(m).map(|d| d.as_secs()).unwrap_or(0);
            o += &format!("  updated {age}s ago  {}\n", p.display());
            o += &match crate::sessions::watcher::tail_info(&p) {
                Some(t) => format!(
                    "    tail parses: type={} sessionId={}\n",
                    t.entry.get("type").and_then(|x| x.as_str()).unwrap_or("?"),
                    t.entry.get("sessionId").and_then(|x| x.as_str()).unwrap_or("(missing, the file name is used)")
                ),
                None => "    tail does not parse (no valid JSON near the end; please report this file)\n".into(),
            };
        }
    }
    o
}

/// Iterates the registry, so a new provider shows up here without editing diagnostics.
pub fn providers(all: &[Arc<dyn crate::providers::UsageProvider>]) -> String {
    all.iter().map(|p| format!("{}: {}\n", p.id().as_str(), p.probe())).collect()
}

pub fn watch_log(dir: &Path) -> String {
    match std::fs::read_to_string(dir.join("watch.log")) {
        Ok(t) if !t.trim().is_empty() => {
            let lines: Vec<&str> = t.lines().collect();
            lines[lines.len().saturating_sub(20)..].join("\n")
        }
        _ => "(empty: the app has not run yet, which is normal on first use)".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_and_port_read_from_the_given_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert!(config(dir.path()).contains("not found"));
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        std::fs::write(dir.path().join("config.json"), format!(r#"{{"port": {port}, "tray_mode": "bars"}}"#)).unwrap();
        assert!(config(dir.path()).contains("tray_mode=bars"));
        assert!(super::port(dir.path()).contains("in use"));
    }

    #[test]
    fn watch_log_keeps_the_last_twenty_lines() {
        let dir = tempfile::tempdir().unwrap();
        let text: String = (0..30).map(|i| format!("line {i}\n")).collect();
        std::fs::write(dir.path().join("watch.log"), text).unwrap();
        let out = watch_log(dir.path());
        assert!(out.starts_with("line 10") && out.ends_with("line 29"));
    }
}
