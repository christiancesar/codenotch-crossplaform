//! X11 edge reservation via `_NET_WM_STRUT_PARTIAL`.
//!
//! On X11 the notch reserves a thin strip on the right screen edge so maximized
//! windows tile around it like a panel. The strut is set while the notch is
//! visible and cleared when it hides.

use tauri::Manager;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as XprotoConnectionExt, PropMode, Window as XWindow};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

/// Width of the visible pill in logical points. The hover card lives to the
/// left of this strip and must not push maximized windows further away than
/// the pill itself.
pub const PILL_W: f64 = 70.0;

/// The 12 CARDINAL values of `_NET_WM_STRUT_PARTIAL`:
/// left, right, top, bottom,
/// left_start_y, left_end_y, right_start_y, right_end_y,
/// top_start_x, top_end_x, bottom_start_x, bottom_end_x.
pub type StrutArray = [u32; 12];

/// Screen edge the notch is pinned to. Only `Right` is implemented for A.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

/// Build the EWMH `_NET_WM_STRUT_PARTIAL` array for the notch.
///
/// `screen_*` give the X screen area in physical pixels. `notch_*` give the
/// notch window geometry in the same coordinate space. `pill_thickness` is the
/// width (for a vertical edge) or height (for a horizontal edge) of the
/// reserved strip in physical pixels.
///
/// The returned array can be tested without an X connection.
pub fn build_strut(
    screen_x: i32,
    screen_y: i32,
    screen_w: u32,
    screen_h: u32,
    edge: Edge,
    notch_x: i32,
    notch_y: i32,
    notch_w: u32,
    notch_h: u32,
    pill_thickness: u32,
) -> StrutArray {
    let mut strut = [0u32; 12];
    let screen_bottom = screen_y.saturating_add(screen_h as i32);
    let screen_right = screen_x.saturating_add(screen_w as i32);

    match edge {
        Edge::Right => {
            // Reserve a vertical strip along the right edge.
            strut[1] = pill_thickness;
            let start = (notch_y - screen_y).max(0) as u32;
            let end = (notch_y + notch_h as i32 - 1 - screen_y)
                .min(screen_bottom - screen_y - 1)
                .max(start as i32) as u32;
            strut[6] = start; // right_start_y
            strut[7] = end; // right_end_y
        }
        Edge::Left => {
            strut[0] = pill_thickness;
            let start = (notch_y - screen_y).max(0) as u32;
            let end = (notch_y + notch_h as i32 - 1 - screen_y)
                .min(screen_bottom - screen_y - 1)
                .max(start as i32) as u32;
            strut[4] = start; // left_start_y
            strut[5] = end; // left_end_y
        }
        Edge::Top => {
            strut[2] = pill_thickness;
            let start = (notch_x - screen_x).max(0) as u32;
            let end = (notch_x + notch_w as i32 - 1 - screen_x)
                .min(screen_right - screen_x - 1)
                .max(start as i32) as u32;
            strut[8] = start; // top_start_x
            strut[9] = end; // top_end_x
        }
        Edge::Bottom => {
            strut[3] = pill_thickness;
            let start = (notch_x - screen_x).max(0) as u32;
            let end = (notch_x + notch_w as i32 - 1 - screen_x)
                .min(screen_right - screen_x - 1)
                .max(start as i32) as u32;
            strut[10] = start; // bottom_start_x
            strut[11] = end; // bottom_end_x
        }
    }

    strut
}

