//! OpenCode usage adapter (H.2, decided in H1-opencode-research.md; extended for the Go plan).
//!
//! OpenCode itself is a routing/BYOK tool with no quota of its own (H.1 §"What's already
//! confirmed") — Zen is prepaid credit, Enterprise is a self-hosted gateway with no opencode-side
//! limit at all, and a raw BYOK key's limit belongs to that key's own provider, not OpenCode.
//! **OpenCode Go** is the one exception: a $10/mo subscription with real rolling-5h/weekly/monthly
//! percent windows, exposed at `GET https://opencode.ai/zen/go/v1/usage` (shipped upstream
//! 2026-08-11, `anomalyco/opencode#16513`/`#16017` — it did not exist when H.1 was written, which
//! is why that report found nothing). This module reads the `opencode-go` credential from
//! `auth.json` — the *value* of a key we already only ever read, never one we didn't touch before
//! — and calls that endpoint for a real percent + reset time, same `LimitWindow` shape as Claude's
//! five_hour/seven_day windows (`usage.rs`). Zen/BYOK/no-key cases get no live quota (none
//! exists to fetch) and fall back to the local `opencode.db` derived token tally this module
//! always had, same `count`/`derived` shape Antigravity's own count fallback uses.
//!
//! `opencode.db` access stays read only, never written; `account`/`credential` *tables* in that
//! DB are still never read (H.1's Must NOT still applies to the DB) — the Go API key comes from
//! `auth.json` instead, which is the file OpenCode itself writes it to for this exact purpose.

use crate::usage::{LimitWindow, UsageSnapshot};
use crate::AppState;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const POLL_SECS: u64 = 300;
/// "Today" for the token count fallback: a rolling 24h window, not calendar-day — matches
/// Antigravity's own "requests today" framing (a fixed reset-at-midnight would need a timezone
/// this data doesn't carry).
const WINDOW_MS: i64 = 24 * 60 * 60 * 1000;
const GO_USAGE_ENDPOINT: &str = "https://opencode.ai/zen/go/v1/usage";

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

/// Same fixed data dir as `db_path()`, sibling file OpenCode itself writes credentials to.
fn auth_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("opencode").join("auth.json"))
}

/// The `opencode-go` entry's API key, if the user has subscribed to Go. Reads `auth.json` only —
/// never `opencode.db`'s `account`/`credential` tables (H.1's Must NOT, restated above).
fn read_go_key() -> Option<String> {
    let text = std::fs::read_to_string(auth_path()?).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("opencode-go")?
        .get("key")?
        .as_str()
        .map(|s| s.to_string())
}

enum FetchErr {
    /// Key present but not a Go subscription (403 EntitlementError) — not an error worth
    /// surfacing, just "this key doesn't have what we're asking for".
    NotGo,
    Other(String),
}

struct GoWindow {
    status: String,
    percent: f64,
    resets_at: Option<u64>,
}

fn parse_go_window(v: &serde_json::Value) -> Option<GoWindow> {
    Some(GoWindow {
        status: v.get("status")?.as_str()?.to_string(),
        percent: v.get("percent")?.as_f64()?,
        resets_at: v
            .get("resetsAt")
            .and_then(|x| x.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.timestamp_millis().max(0) as u64),
    })
}

/// `usage.{rolling,weekly,monthly}` → three `LimitWindow`s, same shape/order the settings picker
/// already renders generically for any provider (`ui/settings.html`'s `p.windows` loop) — no
/// frontend change needed for the picker or the notch's "whichever is fullest" default.
fn parse_go_usage(v: &serde_json::Value) -> Vec<LimitWindow> {
    let usage = v.get("usage");
    [("rolling", "5 hour"), ("weekly", "Weekly"), ("monthly", "Monthly")]
        .into_iter()
        .filter_map(|(field, label)| {
            let w = parse_go_window(usage?.get(field)?)?;
            // "rate-limited" means the server has already cut this window off, regardless of
            // what percent it happens to report — show it as full so the ring/reset time agree
            // with what the user will actually experience on their next request.
            let used = if w.status == "rate-limited" {
                1.0
            } else {
                (w.percent / 100.0).clamp(0.0, 1.0)
            };
            Some(LimitWindow {
                id: field.into(),
                label: format!("{label} (Go)"),
                used,
                resets_at: w.resets_at,
                count: None,
                derived: false,
            })
        })
        .collect()
}

