//! Actions shared by commands, the tray menu and setup: they touch windows and the tray, so they
//! live here, next to the AppHandle.

use super::events::{LangChanged, PointerLeft};
use super::state::AppState;
use crate::config::{Slot, TrayMode};
use crate::platform::{Platform, Window};
use crate::providers::ProviderId;
use crate::tray::{menu, readings, render};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub const NOTCH: &str = "notch";
pub const SETTINGS: &str = "settings";

pub fn place_notch(app: &AppHandle) {
    let st = app.state::<AppState>();
    let ratio = st.config.lock().unwrap().notch_y;
    if let Some(w) = app.get_webview_window(NOTCH) {
        crate::notch::placement::place(&w, ratio, st.notch.height());
    }
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
    menu::refresh(app, lang.clone());
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
