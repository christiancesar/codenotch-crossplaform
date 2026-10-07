use crate::providers::{LimitWindow, Reading};
use rusqlite::Connection;

/// "Today" for the tally: a rolling 24 h, since the data carries no timezone for a calendar day.
const WINDOW_MS: i64 = 24 * 60 * 60 * 1000;

/// Sum of input + output tokens and the latest update across sessions touched in the window.
fn windowed_totals(conn: &Connection, now: u64) -> Option<(i64, u64)> {
    conn.query_row(
        "SELECT COALESCE(SUM(tokens_input + tokens_output), 0), COALESCE(MAX(time_updated), 0)
         FROM session WHERE time_updated >= ?1",
        [now as i64 - WINDOW_MS],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? as u64)),
    )
    .ok()
}

/// A count with no published limit: the ring draws only its track, and `derived` marks the
/// number as Codenotch's own.
pub fn reading(conn: &Connection, now: u64) -> Option<Reading> {
    let (tokens, latest) = windowed_totals(conn, now)?;
    Some(Reading {
        windows: vec![LimitWindow {
            id: "tokens".into(),
            label: "Tokens today · no limit published".into(),
            used: 0.0,
            resets_at: None,
            count: Some(tokens),
            derived: true,
        }],
        note: "OpenCode publishes no quota — token count is Codenotch's own tally".into(),
        recorded_at: Some(if latest > 0 { latest } else { now }),
        current: true,
        rate_limited: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_only_sessions_inside_the_window() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE session (id text PRIMARY KEY, time_updated integer NOT NULL, tokens_input integer DEFAULT 0 NOT NULL, tokens_output integer DEFAULT 0 NOT NULL);").unwrap();
        let now: i64 = 1_791_000_000_000;
        conn.execute(
            "INSERT INTO session VALUES ('a', ?1, 100, 50), ('b', ?2, 5, 5), ('old', ?3, 999, 999)",
            rusqlite::params![now, now - 1000, now - WINDOW_MS - 1000],
        )
        .unwrap();
        let r = reading(&conn, now as u64).unwrap();
        assert_eq!(r.windows[0].count, Some(160));
        assert_eq!(r.recorded_at, Some(now as u64));
        assert!(r.windows[0].derived);
    }
}
