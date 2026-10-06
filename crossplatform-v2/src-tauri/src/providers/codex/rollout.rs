//! The fallback: rate limits Codex wrote into the thread's rollout log, lines like
//! `{"timestamp":"…","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":0.0,
//! "window_minutes":300,"resets_at":1790585719},"secondary":…,"plan_type":"free"}}}`.
//! The newest file is found by walking the dated directories newest-first, no SQLite involved.

use super::parse::{label_for, num};
use crate::providers::LimitWindow;
use crate::support::time::parse_iso;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const TAIL_BYTES: u64 = 256 * 1024;

pub struct Snapshot {
    pub windows: Vec<LimitWindow>,
    pub recorded_at: Option<u64>,
    pub plan: Option<String>,
}

fn list_dirs_desc(p: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> =
        std::fs::read_dir(p).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect()).unwrap_or_default();
    v.sort_by(|a, b| b.cmp(a));
    v
}

/// The most recently modified rollout among the three most recent days that have a directory.
pub fn newest(sessions: &Path) -> Option<PathBuf> {
    let days: Vec<PathBuf> = list_dirs_desc(sessions)
        .into_iter()
        .flat_map(|y| list_dirs_desc(&y))
        .flat_map(|m| list_dirs_desc(&m))
        .take(3)
        .collect();
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for e in days.iter().flat_map(|d| std::fs::read_dir(d).into_iter().flatten().flatten()) {
        let name = e.file_name().to_string_lossy().to_string();
        if !(name.starts_with("rollout-") && name.ends_with(".jsonl")) {
            continue;
        }
        let Some(mt) = e.metadata().ok().and_then(|m| m.modified().ok()) else { continue };
        if best.as_ref().map(|(t, _)| mt > *t).unwrap_or(true) {
            best = Some((mt, e.path()));
        }
    }
    best.map(|(_, p)| p)
}

pub fn tail_text(path: &Path) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = f.seek(SeekFrom::Start(len.saturating_sub(TAIL_BYTES)));
    let mut raw = Vec::new();
    f.read_to_end(&mut raw).ok()?;
    Some(String::from_utf8_lossy(&raw).into_owned())
}

/// The last rate_limits entry with at least one usable window. A snapshot with only null windows
/// is skipped: it must not become a fabricated zero ring.
pub fn snapshot(text: &str, now: u64) -> Option<Snapshot> {
    for line in text.lines().rev().filter(|l| l.contains("rate_limits")) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        let Some(rl) = v.get("rate_limits").or_else(|| v.pointer("/payload/rate_limits")).filter(|x| x.is_object()) else {
            continue;
        };
        let mut windows = Vec::new();
        for id in ["primary", "secondary"] {
            let Some(w) = rl.get(id).filter(|x| x.is_object()) else { continue };
            let Some(pct) = num(w.get("used_percent")) else { continue };
            // resets_at is absolute seconds; the documented resets_in_seconds is accepted too
            let resets_at = num(w.get("resets_at"))
                .map(|s| (s * 1000.0) as u64)
                .or_else(|| num(w.get("resets_in_seconds")).map(|s| now + (s * 1000.0) as u64));
            windows.push(LimitWindow {
                id: id.into(),
                label: label_for(num(w.get("window_minutes")), id),
                used: (pct / 100.0).clamp(0.0, 1.0),
                resets_at,
                ..Default::default()
            });
        }
        if windows.is_empty() {
            continue;
        }
        return Some(Snapshot {
            windows,
            recorded_at: parse_iso(v.get("timestamp")),
            plan: rl.get("plan_type").and_then(|x| x.as_str()).map(String::from),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROLLOUT: &str = include_str!("../../../tests/fixtures/vendors/codex-rollout.jsonl");

    #[test]
    fn recorded_rollout_parses_with_its_own_timestamp() {
        let s = snapshot(ROLLOUT, 0).expect("fixture has a snapshot");
        assert_eq!(s.windows.len(), 1);
        assert_eq!((s.windows[0].id.as_str(), s.windows[0].label.as_str()), ("primary", "5h limit"));
        assert!((s.windows[0].used - 0.42).abs() < 1e-9);
        assert_eq!(s.windows[0].resets_at, Some(1_893_456_000_000));
        assert_eq!(s.plan.as_deref(), Some("free"));
        assert!(s.recorded_at.unwrap() > 0);
    }

    #[test]
    fn null_windows_mean_nothing_metered() {
        let line = r#"{"timestamp":"2026-02-23T20:45:18.178Z","type":"event_msg","payload":{"type":"token_count","info":null,"rate_limits":{"limit_id":"codex","primary":null,"secondary":null,"plan_type":null}}}"#;
        assert!(snapshot(line, 0).is_none());
    }

    #[test]
    fn newest_picks_the_latest_file_in_the_latest_days() {
        let dir = tempfile::tempdir().unwrap();
        let day = |d: &str| {
            let p = dir.path().join(d);
            std::fs::create_dir_all(&p).unwrap();
            p
        };
        std::fs::write(day("2026/09/30").join("rollout-a.jsonl"), "").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(day("2026/10/01").join("rollout-b.jsonl"), "").unwrap();
        std::fs::write(day("2026/10/01").join("notes.txt"), "").unwrap();
        assert_eq!(newest(dir.path()).unwrap().file_name().unwrap(), "rollout-b.jsonl");
    }
}
