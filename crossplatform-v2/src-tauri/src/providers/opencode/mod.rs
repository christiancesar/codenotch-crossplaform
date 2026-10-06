//! OpenCode. OpenCode has no quota of its own (Zen is prepaid credit, BYOK limits belong to the
//! key's provider), with one exception: the Go subscription, whose rolling 5h / weekly / monthly
//! percent windows come from `opencode.ai/zen/go/v1/usage` with the `opencode-go` key in
//! OpenCode's own auth.json. Everyone else gets a derived token tally from the local
//! `opencode.db`, marked as ours. The DB's account and credential tables are never read.

mod activity;
mod go;
mod tally;

use super::{Activity, FetchError, ProviderId, Reading, UsageProvider};
use crate::support::sqlite::LiveQuery;
use std::sync::Mutex;
use crate::support::sqlite::open_ro;
use std::path::PathBuf;

pub struct OpenCode {
    activity: Mutex<LiveQuery<Vec<Activity>>>,
}

impl Default for OpenCode {
    fn default() -> Self {
        OpenCode { activity: Mutex::new(LiveQuery::new(db_path().unwrap_or_default())) }
    }
}

fn data_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("opencode"))
}

fn db_path() -> Option<PathBuf> {
    data_dir().map(|d| d.join("opencode.db"))
}

fn tally() -> Result<Reading, FetchError> {
    let conn = db_path().and_then(|p| open_ro(&p, "session")).ok_or(FetchError::Absent)?;
    tally::reading(&conn, crate::support::time::now_ms())
        .ok_or_else(|| FetchError::Other("Could not read opencode.db".into()))
}

impl UsageProvider for OpenCode {
    fn id(&self) -> ProviderId {
        ProviderId::Opencode
    }

    fn is_present(&self) -> bool {
        db_path().is_some_and(|p| p.is_file())
    }

    fn activity(&self) -> Vec<Activity> {
        let now = crate::support::time::now_ms();
        self.activity.lock().unwrap().get(|c| activity::query(c, now))
    }

    fn fetch(&self) -> Result<Reading, FetchError> {
        if !db_path().map(|p| p.is_file()).unwrap_or(false) {
            return Err(FetchError::Absent);
        }
        let Some(key) = data_dir().and_then(|d| go::read_key(&d.join("auth.json"))) else {
            return tally();
        };
        match go::fetch(&key) {
            Ok(windows) => Ok(Reading::live(windows, "OpenCode Go")),
            // A key without the Go entitlement: nothing to ask for, not an error
            Err(go::Error::NotGo) => tally(),
            Err(go::Error::Other(msg)) => tally().map(|mut r| {
                r.current = false;
                r.note = format!("Go usage check failed ({msg}); showing local token tally");
                r
            }),
        }
    }

    /// Presence only: never a row count or any value from the database.
    fn probe(&self) -> String {
        let Some(p) = db_path() else { return "OpenCode: cannot locate the local data dir".into() };
        if !p.is_file() {
            return format!("OpenCode: {} not found (not installed)", p.display());
        }
        match open_ro(&p, "session") {
            Some(_) => format!("OpenCode: {} opened read-only", p.display()),
            None => format!("OpenCode: {} exists but could not be opened read-only", p.display()),
        }
    }
}