/// Best-effort: read the notch XID and set the strut from its current geometry.
pub fn set_strut_for_notch(app: &tauri::AppHandle) {
    crate::applog("x11 strut: set_strut_for_notch called");
    let Some(window) = app.get_webview_window("notch") else {
        crate::applog("x11 strut: no notch window");
        return;
    };
    let Some(xid) = xid_of(&window) else {
        crate::applog("x11 strut: no xid");
        return;
    };
    crate::applog(&format!("x11 strut: xid={xid}"));

    let scale = window.scale_factor().unwrap_or(1.0);
    let pill_thickness = (PILL_W * scale).round() as u32;

    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };
    let Ok(pos) = window.outer_position() else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };
    // The very first placement call runs before GTK has realized the window, so
    // outer_size() answers 0x0. Applying a strut with that geometry would reserve
    // a zero-height strip and momentarily desync the workarea from where the pill
    // actually ends up; wait for the resize/move that follows realization instead.
    if size.width == 0 || size.height == 0 {
        crate::applog("x11 strut: skipping — window not realized yet (size 0x0)");
        return;
    }

    // _NET_WM_STRUT[_PARTIAL]'s "right" value reserves N pixels from the right edge
    // of the X11 root window — which spans every monitor combined, not just this
    // one. On a multi-monitor setup where the notch's monitor is not the physically
    // rightmost, "reserve 70px from the right" carves that strip out of whichever
    // monitor actually sits at the root's right edge instead — an empty notch-sized
    // gap on a screen the notch was never near. There is no single-edge way to
    // express "reserve the right edge of monitor N" when another monitor extends
    // further right, so skip the strut entirely rather than reserve the wrong
    // screen's space; the notch still floats always-on-top there.
    let root_w = match root_screen_width() {
        Some(w) => w,
        None => {
            crate::applog("x11 strut: skipping — could not read root screen width");
            return;
        }
    };
    let monitor_right = monitor.position().x + monitor.size().width as i32;
    if monitor_right < root_w as i32 {
        crate::applog(&format!(
            "x11 strut: skipping — notch's monitor ends at x={monitor_right}, root screen is {root_w}px wide (another display extends further right)"
        ));
        clear_strut_for_notch(app);
        return;
    }

    let strut = build_strut(
        monitor.position().x,
        monitor.position().y,
        monitor.size().width,
        monitor.size().height,
        Edge::Right,
        pos.x,
        pos.y,
        size.width,
        size.height,
        pill_thickness,
    );

    crate::applog(&format!(
        "x11 strut: applying strut={strut:?} for pos=({},{}) size=({}x{}) pill={pill_thickness}",
        pos.x, pos.y, size.width, size.height
    ));
    if let Err(e) = apply_strut(xid, &strut) {
        crate::applog(&format!("x11 strut set failed: {e}"));
    } else {
        crate::applog("x11 strut: applied ok");
    }
}

/// Best-effort: remove the strut property before the notch hides.
pub fn clear_strut_for_notch(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("notch") else {
        return;
    };
    let Some(xid) = xid_of(&window) else { return };
    if let Err(e) = clear_strut(xid) {
        crate::applog(&format!("x11 strut clear failed: {e}"));
    }
}

/// Tauri does not expose the XID directly; reach through its GTK window to the
/// GDK X11 surface. This matches the A.1 ticket and avoids relying on the raw
/// window handle, which is not always available during early setup.
fn xid_of(window: &tauri::WebviewWindow) -> Option<u64> {
    use gtk::glib::Cast;
    use gtk::prelude::WidgetExt;

    let gtk_window = match window.gtk_window() {
        Ok(w) => w,
        Err(e) => {
            crate::applog(&format!("x11 strut: gtk_window failed: {e}"));
            return None;
        }
    };
    if !gtk_window.is_realized() {
        crate::applog("x11 strut: realizing gtk window");
        gtk_window.realize();
    }
    let gdk_window = match gtk_window.window() {
        Some(w) => w,
        None => {
            crate::applog("x11 strut: gdk window not realized");
            return None;
        }
    };
    let x11_window = match gdk_window.downcast::<gdkx11::X11Window>() {
        Ok(w) => w,
        Err(_) => {
            crate::applog("x11 strut: not an X11 window");
            return None;
        }
    };
    let xid = x11_window.xid();
    crate::applog(&format!("x11 strut: gdk xid={xid}"));
    Some(xid)
}

/// Width in pixels of the X11 root window — the combined width of every monitor,
/// which is the coordinate space `_NET_WM_STRUT[_PARTIAL]`'s "right" value is
/// measured against (not any single monitor's width).
fn root_screen_width() -> Option<u32> {
    let (conn, screen_num) = RustConnection::connect(None).ok()?;
    conn.setup()
        .roots
        .get(screen_num)
        .map(|screen| screen.width_in_pixels as u32)
}

