use serde::{Deserialize, Serialize};
use specta::Type;

/// Ids are frozen: config slots, glyph names and the page all use these strings. Antigravity is
/// "gemini" for historical reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    Claude,
    Codex,
    Cursor,
    Gemini,
    Opencode,
}

impl ProviderId {
    pub const ALL: [ProviderId; 5] =
        [ProviderId::Claude, ProviderId::Codex, ProviderId::Cursor, ProviderId::Gemini, ProviderId::Opencode];

    pub fn as_str(self) -> &'static str {
        match self {
            ProviderId::Claude => "claude",
            ProviderId::Codex => "codex",
            ProviderId::Cursor => "cursor",
            ProviderId::Gemini => "gemini",
            ProviderId::Opencode => "opencode",
        }
    }

    /// Snapshot file in the config directory. Claude's is `usage.json` and Antigravity's
    /// `antigravity.json`, as v0.3 wrote them.
    pub fn snapshot_file(self) -> &'static str {
        match self {
            ProviderId::Claude => "usage.json",
            ProviderId::Codex => "codex.json",
            ProviderId::Cursor => "cursor.json",
            ProviderId::Gemini => "antigravity.json",
            ProviderId::Opencode => "opencode.json",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
pub enum ProviderStatus {
    #[serde(rename = "ok")]
    Ok,
    /// A reading that is not current: restored after a restart, or kept through a failure
    #[serde(rename = "stale")]
    Stale,
    #[serde(rename = "needsAuth")]
    NeedsAuth,
    #[serde(rename = "backoff")]
    Backoff,
    /// Nothing to show and the last attempt failed
    #[default]
    #[serde(rename = "error")]
    Error,
    /// The tool is not installed; the provider is left off the notch
    #[serde(rename = "absent")]
    Absent,
    /// Installed and signed in, but nothing metered yet
    #[serde(rename = "none")]
    None,
}

impl ProviderStatus {
    pub fn parse(s: &str) -> ProviderStatus {
        match s {
            "ok" => ProviderStatus::Ok,
            "stale" => ProviderStatus::Stale,
            "needsAuth" => ProviderStatus::NeedsAuth,
            "backoff" => ProviderStatus::Backoff,
            "absent" => ProviderStatus::Absent,
            "none" => ProviderStatus::None,
            _ => ProviderStatus::Error,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderStatus::Ok => "ok",
            ProviderStatus::Stale => "stale",
            ProviderStatus::NeedsAuth => "needsAuth",
            ProviderStatus::Backoff => "backoff",
            ProviderStatus::Error => "error",
            ProviderStatus::Absent => "absent",
            ProviderStatus::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
pub struct LimitWindow {
    pub id: String,
    pub label: String,
    /// Fraction used, 0.0 to 1.0
    pub used: f64,
    /// Reset time in epoch ms, None when unknown
    pub resets_at: Option<u64>,
    /// A pure count with no published denominator (requests today): the cell shows ~N and the
    /// ring draws only its track
    pub count: Option<i64>,
    /// The number is ours, not the vendor's; the card prefixes it with ~
    pub derived: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
pub struct UsageSnapshot {
    pub status: ProviderStatus,
    pub windows: Vec<LimitWindow>,
    pub fetched_at: u64,
    pub note: String,
    /// No request before this epoch ms (429 backoff); survives restarts
    pub backoff_until: u64,
}

/// What one successful read returns.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub windows: Vec<LimitWindow>,
    pub note: String,
    /// When the numbers were recorded; None means now
    pub recorded_at: Option<u64>,
    /// false for last-known numbers (a local log from the tool's last run): shown as stale
    pub current: bool,
    /// The live source answered 429 and these numbers came from a fallback; the next request
    /// still has to wait. Server hint in seconds, 0 when absent.
    pub rate_limited: Option<u64>,
}

impl Reading {
    /// Live numbers read just now.
    pub fn live(windows: Vec<LimitWindow>, note: impl Into<String>) -> Reading {
        Reading { windows, note: note.into(), recorded_at: None, current: true, rate_limited: None }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FetchError {
    /// No credential, or the vendor refused it. The note says which, for the card.
    NeedsAuth(String),
    /// HTTP 429. `retry_after_secs` is the server hint, 0 when absent; it only ever raises the
    /// backoff, never lowers it.
    RateLimited { retry_after_secs: u64 },
    /// The tool is not installed at all
    Absent,
    /// Installed, but there is nothing metered to show yet. The note says so.
    NothingMetered(String),
    /// Anything else (network, unexpected HTTP status, unreadable body), shown as the note
    Other(String),
}
