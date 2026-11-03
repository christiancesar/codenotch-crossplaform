//! OpenCode. OpenCode has no quota of its own (Zen is prepaid credit, BYOK limits belong to the
//! key's provider), with one exception: the Go subscription, whose rolling 5h / weekly / monthly
//! percent windows come from `opencode.ai/zen/go/v1/usage` with the `opencode-go` key in
//! OpenCode's own credential file. Everyone else gets a derived token tally from the local
//! `opencode.db`, marked as ours. The DB's account and credential tables are never read.

mod activity;
mod go;
mod tally;

use super::{Activity, FetchError, ProviderId, Reading, UsageProvider};
use crate::support::sqlite::LiveQuery;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const SESSION_TABLE: &str = "session_v2";

pub(super) fn open_sessions(path: &Path) -> Option<Connection> {
    crate::support::sqlite::open_ro(path, SESSION_TABLE)
}

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
    let conn = db_path().and_then(|p| open_sessions(&p)).ok_or(FetchError::Absent)?;
    tally::reading(&conn, crate::support::time::now_ms())
        .ok_or_else(|| FetchError::Other("Could not read opencode.db".into()))
}

fn go_key(dir: &Path) -> Option<String> {
    go::read_key(&dir.join("account.json"))
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
        let Some(key) = data_dir().and_then(|d| go_key(&d)) else {
            return tally();
        };
        match go::fetch(&key) {
            Ok(windows) => Ok(Reading::live(windows, "OpenCode Go")),
            // A key without the Go entitlement: nothing to ask for, not an error
            Err(go::Error::NotGo) => tally(),
            // A dead key is the user's to fix, and the app would otherwise look like it simply
            // stopped refreshing the Go windows.
            Err(go::Error::KeyRejected(why)) => tally().map(|mut r| {
                r.current = false;
                r.note = format!("Go key rejected ({why}); showing local token tally");
                r
            }),
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
        match open_sessions(&p) {
            Some(_) => format!("OpenCode: {} opened read-only", p.display()),
            None => format!("OpenCode: {} exists but could not be opened read-only", p.display()),
        }
    }
}
