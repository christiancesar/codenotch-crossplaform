//! Typed events sent to the webview.

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

/// Phase 1 spike only: proves an event with a u64 field reaches the page typed as `number`.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "spike_tick")]
pub struct SpikeTick {
    pub fetched_at: u64,
}
