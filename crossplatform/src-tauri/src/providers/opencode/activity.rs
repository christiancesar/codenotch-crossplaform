//! Coarse liveness: a session whose `time_updated` moved within 90 s is busy (it advances on
//! every step), compacting counts too, archived sessions never do. OpenCode records no "waiting
//! on you" signal.

use crate::providers::opencode::table;
use crate::providers::{Activity, ActivityState, ProviderId};
use rusqlite::Connection;

const FRESH_MS: u64 = 90_000;

pub fn query(conn: &Connection, now: u64) -> Option<Vec<Activity>> {
    let sql = format!(
        "SELECT title, time_updated, time_compacting FROM {} WHERE time_archived IS NULL ORDER BY time_updated DESC LIMIT 20",
        table(conn)
    );
    let mut stmt = conn.prepare(&sql).ok()?;
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
        c.execute_batch("CREATE TABLE session_v2 (title TEXT, time_updated INTEGER, time_compacting INTEGER, time_archived INTEGER);").unwrap();
        let now = 1_000_000i64;
        c.execute(
            "INSERT INTO session_v2 VALUES ('live', ?1, NULL, NULL), ('', 0, 5, NULL), ('old', 0, NULL, NULL), ('gone', ?1, NULL, 9)",
            [now - 1_000],
        )
        .unwrap();
        let a = query(&c, now as u64).unwrap();
        assert_eq!(a.iter().map(|x| (x.name.as_str(), x.detail.as_str())).collect::<Vec<_>>(), vec![("live", "Working"), ("OpenCode", "Compacting")]);
    }

    /// A database the V1 CLI wrote must still list its sessions.
    #[test]
    fn the_v1_table_is_read_too() {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE session (title TEXT, time_updated INTEGER, time_compacting INTEGER, time_archived INTEGER);").unwrap();
        c.execute("INSERT INTO session VALUES ('v1', ?1, NULL, NULL)", [999_500i64]).unwrap();
        let a = query(&c, 1_000_000).unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].name, "v1");
    }
}
