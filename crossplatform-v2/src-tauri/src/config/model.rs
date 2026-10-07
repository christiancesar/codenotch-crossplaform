use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;

/// How small the notch may be drawn, as a multiple of its designed size. Below roughly 0.4 the
/// rings stop being readable at 100 % display scaling.
pub const SCALE_MIN: f64 = 0.40;
pub const SCALE_MAX: f64 = 1.00;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum TrayMode {
    /// The plain mark
    Off,
    /// Up to two readings as digits
    Numbers,
    /// A column per reading
    Bars,
}

/// The app's colours: the settings window, the notch and its card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Follows the desktop's light or dark preference
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    /// Unknown strings follow the system
    pub fn parse(s: &str) -> Theme {
        match s {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

impl TrayMode {
    pub fn as_str(self) -> &'static str {
        match self {
            TrayMode::Off => "off",
            TrayMode::Numbers => "numbers",
            TrayMode::Bars => "bars",
        }
    }

    /// Unknown strings fall back to the plain mark, the one mode that never shows a wrong number.
    pub fn parse(s: &str) -> TrayMode {
        match s {
            "numbers" => TrayMode::Numbers,
            "bars" => TrayMode::Bars,
            _ => TrayMode::Off,
        }
    }
}

/// One reading shown on the tray icon or as a notch ring. `window` is a window id as the provider
/// reports it ("session", "weekly_all"...), or empty / "top" for whichever window is fullest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Slot {
    pub provider: String,
    pub window: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub port: u16,
    /// "auto" | "en" | "zh" | "ja" | "ko" | "pt"
    pub lang: String,
    pub theme: Theme,
    /// Window centre as a fraction of the primary monitor's height, 0 = top, 1 = bottom
    pub notch_y: f64,
    pub scale: f64,
    pub tray_mode: TrayMode,
    pub tray_slots: Vec<Slot>,
    /// Empty means every provider with something to report
    pub notch_slots: Vec<Slot>,
    pub notch_visible: bool,
    pub tray_visible: bool,
    /// Keys this version does not use (v0.3's `bar_*`, `drag_enabled`, `*_providers`, or anything
    /// a newer build wrote). Kept so saving never deletes someone else's setting.
    pub extra: Map<String, Value>,
}

pub const DEFAULT_PORT: u16 = 48666;

impl Default for Config {
    fn default() -> Self {
        Config {
            port: DEFAULT_PORT,
            lang: "auto".into(),
            theme: Theme::System,
            notch_y: 0.5,
            scale: 1.0,
            // A fresh install shows readings straight away; upgrades keep their mark (migrate.rs)
            tray_mode: TrayMode::Numbers,
            tray_slots: ["claude", "codex"]
                .into_iter()
                .map(|p| Slot { provider: p.into(), window: String::new() })
                .collect(),
            notch_slots: Vec::new(),
            notch_visible: true,
            tray_visible: true,
            extra: Map::new(),
        }
    }
}

impl Config {
    /// Also writes v0.3's `tray_providers`, so going back to v0.3 still shows the same providers.
    pub fn set_tray_slots(&mut self, slots: Vec<Slot>) {
        self.extra.insert("tray_providers".into(), Value::from(slots.iter().map(|s| s.provider.clone()).collect::<Vec<_>>()));
        self.tray_slots = slots;
    }

    /// Same for v0.3's `notch_providers`.
    pub fn set_notch_slots(&mut self, slots: Vec<Slot>) {
        self.extra.insert("notch_providers".into(), Value::from(slots.iter().map(|s| s.provider.clone()).collect::<Vec<_>>()));
        self.notch_slots = slots;
    }

    /// Rules that hold for any file, hand-edited ones included.
    pub fn normalized(mut self) -> Self {
        // Both hidden would leave the app unreachable: no pill, no tray icon, no way to settings
        if !self.notch_visible && !self.tray_visible {
            self.tray_visible = true;
        }
        self.scale = if self.scale.is_finite() { self.scale.clamp(SCALE_MIN, SCALE_MAX) } else { 1.0 };
        self.notch_y = if self.notch_y.is_finite() { self.notch_y.clamp(0.0, 1.0) } else { 0.5 };
        self
    }
}