fn apply_strut(xid: u64, strut: &StrutArray) -> Result<(), Box<dyn std::error::Error>> {
    let (conn, _) = RustConnection::connect(None)?;
    let cardinal = conn.intern_atom(false, b"CARDINAL")?.reply()?.atom;

    // _NET_WM_STRUT_PARTIAL is the modern, precise property required by A.1.
    let partial = conn
        .intern_atom(false, b"_NET_WM_STRUT_PARTIAL")?
        .reply()?
        .atom;
    conn.change_property32(PropMode::REPLACE, xid as XWindow, partial, cardinal, strut)?;

    // _NET_WM_STRUT (the four-value legacy form) keeps some WMs happy; derive it
    // from the partial values so both properties agree on which edges are reserved.
    let legacy = conn.intern_atom(false, b"_NET_WM_STRUT")?.reply()?.atom;
    let legacy_values = [strut[0], strut[1], strut[2], strut[3]];
    conn.change_property32(
        PropMode::REPLACE,
        xid as XWindow,
        legacy,
        cardinal,
        &legacy_values,
    )?;

    conn.flush()?;
    Ok(())
}

fn clear_strut(xid: u64) -> Result<(), Box<dyn std::error::Error>> {
    let (conn, _) = RustConnection::connect(None)?;
    let partial = conn
        .intern_atom(false, b"_NET_WM_STRUT_PARTIAL")?
        .reply()?
        .atom;
    conn.delete_property(xid as XWindow, partial)?;
    let legacy = conn.intern_atom(false, b"_NET_WM_STRUT")?.reply()?.atom;
    conn.delete_property(xid as XWindow, legacy)?;
    conn.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_strut, Edge, StrutArray};

    #[test]
    fn right_edge_reserves_pill_width_and_vertical_span() {
        // 1920x1080 screen, notch flush on the right edge, 70 px pill.
        let strut = build_strut(0, 0, 1920, 1080, Edge::Right, 1850, 310, 70, 460, 70);
        assert_eq!(
            strut,
            [
                0, 70, 0, 0, // left, right, top, bottom
                0, 0, // left_start_y, left_end_y
                310, 769, // right_start_y, right_end_y = 310 + 460 - 1
                0, 0, 0, 0, // top_start_x, top_end_x, bottom_start_x, bottom_end_x
            ]
        );
    }

    #[test]
    fn vertical_span_is_clamped_to_screen() {
        // Notch slightly below the top and taller than the screen.
        let strut = build_strut(0, 0, 1920, 1080, Edge::Right, 1850, 900, 70, 460, 70);
        assert_eq!(strut[6], 900); // right_start_y
        assert_eq!(strut[7], 1079); // clamped to screen_h - 1
    }

    #[test]
    fn multi_monitor_screen_offset_is_removed() {
        // Monitor starts at y=200 in the X screen.
        let strut = build_strut(0, 200, 1920, 1080, Edge::Right, 1850, 510, 70, 460, 70);
        assert_eq!(strut[6], 310); // 510 - 200
        assert_eq!(strut[7], 769); // 510 + 460 - 1 - 200
    }

    #[test]
    fn left_edge_is_symmetric() {
        let strut = build_strut(0, 0, 1920, 1080, Edge::Left, 0, 310, 70, 460, 70);
        assert_eq!(strut[0], 70);
        assert_eq!(strut[4], 310);
        assert_eq!(strut[5], 769);
    }

    #[test]
    fn top_edge_reserves_height_and_horizontal_span() {
        let strut = build_strut(0, 0, 1920, 1080, Edge::Top, 310, 0, 460, 70, 70);
        assert_eq!(strut[2], 70);
        assert_eq!(strut[8], 310);
        assert_eq!(strut[9], 769);
    }

    #[test]
    fn bottom_edge_is_symmetric() {
        let strut = build_strut(0, 0, 1920, 1080, Edge::Bottom, 310, 1010, 460, 70, 70);
        assert_eq!(strut[3], 70);
        assert_eq!(strut[10], 310);
        assert_eq!(strut[11], 769);
    }
}
