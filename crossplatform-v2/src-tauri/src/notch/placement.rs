//! Pinned to the right edge of the primary monitor at the saved vertical ratio.

use super::{NOTCH_H, NOTCH_W};

/// Top-left for a window of physical size `win` whose centre sits at `ratio` of the monitor's
/// height, kept fully on the monitor.
pub fn position(mon_pos: (i32, i32), mon_size: (u32, u32), win: (u32, u32), ratio: f64) -> (i32, i32) {
    let (mw, mh) = (mon_size.0 as f64, mon_size.1 as f64);
    let (ww, wh) = (win.0 as f64, win.1 as f64);
    let half = (wh / 2.0).min(mh / 2.0);
    let centre = (mon_pos.1 as f64 + mh * ratio.clamp(0.0, 1.0)).clamp(mon_pos.1 as f64 + half, mon_pos.1 as f64 + mh - half);
    ((mon_pos.0 as f64 + mw - ww).round() as i32, (centre - wh / 2.0).round() as i32)
}

/// The saved ratio after a drag left the window's top at `top`.
pub fn ratio_after_drag(top: i32, win_h: u32, mon_y: i32, mon_h: u32) -> f64 {
    ((top + win_h as i32 / 2 - mon_y) as f64 / mon_h as f64).clamp(0.0, 1.0)
}

/// Sizes the window from the monitor's scale and places it. The physical size is pinned straight
/// from the monitor's scale factor: with two monitors at different scales the window could end up
/// converted with the other monitor's factor, leaving the WebView ~256 logical px wide.
pub fn place(w: &tauri::WebviewWindow, ratio: f64) -> Option<(i32, i32)> {
    let mon = w.primary_monitor().ok().flatten()?;
    let s = mon.scale_factor();
    let target = tauri::PhysicalSize::new((NOTCH_W * s).round() as u32, (NOTCH_H * s).round() as u32);
    let _ = w.set_size(target);
    // The measured size can still be 0x0 before the window is mapped; the target is what it will be
    let size = w.outer_size().ok().filter(|z| z.width > 0 && z.height > 0).unwrap_or(target);
    let (x, y) = position((mon.position().x, mon.position().y), (mon.size().width, mon.size().height), (size.width, size.height), ratio);
    let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
    crate::diagnostics::log(&format!(
        "notch placed: pos=({x},{y}) size=({}x{}) scale={s} monitor=({},{} {}x{}) gdk={}",
        size.width,
        size.height,
        mon.position().x,
        mon.position().y,
        mon.size().width,
        mon.size().height,
        std::env::var("GDK_BACKEND").unwrap_or_default()
    ));
    Some((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_right_and_clamped_vertically() {
        // The Ubuntu 26.04 machine: 1920x1080 at 100 %
        assert_eq!(position((0, 0), (1920, 1080), (340, 460), 0.5), (1580, 310));
        assert_eq!(position((0, 0), (1920, 1080), (340, 460), 0.0), (1580, 0));
        assert_eq!(position((0, 0), (1920, 1080), (340, 460), 1.0), (1580, 620));
        // A second monitor to the left and above
        assert_eq!(position((-2560, -200), (2560, 1440), (510, 690), 0.5), (-510, 175));
    }

    #[test]
    fn drag_ratio_round_trips_through_position() {
        let r = ratio_after_drag(385, 460, 0, 1080);
        assert_eq!(position((0, 0), (1920, 1080), (340, 460), r).1, 385);
    }
}
