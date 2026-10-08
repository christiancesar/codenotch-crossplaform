//! Starts every background loop and wires its callback to state and events.

use super::events::{ActivityChanged, GlyphsChanged, MonitorsChanged, PointerLeft, SessionsChanged, UsageChanged};
use super::state::{AppState, ProviderSlot};
use crate::providers::{scheduler, UsageProvider};
use crate::sessions::{hook_server, sweep, watcher, HookEvent};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

/// Provider threads start before AppState exists (it holds their handles), so they read the
/// session store through this shared Arc.
pub fn start_providers(app: &AppHandle, dir: &std::path::Path, sessions: Arc<Mutex<crate::sessions::Store>>) -> Vec<ProviderSlot> {
    crate::providers::all()
        .into_iter()
        .map(|provider| {
            let a = app.clone();
            let s = sessions.clone();
            let handle = scheduler::spawn(
                provider.clone(),
                dir.to_path_buf(),
                Arc::new(move || s.lock().unwrap().is_active()),
                Arc::new(move |id, snap| {
                    let _ = UsageChanged { provider: id, snapshot: snap.clone() }.emit(&a);
                }),
            );
            ProviderSlot { provider, handle }
        })
        .collect()
}

fn broadcast_sessions(app: &AppHandle) {
    let snap = app.state::<AppState>().sessions.lock().unwrap().snapshot();
    let _ = SessionsChanged(snap).emit(app);
}

pub fn start_sessions(app: &AppHandle, port: u16) {
    let a = app.clone();
    let sink: hook_server::Sink = Arc::new(move |ev: HookEvent| {
        // The first pushes go to watch.log so `doctor` can tell "nothing arrived" from "arrived
        // but was not shown"; then silence, to keep the log small
        static LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let line = (LOGGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 30)
            .then(|| format!("push {} src={:?} session={} cwd={}", ev.e, ev.src, ev.session_id, ev.cwd));
        let changed = a.state::<AppState>().sessions.lock().unwrap().apply(ev, crate::support::time::now_ms());
        if let Some(line) = line {
            crate::diagnostics::watch_log(&format!("{line} changed={changed}"));
        }
        if changed {
            broadcast_sessions(&a);
        }
    });
    hook_server::start(port, sink.clone());
    watcher::start(sink);
    let a = app.clone();
    sweep::start(app.state::<AppState>().sessions.clone(), Arc::new(move || broadcast_sessions(&a)));
}

pub fn start_activity(app: &AppHandle) {
    let providers: Vec<Arc<dyn UsageProvider>> = app.state::<AppState>().providers.iter().map(|p| p.provider.clone()).collect();
    let a = app.clone();
    crate::providers::activity::start(
        providers,
        Arc::new(move |found| {
            *a.state::<AppState>().activity.lock().unwrap() = found.to_vec();
            let _ = ActivityChanged(found.to_vec()).emit(&a);
        }),
    );
}

pub fn reload_glyphs(app: &AppHandle) {
    let a = app.clone();
    std::thread::spawn(move || {
        let m = crate::glyphs::collect();
        *a.state::<AppState>().glyphs.lock().unwrap() = m.clone();
        let _ = GlyphsChanged(m).emit(&a);
    });
}

/// Screens come and go (a dock, a projector) and the OS sends no event the app could listen to on
/// both platforms, so the list is compared every couple of seconds. A change moves the notch
/// (back to the chosen screen, or to the primary while it is away) and redraws the picker.
pub fn start_monitor_watch(app: &AppHandle) {
    let a = app.clone();
    std::thread::Builder::new()
        .name("monitor-watch".into())
        .spawn(move || {
            use crate::platform::{Platform, Processes};
            Platform.lower_current_thread_priority();
            let mut last = crate::notch::monitors::list(&a);
            loop {
                std::thread::sleep(Duration::from_secs(2));
                let now = crate::notch::monitors::list(&a);
                if now != last {
                    crate::diagnostics::log(&format!("monitors changed: {}", now.iter().map(|m| m.id.as_str()).collect::<Vec<_>>().join(", ")));
                    super::ui::place_notch(&a);
                    let _ = MonitorsChanged(now.clone()).emit(&a);
                    last = now;
                }
            }
        })
        .expect("failed to start the monitor watch");
}

pub fn start_watchdog(app: &AppHandle) {
    let Some(w) = app.get_webview_window(super::ui::NOTCH) else { return };
    let a = app.clone();
    crate::notch::watchdog::start(w, app.state::<AppState>().notch.clone(), move || {
        let _ = PointerLeft.emit(&a);
    });
}

/// Repaints the tray when a reading or the settings change. Every 2 s, but it only touches the
/// icon when something moved.
pub fn start_tray_updater(app: &AppHandle) {
    let a = app.clone();
    std::thread::spawn(move || {
        let mut last = None;
        loop {
            std::thread::sleep(Duration::from_secs(2));
            let st = a.state::<AppState>();
            let (mode, slots) = {
                let c = st.config.lock().unwrap();
                (c.tray_mode, c.tray_slots.clone())
            };
            let values = super::ui::tray_values(&st, &slots);
            let key = (mode, slots.clone(), values.clone());
            if last.as_ref() != Some(&key) {
                super::ui::paint_tray(&a, mode, &slots, &values);
                last = Some(key);
            }
        }
    });
}
