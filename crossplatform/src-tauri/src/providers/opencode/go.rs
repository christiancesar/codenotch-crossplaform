use crate::providers::LimitWindow;
use crate::support::time::parse_iso;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

const ENDPOINT: &str = "https://opencode.ai/zen/go/v1/usage";

pub enum Error {
    /// 403: the key exists but has no Go subscription
    NotGo,
    /// 401: expired, revoked or mistyped. Nothing retries until the user replaces the key, so the
    /// caller says so instead of showing a bare status code.
    KeyRejected(String),
    Other(String),
}

/// The API answers a dead key with `{"error":{"code":"api_key_expired",...}}`. Only the code is
/// read: the message is prose meant for a browser, and the code is what tells the cases apart.
fn rejected_reason(v: &Value) -> String {
    match v.pointer("/error/code").and_then(|c| c.as_str()) {
        Some("api_key_expired") => "the key expired; create a new one in the OpenCode console".into(),
        Some(code) => code.into(),
        None => "the key was rejected".into(),
    }
}

/// The `opencode-go` entry's key only; other entries in the credential file are never touched.
///
/// V2 writes `account.json`, keyed by account id with an `active` map per service. The active
/// entry is the one to use; falling back to the first Go account covers a file whose `active`
/// entry was dropped while the account itself stayed.
pub fn read_key(account: &Path) -> Option<String> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(account).ok()?).ok()?;
    let accounts = v.get("accounts")?.as_object()?;
    let active = v.get("active").and_then(|a| a.get("opencode-go")).and_then(|a| a.as_str());
    active
        .and_then(|id| accounts.get(id))
        .or_else(|| accounts.values().find(|a| a.get("serviceID").and_then(|s| s.as_str()) == Some("opencode-go")))
        .and_then(|a| a.get("credential"))
        .and_then(|c| c.get("key"))
        .and_then(|k| k.as_str())
        .map(String::from)
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
        Err(ureq::Error::Status(401, r)) => {
            let reason = r.into_json::<Value>().map(|v| rejected_reason(&v)).unwrap_or_else(|_| "the key was rejected".into());
            Err(Error::KeyRejected(reason))
        }
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
    fn a_rate_limited_window_is_full_whatever_its_percent() {        let body: Value = serde_json::from_str(r#"{"usage":{"rolling":{"status":"rate-limited","percent":83,"resetsAt":"2026-09-16T01:16:22.384Z"}}}"#).unwrap();
        assert_eq!(windows(&body)[0].used, 1.0);
        assert!(windows(&serde_json::json!({"error": {"type": "AuthError"}})).is_empty());
    }

    /// The exact body the API returned for an expired key on 2026-11-03.
    #[test]
    fn an_expired_key_is_explained_not_just_a_status_code() {
        let body: Value = serde_json::from_str(
            r#"{"error":{"type":"authentication_error","code":"api_key_expired","message":"This API key has expired."}}"#,
        )
        .unwrap();
        assert_eq!(rejected_reason(&body), "the key expired; create a new one in the OpenCode console");
        assert_eq!(rejected_reason(&serde_json::json!({"error":{"code":"api_key_revoked"}})), "api_key_revoked");
        assert_eq!(rejected_reason(&serde_json::json!({"unexpected":true})), "the key was rejected");
    }

    /// The V2 shape, with a second account that must not win.
    #[test]
    fn the_account_file_yields_the_active_go_key() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("account.json");
        std::fs::write(
            &p,
            r#"{"version":2,"accounts":{
                 "a1":{"id":"a1","serviceID":"opencode","credential":{"type":"api","key":"sk-paid"}},
                 "a2":{"id":"a2","serviceID":"opencode-go","credential":{"type":"api","key":"sk-go"}}},
               "active":{"opencode-go":"a2","opencode":"a1"}}"#,
        )
        .unwrap();
        assert_eq!(read_key(&p).as_deref(), Some("sk-go"));
    }

    /// No `active` entry left: the subscription is still in the file, so its key is used.
    #[test]
    fn a_go_account_is_found_without_an_active_entry() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("account.json");
        std::fs::write(
            &p,
            r#"{"version":2,"accounts":{"a2":{"id":"a2","serviceID":"opencode-go","credential":{"type":"api","key":"sk-go"}}}}"#,
        )
        .unwrap();
        assert_eq!(read_key(&p).as_deref(), Some("sk-go"));
        std::fs::write(&p, r#"{"version":2,"accounts":{},"active":{}}"#).unwrap();
        assert_eq!(read_key(&p), None);
    }
}
