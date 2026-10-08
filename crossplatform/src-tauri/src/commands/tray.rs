use crate::app::state::AppState;
use crate::config::{Slot, TrayMode};
use crate::providers::{ProviderId, ProviderStatus};
use crate::tray::render;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};

#[derive(Serialize, Type)]
pub struct TrayWindowOption {
    pub id: String,
    pub label: String,
    pub used: u32,
}

/// A provider and the windows it reports now, for the pickers: built from live readings, so a
/// provider that gains a window shows it.
#[derive(Serialize, Type)]
pub struct TrayOption {
    pub id: ProviderId,
    pub label: String,
    pub status: ProviderStatus,
    pub windows: Vec<TrayWindowOption>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct TrayConfig {
    pub mode: TrayMode,
    pub slots: Vec<Slot>,
}

#[tauri::command]
#[specta::specta]
pub fn get_tray_options(state: State<AppState>) -> Vec<TrayOption> {
    ProviderId::ALL
        .into_iter()
        .map(|id| {
            let snap = state.snapshot(id);
            TrayOption {
                id,
                label: crate::app::ui::label(id.as_str()).into(),
                status: snap.status,
                // A count window has no percentage to draw
                windows: snap
                    .windows
                    .iter()
                    .filter(|w| w.count.is_none())
                    .map(|w| TrayWindowOption { id: w.id.clone(), label: w.label.clone(), used: (w.used * 100.0).round().clamp(0.0, 100.0) as u32 })
                    .collect(),
            }
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub fn get_tray_config(state: State<AppState>) -> TrayConfig {
    let c = state.config.lock().unwrap();
    TrayConfig { mode: c.tray_mode, slots: c.tray_slots.clone() }
}

#[tauri::command]
#[specta::specta]
pub fn set_tray_config(app: AppHandle, state: State<AppState>, config: TrayConfig) {
    state.update_config(|c| {
        c.tray_mode = config.mode;
        c.set_tray_slots(config.slots);
    });
    crate::app::ui::repaint_tray(&app);
}

/// The real icon as a picture, so the settings preview cannot drift from the taskbar. None for
/// the plain mark, which the page draws itself.
#[tauri::command]
#[specta::specta]
pub fn get_tray_preview(state: State<AppState>, config: TrayConfig) -> Option<String> {
    let values = crate::app::ui::tray_values(&state, &config.slots);
    match config.mode {
        TrayMode::Bars if !values.is_empty() => render::to_data_url(&render::bars_rgba(&values)),
        TrayMode::Numbers if !values.is_empty() => render::to_data_url(&render::numbers_rgba(&values)),
        _ => None,
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_app_icon() -> Option<String> {
    render::app_mark_data_url()
}
