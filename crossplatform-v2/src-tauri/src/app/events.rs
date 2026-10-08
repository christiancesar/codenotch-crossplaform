//! Typed events sent to the webviews. Names are snake_case on both sides.

use crate::config::Slot;
use crate::glyphs::Glyph;
use crate::providers::{Activity, ProviderId, UsageSnapshot};
use crate::sessions::SessionsSnapshot;
use serde::Serialize;
use specta::Type;
use std::collections::HashMap;
use tauri_specta::Event;

#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "usage")]
pub struct UsageChanged {
    pub provider: ProviderId,
    pub snapshot: UsageSnapshot,
}

#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "sessions")]
pub struct SessionsChanged(pub SessionsSnapshot);

#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "activity")]
pub struct ActivityChanged(pub Vec<Activity>);

#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "glyphs")]
pub struct GlyphsChanged(pub HashMap<String, Glyph>);

/// The cursor left the notch (or focus moved away): close the card
#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "pointer_left")]
pub struct PointerLeft;

#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "drag_end")]
pub struct DragEnded {
    pub moved: bool,
}

/// The notch size changed (possibly from the settings window)
#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "scale")]
pub struct ScaleChanged(pub f64);

#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "notch_slots")]
pub struct NotchSlotsChanged(pub Vec<Slot>);

#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "lang")]
pub struct LangChanged {
    pub lang: String,
    pub resolved: String,
}

/// The theme changed in Settings: every window applies it
#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "theme")]
pub struct ThemeChanged(pub crate::config::Theme);

/// The fold-when-idle switch changed in Settings: the notch follows it at once
#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "notch_collapse")]
pub struct NotchCollapseChanged(pub bool);

/// Something the user should read on the notch (a refused second instance, say)
#[derive(Debug, Clone, Serialize, Type, Event)]
#[tauri_specta(event_name = "notice")]
pub struct Notice(pub String);
