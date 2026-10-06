//! Cursor, official: the editor's own session from its VS Code-style global state database,
//! sent as a cookie to `cursor.com/api/usage-summary`. Read only; the token never reaches logs,
//! events or the UI, and is re-read every time because the editor rotates it.

mod api;
mod credentials;
mod parse;

use super::{FetchError, ProviderId, Reading, UsageProvider};
use crate::support::text::cap;

pub struct Cursor;

impl UsageProvider for Cursor {
    fn id(&self) -> ProviderId {
        ProviderId::Cursor
    }

    fn fetch(&self) -> Result<Reading, FetchError> {
        let Some(path) = credentials::store_path().filter(|p| p.is_file()) else {
            return Err(FetchError::Absent);
        };
        let Some(creds) = credentials::read_at(&path) else {
            return Err(FetchError::NeedsAuth("Sign in to Cursor (the editor) to see usage.".into()));
        };
        match api::fetch(&creds.cookie) {
            Ok(v) => {
                let (windows, note) = parse::summary(&v);
                if windows.is_empty() {
                    return Err(FetchError::NothingMetered(note));
                }
                let plan = v.get("membershipType").and_then(|x| x.as_str()).map(String::from).or(creds.plan);
                Ok(Reading::live(windows, plan.map(|p| format!("{} · via Cursor", cap(&p))).unwrap_or_default()))
            }
            Err(api::Error::Rejected) => {
                Err(FetchError::NeedsAuth("Cursor session was rejected — sign in again in the editor".into()))
            }
            Err(api::Error::Other(msg)) => Err(FetchError::Other(msg)),
        }
    }

    fn probe(&self) -> String {
        credentials::probe()
    }
}
