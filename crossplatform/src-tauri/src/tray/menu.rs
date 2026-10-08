//! Deliberately short: settings, refresh, quit. Everything else lives in the settings window,
//! which can explain each choice.

use crate::i18n::tr;
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder};
use tauri::{AppHandle, Wry};

pub const TRAY_ID: &str = "main";

pub fn build(app: &AppHandle, lang: &str) -> tauri::Result<Menu<Wry>> {
    MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id("settings", tr(lang, "settings")).build(app)?)
        .item(&MenuItemBuilder::with_id("refresh", tr(lang, "refresh")).build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("quit", tr(lang, "quit")).build(app)?)
        .build()
}

/// Always on the main thread: a menu built or swapped from another thread leaves the tray holding
/// a menu that never opens again, and a language change (which triggers this) comes from the
/// settings window's thread.
pub fn refresh(app: &AppHandle, lang: String) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let (Some(tray), Ok(menu)) = (handle.tray_by_id(TRAY_ID), build(&handle, &lang)) {
            let _ = tray.set_menu(Some(menu));
        }
    });
}
