//! The typed IPC contract: every command and event the webviews can reach is registered here,
//! and `src/libs/ipc/bindings.ts` is generated from this list.

use crate::commands::*;
use tauri_specta::{collect_commands, collect_events, Builder};

pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            usage::get_usage,
            usage::refresh_usage,
            sessions::get_sessions,
            sessions::get_activity,
            sessions::focus_session,
            sessions::dismiss_session,
            notch::set_hot,
            notch::drag_begin,
            notch::report_dpr,
            notch::set_notch_height,
            notch::get_scale,
            notch::set_scale,
            notch::get_notch_slots,
            notch::set_notch_slots,
            notch::reset_notch_position,
            notch::starts_collapsed,
            tray::get_tray_options,
            tray::get_tray_config,
            tray::set_tray_config,
            tray::get_tray_preview,
            tray::get_app_icon,
            settings::get_lang,
            settings::set_lang,
            settings::get_ui_flags,
            settings::set_ui_flags,
            settings::get_autostart,
            settings::set_autostart,
            settings::get_hooks_installed,
            settings::set_hooks_installed,
            settings::open_settings,
            system::app_version,
            system::get_glyphs,
            system::open_data_dir,
            system::open_provider_page,
            system::log_js,
        ])
        .events(collect_events![
            super::events::UsageChanged,
            super::events::SessionsChanged,
            super::events::ActivityChanged,
            super::events::GlyphsChanged,
            super::events::PointerLeft,
            super::events::DragEnded,
            super::events::ScaleChanged,
            super::events::NotchSlotsChanged,
            super::events::LangChanged,
            super::events::Notice,
        ])
        // Epoch-ms timestamps and counters, all far below 2^53, so `number` is exact for them
        .dangerously_cast_bigints_to_number()
}

pub fn export(builder: &Builder<tauri::Wry>) -> Result<(), String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/libs/ipc/bindings.ts");
    builder.export(specta_typescript::Typescript::default(), path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    /// Regenerates the bindings on every `cargo test`; CI then fails on a diff, so a Rust change
    /// that alters the contract cannot land without the matching bindings.ts.
    #[test]
    fn bindings_export() {
        super::export(&super::builder()).unwrap();
    }
}
