//! Deliberately short: settings, refresh, where the notch sits, quit. Everything else lives in the
//! settings window, which can explain each choice. The notch position is here too because moving
//! it is something you try a few times in a row, and Settings would be in the way.

use crate::app::state::AppState;
use crate::config::NotchEdge;
use crate::i18n::tr;
use tauri::menu::{CheckMenuItemBuilder, Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Manager, Wry};

pub const TRAY_ID: &str = "main";
/// Menu ids of the position choices: `edge:<edge>`
const EDGE_PREFIX: &str = "edge:";

/// The edge a menu id chooses, if it is one of the position items.
pub fn edge_of(id: &str) -> Option<NotchEdge> {
    id.strip_prefix(EDGE_PREFIX).map(NotchEdge::parse)
}

pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let (lang, current) = {
        let c = app.state::<AppState>().config.lock().unwrap().clone();
        (c.lang, c.notch_edge)
    };
    let lang = lang.as_str();
    let mut position = SubmenuBuilder::new(app, tr(lang, "notchPosition"));
    for edge in NotchEdge::ALL {
        position = position.item(
            &CheckMenuItemBuilder::with_id(format!("{EDGE_PREFIX}{}", edge.as_str()), tr(lang, edge.as_str()))
                .checked(edge == current)
                .build(app)?,
        );
    }
    MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id("settings", tr(lang, "settings")).build(app)?)
        .item(&MenuItemBuilder::with_id("refresh", tr(lang, "refresh")).build(app)?)
        .item(&position.build()?)
        .separator()
        .item(&MenuItemBuilder::with_id("quit", tr(lang, "quit")).build(app)?)
        .build()
}

/// Rebuilt after a language or position change, so labels and the check mark follow. Always on
/// the main thread: a menu built or swapped from another thread leaves the tray holding a menu
/// that never opens again, and those changes come from the settings window's thread.
pub fn refresh(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let (Some(tray), Ok(menu)) = (handle.tray_by_id(TRAY_ID), build(&handle)) {
            let _ = tray.set_menu(Some(menu));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_ids_name_their_edge() {
        assert_eq!(edge_of("edge:top"), Some(NotchEdge::Top));
        assert_eq!(edge_of("edge:bottom"), Some(NotchEdge::Bottom));
        assert_eq!(edge_of("settings"), None);
    }
}
