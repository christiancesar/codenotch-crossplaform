//! Pinned to the chosen edge of the chosen screen at the saved ratio along it, or, for the centre
//! widget, at its saved spot on that screen.

use super::window_size;
use crate::config::NotchEdge;

/// Where the widget goes before it is first dragged: the middle of the screen across, a third down
pub const WIDGET_HOME: (f64, f64) = (0.5, 0.33);
/// The widget may go this far down its window's height below the screen's bottom: the window is
/// mostly room for the card, and the widget at its top stays on screen
const WIDGET_OVERHANG: f64 = 0.7;

/// The range a window `len` long may start in on a side starting at `start`, `side` long, letting
/// it hang `over` past the far end
fn span(start: i32, side: u32, len: u32, over: f64) -> (i32, i32) {
    (start, start + (side as i32 - (len as f64 * (1.0 - over)) as i32).max(0))
}

/// Top-left of the centre widget's window for its stored `spot` (fractions of the screen: x the
/// window's middle, y its top), kept so the widget stays on the screen.
fn widget_position(mon_pos: (i32, i32), mon_size: (u32, u32), win: (u32, u32), spot: (f64, f64)) -> (i32, i32) {
    let (x0, x1) = span(mon_pos.0, mon_size.0, win.0, 0.0);
    let (y0, y1) = span(mon_pos.1, mon_size.1, win.1, WIDGET_OVERHANG);
    let x = mon_pos.0 as f64 + mon_size.0 as f64 * spot.0.clamp(0.0, 1.0) - win.0 as f64 / 2.0;
    let y = mon_pos.1 as f64 + mon_size.1 as f64 * spot.1.clamp(0.0, 1.0);
    ((x.round() as i32).clamp(x0, x1), (y.round() as i32).clamp(y0, y1))
}

/// The bounds a drag keeps the window's top-left in: ((x min, x max), (y min, y max))
pub fn drag_bounds(edge: NotchEdge, mon_pos: (i32, i32), mon_size: (u32, u32), win: (u32, u32)) -> ((i32, i32), (i32, i32)) {
    let over = if edge.is_widget() { WIDGET_OVERHANG } else { 0.0 };
    (span(mon_pos.0, mon_size.0, win.0, 0.0), span(mon_pos.1, mon_size.1, win.1, over))
}

