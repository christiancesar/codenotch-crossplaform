//! Polling, backoff and persistence for every provider.

use super::{FetchError, ProviderId, ProviderStatus, Reading, UsageProvider, UsageSnapshot};
use crate::support::time::{now_ms, sleep_interruptible};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

const BACKOFF_BASE_SECS: u64 = 60;
const BACKOFF_CAP_SECS: u64 = 900;

/// 60 s doubling per consecutive 429, capped at 15 min. A server can answer `Retry-After: 0`, so
/// the hint only raises the wait, never lowers it.
pub fn backoff_secs(consecutive: u32, retry_after: u64) -> u64 {
    let exp = BACKOFF_BASE_SECS.saturating_mul(1u64 << consecutive.min(4));
    exp.clamp(BACKOFF_BASE_SECS, BACKOFF_CAP_SECS).max(retry_after)
}

/// The next snapshot after one read. A failure never invents a number: windows already shown
/// stay, marked stale, and the note says why.
pub fn apply(prev: &UsageSnapshot, result: Result<Reading, FetchError>, now: u64, consecutive_429: &mut u32) -> UsageSnapshot {
    let mut next = prev.clone();
    let rate_limited = match &result {
        Err(FetchError::RateLimited { .. }) => true,
        Ok(r) => r.rate_limited.is_some(),
        _ => false,
    };
    if !rate_limited {
        *consecutive_429 = 0;
    }
    match result {
        Ok(r) => {
            next.status = if r.current { ProviderStatus::Ok } else { ProviderStatus::Stale };
            next.windows = r.windows;
            next.fetched_at = r.recorded_at.unwrap_or(now);
            next.note = r.note;
            next.backoff_until = match r.rate_limited {
                Some(hint) => {
                    let wait = backoff_secs(*consecutive_429, hint);
                    *consecutive_429 += 1;
                    now + wait * 1000
                }
                None => 0,
            };
        }
        Err(FetchError::NeedsAuth(note)) => {
            next.status = ProviderStatus::NeedsAuth;
            next.note = note;
        }
        Err(FetchError::RateLimited { retry_after_secs }) => {
            let wait = backoff_secs(*consecutive_429, retry_after_secs);
            *consecutive_429 += 1;
            next.status = if prev.windows.is_empty() { ProviderStatus::Backoff } else { ProviderStatus::Stale };
            next.note = format!("Rate limited, retrying in {wait}s");
            next.backoff_until = now + wait * 1000;
        }
        Err(FetchError::Absent) => {
            next = UsageSnapshot { status: ProviderStatus::Absent, ..Default::default() };
        }
        Err(FetchError::NothingMetered(note)) => {
            next = UsageSnapshot { status: ProviderStatus::None, note, ..Default::default() };
        }
        Err(FetchError::Other(msg)) => {
            next.status = if prev.windows.is_empty() { ProviderStatus::Error } else { ProviderStatus::Stale };
            next.note = msg;
        }
    }
    next
}

pub type OnChange = Arc<dyn Fn(ProviderId, &UsageSnapshot) + Send + Sync>;
pub type IsActive = Arc<dyn Fn() -> bool + Send + Sync>;

/// One provider's live state, shared with the commands that read it.
pub struct Handle {
    pub snapshot: Arc<Mutex<UsageSnapshot>>,
    /// Set to cut the current wait short (tray Refresh, hover on a stale card)
    pub wake: Arc<AtomicBool>,
}

