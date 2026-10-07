//! Antigravity ("gemini" in ids). Preferred: the official CLI, `agy --print /usage`, run on
//! demand (startup, tray Refresh, hover) with a 5-minute TTL, since every read starts a process.
//! Without the CLI, most honest source first:
//!   1. the local language server bridge (its quota RPC answers for any signed-in IDE);
//!   2. after the bridge has answered once and stops (the IDE closed), the last reading kept stale;
//!   3. the Google credential from the OS store: tier name, and quota for licensed accounts;
//!   4. a count of today's model requests from the transcripts. A count, not a percentage: there
//!      is no published denominator, so the ring draws only its track.
//! Read only; token values are never cached and never logged.

mod activity;
mod bridge;
mod cli;
mod direct;
mod transcripts;

use super::{Activity, FetchError, LimitWindow, ProviderId, ProviderStatus, Reading, UsageProvider, UsageSnapshot};
use crate::support::time::now_ms;
use std::sync::Mutex;

const CLI_TTL_MS: u64 = 5 * 60 * 1000;
const CLI_NOTE: &str = "via Antigravity CLI";
/// In CLI mode reads only happen on request; this is just how long the thread waits for one
const CLI_IDLE_SECS: u64 = 24 * 60 * 60;

#[derive(Default)]
struct Legacy {
    endpoint: Option<bridge::Endpoint>,
    /// The last bridge reading and when it was taken: kept, stale, once the IDE closes, rather
    /// than degrading to a count (8 % turning into "31" looks broken)
    last_bridge: Option<(Vec<LimitWindow>, u64)>,
}

#[derive(Default)]
pub struct Antigravity {
    legacy: Mutex<Legacy>,
    last_cli_attempt: Mutex<Option<u64>>,
}

impl Antigravity {
    fn fetch_cli(&self, agy: &std::path::Path) -> Result<Reading, FetchError> {
        *self.last_cli_attempt.lock().unwrap() = Some(now_ms());
        let dir = crate::storage::paths::config_dir().join("quota-work");
        std::fs::create_dir_all(&dir).map_err(|e| FetchError::Other(format!("{CLI_NOTE} — cannot create its working directory: {e}")))?;
        match cli::read_quota(agy, &dir) {
            Ok(windows) => Ok(Reading::live(windows, CLI_NOTE)),
            Err(e) => Err(FetchError::Other(format!("{CLI_NOTE} — {e}."))),
        }
    }

    fn fetch_legacy(&self) -> Result<Reading, FetchError> {
        if !legacy_present() {
            return Err(FetchError::Absent);
        }
        let mut rt = self.legacy.lock().unwrap();
        // 1. The bridge: the cached endpoint first; the port changes on every launch, so a miss is normal
        let mut bridge_err = None;
        let candidates = rt.endpoint.clone().into_iter().chain(std::iter::once_with(bridge::discover).flatten());
        for ep in candidates {
            match bridge::quota(&ep) {
                Ok(w) => {
                    rt.last_bridge = Some((w.clone(), now_ms()));
                    rt.endpoint = Some(ep);
                    return Ok(Reading::live(w, "via Antigravity"));
                }
                Err(e) => {
                    rt.endpoint = None;
                    bridge_err = Some(e);
                }
            }
        }
        if let Some(e) = bridge_err {
            crate::diagnostics::log(&format!("antigravity: local bridge failed ({e})"));
        }
        // 2. The bridge answered before and does not now
        if let Some((windows, at)) = rt.last_bridge.clone() {
            return Ok(Reading { windows, note: "Antigravity is closed — last reading kept".into(), recorded_at: Some(at), current: false, rate_limited: None });
        }
        drop(rt);
        // 3. The Google credential
        let mut tier = None;
        match direct::read_credentials() {
            Some(c) if !c.expired => match direct::load_tier(&c.access_token) {
                Ok(t) => {
                    if let Some(w) = direct::quota(&c.access_token) {
                        return Ok(Reading::live(w, format!("{t} · via Google")));
                    }
                    tier = Some(t);
                }
                Err(direct::TierError::Rejected) => {
                    return Err(FetchError::NeedsAuth("Antigravity's Google session was rejected — sign in again in Antigravity".into()))
                }
                Err(direct::TierError::Other(e)) => crate::diagnostics::log(&format!("antigravity: loadCodeAssist {e}")),
            },
            // Expired is not signed out: Antigravity refreshes it on its next run
            Some(c) => tier = Some(if c.auth_method == "consumer" { "Personal".into() } else { c.auth_method }),
            None => {}
        }
        // 4. The count
        let (n, latest) = transcripts::requests_today(&transcripts::state_roots());
        Ok(Reading {
            windows: vec![LimitWindow {
                id: "requests".into(),
                label: "Requests today · no limit published".into(),
                used: 0.0,
                resets_at: None,
                count: Some(n as i64),
                derived: true,
            }],
            note: match tier {
                Some(t) => format!("{t} · Google publishes no quota for this account"),
                None => "Open Antigravity to read its quota".into(),
            },
            recorded_at: Some(latest.unwrap_or_else(now_ms)),
            current: true,
            rate_limited: None,
        })
    }
}

