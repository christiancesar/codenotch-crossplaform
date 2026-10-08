//! Today's model requests, counted from every install's
//! `~/.gemini/antigravity*/brain/*/.system_generated/logs/transcript.jsonl`.

use std::path::{Path, PathBuf};

/// Every install's state directory (antigravity, antigravity-ide, antigravity-cli...), not just
/// the first: switching flavour leaves the old directory behind, so the first can be empty while
/// the transcripts sit in the next.
pub fn state_roots() -> Vec<PathBuf> {
    dirs::home_dir().map(|h| state_roots_in(&h)).unwrap_or_default()
}

fn state_roots_in(home: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(home.join(".gemini")) else { return vec![] };
    let mut out: Vec<PathBuf> = rd
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("antigravity"))
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    out.sort();
    out
}

/// `(count today, latest step in epoch ms)`. `created_at` is UTC; "today" is the local day.
pub fn requests_today(roots: &[PathBuf]) -> (u64, Option<u64>) {
    requests_on(roots, chrono::Local::now().date_naive())
}

fn requests_on(roots: &[PathBuf], today: chrono::NaiveDate) -> (u64, Option<u64>) {
    use chrono::{Local, TimeZone};
    let mut count = 0u64;
    let mut latest: Option<u64> = None;
    let transcripts = roots
        .iter()
        .filter_map(|r| std::fs::read_dir(r.join("brain")).ok())
        .flat_map(|rd| rd.flatten())
        .map(|e| e.path().join(".system_generated").join("logs").join("transcript.jsonl"));
    for p in transcripts {
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        for line in text.lines().filter(|l| l.contains("\"MODEL\"")) {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
            if v.get("source").and_then(|x| x.as_str()) != Some("MODEL") {
                continue;
            }
            let Some(ms) = crate::support::time::parse_iso(v.get("created_at")) else { continue };
            latest = Some(latest.map_or(ms, |l| l.max(ms)));
            if Local.timestamp_millis_opt(ms as i64).single().map(|l| l.date_naive()) == Some(today) {
                count += 1;
            }
        }
    }
    (count, latest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flavour(home: &Path, name: &str) -> PathBuf {
        let d = home.join(".gemini").join(name);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn trajectory(root: &Path, id: &str, lines: &[String]) {
        let logs = root.join("brain").join(id).join(".system_generated").join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        std::fs::write(logs.join("transcript.jsonl"), lines.join("\n")).unwrap();
    }

    fn step(source: &str, at: chrono::DateTime<chrono::Utc>) -> String {
        format!(r#"{{"source":"{source}","created_at":"{}"}}"#, at.to_rfc3339())
    }

    fn names(roots: &[PathBuf]) -> Vec<String> {
        roots.iter().map(|p| p.file_name().unwrap().to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn every_flavour_is_found_and_nothing_else() {
        let h = tempfile::tempdir().unwrap();
        assert!(state_roots_in(h.path()).is_empty());
        for f in ["antigravity-cli", "antigravity", "antigravity-ide", "config"] {
            flavour(h.path(), f);
        }
        std::fs::write(h.path().join(".gemini/antigravity-notes.txt"), "").unwrap();
        assert_eq!(names(&state_roots_in(h.path())), ["antigravity", "antigravity-cli", "antigravity-ide"]);
    }

    #[test]
    fn requests_are_summed_across_installs_and_an_empty_one_hides_nothing() {
        let h = tempfile::tempdir().unwrap();
        let now = chrono::Utc::now();
        let old = now - chrono::TimeDelta::days(3);
        std::fs::create_dir_all(flavour(h.path(), "antigravity").join("brain")).unwrap();
        trajectory(&flavour(h.path(), "antigravity-ide"), "a", &[step("MODEL", now), step("USER", now), step("MODEL", old)]);
        trajectory(&flavour(h.path(), "antigravity-cli"), "b", &[step("MODEL", now), step("MODEL", now)]);
        let (count, latest) = requests_today(&state_roots_in(h.path()));
        assert_eq!(count, 3);
        assert_eq!(latest, Some(now.timestamp_millis() as u64));
    }
}
