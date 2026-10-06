use crate::app::state::AppState;
use crate::platform::{Autostart, Platform};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

#[derive(Serialize, Type)]
pub struct LangInfo {
    /// What the user chose, possibly "auto"
    pub lang: String,
    /// The language actually used
    pub resolved: String,
}

#[derive(Serialize, Type)]
pub struct UiFlags {
    pub notch_visible: bool,
    pub tray_visible: bool,
}

#[tauri::command]
#[specta::specta]
pub fn get_lang(state: State<AppState>) -> LangInfo {
    let lang = state.config.lock().unwrap().lang.clone();
    LangInfo { resolved: crate::i18n::resolve(&lang).into(), lang }
}

#[tauri::command]
#[specta::specta]
pub fn set_lang(app: AppHandle, lang: String) {
    crate::app::ui::apply_lang(&app, lang);
}

#[tauri::command]
#[specta::specta]
pub fn get_ui_flags(state: State<AppState>) -> UiFlags {
    let c = state.config.lock().unwrap();
    UiFlags { notch_visible: c.notch_visible, tray_visible: c.tray_visible }
}

/// Hiding both would leave the app with nothing to click, so the tray stays whenever the notch
/// is off. The answer is what was stored, so the window shows the corrected state.
#[tauri::command]
#[specta::specta]
pub fn set_ui_flags(app: AppHandle, state: State<AppState>, notch_visible: bool, tray_visible: bool) -> UiFlags {
    let flags = state.update_config(|c| {
        c.notch_visible = notch_visible;
        c.tray_visible = tray_visible || !notch_visible;
        UiFlags { notch_visible: c.notch_visible, tray_visible: c.tray_visible }
    });
    crate::app::ui::apply_visibility(&app);
    flags
}

#[tauri::command]
#[specta::specta]
pub fn get_autostart() -> bool {
    Platform.is_enabled()
}

#[tauri::command]
#[specta::specta]
pub fn set_autostart(on: bool) -> Result<String, String> {
    if on {
        Platform.enable()
    } else {
        Platform.disable()
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_hooks_installed() -> bool {
    crate::sessions::hooks_install::is_installed()
}

#[tauri::command]
#[specta::specta]
pub fn set_hooks_installed(on: bool) -> Result<String, String> {
    if on {
        crate::sessions::hooks_install::install()
    } else {
        crate::sessions::hooks_install::uninstall()
    }
}

#[tauri::command]
#[specta::specta]
pub fn open_settings(app: AppHandle) {
    crate::app::ui::open_settings(&app);
}
