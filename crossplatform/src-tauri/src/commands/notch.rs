use crate::app::events::{DragEnded, NotchSlotsChanged, ScaleChanged};
use crate::app::state::AppState;
use crate::config::{NotchEdge, Slot, SCALE_MAX, SCALE_MIN};
use crate::notch::monitors::MonitorInfo;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

/// Hot rectangles in window-relative physical pixels (the page multiplies by its own DPR): the
/// pill, plus the card while it is open. Empty is click-through.
#[tauri::command]
#[specta::specta]
pub fn set_hot(app: AppHandle, state: State<AppState>, rects: Vec<[f64; 4]>, expanded: bool) {
    crate::app::ui::apply_input_region(&app, &rects);
    *state.notch.hot.lock().unwrap() = rects;
    let was = state.notch.expanded.swap(expanded, Ordering::Relaxed);
    // Opening the card asks providers whose read is cached (Antigravity's CLI) for a fresh one
    if expanded && !was {
        for p in state.providers.iter().filter(|p| p.provider.refresh_on_hover()) {
            p.handle.wake.store(true, Ordering::Relaxed);
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn drag_begin(app: AppHandle, state: State<AppState>) {
    let Some(w) = app.get_webview_window(crate::app::ui::NOTCH) else { return };
    let Some(mon) = crate::app::ui::notch_monitor(&app) else { return };
    let a = app.clone();
    let edge = state.config.lock().unwrap().notch_edge;
    crate::notch::drag::begin(w, mon, state.notch.clone(), edge, move |ratio| {
        if let Some(r) = ratio {
            a.state::<AppState>().update_config(|c| c.notch_y = r);
        }
        let _ = DragEnded { moved: ratio.is_some() }.emit(&a);
    });
}

/// The page's pill no longer fits (five providers at full size): grow the window to `length`
/// logical px along its edge, at least the minimum and at most the monitor's side, keeping the
/// saved centre.
#[tauri::command]
#[specta::specta]
pub fn set_notch_length(app: AppHandle, state: State<AppState>, length: f64) {
    let edge = state.config.lock().unwrap().notch_edge;
    let min = crate::notch::min_length(edge);
    let side = crate::app::ui::notch_monitor(&app)
        .map(|m| if edge.is_horizontal() { m.size().width } else { m.size().height } as f64 / m.scale_factor())
        .unwrap_or(min);
    let l = crate::notch::clamp_length(length, side, min);
    if (state.notch.length(edge) - l).abs() < 1.0 {
        return;
    }
    *state.notch.length.lock().unwrap() = l;
    crate::app::ui::place_notch(&app);
}

/// The connected screens, as the OS arranges them
#[tauri::command]
#[specta::specta]
pub fn get_monitors(app: AppHandle) -> Vec<MonitorInfo> {
    crate::notch::monitors::list(&app)
}

/// The stored screen; null until one is chosen (the primary stands in)
#[tauri::command]
#[specta::specta]
pub fn get_notch_monitor(state: State<AppState>) -> Option<String> {
    state.config.lock().unwrap().notch_monitor.clone()
}

#[tauri::command]
#[specta::specta]
pub fn set_notch_monitor(app: AppHandle, id: String) {
    crate::app::ui::set_notch_monitor(&app, id);
}

#[tauri::command]
#[specta::specta]
pub fn get_notch_edge(state: State<AppState>) -> NotchEdge {
    state.config.lock().unwrap().notch_edge
}

/// Moves the notch to another screen edge (Settings; the tray menu does the same)
#[tauri::command]
#[specta::specta]
pub fn set_notch_edge(app: AppHandle, edge: NotchEdge) {
    crate::app::ui::set_notch_edge(&app, edge);
}

/// With two monitors at different scales the WebView can pick the other monitor's DPR, leaving
/// the page 255 CSS px wide instead of 340. The page reports its DPR, and a zoom pulls it back
/// to the primary monitor's scale; at most three corrections, in case it never follows.
#[tauri::command]
#[specta::specta]
pub fn report_dpr(app: AppHandle, state: State<AppState>, dpr: f64, width: f64, height: f64) {
    let Some(win) = app.get_webview_window(crate::app::ui::NOTCH) else { return };
    let want = crate::app::ui::notch_monitor(&app).map(|m| m.scale_factor()).unwrap_or_else(|| win.scale_factor().unwrap_or(1.0));
    let mut z = state.notch.zoom.lock().unwrap();
    if *z <= 0.0 {
        *z = 1.0;
    }
    let target = if dpr > 0.0 { want / (dpr / *z) } else { 1.0 };
    crate::diagnostics::log(&format!("dpr report: dpr={dpr:.3} viewport={width:.0}x{height:.0} monitor_scale={want:.3} zoom={:.3} -> {target:.3}", *z));
    static APPLIED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    if (dpr - want).abs() > 0.02 && (target - *z).abs() > 0.01 && (0.25..=4.0).contains(&target) && APPLIED.fetch_add(1, Ordering::Relaxed) < 3 && win.set_zoom(target).is_ok() {
        *z = target;
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_scale(state: State<AppState>) -> f64 {
    state.config.lock().unwrap().scale
}

/// Only the value is stored: the page scales the pill with CSS, so the window never resizes and
/// the slider does not move under the cursor.
#[tauri::command]
#[specta::specta]
pub fn set_scale(app: AppHandle, state: State<AppState>, scale: f64) {
    let v = state.update_config(|c| {
        c.scale = scale.clamp(SCALE_MIN, SCALE_MAX);
        c.scale
    });
    let _ = ScaleChanged(v).emit(&app);
}

#[tauri::command]
#[specta::specta]
pub fn get_notch_slots(state: State<AppState>) -> Vec<Slot> {
    state.config.lock().unwrap().notch_slots.clone()
}

#[tauri::command]
#[specta::specta]
pub fn set_notch_slots(app: AppHandle, state: State<AppState>, slots: Vec<Slot>) {
    let list = state.update_config(|c| {
        c.set_notch_slots(slots);
        c.notch_slots.clone()
    });
    let _ = NotchSlotsChanged(list).emit(&app);
}

#[tauri::command]
#[specta::specta]
pub fn reset_notch_position(app: AppHandle, state: State<AppState>) {
    state.update_config(|c| c.notch_y = 0.5);
    crate::app::ui::place_notch(&app);
    if let Some(w) = app.get_webview_window(crate::app::ui::NOTCH) {
        let _ = w.show();
    }
}

/// Whether the idle notch folds to the thin tab on the edge (Settings, on by default).
#[tauri::command]
#[specta::specta]
pub fn starts_collapsed(state: State<AppState>) -> bool {
    state.config.lock().unwrap().notch_collapse
}
