use std::time::Duration;

const ENDPOINT: &str = "https://cursor.com/api/usage-summary";

pub enum Error {
    Rejected,
    Other(String),
}

pub fn fetch(cookie: &str) -> Result<serde_json::Value, Error> {
    match ureq::get(ENDPOINT).set("Cookie", cookie).set("Accept", "application/json").timeout(Duration::from_secs(15)).call() {
        Ok(r) => r.into_json().map_err(|e| Error::Other(format!("parse: {e}"))),
        Err(ureq::Error::Status(401 | 403, _)) => Err(Error::Rejected),
        Err(ureq::Error::Status(code, _)) => Err(Error::Other(format!("HTTP {code}"))),
        Err(e) => Err(Error::Other(e.to_string())),
    }
}
