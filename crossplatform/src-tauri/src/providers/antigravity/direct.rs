//! The Google credential Antigravity stores under `gemini:antigravity` (Go keyring's
//! service:user naming): `{auth_method, token:{access_token, expiry}}`, raw JSON or with a
//! `go-keyring-base64:` prefix. `:loadCodeAssist` gives the tier name; `:retrieveUserQuotaSummary`
//! answers 200 only for licensed accounts and is parsed defensively.

use crate::platform::{Credentials, Platform};
use crate::providers::LimitWindow;
use crate::support::time::{now_ms, parse_iso};
use std::time::Duration;

const LOAD_CODE_ASSIST: &str = "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
const QUOTA_SUMMARY: &str = "https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary";

pub struct Creds {
    pub access_token: String,
    pub expired: bool,
    pub auth_method: String,
}

pub fn read_credentials() -> Option<Creds> {
    decode(&Platform.read_generic("gemini:antigravity")?, now_ms())
}

fn decode(raw: &[u8], now: u64) -> Option<Creds> {
    // Some writers store the blob as UTF-16LE. ASCII in UTF-16LE is also valid UTF-8 (every other
    // byte a NUL), so a UTF-8 check alone never notices; v0.3 fell into that and read nothing.
    let utf16 = raw.len() >= 2 && raw.len() % 2 == 0 && raw.iter().skip(1).step_by(2).all(|b| *b == 0);
    let mut text = if utf16 {
        let u16s: Vec<u16> = raw.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&u16s)
    } else {
        String::from_utf8_lossy(raw).into_owned()
    };
    text = text.trim_matches('\0').trim().to_string();
    if let Some(rest) = text.strip_prefix("go-keyring-base64:") {
        text = String::from_utf8(crate::support::base64::decode(rest.trim())?).ok()?;
    }
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let access_token = v.pointer("/token/access_token")?.as_str()?.to_string();
    let expired = v
        .pointer("/token/expiry")
        .and_then(|x| x.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| (d.timestamp_millis().max(0) as u64) <= now)
        .unwrap_or(false);
    let auth_method = v.get("auth_method").and_then(|x| x.as_str()).unwrap_or("").to_string();
    Some(Creds { access_token, expired, auth_method })
}

pub enum TierError {
    Rejected,
    Other(String),
}

/// "Personal", "Pro"...; the plugin type is GEMINI, not ANTIGRAVITY.
pub fn load_tier(token: &str) -> Result<String, TierError> {
    let res = ureq::post(LOAD_CODE_ASSIST)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json")
        .timeout(Duration::from_secs(15))
        .send_string(r#"{"metadata":{"pluginType":"GEMINI"}}"#);
    match res {
        Ok(r) => {
            let v: serde_json::Value = r.into_json().map_err(|e| TierError::Other(e.to_string()))?;
            Ok(tier_name(&v))
        }
        Err(ureq::Error::Status(401 | 403, _)) => Err(TierError::Rejected),
        Err(ureq::Error::Status(code, _)) => Err(TierError::Other(format!("HTTP {code}"))),
        Err(e) => Err(TierError::Other(e.to_string())),
    }
}

fn tier_name(v: &serde_json::Value) -> String {
    v.get("currentTier")
        .or_else(|| {
            let a = v.get("allowedTiers")?.as_array()?;
            a.iter().find(|t| t.get("isDefault").and_then(|x| x.as_bool()) == Some(true)).or(a.first())
        })
        .and_then(|t| t.get("name"))
        .and_then(|x| x.as_str())
        .unwrap_or("Gemini")
        .to_string()
}

/// Licensed accounts only; a personal account gets 403, which is None, not an error.
pub fn quota(token: &str) -> Option<Vec<LimitWindow>> {
    let r = ureq::post(QUOTA_SUMMARY)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json")
        .timeout(Duration::from_secs(15))
        .send_string("{}")
        .ok()?;
    let w = quota_windows(&r.into_json().ok()?);
    (!w.is_empty()).then_some(w)
}

/// A bucket with no positive limit, or used beyond 1.5x the limit, is a reply of the wrong shape
/// and draws no ring.
fn quota_windows(v: &serde_json::Value) -> Vec<LimitWindow> {
    let grouped = v.get("quotaGroups").and_then(|g| g.as_array()).into_iter().flatten().filter_map(|g| g.get("buckets")?.as_array()).flatten();
    let flat = v.get("buckets").and_then(|b| b.as_array()).into_iter().flatten();
    grouped
        .chain(flat)
        .filter_map(|b| {
            let limit = b.get("limit")?.as_f64()?;
            let used = b.get("used")?.as_f64()?;
            if limit <= 0.0 || used < 0.0 || used > limit * 1.5 {
                return None;
            }
            let label = b.get("displayName").or_else(|| b.get("name")).and_then(|x| x.as_str()).unwrap_or("Usage").to_string();
            Some(LimitWindow {
                id: b.get("name").and_then(|x| x.as_str()).unwrap_or(&label).to_string(),
                label,
                used: (used / limit).clamp(0.0, 1.0),
                resets_at: parse_iso(b.get("resetTime")),
                ..Default::default()
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const CRED: &str = r#"{"auth_method":"consumer","token":{"access_token":"ya29.x","expiry":"2026-10-06T12:00:00+02:00"}}"#;

    #[test]
    fn decodes_plain_base64_and_utf16_blobs() {
        let before = 1_791_280_799_000;
        let c = decode(CRED.as_bytes(), before).unwrap();
        assert_eq!((c.access_token.as_str(), c.auth_method.as_str(), c.expired), ("ya29.x", "consumer", false));
        assert!(decode(CRED.as_bytes(), before + 1_000).unwrap().expired);
        let b64 = format!("go-keyring-base64:{}", crate::support::base64::encode(CRED.as_bytes()));
        assert_eq!(decode(b64.as_bytes(), 0).unwrap().access_token, "ya29.x");
        let utf16: Vec<u8> = CRED.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        assert_eq!(decode(&utf16, 0).unwrap().access_token, "ya29.x");
    }

    #[test]
    fn tier_prefers_current_then_default_then_first() {
        assert_eq!(tier_name(&json!({"currentTier": {"name": "Pro"}})), "Pro");
        assert_eq!(tier_name(&json!({"allowedTiers": [{"name": "A"}, {"name": "B", "isDefault": true}]})), "B");
        assert_eq!(tier_name(&json!({})), "Gemini");
    }

    #[test]
    fn implausible_buckets_are_dropped() {
        let v = json!({"quotaGroups": [{"buckets": [
            {"name": "daily", "displayName": "Daily", "limit": 100.0, "used": 25.0, "resetTime": "2026-10-07T00:00:00Z"},
            {"name": "zero", "limit": 0.0, "used": 1.0},
            {"name": "wild", "limit": 10.0, "used": 99.0}
        ]}], "buckets": [{"name": "flat", "limit": 4.0, "used": 1.0}]});
        let w = quota_windows(&v);
        assert_eq!(w.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), vec!["daily", "flat"]);
        assert!((w[0].used - 0.25).abs() < 1e-9 && w[0].resets_at.is_some());
    }
}
