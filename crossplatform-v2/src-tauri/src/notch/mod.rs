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
/// Collapsed and expanded are CSS states inside it. The height is the minimum: the page asks for
/// more (`set_notch_height`) when the pill is taller, five providers at full size say.
pub const NOTCH_W: f64 = 340.0;
pub const NOTCH_H: f64 = 460.0;

/// The height the page asked for, kept within the window minimum and the monitor.
pub fn clamp_height(asked: f64, monitor_logical_h: f64) -> f64 {
    if !asked.is_finite() {
        return NOTCH_H;
    }
    asked.ceil().max(NOTCH_H).min(monitor_logical_h.max(NOTCH_H))
}

#[derive(Default)]
pub struct NotchState {
    /// Window-relative physical rectangles the page reported: the pill, plus the card when open
    pub hot: Mutex<Vec<[f64; 4]>>,
    pub expanded: AtomicBool,
    pub dragging: AtomicBool,
    /// WebView zoom currently applied by the DPR correction (1.0 = none)
    pub zoom: Mutex<f64>,
    /// Logical window height; 0 until the page asks, which reads as NOTCH_H
    pub height: Mutex<f64>,
}

impl NotchState {
    pub fn height(&self) -> f64 {
        let h = *self.height.lock().unwrap();
        if h > 0.0 { h } else { NOTCH_H }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_stays_between_the_minimum_and_the_monitor() {
        assert_eq!(clamp_height(300.0, 1080.0), NOTCH_H);
        assert_eq!(clamp_height(587.3, 1080.0), 588.0);
        assert_eq!(clamp_height(2000.0, 1080.0), 1080.0);
        assert_eq!(clamp_height(f64::NAN, 1080.0), NOTCH_H);
        // A monitor shorter than the minimum still gets the minimum
        assert_eq!(clamp_height(900.0, 400.0), NOTCH_H);
    }
}
