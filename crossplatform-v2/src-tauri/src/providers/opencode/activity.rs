//! Coarse liveness: a session whose `time_updated` moved within 90 s is busy (it advances on
//! every step), compacting counts too, archived sessions never do. OpenCode records no "waiting
//! on you" signal.

use crate::providers::{Activity, ActivityState, ProviderId};
use rusqlite::Connection;

const FRESH_MS: u64 = 90_000;

pub fn query(conn: &Connection, now: u64) -> Option<Vec<Activity>> {
    let mut stmt = conn
        .prepare("SELECT title, time_updated, time_compacting FROM session WHERE time_archived IS NULL ORDER BY time_updated DESC LIMIT 20")
        .ok()?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, Option<i64>>(2)?))).ok()?;
    let mut out: Vec<Activity> = rows
        .flatten()
        .filter_map(|(title, updated, compacting)| {
            let updated = updated.max(0) as u64;
            let compacting = compacting.is_some();
            (compacting || now.saturating_sub(updated) <= FRESH_MS).then(|| Activity {
                provider: ProviderId::Opencode,
                state: ActivityState::Busy,
                name: if title.trim().is_empty() { "OpenCode".into() } else { title },
                detail: if compacting { "Compacting".into() } else { "Working".into() },
                since: updated,
            })
        })
        .collect();
    out.sort_by(|a, b| b.since.cmp(&a.since));
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_and_compacting_sessions_are_busy() {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE session (title TEXT, time_updated INTEGER, time_compacting INTEGER, time_archived INTEGER);").unwrap();
        let now = 1_000_000i64;
        c.execute(
            "INSERT INTO session VALUES ('live', ?1, NULL, NULL), ('', 0, 5, NULL), ('old', 0, NULL, NULL), ('gone', ?1, NULL, 9)",
            [now - 1_000],
        )
        .unwrap();
        let a = query(&c, now as u64).unwrap();
        assert_eq!(a.iter().map(|x| (x.name.as_str(), x.detail.as_str())).collect::<Vec<_>>(), vec![("live", "Working"), ("OpenCode", "Compacting")]);
    }
}