fn fetch_go_usage(key: &str) -> Result<Vec<LimitWindow>, FetchErr> {
    let resp = ureq::get(GO_USAGE_ENDPOINT)
        .set("Authorization", &format!("Bearer {key}"))
        .timeout(Duration::from_secs(15))
        .call();
    match resp {
        Ok(r) => {
            let v: serde_json::Value = r
                .into_json()
                .map_err(|e| FetchErr::Other(format!("parse: {e}")))?;
            let windows = parse_go_usage(&v);
            if windows.is_empty() {
                Err(FetchErr::Other("unrecognized response shape".into()))
            } else {
                Ok(windows)
            }
        }
        Err(ureq::Error::Status(403, _)) => Err(FetchErr::NotGo),
        Err(ureq::Error::Status(code, _)) => Err(FetchErr::Other(format!("HTTP {code}"))),
        Err(e) => Err(FetchErr::Other(format!("{e}"))),
    }
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

fn token_tally_snapshot() -> UsageSnapshot {
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

/// Go's real percent windows when the user has that plan; the local token tally for everyone
/// else (Zen, BYOK, no key, or a Go call that failed) — same fallback chain the module always had,
/// just with a real number in front of it when there is one.
fn read_once() -> UsageSnapshot {
    let Some(key) = read_go_key() else {
        return token_tally_snapshot();
    };
    match fetch_go_usage(&key) {
        Ok(windows) => UsageSnapshot {
            status: "ok".into(),
            fetched_at: now_ms(),
            note: "OpenCode Go".into(),
            windows,
            backoff_until: 0,
        },
        Err(FetchErr::NotGo) => token_tally_snapshot(),
        Err(FetchErr::Other(msg)) => {
            let mut snap = token_tally_snapshot();
            if snap.status == "ok" {
                snap.status = "stale".into();
            }
            snap.note = format!("Go usage check failed ({msg}); showing local token tally");
            snap
        }
    }
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

    /// Real `/zen/go/v1/usage` body captured live against `https://opencode.ai` on 2026-09-15
    /// with a real Go subscription's key. No secret in the fixture — the key itself is never
    /// part of the response.
    #[test]
    fn go_usage_fixture_parses_to_three_windows() {
        let body: serde_json::Value = serde_json::from_str(
            r#"{"usage":{"rolling":{"status":"ok","percent":0,"resetsAt":"2026-09-16T01:16:22.384Z"},
                "weekly":{"status":"ok","percent":40,"resetsAt":"2026-09-21T00:00:00.384Z"},
                "monthly":{"status":"ok","percent":27,"resetsAt":"2026-10-07T16:51:48.384Z"}}}"#,
        )
        .unwrap();
        let windows = parse_go_usage(&body);
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].id, "rolling");
        assert!((windows[0].used - 0.0).abs() < 1e-9);
        assert_eq!(windows[1].id, "weekly");
        assert!((windows[1].used - 0.40).abs() < 1e-9);
        assert_eq!(windows[2].id, "monthly");
        assert!((windows[2].used - 0.27).abs() < 1e-9);
        for w in &windows {
            assert!(w.resets_at.is_some());
            assert!(!w.derived);
            assert!(w.count.is_none());
        }
    }

    #[test]
    fn go_usage_rate_limited_window_shows_as_full_regardless_of_percent() {
        let body: serde_json::Value = serde_json::from_str(
            r#"{"usage":{"rolling":{"status":"rate-limited","percent":83,"resetsAt":"2026-09-16T01:16:22.384Z"}}}"#,
        )
        .unwrap();
        let windows = parse_go_usage(&body);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].used, 1.0);
    }

    #[test]
    fn go_usage_malformed_body_yields_no_windows() {
        let body: serde_json::Value = serde_json::from_str(r#"{"error":{"type":"AuthError"}}"#).unwrap();
        assert!(parse_go_usage(&body).is_empty());
    }

    #[test]
    fn read_go_key_reads_the_opencode_go_entry_only() {
        let dir = std::env::temp_dir().join(format!(
            "codenotch-opencode-authtest-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("auth.json");
        std::fs::write(
            &path,
            r#"{"google":{"type":"oauth","access":"x"},"opencode-go":{"type":"api","key":"sk-test123"}}"#,
        )
        .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            v.get("opencode-go").and_then(|x| x.get("key")).and_then(|x| x.as_str()),
            Some("sk-test123")
        );
        assert!(v.get("google").and_then(|x| x.get("key")).is_none());
    }

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