/// Any `~/.gemini/antigravity*` install, or its credential in the OS store.
fn legacy_present() -> bool {
    !transcripts::state_roots().is_empty() || direct::read_credentials().is_some()
}

impl UsageProvider for Antigravity {
    fn id(&self) -> ProviderId {
        ProviderId::Gemini
    }

    fn is_present(&self) -> bool {
        cli::find_agy().is_some() || legacy_present()
    }

    fn activity(&self) -> Vec<Activity> {
        activity::read(now_ms())
    }

    fn fetch(&self) -> Result<Reading, FetchError> {
        match cli::find_agy() {
            Some(agy) => self.fetch_cli(&agy),
            None => self.fetch_legacy(),
        }
    }

    fn poll_secs(&self, _session_active: bool) -> u64 {
        if cli::find_agy().is_some() {
            CLI_IDLE_SECS
        } else {
            300
        }
    }

    /// The CLI starts a process, so a fresh reading or a recent attempt answers for five minutes
    fn should_fetch(&self, current: &UsageSnapshot, now: u64) -> bool {
        if cli::find_agy().is_none() {
            return true;
        }
        let recent_attempt = self.last_cli_attempt.lock().unwrap().is_some_and(|t| now.saturating_sub(t) < CLI_TTL_MS);
        let fresh = !current.windows.is_empty() && now.saturating_sub(current.fetched_at) < CLI_TTL_MS;
        !recent_attempt && !fresh
    }

    fn refresh_on_hover(&self) -> bool {
        cli::find_agy().is_some()
    }

    fn restore(&self, snap: UsageSnapshot) -> UsageSnapshot {
        if cli::find_agy().is_none() {
            let mut snap = snap;
            if !snap.windows.is_empty() {
                snap.status = ProviderStatus::Stale;
            }
            return snap;
        }
        restore_cli(snap, now_ms())
    }

    fn probe(&self) -> String {
        match cli::find_agy() {
            Some(agy) => format!("Antigravity: official CLI installed at {}", agy.display()),
            None => format!(
                "Antigravity: state dirs {} | OS credential store gemini:antigravity {} | language_server {}",
                match transcripts::state_roots() {
                    r if r.is_empty() => "none under ~/.gemini".to_string(),
                    r => r.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "),
                },
                match direct::read_credentials() {
                    Some(c) => format!("found ({}, {})", c.auth_method, if c.expired { "expired" } else { "valid" }),
                    None => "not found".into(),
                },
                match bridge::discover() {
                    Some(e) => format!("running, ports {:?}", e.ports),
                    None => "not running".into(),
                }
            ),
        }
    }
}

/// In CLI mode only a CLI reading is restored, and it counts as live while inside the TTL.
fn restore_cli(snap: UsageSnapshot, now: u64) -> UsageSnapshot {
    let mut snap = if snap.note.starts_with(CLI_NOTE) { snap } else { UsageSnapshot::default() };
    snap.status = if snap.windows.is_empty() {
        ProviderStatus::Error
    } else if now.saturating_sub(snap.fetched_at) < CLI_TTL_MS {
        ProviderStatus::Ok
    } else {
        ProviderStatus::Stale
    };
    if snap.windows.is_empty() {
        snap.note = "Waiting for Antigravity CLI quota".into();
    }
    snap
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win() -> LimitWindow {
        LimitWindow { id: "w".into(), label: "Gemini · Weekly".into(), used: 0.1, ..Default::default() }
    }

    #[test]
    fn cli_mode_restores_only_its_own_readings() {
        let mine = UsageSnapshot { windows: vec![win()], fetched_at: 1_000, note: "via Antigravity CLI".into(), ..Default::default() };
        assert_eq!(restore_cli(mine.clone(), 1_000 + CLI_TTL_MS - 1).status, ProviderStatus::Ok);
        assert_eq!(restore_cli(mine, 1_000 + CLI_TTL_MS).status, ProviderStatus::Stale);
        let bridge = UsageSnapshot { windows: vec![win()], note: "via Antigravity".into(), ..Default::default() };
        let r = restore_cli(bridge, 0);
        assert_eq!((r.status, r.windows.len(), r.note.as_str()), (ProviderStatus::Error, 0, "Waiting for Antigravity CLI quota"));
    }

    #[test]
    fn the_real_v031_snapshot_restores_in_cli_mode() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/persisted/v0.3.1");
        let snap = crate::providers::load_snapshot(&dir, ProviderId::Gemini);
        let r = restore_cli(snap, u64::MAX);
        assert_eq!((r.status, r.windows.len()), (ProviderStatus::Stale, 2));
    }
}
