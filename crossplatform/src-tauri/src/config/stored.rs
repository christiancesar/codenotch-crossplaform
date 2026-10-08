//! `config.json` exactly as it sits on disk. The model can change freely; this shape only
//! changes through a migration.

use super::model::{Config, NotchEdge, Slot, Theme, TrayMode, DEFAULT_PORT};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSlot {
    pub provider: String,
    #[serde(default)]
    pub window: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConfig {
    /// Top-level and named `port`: codenotch-hook finds it with a text scan
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_lang")]
    pub lang: String,
    /// Absent means "system", and "system" is never written: a config that never chose a theme
    /// saves byte for byte as before
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default = "default_notch_y")]
    pub notch_y: f64,
    /// Absent means "right", and "right" is never written, like `theme`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notch_edge: Option<String>,
    /// Absent until a screen is chosen; the primary stands in meanwhile
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notch_monitor: Option<String>,
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(default = "default_tray_mode")]
    pub tray_mode: String,
    #[serde(default)]
    pub tray_slots: Vec<StoredSlot>,
    #[serde(default)]
    pub notch_slots: Vec<StoredSlot>,
    #[serde(default = "yes")]
    pub notch_visible: bool,
    /// Absent means on, and on is never written: a config that never turned it off saves byte
    /// for byte as before
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notch_collapse: Option<bool>,
    #[serde(default = "yes")]
    pub tray_visible: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}
fn default_lang() -> String {
    "auto".into()
}
fn default_notch_y() -> f64 {
    0.5
}
fn default_scale() -> f64 {
    1.0
}
fn default_tray_mode() -> String {
    "numbers".into()
}
fn yes() -> bool {
    true
}

fn slots_in(v: Vec<StoredSlot>) -> Vec<Slot> {
    v.into_iter().map(|s| Slot { provider: s.provider, window: s.window }).collect()
}
fn slots_out(v: Vec<Slot>) -> Vec<StoredSlot> {
    v.into_iter().map(|s| StoredSlot { provider: s.provider, window: s.window }).collect()
}

impl From<StoredConfig> for Config {
    fn from(s: StoredConfig) -> Self {
        Config {
            port: s.port,
            lang: s.lang,
            theme: s.theme.as_deref().map(Theme::parse).unwrap_or_default(),
            notch_y: s.notch_y,
            notch_edge: s.notch_edge.as_deref().map(NotchEdge::parse).unwrap_or_default(),
            notch_monitor: s.notch_monitor.filter(|m| !m.is_empty()),
            scale: s.scale,
            tray_mode: TrayMode::parse(&s.tray_mode),
            tray_slots: slots_in(s.tray_slots),
            notch_slots: slots_in(s.notch_slots),
            notch_visible: s.notch_visible,
            notch_collapse: s.notch_collapse.unwrap_or(true),
            tray_visible: s.tray_visible,
            extra: s.extra,
        }
        .normalized()
    }
}

impl From<Config> for StoredConfig {
    fn from(c: Config) -> Self {
        StoredConfig {
            port: c.port,
            lang: c.lang,
            theme: (c.theme != Theme::System).then(|| c.theme.as_str().into()),
            notch_y: c.notch_y,
            notch_edge: (c.notch_edge != NotchEdge::Right).then(|| c.notch_edge.as_str().into()),
            notch_monitor: c.notch_monitor,
            scale: c.scale,
            tray_mode: c.tray_mode.as_str().into(),
            tray_slots: slots_out(c.tray_slots),
            notch_slots: slots_out(c.notch_slots),
            notch_visible: c.notch_visible,
            notch_collapse: (!c.notch_collapse).then_some(false),
            tray_visible: c.tray_visible,
            extra: c.extra,
        }
    }
}
