use crate::providers::LimitWindow;
use crate::support::time::parse_iso;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

const ENDPOINT: &str = "https://opencode.ai/zen/go/v1/usage";

pub enum Error {
    /// 403: the key exists but has no Go subscription
    NotGo,
    Other(String),
}

/// The `opencode-go` entry's key only; other entries in auth.json are never touched.
pub fn read_key(auth: &Path) -> Option<String> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(auth).ok()?).ok()?;
    v.get("opencode-go")?.get("key")?.as_str().map(String::from)
}

/// `usage.{rolling,weekly,monthly}`, each `{status, percent, resetsAt}`.
pub fn windows(v: &Value) -> Vec<LimitWindow> {
    [("rolling", "5 hour"), ("weekly", "Weekly"), ("monthly", "Monthly")]
        .into_iter()
        .filter_map(|(field, label)| {
            let w = v.get("usage")?.get(field)?;
            let status = w.get("status")?.as_str()?;
            let percent = w.get("percent")?.as_f64()?;
            // "rate-limited" means the server already cut this window off whatever percent it
            // reports, so the ring shows it full and agrees with the next request's outcome
            let used = if status == "rate-limited" { 1.0 } else { (percent / 100.0).clamp(0.0, 1.0) };
            Some(LimitWindow {
                id: field.into(),
                label: format!("{label} (Go)"),
                used,
                resets_at: parse_iso(w.get("resetsAt")),
                ..Default::default()
            })
        })
        .collect()
}

pub fn fetch(key: &str) -> Result<Vec<LimitWindow>, Error> {
    match ureq::get(ENDPOINT).set("Authorization", &format!("Bearer {key}")).timeout(Duration::from_secs(15)).call() {
        Ok(r) => {
            let v: Value = r.into_json().map_err(|e| Error::Other(format!("parse: {e}")))?;
            let w = windows(&v);
            if w.is_empty() {
                Err(Error::Other("unrecognized response shape".into()))
            } else {
                Ok(w)
            }
        }
        Err(ureq::Error::Status(403, _)) => Err(Error::NotGo),
        Err(ureq::Error::Status(code, _)) => Err(Error::Other(format!("HTTP {code}"))),
        Err(e) => Err(Error::Other(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real body captured with a Go subscription's key on 2026-09-15 (the key is never in it).
    #[test]
    fn recorded_body_parses_to_three_windows() {
        let body: Value = serde_json::from_str(
            r#"{"usage":{"rolling":{"status":"ok","percent":0,"resetsAt":"2026-09-16T01:16:22.384Z"},
                "weekly":{"status":"ok","percent":40,"resetsAt":"2026-09-21T00:00:00.384Z"},
                "monthly":{"status":"ok","percent":27,"resetsAt":"2026-10-07T16:51:48.384Z"}}}"#,
        )
        .unwrap();
        let w = windows(&body);
        assert_eq!(w.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), vec!["rolling", "weekly", "monthly"]);
        assert!((w[1].used - 0.40).abs() < 1e-9 && (w[2].used - 0.27).abs() < 1e-9);
        assert!(w.iter().all(|x| x.resets_at.is_some() && !x.derived && x.count.is_none()));
    }

    #[test]
    fn a_rate_limited_window_is_full_whatever_its_percent() {
        let body: Value = serde_json::from_str(r#"{"usage":{"rolling":{"status":"rate-limited","percent":83,"resetsAt":"2026-09-16T01:16:22.384Z"}}}"#).unwrap();
        assert_eq!(windows(&body)[0].used, 1.0);
        assert!(windows(&serde_json::json!({"error": {"type": "AuthError"}})).is_empty());
    }

    #[test]
    fn only_the_go_entry_is_read() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("auth.json");
        std::fs::write(&p, r#"{"google":{"type":"oauth","access":"x"},"opencode-go":{"type":"api","key":"sk-test123"}}"#).unwrap();
        assert_eq!(read_key(&p).as_deref(), Some("sk-test123"));
        std::fs::write(&p, r#"{"google":{"key":"x"}}"#).unwrap();
        assert_eq!(read_key(&p), None);
    }
}