/// Restores the snapshot from disk, announces it, then polls on its own thread.
pub fn spawn(provider: Arc<dyn UsageProvider>, dir: PathBuf, is_active: IsActive, on_change: OnChange) -> Handle {
    let id = provider.id();
    let restored = provider.restore(super::load_snapshot(&dir, id));
    let snapshot = Arc::new(Mutex::new(restored));
    let wake = Arc::new(AtomicBool::new(false));
    let handle = Handle { snapshot: snapshot.clone(), wake: wake.clone() };

    std::thread::Builder::new()
        .name(format!("provider-{}", id.as_str()))
        .spawn(move || {
            // Stale beats blank: show the restored reading before the first request
            on_change(id, &snapshot.lock().unwrap().clone());
            let mut consecutive_429 = 0u32;
            loop {
                let backoff_until = snapshot.lock().unwrap().backoff_until;
                let now = now_ms();
                if backoff_until > now {
                    sleep_interruptible(((backoff_until - now) / 1000).clamp(1, 30), &wake);
                    continue;
                }
                if !provider.should_fetch(&snapshot.lock().unwrap(), now) {
                    sleep_interruptible(provider.poll_secs(is_active()), &wake);
                    continue;
                }
                let result = provider.fetch();
                let next = {
                    let mut cur = snapshot.lock().unwrap();
                    *cur = apply(&cur, result, now_ms(), &mut consecutive_429);
                    cur.clone()
                };
                if let Err(e) = super::save_snapshot(&dir, id, &next) {
                    crate::diagnostics::log(&format!("{}: saving the snapshot failed: {e}", id.as_str()));
                }
                on_change(id, &next);
                sleep_interruptible(provider.poll_secs(is_active()), &wake);
            }
        })
        .expect("failed to start a provider thread");
    handle
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::LimitWindow;

    fn win() -> LimitWindow {
        LimitWindow { id: "session".into(), label: "Current session".into(), used: 0.4, ..Default::default() }
    }

    #[test]
    fn backoff_doubles_from_a_minute_to_fifteen_and_hints_only_raise_it() {
        let waits: Vec<u64> = (0..6).map(|n| backoff_secs(n, 0)).collect();
        assert_eq!(waits, vec![60, 120, 240, 480, 900, 900]);
        assert_eq!(backoff_secs(0, 1200), 1200);
        assert_eq!(backoff_secs(2, 5), 240);
    }

    #[test]
    fn rate_limits_keep_the_reading_and_set_a_deadline() {
        let prev = UsageSnapshot { status: ProviderStatus::Ok, windows: vec![win()], ..Default::default() };
        let mut n = 0;
        let a = apply(&prev, Err(FetchError::RateLimited { retry_after_secs: 0 }), 1_000, &mut n);
        assert_eq!((a.status, a.backoff_until, n), (ProviderStatus::Stale, 61_000, 1));
        assert_eq!(a.windows, prev.windows);
        let b = apply(&a, Err(FetchError::RateLimited { retry_after_secs: 0 }), 2_000, &mut n);
        assert_eq!(b.backoff_until, 122_000);
        let c = apply(&b, Ok(Reading::live(vec![win()], "")), 3_000, &mut n);
        assert_eq!((c.status, c.backoff_until, c.fetched_at, n), (ProviderStatus::Ok, 0, 3_000, 0));
    }

    #[test]
    fn a_fallback_reading_keeps_its_own_time_and_can_carry_a_rate_limit() {
        let mut n = 0;
        let old_log = Reading { windows: vec![win()], note: "from last run".into(), recorded_at: Some(7), current: false, rate_limited: Some(0) };
        let s = apply(&UsageSnapshot::default(), Ok(old_log), 100_000, &mut n);
        assert_eq!((s.status, s.fetched_at, s.backoff_until, n), (ProviderStatus::Stale, 7, 160_000, 1));
        let nothing = apply(&s, Err(FetchError::NothingMetered("no snapshot yet".into())), 0, &mut n);
        assert_eq!((nothing.status, nothing.windows.len()), (ProviderStatus::None, 0));
    }

    #[test]
    fn failures_never_invent_or_drop_a_reading() {
        let empty = UsageSnapshot::default();
        let mut n = 0;
        assert_eq!(apply(&empty, Err(FetchError::Other("HTTP 500".into())), 0, &mut n).status, ProviderStatus::Error);
        let shown = UsageSnapshot { status: ProviderStatus::Ok, windows: vec![win()], fetched_at: 5, ..Default::default() };
        let s = apply(&shown, Err(FetchError::Other("timeout".into())), 9, &mut n);
        assert_eq!((s.status, s.fetched_at, s.windows.len()), (ProviderStatus::Stale, 5, 1));
        let a = apply(&shown, Err(FetchError::NeedsAuth("Credential rejected".into())), 9, &mut n);
        assert_eq!((a.status, a.note.as_str(), a.windows.len()), (ProviderStatus::NeedsAuth, "Credential rejected", 1));
    }
}
