//! Pinned to the chosen edge of the primary monitor at the saved ratio along it.

use super::window_size;
use crate::config::NotchEdge;

/// Top-left for a window of physical size `win` flush with `edge`, its centre at `ratio` of the
/// monitor's side along that edge, kept fully on the monitor.
pub fn position(edge: NotchEdge, mon_pos: (i32, i32), mon_size: (u32, u32), win: (u32, u32), ratio: f64) -> (i32, i32) {
    let (mx, my) = (mon_pos.0 as f64, mon_pos.1 as f64);
    let (mw, mh) = (mon_size.0 as f64, mon_size.1 as f64);
    let (ww, wh) = (win.0 as f64, win.1 as f64);
    // Centre along an axis that starts at `start`, is `side` long and holds a window `len` long
    let along = |start: f64, side: f64, len: f64| {
        let half = (len / 2.0).min(side / 2.0);
        (start + side * ratio.clamp(0.0, 1.0)).clamp(start + half, start + side - half) - len / 2.0
    };
    let (x, y) = match edge {
        NotchEdge::Right => (mx + mw - ww, along(my, mh, wh)),
        NotchEdge::Left => (mx, along(my, mh, wh)),
        NotchEdge::Top => (along(mx, mw, ww), my),
        NotchEdge::Bottom => (along(mx, mw, ww), my + mh - wh),
    };
    (x.round() as i32, y.round() as i32)
}

/// The saved ratio after a drag left the window's top-left at `pos`.
pub fn ratio_after_drag(edge: NotchEdge, pos: (i32, i32), win: (u32, u32), mon_pos: (i32, i32), mon_size: (u32, u32)) -> f64 {
    let (lead, len, start, side) = if edge.is_horizontal() {
        (pos.0, win.0, mon_pos.0, mon_size.0)
    } else {
        (pos.1, win.1, mon_pos.1, mon_size.1)
    };
    ((lead + len as i32 / 2 - start) as f64 / side as f64).clamp(0.0, 1.0)
}

/// Sizes the window from the monitor's scale and places it. The physical size is pinned straight
/// from the monitor's scale factor: with two monitors at different scales the window could end up
/// converted with the other monitor's factor, leaving the WebView ~256 logical px wide.
pub fn place(w: &tauri::WebviewWindow, edge: NotchEdge, ratio: f64, length: f64) -> Option<(i32, i32)> {
    let mon = w.primary_monitor().ok().flatten()?;
    let s = mon.scale_factor();
    let (lw, lh) = window_size(edge, length);
    let target = tauri::PhysicalSize::new((lw * s).round() as u32, (lh * s).round() as u32);
    let _ = w.set_size(target);
    // The measured size can still be 0x0 before the window is mapped, and lags a resize just
    // asked for: the target is what it will be
    let size = target;
    let (x, y) = position(edge, (mon.position().x, mon.position().y), (mon.size().width, mon.size().height), (size.width, size.height), ratio);
    let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
    crate::diagnostics::log(&format!(
        "notch placed: edge={} pos=({x},{y}) size=({}x{}) scale={s} monitor=({},{} {}x{}) gdk={}",
        edge.as_str(),
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
        assert_eq!(position(NotchEdge::Right, (0, 0), (1920, 1080), (340, 460), 0.5), (1580, 310));
        assert_eq!(position(NotchEdge::Right, (0, 0), (1920, 1080), (340, 460), 0.0), (1580, 0));
        assert_eq!(position(NotchEdge::Right, (0, 0), (1920, 1080), (340, 460), 1.0), (1580, 620));
        // A second monitor to the left and above
        assert_eq!(position(NotchEdge::Right, (-2560, -200), (2560, 1440), (510, 690), 0.5), (-510, 175));
    }

    #[test]
    fn every_edge_is_flush_and_centred_along_it() {
        assert_eq!(position(NotchEdge::Left, (0, 0), (1920, 1080), (340, 460), 0.5), (0, 310));
        assert_eq!(position(NotchEdge::Top, (0, 0), (1920, 1080), (520, 380), 0.5), (700, 0));
        assert_eq!(position(NotchEdge::Bottom, (0, 0), (1920, 1080), (520, 380), 0.5), (700, 700));
        // Kept on the monitor at the ends
        assert_eq!(position(NotchEdge::Top, (0, 0), (1920, 1080), (520, 380), 0.0), (0, 0));
        assert_eq!(position(NotchEdge::Bottom, (100, 0), (1920, 1080), (520, 380), 1.0), (1500, 700));
    }

    #[test]
    fn drag_ratio_round_trips_through_position() {
        let r = ratio_after_drag(NotchEdge::Right, (1580, 385), (340, 460), (0, 0), (1920, 1080));
        assert_eq!(position(NotchEdge::Right, (0, 0), (1920, 1080), (340, 460), r).1, 385);
        let r = ratio_after_drag(NotchEdge::Top, (900, 0), (520, 380), (0, 0), (1920, 1080));
        assert_eq!(position(NotchEdge::Top, (0, 0), (1920, 1080), (520, 380), r).0, 900);
    }
}