/// The widget's stored spot after a drag left its window's top-left at `pos`.
pub fn widget_after_drag(pos: (i32, i32), win: (u32, u32), mon_pos: (i32, i32), mon_size: (u32, u32)) -> (f64, f64) {
    let x = (pos.0 + win.0 as i32 / 2 - mon_pos.0) as f64 / mon_size.0 as f64;
    let y = (pos.1 - mon_pos.1) as f64 / mon_size.1 as f64;
    (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
}

/// Top-left for a window of physical size `win` flush with `edge`, its centre at `ratio` of the
/// monitor's side along that edge, kept fully on the monitor; for the centre widget, at `spot`.
pub fn position(edge: NotchEdge, mon_pos: (i32, i32), mon_size: (u32, u32), win: (u32, u32), ratio: f64, spot: Option<(f64, f64)>) -> (i32, i32) {
    if edge.is_widget() {
        return widget_position(mon_pos, mon_size, win, spot.unwrap_or(WIDGET_HOME));
    }
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
        NotchEdge::Center => unreachable!("the widget is placed by widget_position"),
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

/// Sizes the window from `mon`'s scale and places it on `mon`. The physical size is pinned straight
/// from that monitor's scale factor: with two monitors at different scales the window could end up
/// converted with the other monitor's factor, leaving the WebView ~256 logical px wide.
pub fn place(w: &tauri::WebviewWindow, mon: &tauri::Monitor, edge: NotchEdge, ratio: f64, spot: Option<(f64, f64)>, length: f64) -> (i32, i32) {
    let s = mon.scale_factor();
    let (lw, lh) = window_size(edge, length);
    let target = tauri::PhysicalSize::new((lw * s).round() as u32, (lh * s).round() as u32);
    let _ = w.set_size(target);
    // The measured size can still be 0x0 before the window is mapped, and lags a resize just
    // asked for: the target is what it will be
    let size = target;
    let (x, y) = position(edge, (mon.position().x, mon.position().y), (mon.size().width, mon.size().height), (size.width, size.height), ratio, spot);
    let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
    crate::diagnostics::log(&format!(
        "notch placed: edge={} pos=({x},{y}) size=({}x{}) scale={s} monitor={} ({},{} {}x{}) gdk={}",
        edge.as_str(),
        size.width,
        size.height,
        mon.name().map(String::as_str).unwrap_or("?"),
        mon.position().x,
        mon.position().y,
        mon.size().width,
        mon.size().height,
        std::env::var("GDK_BACKEND").unwrap_or_default()
    ));
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_right_and_clamped_vertically() {
        // The Ubuntu 26.04 machine: 1920x1080 at 100 %
        assert_eq!(position(NotchEdge::Right, (0, 0), (1920, 1080), (340, 460), 0.5, None), (1580, 310));
        assert_eq!(position(NotchEdge::Right, (0, 0), (1920, 1080), (340, 460), 0.0, None), (1580, 0));
        assert_eq!(position(NotchEdge::Right, (0, 0), (1920, 1080), (340, 460), 1.0, None), (1580, 620));
        // A second monitor to the left and above
        assert_eq!(position(NotchEdge::Right, (-2560, -200), (2560, 1440), (510, 690), 0.5, None), (-510, 175));
    }

    #[test]
    fn every_edge_is_flush_and_centred_along_it() {
        assert_eq!(position(NotchEdge::Left, (0, 0), (1920, 1080), (340, 460), 0.5, None), (0, 310));
        assert_eq!(position(NotchEdge::Top, (0, 0), (1920, 1080), (520, 480), 0.5, None), (700, 0));
        assert_eq!(position(NotchEdge::Bottom, (0, 0), (1920, 1080), (520, 480), 0.5, None), (700, 600));
        // Kept on the monitor at the ends
        assert_eq!(position(NotchEdge::Top, (0, 0), (1920, 1080), (520, 480), 0.0, None), (0, 0));
        assert_eq!(position(NotchEdge::Bottom, (100, 0), (1920, 1080), (520, 480), 1.0, None), (1500, 600));
    }

    #[test]
    fn the_widget_sits_at_its_spot_and_stays_on_screen() {
        let mon = ((0, 0), (1920, 1080));
        // Never dragged: the middle across, a third down
        assert_eq!(position(NotchEdge::Center, mon.0, mon.1, (520, 480), 0.5, None), (700, 356));
        assert_eq!(position(NotchEdge::Center, mon.0, mon.1, (520, 480), 0.5, Some((0.25, 0.1))), (220, 108));
        // Its window may hang below the screen, the widget at its top may not leave it
        assert_eq!(position(NotchEdge::Center, mon.0, mon.1, (520, 480), 0.5, Some((1.0, 1.0))), (1400, 1080 - 144));
        let spot = widget_after_drag((220, 108), (520, 480), mon.0, mon.1);
        assert_eq!(position(NotchEdge::Center, mon.0, mon.1, (520, 480), 0.5, Some(spot)), (220, 108));
    }

    #[test]
    fn drag_ratio_round_trips_through_position() {
        let r = ratio_after_drag(NotchEdge::Right, (1580, 385), (340, 460), (0, 0), (1920, 1080));
        assert_eq!(position(NotchEdge::Right, (0, 0), (1920, 1080), (340, 460), r, None).1, 385);
        let r = ratio_after_drag(NotchEdge::Top, (900, 0), (520, 480), (0, 0), (1920, 1080));
        assert_eq!(position(NotchEdge::Top, (0, 0), (1920, 1080), (520, 480), r, None).0, 900);
    }
}
