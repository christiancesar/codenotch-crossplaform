use crate::app::state::AppState;
use crate::providers::{ProviderId, UsageSnapshot};
use serde::Serialize;
use specta::Type;
use tauri::State;

#[derive(Serialize, Type)]
pub struct ProviderUsage {
    pub provider: ProviderId,
    pub snapshot: UsageSnapshot,
}

/// Every provider's current reading, in display order.
#[tauri::command]
#[specta::specta]
pub fn get_usage(state: State<AppState>) -> Vec<ProviderUsage> {
    state.providers.iter().map(|p| ProviderUsage { provider: p.provider.id(), snapshot: p.handle.snapshot.lock().unwrap().clone() }).collect()
}

#[tauri::command]
#[specta::specta]
pub fn refresh_usage(app: tauri::AppHandle) {
    crate::app::ui::refresh_all(&app);
}
