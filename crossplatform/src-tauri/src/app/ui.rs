//! Actions shared by commands, the tray menu and setup: they touch windows and the tray, so they
//! live here, next to the AppHandle.

use super::events::{LangChanged, NotchEdgeChanged, PointerLeft};
use super::state::AppState;
use crate::config::{NotchEdge, Slot, TrayMode};
use crate::platform::{Platform, Window};
use crate::providers::ProviderId;
use crate::tray::{menu, readings, render};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub const NOTCH: &str = "notch";
pub const SETTINGS: &str = "settings";

/// The screen the notch is on now: the chosen one while connected, the primary otherwise.
pub fn notch_monitor(app: &AppHandle) -> Option<tauri::Monitor> {
    let stored = app.state::<AppState>().config.lock().unwrap().notch_monitor.clone();
    crate::notch::monitors::chosen(app, stored.as_deref())
}

pub fn place_notch(app: &AppHandle) {
    let st = app.state::<AppState>();
    let (edge, ratio, spot) = {
        let c = st.config.lock().unwrap();
        (c.notch_edge, c.notch_y, c.notch_widget)
    };
    if let (Some(w), Some(mon)) = (app.get_webview_window(NOTCH), notch_monitor(app)) {
        crate::notch::placement::place(&w, &mon, edge, ratio, spot, st.notch.length(edge));
    }
    apply_notch_layer(app);
}

/// On an edge the notch stays over every window. The centre widget stays under them, out of the
/// way, and comes up only while its card is open, so the card can be read; it goes back down
/// when the card closes.
pub fn apply_notch_layer(app: &AppHandle) {
    let st = app.state::<AppState>();
    let widget = st.config.lock().unwrap().notch_edge.is_widget();
    let behind = widget && !st.notch.expanded.load(Ordering::Relaxed);
    if let Some(w) = app.get_webview_window(NOTCH) {
        // Unset the other first: a window asked to be both keeps whichever the OS saw last
        if behind {
            let _ = w.set_always_on_top(false);
            let _ = w.set_always_on_bottom(true);
        } else {
            let _ = w.set_always_on_bottom(false);
            let _ = w.set_always_on_top(true);
        }
    }
}

/// Moves the notch to another screen (Settings). The length the page asked for is kept within the
/// new screen's side by the placement; the card closes, since it was laid out for the old spot.
pub fn set_notch_monitor(app: &AppHandle, id: String) {
    let st = app.state::<AppState>();
    let changed = st.update_config(|c| c.notch_monitor.replace(id.clone()).as_deref() != Some(id.as_str()));
    if !changed {
        return;
    }
    st.notch.expanded.store(false, Ordering::Relaxed);
    let _ = PointerLeft.emit(app);
    place_notch(app);
}

/// Moves the notch to another screen edge, from Settings or the tray. The length the page asked
/// for was for the old orientation, so it starts from the minimum until the page asks again; the
/// card closes, since it was laid out for the old edge.
pub fn set_notch_edge(app: &AppHandle, edge: NotchEdge) {
    let st = app.state::<AppState>();
    let was = st.update_config(|c| std::mem::replace(&mut c.notch_edge, edge));
    if was == edge {
        return;
    }
    *st.notch.length.lock().unwrap() = 0.0;
    st.notch.expanded.store(false, Ordering::Relaxed);
    let _ = PointerLeft.emit(app);
    place_notch(app);
    let _ = NotchEdgeChanged(edge).emit(app);
    menu::refresh(app);
}

/// Puts the two visibility switches into effect.
pub fn apply_visibility(app: &AppHandle) {
    let st = app.state::<AppState>();
    let (notch, tray) = {
        let c = st.config.lock().unwrap();
        (c.notch_visible, c.tray_visible)
    };
    if let Some(w) = app.get_webview_window(NOTCH) {
        if notch {
            let _ = w.show();
            place_notch(app);
        } else {
            // Always reopen collapsed, never mid-expand
            st.notch.expanded.store(false, Ordering::Relaxed);
            let _ = PointerLeft.emit(app);
            let _ = w.hide();
        }
    }
    if let Some(t) = app.tray_by_id(menu::TRAY_ID) {
        let _ = t.set_visible(tray);
    }
}

pub fn label(id: &str) -> &'static str {
    match id {
        "codex" => "Codex",
        "cursor" => "Cursor",
        "gemini" => "Antigravity",
        "opencode" => "OpenCode",
        _ => "Claude",
    }
}

/// Current values for the tray slots.
pub fn tray_values(st: &AppState, slots: &[Slot]) -> Vec<Option<u32>> {
    slots
        .iter()
        .map(|s| {
            let snap = ProviderId::ALL.into_iter().find(|id| id.as_str() == s.provider).map(|id| st.snapshot(id)).unwrap_or_default();
            readings::for_slot(s, &snap)
        })
        .collect()
}

/// Draws the icon and writes the tooltip, which lists every slot, including any the digit
/// layout could not fit.
pub fn paint_tray(app: &AppHandle, mode: TrayMode, slots: &[Slot], values: &[Option<u32>]) {
    let Some(tray) = app.tray_by_id(menu::TRAY_ID) else { return };
    let icon = match mode {
        TrayMode::Numbers if !values.is_empty() => Some(render::numbers(values)),
        TrayMode::Bars if !values.is_empty() => Some(render::bars(values)),
        _ => render::app_mark(),
    };
    if let Some(icon) = icon {
        if let Err(e) = tray.set_icon(Some(icon)) {
            crate::diagnostics::log(&format!("tray: set_icon failed mode={} values={values:?}: {e}", mode.as_str()));
        }
    }
    let parts: Vec<String> = slots
        .iter()
        .zip(values)
        .map(|(s, v)| format!("{} {}", label(&s.provider), v.map(|p| format!("{p}%")).unwrap_or_else(|| "—".into())))
        .collect();
    let tip = if parts.is_empty() { concat!("Codenotch v", env!("CARGO_PKG_VERSION")).to_string() } else { format!("Codenotch — {}", parts.join(" · ")) };
    let _ = tray.set_tooltip(Some(&tip));
}

pub fn repaint_tray(app: &AppHandle) {
    let st = app.state::<AppState>();
    let (mode, slots) = {
        let c = st.config.lock().unwrap();
        (c.tray_mode, c.tray_slots.clone())
    };
    let values = tray_values(&st, &slots);
    paint_tray(app, mode, &slots, &values);
}

pub fn apply_lang(app: &AppHandle, lang: String) {
    let st = app.state::<AppState>();
    st.update_config(|c| c.lang = lang.clone());
    menu::refresh(app);
    let _ = LangChanged { resolved: crate::i18n::resolve(&lang).into(), lang }.emit(app);
}

pub fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(SETTINGS) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Every provider reads again now. A rate-limit deadline still holds: the scheduler waits it out.
pub fn refresh_all(app: &AppHandle) {
    for p in &app.state::<AppState>().providers {
        p.handle.wake.store(true, Ordering::Relaxed);
    }
}

/// Clicks reach the notch only where the page says it has something.
pub fn apply_input_region(app: &AppHandle, rects: &[[f64; 4]]) {
    if Platform.shapes_input() {
        if let Some(w) = app.get_webview_window(NOTCH) {
            Platform.set_input_region(&w, crate::notch::hit_test::input_region(rects));
        }
    }
}
