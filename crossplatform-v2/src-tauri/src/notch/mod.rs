//! The notch window: where it sits on the screen edge, which parts of it take input, the vertical
//! drag, and the watchdog that closes the hover card. Works on a WebviewWindow; `app/` decides
//! when to call what and turns the callbacks into events.

pub mod drag;
pub mod hit_test;
pub mod placement;
pub mod watchdog;

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

/// Logical size of the notch window: the pill column on the right plus room for the hover card.
/// The window is never resized; collapsed and expanded are CSS states inside it.
pub const NOTCH_W: f64 = 340.0;
pub const NOTCH_H: f64 = 460.0;

#[derive(Default)]
pub struct NotchState {
    /// Window-relative physical rectangles the page reported: the pill, plus the card when open
    pub hot: Mutex<Vec<[f64; 4]>>,
    pub expanded: AtomicBool,
    pub dragging: AtomicBool,
    /// WebView zoom currently applied by the DPR correction (1.0 = none)
    pub zoom: Mutex<f64>,
}
