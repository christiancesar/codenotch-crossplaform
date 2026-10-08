//! The only place that holds the AppHandle: builder, plugins, setup, and the actions that touch
//! windows and the tray.

pub mod events;
mod ipc;
pub mod state;
pub mod ui;
mod workers;

use crate::platform::{Platform, Window};
use events::Notice;
use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub fn run() {
    Platform.prepare_process();
    let args: Vec<String> = std::env::args().collect();
    if crate::cli::dispatch(&args).is_some() {
        return;
    }
    let dir = crate::storage::paths::config_dir();
    // A launch that is not a hook's relaunch lifts the tray's Quit: hooks may relaunch again
    let _ = std::fs::remove_file(dir.join(crate::storage::paths::USER_QUIT));

    let builder = ipc::builder();
    #[cfg(debug_assertions)]
    ipc::export(&builder).expect("failed to export IPC bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            crate::diagnostics::log("single instance: another launch was refused");
            let _ = Notice("Codenotch is already running — quit it from the tray before starting another build".into()).emit(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            setup(app.handle(), dir.clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup(app: &AppHandle, dir: std::path::PathBuf) -> tauri::Result<()> {
    let loaded = crate::config::load(&dir);
    // Only a missing file is written now (the hook reads the port from it). v0.3 saved
    // unconditionally here and put defaults over a file it had failed to read.
    if loaded.missing {
        if let Err(e) = crate::config::save(&dir, &loaded.config) {
            crate::diagnostics::log(&format!("config: first save failed: {e}"));
        }
    }
    if let Some(p) = &loaded.quarantined {
        let msg = format!("Your settings file could not be read and was set aside as {}; running on defaults", p.display());
        let a = app.clone();
        // After the page has had time to listen
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let _ = Notice(msg).emit(&a);
        });
    }
    // A dev build would point the user's hooks at target/debug, so only an installed app wires them
    if !cfg!(debug_assertions) {
        if let Some(r) = crate::sessions::hooks_install::ensure(&dir) {
            crate::diagnostics::log(&format!("hooks: {}", r.unwrap_or_else(|e| format!("not wired: {e}"))));
        }
    }
    let port = loaded.config.port;
    let lang = loaded.config.lang.clone();
    let sessions = Arc::new(Mutex::new(crate::sessions::Store::default()));
    let providers = workers::start_providers(app, &dir, sessions.clone());
    app.manage(AppState {
        dir,
        config: Mutex::new(loaded.config),
        sessions,
        providers,
        activity: Mutex::new(Vec::new()),
        glyphs: Mutex::new(crate::glyphs::collect()),
        notch: Arc::new(crate::notch::NotchState::default()),
    });

    if let Some(w) = app.get_webview_window(ui::NOTCH) {
        ui::place_notch(app);
        Platform.no_activate(&w);
        // Click-through until the page reports its pill
        ui::apply_input_region(app, &[]);
    }
    // Closing destroys a Tauri window by default, and a destroyed one cannot be shown again
    if let Some(w) = app.get_webview_window(ui::SETTINGS) {
        let hide = w.clone();
        w.on_window_event(move |e| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = e {
                api.prevent_close();
                let _ = hide.hide();
            }
        });
    }

    TrayIconBuilder::with_id(crate::tray::menu::TRAY_ID)
        .icon(crate::tray::render::app_mark().unwrap_or_else(|| app.default_window_icon().cloned().expect("an app icon")))
        .tooltip(concat!("Codenotch v", env!("CARGO_PKG_VERSION")))
        .menu(&crate::tray::menu::build(app, &lang)?)
        .show_menu_on_left_click(Platform.tray_menu_on_left_click())
        .on_tray_icon_event(|tray, ev| {
            if let tauri::tray::TrayIconEvent::DoubleClick { button: tauri::tray::MouseButton::Left, .. } = ev {
                ui::open_settings(tray.app_handle());
            }
        })
        .on_menu_event(|app, ev| match ev.id().as_ref() {
            "settings" => ui::open_settings(app),
            "refresh" => {
                ui::refresh_all(app);
                workers::reload_glyphs(app);
            }
            "quit" => {
                // Without this the next Claude Code hook finds no server and starts the app again
                let st = app.state::<AppState>();
                let _ = std::fs::write(st.dir.join(crate::storage::paths::USER_QUIT), b"");
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    ui::apply_visibility(app);
    ui::repaint_tray(app);
    workers::start_tray_updater(app);
    workers::start_sessions(app, port);
    workers::start_activity(app);
    workers::start_watchdog(app);
    Ok(())
}
