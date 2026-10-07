use crate::app::state::AppState;
use crate::glyphs::Glyph;
use crate::providers::ProviderId;
use std::collections::HashMap;
use tauri::State;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
#[specta::specta]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
#[specta::specta]
pub fn get_glyphs(state: State<AppState>) -> HashMap<String, Glyph> {
    state.glyphs.lock().unwrap().clone()
}

/// The data folder, with the glyph override directory created so it can be found.
#[tauri::command]
#[specta::specta]
pub fn open_data_dir(app: tauri::AppHandle, state: State<AppState>) -> Result<(), String> {
    let _ = std::fs::create_dir_all(crate::glyphs::user_dir());
    app.opener().open_path(state.dir.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
}

/// A click on a provider's cell opens its usage page.
#[tauri::command]
#[specta::specta]
pub fn open_provider_page(app: tauri::AppHandle, provider: ProviderId) -> Result<(), String> {
    let url = match provider {
        ProviderId::Codex => "https://chatgpt.com/#settings/Account",
        ProviderId::Cursor => "https://cursor.com/dashboard",
        ProviderId::Gemini => "https://antigravity.google",
        ProviderId::Opencode => "https://opencode.ai",
        ProviderId::Claude => "https://claude.ai/settings/usage",
    };
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

/// The page's own diagnostics go to run.log.
#[tauri::command]
#[specta::specta]
pub fn log_js(msg: String) {
    crate::diagnostics::log(&format!("js: {}", msg.chars().take(600).collect::<String>()));
}
