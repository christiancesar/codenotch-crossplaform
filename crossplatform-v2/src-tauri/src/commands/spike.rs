//! Phase 1 spike for the typed IPC contract; removed once real commands cover what it checks.
//! Finding: tauri-specta rc.25 always renders argument keys in camelCase, so commands keep
//! Tauri's default argument casing. Callers never see the keys, the wrappers are positional.

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SpikeReply {
    pub echoed_text: String,
    pub fetched_at: u64,
    pub count: Option<i64>,
}

/// Checks three things at once: a multi-word argument round-trips, u64/i64 arrive as `number`,
/// and an event emitted from a command reaches the page.
#[tauri::command]
#[specta::specta]
pub fn spike_echo(app: tauri::AppHandle, input_text: String, fetched_at: u64) -> SpikeReply {
    let _ = crate::app::events::SpikeTick { fetched_at }.emit(&app);
    SpikeReply { echoed_text: input_text, fetched_at, count: Some(1) }
}
