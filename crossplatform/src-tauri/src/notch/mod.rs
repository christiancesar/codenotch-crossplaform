//! The notch window: where it sits on the screen edge, which parts of it take input, the drag
//! along the edge, and the watchdog that closes the hover card. Works on a WebviewWindow; `app/`
//! decides when to call what and turns the callbacks into events.

pub mod drag;
pub mod hit_test;
pub mod placement;
pub mod watchdog;

use crate::config::NotchEdge;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

/// Logical depth of the notch window away from a left or right edge: the pill column plus room
/// for the hover card beside it. Collapsed and expanded are CSS states inside it.
pub const COLUMN_DEPTH: f64 = 340.0;
/// Its minimum length along that edge; the page asks for more (`set_notch_length`) when the pill
/// is longer, five providers at full size say.
pub const COLUMN_LENGTH: f64 = 460.0;
/// The same for the top and bottom edges: the pill row is deeper than the column and the card
/// hangs under (or over) it, so the window is shallower than it is long.
pub const ROW_DEPTH: f64 = 480.0;
pub const ROW_LENGTH: f64 = 520.0;

/// The smallest the window may be along `edge`.
pub fn min_length(edge: NotchEdge) -> f64 {
    if edge.is_horizontal() {
        ROW_LENGTH
    } else {
        COLUMN_LENGTH
    }
}

/// Logical (width, height) of the window on `edge` for `length` along it.
pub fn window_size(edge: NotchEdge, length: f64) -> (f64, f64) {
    if edge.is_horizontal() {
        (length, ROW_DEPTH)
    } else {
        (COLUMN_DEPTH, length)
    }
}

/// The length the page asked for, kept within the window minimum and the monitor's side.
pub fn clamp_length(asked: f64, monitor_side: f64, min: f64) -> f64 {
    if !asked.is_finite() {
        return min;
    }
    asked.ceil().max(min).min(monitor_side.max(min))
}

#[derive(Default)]
pub struct NotchState {
    /// Window-relative physical rectangles the page reported: the pill, plus the card when open
    pub hot: Mutex<Vec<[f64; 4]>>,
    pub expanded: AtomicBool,
    pub dragging: AtomicBool,
    /// WebView zoom currently applied by the DPR correction (1.0 = none)
    pub zoom: Mutex<f64>,
    /// Logical window length along the edge; 0 until the page asks, which reads as the minimum
    pub length: Mutex<f64>,
}

impl NotchState {
    pub fn length(&self, edge: NotchEdge) -> f64 {
        let l = *self.length.lock().unwrap();
        l.max(min_length(edge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_stays_between_the_minimum_and_the_monitor() {
        assert_eq!(clamp_length(300.0, 1080.0, COLUMN_LENGTH), COLUMN_LENGTH);
        assert_eq!(clamp_length(587.3, 1080.0, COLUMN_LENGTH), 588.0);
        assert_eq!(clamp_length(2000.0, 1080.0, COLUMN_LENGTH), 1080.0);
        assert_eq!(clamp_length(f64::NAN, 1080.0, COLUMN_LENGTH), COLUMN_LENGTH);
        // A monitor shorter than the minimum still gets the minimum
        assert_eq!(clamp_length(900.0, 400.0, COLUMN_LENGTH), COLUMN_LENGTH);
    }

    #[test]
    fn a_row_is_wide_and_a_column_tall() {
        assert_eq!(window_size(NotchEdge::Right, 600.0), (COLUMN_DEPTH, 600.0));
        assert_eq!(window_size(NotchEdge::Top, 600.0), (600.0, ROW_DEPTH));
        let s = NotchState::default();
        assert_eq!((s.length(NotchEdge::Left), s.length(NotchEdge::Bottom)), (COLUMN_LENGTH, ROW_LENGTH));
    }
}
