//! Usage sources. Each provider only knows how to read its vendor once; polling, backoff,
//! persistence and change notification live in the scheduler, written once for all of them.
//! Nothing here knows Tauri: `app/` wires the scheduler's callback to events.

pub mod antigravity;
pub mod claude;
pub mod codex;
pub mod cursor;
pub mod model;
pub mod opencode;
pub mod scheduler;
mod stored;

pub use model::{FetchError, LimitWindow, ProviderId, ProviderStatus, Reading, UsageSnapshot};

use crate::storage::versioned::{self, Loaded};
use std::path::Path;

pub trait UsageProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    /// One read. No sleeping, no emitting, no disk writes.
    fn fetch(&self) -> Result<Reading, FetchError>;
    /// One line for `codenotch doctor`. Never prints a secret.
    fn probe(&self) -> String;
    /// Seconds until the next read.
    fn poll_secs(&self, _session_active: bool) -> u64 {
        300
    }
    /// Whether a scheduled or requested read should run now. A provider whose read is
    /// expensive (it starts a process) answers false while its last reading is fresh enough.
    fn should_fetch(&self, _current: &UsageSnapshot, _now: u64) -> bool {
        true
    }
    /// Whether hovering the notch asks for a read. Only cheap or cached reads should.
    fn refresh_on_hover(&self) -> bool {
        false
    }
    /// The snapshot restored from disk at startup. An old reading is never presented as live.
    fn restore(&self, mut snap: UsageSnapshot) -> UsageSnapshot {
        if !snap.windows.is_empty() {
            snap.status = ProviderStatus::Stale;
        }
        snap
    }
}

pub fn load_snapshot(dir: &Path, id: ProviderId) -> UsageSnapshot {
    match versioned::load::<stored::StoredSnapshot>(&dir.join(id.snapshot_file()), &[]) {
        Loaded::Ok(s) => s.into(),
        Loaded::Missing | Loaded::Quarantined(_) => UsageSnapshot::default(),
    }
}

pub fn save_snapshot(dir: &Path, id: ProviderId, snap: &UsageSnapshot) -> std::io::Result<()> {
    versioned::save(&dir.join(id.snapshot_file()), &stored::StoredSnapshot::from(snap), &[])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn real(id: ProviderId) -> UsageSnapshot {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/persisted/v0.3.1");
        load_snapshot(&dir, id)
    }

    #[test]
    fn every_real_v031_snapshot_loads_with_its_values() {
        let claude = real(ProviderId::Claude);
        assert_eq!(claude.status, ProviderStatus::Ok);
        assert_eq!(claude.windows.len(), 2);
        assert_eq!(claude.windows[1].id, "weekly_all");
        assert_eq!(claude.windows[1].resets_at, Some(1_791_698_399_761));
        assert_eq!(claude.fetched_at, 1_791_317_623_977);

        assert_eq!(real(ProviderId::Cursor).note, "Free · via Cursor");
        assert_eq!(real(ProviderId::Gemini).windows[0].label, "Gemini · Weekly");
        assert_eq!(real(ProviderId::Opencode).windows.len(), 3);
    }

    #[test]
    fn a_needs_auth_snapshot_without_windows_keeps_status_and_note() {
        // Gate finding 3: v0.3 could not read this real file and started blank
        let codex = real(ProviderId::Codex);
        assert_eq!(codex.status, ProviderStatus::NeedsAuth);
        assert_eq!(codex.note, "Codex sign-in expired — open Codex once to refresh it");
        assert!(codex.windows.is_empty());
    }

    #[test]
    fn save_then_load_is_lossless() {
        let dir = tempfile::tempdir().unwrap();
        for id in ProviderId::ALL {
            let snap = real(id);
            save_snapshot(dir.path(), id, &snap).unwrap();
            assert_eq!(load_snapshot(dir.path(), id), snap, "{id:?}");
        }
    }
}
