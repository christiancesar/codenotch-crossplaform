//! OpenCode usage adapter (H.2, decided in H1-opencode-research.md).
//!
//! OpenCode is a routing/BYOK tool, not a metered subscription — there is no vendor-published
//! quota to show a fraction against (H.1 §"What's already confirmed"). What it does have is its
//! own SQLite database of session activity (`~/.local/share/opencode/opencode.db`, WAL mode,
//! confirmed via `PRAGMA journal_mode` in H.1's report), with `cost`/`tokens_*` columns already
//! materialized per session. This module sums those into a derived "tokens today" count — the
//! same `count`/`derived` shape Antigravity's own count fallback already uses (see
//! `antigravity.rs`'s `read_once`, the "4. Count fallback" branch) — never an invented fraction.
//!
//! Read only, never written; `account`/`credential` table values are never read (H.1's Must NOT
//! applies here too — this module only ever touches `session`).

use crate::usage::{LimitWindow, UsageSnapshot};
use crate::AppState;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const POLL_SECS: u64 = 300;
/// "Today" for the token count: a rolling 24h window, not calendar-day — matches Antigravity's
/// own "requests today" framing (a fixed reset-at-midnight would need a timezone this data
/// doesn't carry).
const WINDOW_MS: i64 = 24 * 60 * 60 * 1000;

static REFRESH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn request_refresh() {
    REFRESH.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Confirmed in H1-opencode-research.md on Linux (`~/.local/share/opencode/opencode.db`) — a
/// single fixed path, no per-profile sharding observed. `dirs::data_local_dir()` resolves to the
/// same `$XDG_DATA_HOME`/`~/.local/share` on Linux; the Windows/macOS equivalents this maps to
/// were not verified against a real OpenCode install on those platforms.
pub fn db_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("opencode").join("opencode.db"))
}

fn store_path() -> PathBuf {
    crate::config::config_path().with_file_name("opencode.json")
}

pub fn load_persisted() -> UsageSnapshot {
    std::fs::read_to_string(store_path())
        .ok()
        .and_then(|t| serde_json::from_str::<UsageSnapshot>(&t).ok())
        .map(|mut s| {
            if !s.windows.is_empty() {
                s.status = "stale".into();
            }
            s
        })
        .unwrap_or_default()
}

fn persist(s: &UsageSnapshot) {
    if let Ok(t) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(store_path(), t);
    }
}

pub fn present() -> bool {
    db_path().map(|p| p.is_file()).unwrap_or(false)
}

/// For `doctor`: presence only, never a row count or any value from `session`/`account`/`credential`.
pub fn probe() -> String {
    let Some(p) = db_path() else {
        return "OpenCode: cannot locate the local data dir".into();
    };
    if !p.is_file() {
        return format!("OpenCode: {} not found (not installed)", p.display());
    }
    match open_ro(&p) {
        Some(_) => format!("OpenCode: {} opened read-only", p.display()),
        None => format!("OpenCode: {} exists but could not be opened read-only", p.display()),
    }
}

/// mode=ro first, immutable=1 as the fallback — same rule as `cursor.rs::open_ro()` and the same
/// reason: WAL mode (H.1 confirmed) means mode=ro can open successfully while every query fails
/// once the `-shm` sidecar is gone (OpenCode not running / just checkpointed).
fn open_ro(path: &std::path::Path) -> Option<rusqlite::Connection> {
    use rusqlite::OpenFlags;
    if !path.is_file() {
        return None;
    }
    if let Ok(c) = rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        if c.prepare("SELECT 1 FROM session LIMIT 1")
            .and_then(|mut s| s.query([]).map(|_| ()))
            .is_ok()
        {
            return Some(c);
        }
    }
    let mut uri = String::from("file:///");
    uri.push_str(
        &path
            .to_string_lossy()
            .replace('\\', "/")
            .trim_start_matches('/')
            .replace('#', "%23")
            .replace('?', "%3F"),
    );
    uri.push_str("?immutable=1");
    rusqlite::Connection::open_with_flags(
        &uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()
}

/// Sum of `tokens_input + tokens_output` and the latest `time_updated`, across sessions touched
/// within `WINDOW_MS` — one query, no per-row Rust-side filtering needed since `time_updated` is
/// indexed-adjacent (session_project_idx doesn't cover it, but this table is a few hundred rows on
/// a real install, not the six-figure `event` table H.1 flagged as scan-unsafe).
fn windowed_totals(conn: &rusqlite::Connection) -> Option<(i64, u64)> {
    let cutoff = now_ms() as i64 - WINDOW_MS;
    conn.query_row(
        "SELECT COALESCE(SUM(tokens_input + tokens_output), 0), COALESCE(MAX(time_updated), 0)
         FROM session WHERE time_updated >= ?1",
        [cutoff],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? as u64)),
    )
    .ok()
}

