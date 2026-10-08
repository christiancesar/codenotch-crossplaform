//! Each step is written to transcript.jsonl only once it completes, so its status is always
//! DONE and useless; a transcript written within 45 s means working (the model can think a long
//! time between steps).

use crate::providers::{Activity, ActivityState, ProviderId};

const STALE_MS: u64 = 45_000;

pub fn read(now: u64) -> Vec<Activity> {
    let newest = super::transcripts::state_roots()
        .into_iter()
        .filter_map(|r| std::fs::read_dir(r.join("brain")).ok())
        .flat_map(|rd| rd.flatten())
        .filter_map(|e| {
            let t = e.path().join(".system_generated").join("logs").join("transcript.jsonl");
            let m = std::fs::metadata(t).ok()?.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?;
            Some(m.as_millis() as u64)
        })
        .max();
    match newest {
        Some(at) if now.saturating_sub(at) <= STALE_MS => vec![Activity {
            provider: ProviderId::Gemini,
            state: ActivityState::Busy,
            name: "Antigravity".into(),
            detail: "Working".into(),
            since: at,
        }],
        _ => vec![],
    }
}
