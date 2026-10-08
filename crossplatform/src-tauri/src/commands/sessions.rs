use crate::app::events::SessionsChanged;
use crate::app::state::AppState;
use crate::platform::{Focus, Platform};
use crate::providers::Activity;
use crate::sessions::SessionsSnapshot;
use tauri::State;
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
pub fn get_sessions(state: State<AppState>) -> SessionsSnapshot {
    state.sessions.lock().unwrap().snapshot()
}

#[tauri::command]
#[specta::specta]
pub fn get_activity(state: State<AppState>) -> Vec<Activity> {
    state.activity.lock().unwrap().clone()
}

/// Raises the session's terminal, or the Claude desktop app for a session without a process.
#[tauri::command]
#[specta::specta]
pub fn focus_session(state: State<AppState>, id: String) -> bool {
    match state.sessions.lock().unwrap().ppid_of(&id) {
        Some(pid) => Platform.focus_terminal(pid),
        None => Platform.focus_claude_desktop(),
    }
}

#[tauri::command]
#[specta::specta]
pub fn dismiss_session(app: tauri::AppHandle, state: State<AppState>, id: String) {
    let snap = {
        let mut s = state.sessions.lock().unwrap();
        s.dismiss(&id);
        s.snapshot()
    };
    let _ = SessionsChanged(snap).emit(&app);
}