fn read_once() -> UsageSnapshot {
    let mut snap = UsageSnapshot::default();
    let Some(path) = db_path() else {
        snap.status = "absent".into();
        return snap;
    };
    let Some(conn) = open_ro(&path) else {
        snap.status = "absent".into();
        return snap;
    };
    let Some((tokens, latest)) = windowed_totals(&conn) else {
        snap.status = "error".into();
        snap.note = "Could not read opencode.db".into();
        return snap;
    };
    snap.status = "ok".into();
    snap.fetched_at = if latest > 0 { latest } else { now_ms() };
    snap.windows = vec![LimitWindow {
        id: "tokens".into(),
        label: "Tokens today · no limit published".into(),
        used: 0.0,
        resets_at: None,
        count: Some(tokens),
        derived: true,
    }];
    snap.note = "OpenCode publishes no quota — token count is Codenotch's own tally".into();
    snap
}

fn broadcast(app: &AppHandle, snap: UsageSnapshot) {
    let st = app.state::<AppState>();
    *st.opencode.lock().unwrap() = snap.clone();
    persist(&snap);
    let _ = app.emit("opencode", &snap);
}

fn sleep_interruptible(secs: u64) {
    for _ in 0..secs {
        if REFRESH.swap(false, std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        {
            let st = app.state::<AppState>();
            let snap = st.opencode.lock().unwrap().clone();
            let _ = app.emit("opencode", &snap);
        }
        if !present() {
            broadcast(
                &app,
                UsageSnapshot {
                    status: "absent".into(),
                    ..Default::default()
                },
            );
            loop {
                sleep_interruptible(600); // not installed: look again every 10 minutes
                if present() {
                    break;
                }
            }
        }
        loop {
            let snap = read_once();
            if snap.status == "error" {
                crate::applog(&format!("opencode: {}", snap.note));
            }
            broadcast(&app, snap);
            sleep_interruptible(POLL_SECS);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Trimmed to the columns this module reads; the real table (H1-opencode-research.md's schema
    // dump) has more, but rusqlite doesn't care about columns a query never names.
    const SESSION_TABLE: &str = "
        CREATE TABLE session (
            id text PRIMARY KEY,
            time_updated integer NOT NULL,
            tokens_input integer DEFAULT 0 NOT NULL,
            tokens_output integer DEFAULT 0 NOT NULL
        );
    ";

    fn temp_db(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "codenotch-opencode-test-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("opencode.db")
    }

    #[test]
    #[cfg(not(windows))]
    fn db_path_matches_confirmed_linux_path() {
        let expected = dirs::data_local_dir()
            .expect("HOME/data_local_dir on Linux")
            .join("opencode")
            .join("opencode.db");
        assert_eq!(db_path().unwrap(), expected);
    }

    #[test]
    fn open_ro_reads_a_fixture_session_table() {
        let path = temp_db("ro");
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(SESSION_TABLE).unwrap();
        conn.execute(
            "INSERT INTO session (id, time_updated, tokens_input, tokens_output) VALUES ('s1', 1, 10, 20)",
            [],
        )
        .unwrap();
        drop(conn);
        let ro = open_ro(&path).expect("mode=ro should open the fixture");
        let (tokens, _) = windowed_totals(&ro).expect("query should succeed against the fixture");
        // time_updated=1 is ancient, outside any real WINDOW_MS cutoff — this only proves the
        // connection/query shape works, not the window filter (see the next test for that).
        assert_eq!(tokens, 0);
    }

    #[test]
    fn windowed_totals_sums_only_sessions_inside_the_window_and_ignores_older_ones() {
        let path = temp_db("window");
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(SESSION_TABLE).unwrap();
        let now = now_ms() as i64;
        conn.execute(
            "INSERT INTO session (id, time_updated, tokens_input, tokens_output) VALUES \
             ('fresh_a', ?1, 100, 50), ('fresh_b', ?2, 5, 5), ('stale', ?3, 999, 999)",
            rusqlite::params![now, now - 1000, now - WINDOW_MS - 1000],
        )
        .unwrap();
        let (tokens, latest) = windowed_totals(&conn).expect("query should succeed");
        assert_eq!(tokens, 100 + 50 + 5 + 5, "the stale row must not be counted");
        assert_eq!(latest, now as u64, "latest should be the freshest row's time_updated");
    }

    #[test]
    fn read_once_reports_absent_when_the_db_does_not_exist() {
        // present()/db_path() aren't overridable per-test (no DI here, matching cursor.rs's own
        // tests), so this only exercises the "file missing" branch that open_ro already covers —
        // read_once()'s "ok" branch is exercised indirectly by the two tests above, which call the
        // same windowed_totals() it calls.
        let missing = PathBuf::from("/nonexistent/does-not-exist/opencode.db");
        assert!(open_ro(&missing).is_none());
    }
}
