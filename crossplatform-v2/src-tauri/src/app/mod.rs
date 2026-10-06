//! The only place that holds an AppHandle: builder, plugins and setup.

pub mod events;
mod ipc;
pub mod state;

use tauri::Manager;

pub fn run() {
    let builder = ipc::builder();

    #[cfg(debug_assertions)]
    ipc::export(&builder).expect("failed to export IPC bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::load())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            for label in ["notch", "settings"] {
                if let Some(w) = app.get_webview_window(label) {
                    let _ = w.show();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
