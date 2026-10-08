//! Codex. Two sources:
//!   1. Live: the session Codex keeps in `~/.codex/auth.json`, sent to
//!      `chatgpt.com/backend-api/wham/usage`. Read only, never refreshed or written back.
//!   2. Fallback: the limits Codex logged on its last turn, at the tail of the newest
//!      `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`. Stale by the line's own timestamp.
//! No sign-in and no session history at all means absent.

mod activity;
mod api;
mod credentials;
mod parse;
mod rollout;

use super::{Activity, FetchError, ProviderId, Reading, UsageProvider};
use crate::platform::{Executables, Platform};
use crate::support::text::cap;
use crate::support::time::now_ms;
use std::path::PathBuf;

/// A rollout line younger than this counts as current
const CURRENT_FOR_MS: u64 = 5 * 60 * 1000;

pub struct Codex {
    activity: std::sync::Mutex<activity::Probe>,
}

impl Default for Codex {
    fn default() -> Self {
        Codex { activity: std::sync::Mutex::new(activity::Probe::new()) }
    }
}

fn codex_home() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".codex"))
}

/// The native binary inside the global npm package (no cmd/node wrapper), then ~/.codex/bin,
/// then PATH.
pub fn find_executable() -> Option<PathBuf> {
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Some(pkg) = dirs::config_dir().map(|d| d.join("npm/node_modules/@openai/codex")) {
        for e in std::fs::read_dir(pkg.join("bin")).into_iter().flatten().flatten() {
            let n = e.file_name().to_string_lossy().to_lowercase();
            if n.starts_with("codex-") && n.contains("windows") && n.ends_with(".exe") {
                cands.push(e.path());
            }
        }
        // Newer packages keep it at vendor/<triple>/codex/codex.exe
        for e in std::fs::read_dir(pkg.join("vendor")).into_iter().flatten().flatten() {
            cands.push(e.path().join("codex").join("codex.exe"));
        }
    }
    if let Some(h) = codex_home() {
        cands.push(h.join("bin").join("codex.exe"));
        cands.push(h.join("bin").join("codex"));
    }
    cands.into_iter().find(|p| p.is_file()).or_else(|| Platform.find_on_path("codex"))
}

fn present() -> bool {
    find_executable().is_some()
        || credentials::auth_path().map(|p| p.is_file()).unwrap_or(false)
        || codex_home().map(|h| h.join("sessions").is_dir()).unwrap_or(false)
}

/// What the live attempt left behind when it produced no numbers.
enum Live {
    NeedsAuth(String),
    RateLimited(u64),
    Failed(String),
}

impl UsageProvider for Codex {
    fn id(&self) -> ProviderId {
        ProviderId::Codex
    }

    fn is_present(&self) -> bool {
        present()
    }

    fn activity(&self) -> Vec<Activity> {
        self.activity.lock().unwrap().read(now_ms())
    }

    fn fetch(&self) -> Result<Reading, FetchError> {
        if !present() {
            return Err(FetchError::Absent);
        }
        let live = match credentials::load() {
            None => None,
            Some(cred) => match api::fetch(&cred) {
                Ok(v) => {
                    let windows = parse::windows_from_usage(&v, now_ms());
                    if !windows.is_empty() {
                        let plan = v.get("plan_type").and_then(|x| x.as_str()).map(String::from).or(cred.plan);
                        let note = plan.map(|p| format!("{} · via Codex", cap(&p))).unwrap_or_default();
                        return Ok(Reading::live(windows, note));
                    }
                    let keys: Vec<String> = v.as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default();
                    crate::diagnostics::log(&format!("codex: usage reply has no windows (top-level keys {keys:?}), falling back to the rollout"));
                    Some(Live::Failed("Codex reported no usage windows".into()))
                }
                Err(api::Error::Rejected) => Some(Live::NeedsAuth(
                    if cred.expired {
                        "Codex sign-in expired — open Codex once to refresh it"
                    } else {
                        "Codex rejected its sign-in — sign in to Codex again"
                    }
                    .into(),
                )),
                Err(api::Error::RateLimited(ra)) => Some(Live::RateLimited(ra)),
                Err(api::Error::Other(e)) => {
                    crate::diagnostics::log(&format!("codex: live read failed ({e}), falling back to the rollout"));
                    Some(Live::Failed(format!("Live read failed ({e})")))
                }
            },
        };

        let live_note = match &live {
            Some(Live::NeedsAuth(n)) | Some(Live::Failed(n)) => Some(n.clone()),
            Some(Live::RateLimited(_)) => Some("Rate limited".to_string()),
            None => None,
        };
        let fallback = codex_home().and_then(|h| rollout::newest(&h.join("sessions"))).and_then(|p| rollout::tail_text(&p)).and_then(|t| rollout::snapshot(&t, now_ms()));
        match fallback {
            Some(snap) => {
                let rec = snap.recorded_at.unwrap_or(0);
                let mut note = match snap.plan {
                    Some(p) => format!("{} · from last Codex run", cap(&p)),
                    None => "from last Codex run".into(),
                };
                if let Some(n) = live_note {
                    note = format!("{n} · {note}");
                }
                Ok(Reading {
                    windows: snap.windows,
                    note,
                    // The recorded time is what counts: the card shows "Updated N ago" from it
                    recorded_at: Some(rec),
                    current: rec > 0 && now_ms().saturating_sub(rec) <= CURRENT_FOR_MS,
                    rate_limited: match live {
                        Some(Live::RateLimited(ra)) => Some(ra),
                        _ => None,
                    },
                })
            }
            None => Err(match live {
                Some(Live::NeedsAuth(n)) => FetchError::NeedsAuth(n),
                Some(Live::RateLimited(ra)) => FetchError::RateLimited { retry_after_secs: ra },
                Some(Live::Failed(n)) => FetchError::Other(n),
                None => FetchError::NothingMetered("Codex has not recorded a usage snapshot yet".into()),
            }),
        }
    }

    fn probe(&self) -> String {
        let auth = credentials::probe();
        let roll = codex_home().and_then(|h| rollout::newest(&h.join("sessions")));
        let age = roll
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok()?.modified().ok())
            .and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
            .map(|d| format!("{} min ago", d.as_secs() / 60))
            .unwrap_or_else(|| "?".into());
        format!(
            "Codex: {auth} | executable {} | newest rollout {} (modified {age})",
            find_executable().map(|p| p.display().to_string()).unwrap_or_else(|| "not found".into()),
            roll.map(|p| p.display().to_string()).unwrap_or_else(|| "none".into()),
        )
    }
}
