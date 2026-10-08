//! Real state from the editor's `composerHeaders` rows: `unfinishedRunAt` is set for the length
//! of a run; `hasBlockingPendingActions` / `hasPendingPlan` mean it waits on you.

use crate::providers::{Activity, ActivityState, ProviderId};
use crate::support::time::now_ms;
use rusqlite::Connection;

pub fn query(conn: &Connection) -> Option<Vec<Activity>> {
    let mut stmt = conn.prepare("SELECT value FROM composerHeaders WHERE isArchived = 0 ORDER BY recency DESC LIMIT 40").ok()?;
    let rows: Vec<String> = stmt.query_map([], |r| r.get::<_, String>(0)).ok()?.flatten().collect();
    let mut out: Vec<Activity> = rows.iter().filter_map(|json| from_header(&serde_json::from_str(json).ok()?)).collect();
    out.sort_by(|a, b| b.since.cmp(&a.since));
    Some(out)
}

fn from_header(v: &serde_json::Value) -> Option<Activity> {
    v.get("composerId")?.as_str()?;
    let flag = |k: &str| v.get(k).and_then(|x| x.as_bool()) == Some(true);
    let blocked = flag("hasBlockingPendingActions") || flag("hasPendingPlan");
    let running = v.get("unfinishedRunAt").and_then(|x| x.as_f64());
    // Forty idle past conversations are not forty things happening now
    if !blocked && running.is_none() {
        return None;
    }
    let since = running
        .or_else(|| v.get("lastUpdatedAt").and_then(|x| x.as_f64()))
        .or_else(|| v.get("createdAt").and_then(|x| x.as_f64()))
        .map(|ms| ms as u64)
        .unwrap_or_else(now_ms);
    Some(Activity {
        provider: ProviderId::Cursor,
        state: if blocked { ActivityState::Waiting } else { ActivityState::Busy },
        name: v.get("name").and_then(|x| x.as_str()).unwrap_or("Untitled chat").to_string(),
        detail: if blocked { "needs your input".into() } else { v.get("subtitle").and_then(|x| x.as_str()).unwrap_or("Working").to_string() },
        since,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn running_waiting_and_idle_headers() {
        let busy = from_header(&json!({"composerId": "a", "name": "Fix CI", "subtitle": "Editing", "unfinishedRunAt": 5.0})).unwrap();
        assert_eq!((busy.state, busy.detail.as_str(), busy.since), (ActivityState::Busy, "Editing", 5));
        let wait = from_header(&json!({"composerId": "b", "hasPendingPlan": true, "lastUpdatedAt": 7.0})).unwrap();
        assert_eq!((wait.state, wait.name.as_str(), wait.since), (ActivityState::Waiting, "Untitled chat", 7));
        assert!(from_header(&json!({"composerId": "c", "lastUpdatedAt": 1.0})).is_none());
    }
}
