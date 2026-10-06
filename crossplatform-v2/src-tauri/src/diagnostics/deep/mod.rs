//! `doctor deep`: which files move while the apps work, the Codex databases' structure and newest
//! rows, and the processes involved. Long strings are reported as lengths.

use crate::platform::{Platform, Processes};
use crate::support::time::now_ms;
use rusqlite::types::Value as Sql;
use std::path::{Path, PathBuf};

fn mtime_ms(p: &Path) -> Option<u64> {
    std::fs::metadata(p).ok()?.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_millis() as u64)
}

/// Files modified within `within_s` seconds, depth-limited, browser caches skipped.
fn recent_files(root: &Path, depth: usize, within_s: u64, out: &mut Vec<(u64, PathBuf)>) {
    let now = now_ms();
    for e in std::fs::read_dir(root).into_iter().flatten().flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if depth > 0 && !name.starts_with("node_modules") && !["Cache", "Code Cache", "GPUCache"].contains(&name.as_str()) {
                recent_files(&p, depth - 1, within_s, out);
            }
        } else if let Some(age) = mtime_ms(&p).map(|m| now.saturating_sub(m) / 1000).filter(|a| *a <= within_s) {
            out.push((age, p));
        }
    }
}

fn short(v: &Sql) -> String {
    match v {
        Sql::Null => "NULL".into(),
        Sql::Integer(i) => i.to_string(),
        Sql::Real(f) => format!("{f}"),
        Sql::Text(t) if t.len() > 60 => format!("<text {} chars>", t.len()),
        Sql::Text(t) => format!("{t:?}"),
        Sql::Blob(b) => format!("<blob {} bytes>", b.len()),
    }
}

/// Tables, row counts, columns, and per table the newest row by a time-like column.
pub fn dump_sqlite(path: &Path) -> String {
    let mut o = format!(
        "--- {} ({}, modified {}s ago)\n",
        path.display(),
        if path.is_file() { "present" } else { "missing" },
        now_ms().saturating_sub(mtime_ms(path).unwrap_or(0)) / 1000
    );
    let Some(conn) = crate::support::sqlite::open_live(path) else { return o };
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .and_then(|mut s| s.query_map([], |r| r.get(0)).map(|rows| rows.flatten().collect()))
        .unwrap_or_default();
    for t in tables.iter().take(25) {
        let cols: Vec<String> = conn
            .prepare(&format!("PRAGMA table_info(\"{t}\")"))
            .and_then(|mut s| s.query_map([], |r| r.get(1)).map(|rows| rows.flatten().collect()))
            .unwrap_or_default();
        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM \"{t}\""), [], |r| r.get(0)).unwrap_or(-1);
        o += &format!("  table {t} ({count} rows): {}\n", cols.join(", "));
        let timeish = cols.iter().find(|c| {
            let l = c.to_lowercase();
            ["updated", "created", "time", "recency", "modified"].iter().any(|k| l.contains(k)) || l.ends_with("_at")
        });
        if let Some(tc) = timeish {
            let row: Option<Vec<String>> = conn
                .query_row(&format!("SELECT * FROM \"{t}\" ORDER BY \"{tc}\" DESC LIMIT 1"), [], |r| {
                    Ok((0..cols.len()).map(|i| format!("{}={}", cols[i], short(&r.get::<_, Sql>(i).unwrap_or(Sql::Null)))).collect())
                })
                .ok();
            if let Some(parts) = row {
                o += &format!("    newest row (by {tc}): {}\n", parts.join(" | "));
            }
        }
    }
    o
}

/// Scalar keys only; long strings as lengths, nested values as their type.
pub fn dump_json_scalars(path: &Path) -> String {
    let mut o = format!("--- {} (modified {}s ago)\n", path.display(), now_ms().saturating_sub(mtime_ms(path).unwrap_or(0)) / 1000);
    let Some(v) = std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()) else {
        o += "  unreadable or not JSON\n";
        return o;
    };
    fn walk(v: &serde_json::Value, prefix: &str, depth: usize, o: &mut String) {
        for (k, x) in v.as_object().into_iter().flatten().take(60) {
            let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
            match x {
                serde_json::Value::Object(_) if depth < 2 => walk(x, &key, depth + 1, o),
                serde_json::Value::Object(m) => *o += &format!("  {key}: <object {} keys>\n", m.len()),
                serde_json::Value::Array(a) => *o += &format!("  {key}: <array {}>\n", a.len()),
                serde_json::Value::String(s) if s.len() > 40 => *o += &format!("  {key}: <string {} chars>\n", s.len()),
                other => *o += &format!("  {key}: {other}\n"),
            }
        }
    }
    walk(&v, "", 0, &mut o);
    o
}

pub fn run() -> String {
    let mut o = String::from("== doctor deep: working-state signal survey ==\n(run it while the Codex and Claude desktop apps are working)\n\n");
    let home = dirs::home_dir().unwrap_or_default();
    let config = dirs::config_dir().unwrap_or_default();

    o += "## Files modified in the last 120 s\n";
    let mut recent = Vec::new();
    recent_files(&home.join(".codex"), 2, 120, &mut recent);
    for e in std::fs::read_dir(dirs::data_local_dir().unwrap_or_default().join("Packages")).into_iter().flatten().flatten() {
        let n = e.file_name().to_string_lossy().to_lowercase();
        if n.contains("claude") || n.contains("anthropic") {
            recent_files(&e.path().join("LocalCache").join("Roaming").join("Claude"), 3, 120, &mut recent);
        }
    }
    recent_files(&config.join("Claude"), 2, 120, &mut recent);
    recent_files(&config.join("Cursor").join("User").join("globalStorage"), 1, 120, &mut recent);
    recent.sort();
    for (age, p) in recent.iter().take(60) {
        o += &format!("  {age:>4}s ago  {}\n", p.display());
    }
    if recent.is_empty() {
        o += "  (none)\n";
    }

    o += "\n## Codex SQLite databases\n";
    for rel in ["state_5.sqlite", "thread_history_1.sqlite", "sqlite/codex-dev.db", "goals_1.sqlite", "queue_1.sqlite"] {
        o += &dump_sqlite(&home.join(".codex").join(rel));
    }
    o += "\n## Codex global state JSON (scalar keys only)\n";
    o += &dump_json_scalars(&home.join(".codex").join(".codex-global-state.json"));

    o += "\n## Codex processes (first 160 characters of the command line)\n";
    for needle in ["codex", "ChatGPT"] {
        for (pid, cmd) in Platform.command_lines(needle) {
            o += &format!("  {pid}  {}\n", cmd.chars().take(160).collect::<String>());
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_dump_shows_structure_and_hides_long_text() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.sqlite");
        let long = "x".repeat(200);
        rusqlite::Connection::open(&p).unwrap().execute_batch(&format!("CREATE TABLE threads (id TEXT, title TEXT, updated_at INTEGER); INSERT INTO threads VALUES ('t1', '{long}', 5);")).unwrap();
        let o = dump_sqlite(&p);
        assert!(o.contains("table threads (1 rows): id, title, updated_at"));
        assert!(o.contains("title=<text 200 chars>") && !o.contains(&long));
    }

    #[test]
    fn json_dump_prints_scalars_and_shapes_only() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("s.json");
        std::fs::write(&p, format!(r#"{{"a": 1, "token": "{}", "list": [1,2], "nested": {{"b": true}}}}"#, "s".repeat(80))).unwrap();
        let o = dump_json_scalars(&p);
        assert!(o.contains("a: 1") && o.contains("token: <string 80 chars>") && o.contains("list: <array 2>") && o.contains("nested.b: true"));
    }
}
