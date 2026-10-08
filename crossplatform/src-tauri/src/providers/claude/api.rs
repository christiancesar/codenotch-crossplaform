use std::time::Duration;

const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";

pub enum Error {
    /// 401 or 403
    Rejected,
    /// 429 with the Retry-After hint in seconds, 0 when absent
    RateLimited(u64),
    Other(String),
}

pub fn fetch(token: &str) -> Result<serde_json::Value, Error> {
    let resp = ureq::get(ENDPOINT)
        .set("Authorization", &format!("Bearer {token}"))
        .set("anthropic-beta", "oauth-2025-04-20")
        .timeout(Duration::from_secs(15))
        .call();
    match resp {
        Ok(r) => r.into_json().map_err(|e| Error::Other(format!("parse: {e}"))),
        Err(ureq::Error::Status(401 | 403, _)) => Err(Error::Rejected),
        Err(ureq::Error::Status(429, r)) => {
            Err(Error::RateLimited(r.header("retry-after").and_then(|s| s.trim().parse().ok()).unwrap_or(0)))
        }
        Err(ureq::Error::Status(code, _)) => Err(Error::Other(format!("HTTP {code}"))),
        Err(e) => Err(Error::Other(e.to_string())),
    }
}
