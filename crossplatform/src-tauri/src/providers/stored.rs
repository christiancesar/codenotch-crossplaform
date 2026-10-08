//! Snapshot files as they sit on disk (v0.3 format, schema 1).

use super::model::{LimitWindow, ProviderStatus, UsageSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredWindow {
    pub id: String,
    pub label: String,
    pub used: f64,
    #[serde(default)]
    pub resets_at: Option<u64>,
    #[serde(default)]
    pub count: Option<i64>,
    #[serde(default)]
    pub derived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSnapshot {
    #[serde(default)]
    pub status: String,
    /// v0.3 omitted this in some states (a needsAuth Codex snapshot); it made the whole file
    /// unreadable there. Missing now means no windows.
    #[serde(default)]
    pub windows: Vec<StoredWindow>,
    #[serde(default)]
    pub fetched_at: u64,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub backoff_until: u64,
}

impl From<StoredSnapshot> for UsageSnapshot {
    fn from(s: StoredSnapshot) -> Self {
        UsageSnapshot {
            status: ProviderStatus::parse(&s.status),
            windows: s
                .windows
                .into_iter()
                .map(|w| LimitWindow {
                    id: w.id,
                    label: w.label,
                    used: w.used,
                    resets_at: w.resets_at,
                    count: w.count,
                    derived: w.derived,
                })
                .collect(),
            fetched_at: s.fetched_at,
            note: s.note,
            backoff_until: s.backoff_until,
        }
    }
}

impl From<&UsageSnapshot> for StoredSnapshot {
    fn from(s: &UsageSnapshot) -> Self {
        StoredSnapshot {
            status: s.status.as_str().into(),
            windows: s
                .windows
                .iter()
                .map(|w| StoredWindow {
                    id: w.id.clone(),
                    label: w.label.clone(),
                    used: w.used,
                    resets_at: w.resets_at,
                    count: w.count,
                    derived: w.derived,
                })
                .collect(),
            fetched_at: s.fetched_at,
            note: s.note.clone(),
            backoff_until: s.backoff_until,
        }
    }
}
