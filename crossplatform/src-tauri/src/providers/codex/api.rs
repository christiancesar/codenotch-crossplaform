use super::credentials::Credential;
use std::time::Duration;

const ENDPOINT: &str = "https://chatgpt.com/backend-api/wham/usage";

pub enum Error {
    Rejected,
    RateLimited(u64),
    Other(String),
}

pub fn fetch(cred: &Credential) -> Result<serde_json::Value, Error> {
    let resp = ureq::get(ENDPOINT)
        .set("Authorization", &format!("Bearer {}", cred.access_token))
        .set("ChatGPT-Account-Id", &cred.account_id)
        .set("Accept", "application/json")
        .set("Cache-Control", "no-cache, no-store")
        .set("User-Agent", concat!("codenotch/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .call();
    match resp {
        Ok(r) => r.into_json().map_err(|e| Error::Other(format!("parse: {e}"))),
        Err(ureq::Error::Status(code @ (401 | 403), r)) => {
            // 403 can also be an edge node refusing the user agent, so the start of the body is
            // logged instead of folding both into "please sign in" silently
            let head: String = r.into_string().unwrap_or_default().chars().filter(|c| !c.is_control()).take(160).collect();
            crate::diagnostics::log(&format!("codex: usage endpoint HTTP {code}: {head}"));
            Err(Error::Rejected)
        }
        Err(ureq::Error::Status(429, r)) => {
            Err(Error::RateLimited(r.header("retry-after").and_then(|s| s.trim().parse().ok()).unwrap_or(0)))
        }
        Err(ureq::Error::Status(code, _)) => Err(Error::Other(format!("HTTP {code}"))),
        Err(e) => Err(Error::Other(e.to_string())),
    }
}
